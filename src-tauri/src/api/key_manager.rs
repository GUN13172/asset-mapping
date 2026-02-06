use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Local;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// API Key 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStatus {
    pub key: String,
    pub is_exhausted: bool,
    pub exhausted_at: Option<String>, // ISO 8601 日期时间
    pub last_used_at: Option<String>,
}

/// Key 管理器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagerState {
    pub current_index: usize,
    pub keys: Vec<KeyStatus>,
    pub last_reset_date: String, // YYYY-MM-DD
}

/// 全局 Key 管理器（支持多平台）
#[allow(dead_code)]
pub static KEY_MANAGERS: Lazy<Mutex<HashMap<String, KeyManager>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub struct KeyManager {
    platform: String,
    state_file: PathBuf,
}

impl KeyManager {
    pub fn new(platform: &str) -> Self {
        let state_file = Self::get_state_file_path(platform);
        KeyManager { 
            platform: platform.to_string(),
            state_file 
        }
    }

    fn get_state_file_path(platform: &str) -> PathBuf {
        let config_dir = tauri::api::path::config_dir()
            .expect("无法获取配置目录")
            .join("asset-mapping");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).ok();
        }
        
        config_dir.join(format!("{}_key_state.json", platform))
    }

    /// 加载状态
    fn load_state(&self) -> Option<KeyManagerState> {
        if !self.state_file.exists() {
            return None;
        }

        let content = fs::read_to_string(&self.state_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// 保存状态
    fn save_state(&self, state: &KeyManagerState) -> Result<(), String> {
        let content = serde_json::to_string_pretty(state)
            .map_err(|e| format!("序列化状态失败: {}", e))?;
        
        fs::write(&self.state_file, content)
            .map_err(|e| format!("写入状态文件失败: {}", e))?;
        
        Ok(())
    }

    /// 初始化或更新 keys
    pub fn initialize_keys(&self, api_keys: Vec<String>) -> Result<KeyManagerState, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        
        // 尝试加载现有状态
        if let Some(mut state) = self.load_state() {
            // 检查是否需要重置（新的一天）
            if state.last_reset_date != today {
                eprintln!("[{}] 检测到新的一天，重置所有 key 状态", self.platform);
                state = self.reset_all_keys(api_keys, today);
            } else {
                // 更新 keys 列表（可能有新增或删除）
                state = self.update_keys(state, api_keys);
            }
            
            self.save_state(&state)?;
            return Ok(state);
        }

        // 首次初始化
        let state = self.reset_all_keys(api_keys, today);
        self.save_state(&state)?;
        Ok(state)
    }

    /// 重置所有 keys
    fn reset_all_keys(&self, api_keys: Vec<String>, date: String) -> KeyManagerState {
        let keys = api_keys.into_iter().map(|key| KeyStatus {
            key,
            is_exhausted: false,
            exhausted_at: None,
            last_used_at: None,
        }).collect();

        KeyManagerState {
            current_index: 0,
            keys,
            last_reset_date: date,
        }
    }

    /// 更新 keys 列表
    fn update_keys(&self, mut state: KeyManagerState, new_keys: Vec<String>) -> KeyManagerState {
        // 保留现有 key 的状态
        let mut updated_keys = Vec::new();
        
        for new_key in new_keys {
            if let Some(existing) = state.keys.iter().find(|k| k.key == new_key) {
                // 保留现有状态
                updated_keys.push(existing.clone());
            } else {
                // 新增的 key
                updated_keys.push(KeyStatus {
                    key: new_key,
                    is_exhausted: false,
                    exhausted_at: None,
                    last_used_at: None,
                });
            }
        }

        // 调整游标位置
        if state.current_index >= updated_keys.len() {
            state.current_index = 0;
        }

        state.keys = updated_keys;
        state
    }

    /// 获取下一个可用的 key
    pub fn get_next_available_key(&self, api_keys: Vec<String>) -> Result<(String, usize), String> {
        let mut state = self.initialize_keys(api_keys)?;
        
        // 从当前游标开始查找可用的 key
        let start_index = state.current_index;
        let total_keys = state.keys.len();
        
        for offset in 0..total_keys {
            let index = (start_index + offset) % total_keys;
            let key_status = &state.keys[index];
            
            if !key_status.is_exhausted {
                eprintln!("[{}] 使用 Key {} (索引 {}): {}...", 
                         self.platform, offset + 1, index, &key_status.key[..8.min(key_status.key.len())]);
                
                // 更新游标
                state.current_index = index;
                self.save_state(&state)?;
                
                return Ok((key_status.key.clone(), index));
            }
        }

        Err(format!("[{}] 所有 API Key 都已额度耗尽", self.platform))
    }

    /// 标记 key 为已耗尽
    pub fn mark_key_exhausted(&self, key_index: usize, api_keys: Vec<String>) -> Result<(), String> {
        let mut state = self.initialize_keys(api_keys)?;
        
        if key_index < state.keys.len() {
            let now = Local::now().to_rfc3339();
            state.keys[key_index].is_exhausted = true;
            state.keys[key_index].exhausted_at = Some(now);
            
            eprintln!("[{}] 标记 Key {} 为已耗尽: {}...", 
                     self.platform, key_index + 1, &state.keys[key_index].key[..8.min(state.keys[key_index].key.len())]);
            
            // 移动游标到下一个位置
            state.current_index = (key_index + 1) % state.keys.len();
            
            self.save_state(&state)?;
        }

        Ok(())
    }

    /// 更新 key 的最后使用时间
    pub fn update_last_used(&self, key_index: usize, api_keys: Vec<String>) -> Result<(), String> {
        let mut state = self.initialize_keys(api_keys)?;
        
        if key_index < state.keys.len() {
            let now = Local::now().to_rfc3339();
            state.keys[key_index].last_used_at = Some(now);
            self.save_state(&state)?;
        }

        Ok(())
    }

    /// 获取状态摘要
    #[allow(dead_code)]
    pub fn get_status_summary(&self, api_keys: Vec<String>) -> Result<String, String> {
        let state = self.initialize_keys(api_keys)?;
        
        let total = state.keys.len();
        let exhausted = state.keys.iter().filter(|k| k.is_exhausted).count();
        let available = total - exhausted;
        
        Ok(format!(
            "[{}] 总计: {} | 可用: {} | 已耗尽: {} | 游标: {}",
            self.platform, total, available, exhausted, state.current_index + 1
        ))
    }
}

/// 便捷函数：获取下一个可用的 key
pub fn get_next_key(platform: &str, api_keys: Vec<String>) -> Result<(String, usize), String> {
    let manager = KeyManager::new(platform);
    manager.get_next_available_key(api_keys)
}

/// 便捷函数：标记 key 为已耗尽
pub fn mark_exhausted(platform: &str, key_index: usize, api_keys: Vec<String>) -> Result<(), String> {
    let manager = KeyManager::new(platform);
    manager.mark_key_exhausted(key_index, api_keys)
}

/// 便捷函数：更新最后使用时间
pub fn update_used(platform: &str, key_index: usize, api_keys: Vec<String>) -> Result<(), String> {
    let manager = KeyManager::new(platform);
    manager.update_last_used(key_index, api_keys)
}

/// 便捷函数：获取状态摘要
#[allow(dead_code)]
pub fn get_status(platform: &str, api_keys: Vec<String>) -> Result<String, String> {
    let manager = KeyManager::new(platform);
    manager.get_status_summary(api_keys)
}

/// 便捷函数：使用 key 轮询执行操作
/// 
/// 这个函数会自动处理 key 轮询逻辑：
/// 1. 获取下一个可用的 key
/// 2. 执行提供的异步操作
/// 3. 如果操作失败且是配额耗尽错误，标记 key 为已耗尽并尝试下一个
/// 4. 如果操作成功，更新 key 的最后使用时间
pub async fn execute_with_key_rotation<F, Fut, T>(
    platform: &str,
    api_keys: &[String],
    operation: F,
) -> Result<T, String>
where
    F: Fn(&str) -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let api_keys_vec = api_keys.to_vec();
    let max_attempts = api_keys_vec.len();
    
    for attempt in 0..max_attempts {
        // 获取下一个可用的 key
        let (api_key, key_index) = match get_next_key(platform, api_keys_vec.clone()) {
            Ok(result) => result,
            Err(e) => {
                if attempt == 0 {
                    return Err(e);
                }
                // 所有 key 都已耗尽
                return Err(format!("[{}] 所有API Key都无法使用", platform));
            }
        };
        
        // 执行操作
        match operation(&api_key).await {
            Ok(result) => {
                // 成功，更新最后使用时间
                update_used(platform, key_index, api_keys_vec.clone()).ok();
                return Ok(result);
            }
            Err(e) => {
                // 检查是否是配额耗尽错误
                let is_quota_error = e.contains("积分") 
                    || e.contains("quota") 
                    || e.contains("次牛")
                    || e.contains("F币")
                    || e.contains("2004");
                
                if is_quota_error {
                    eprintln!("[{}] Key {} 配额耗尽，尝试下一个...", platform, key_index + 1);
                    mark_exhausted(platform, key_index, api_keys_vec.clone()).ok();
                    continue; // 尝试下一个 key
                } else {
                    // 其他错误，直接返回
                    return Err(e);
                }
            }
        }
    }
    
    Err(format!("[{}] 所有API Key都无法使用", platform))
}
