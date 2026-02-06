use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistory {
    pub id: String,
    pub platform: String,
    pub query: String,
    pub results_count: u64,
    pub timestamp: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryStore {
    records: Vec<QueryHistory>,
}

impl HistoryStore {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
}

// 获取历史记录文件路径
fn get_history_file_path() -> Result<PathBuf, String> {
    let config_dir = tauri::api::path::config_dir()
        .ok_or_else(|| "无法获取配置目录".to_string())?;
    
    let app_dir = config_dir.join("asset-mapping");
    
    // 确保目录存在
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;
    
    Ok(app_dir.join("query_history.json"))
}

// 加载历史记录
fn load_history() -> Result<HistoryStore, String> {
    let file_path = get_history_file_path()?;
    
    if !file_path.exists() {
        return Ok(HistoryStore::new());
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取历史记录失败: {}", e))?;
    
    let store: HistoryStore = serde_json::from_str(&content)
        .unwrap_or_else(|_| HistoryStore::new());
    
    Ok(store)
}

// 保存历史记录
fn save_history(store: &HistoryStore) -> Result<(), String> {
    let file_path = get_history_file_path()?;
    
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("序列化历史记录失败: {}", e))?;
    
    fs::write(&file_path, content)
        .map_err(|e| format!("保存历史记录失败: {}", e))?;
    
    Ok(())
}

// 添加历史记录
pub fn add_history(
    platform: String,
    query: String,
    results_count: u64,
    success: bool,
    error_message: Option<String>,
) -> Result<(), String> {
    let mut store = load_history()?;
    
    let now: DateTime<Utc> = Utc::now();
    let id = format!("{}_{}", platform, now.timestamp_millis());
    
    let record = QueryHistory {
        id,
        platform,
        query,
        results_count,
        timestamp: now.to_rfc3339(),
        success,
        error_message,
    };
    
    // 添加到列表开头（最新的在前面）
    store.records.insert(0, record);
    
    // 限制历史记录数量（最多保留1000条）
    if store.records.len() > 1000 {
        store.records.truncate(1000);
    }
    
    save_history(&store)?;
    
    Ok(())
}

// 获取所有历史记录
pub fn get_all_history() -> Result<Vec<QueryHistory>, String> {
    let store = load_history()?;
    Ok(store.records)
}

// 根据平台筛选历史记录
pub fn get_history_by_platform(platform: &str) -> Result<Vec<QueryHistory>, String> {
    let store = load_history()?;
    let filtered: Vec<QueryHistory> = store.records
        .into_iter()
        .filter(|r| r.platform == platform)
        .collect();
    Ok(filtered)
}

// 搜索历史记录
pub fn search_history(keyword: &str) -> Result<Vec<QueryHistory>, String> {
    let store = load_history()?;
    let keyword_lower = keyword.to_lowercase();
    
    let filtered: Vec<QueryHistory> = store.records
        .into_iter()
        .filter(|r| {
            r.query.to_lowercase().contains(&keyword_lower) ||
            r.platform.to_lowercase().contains(&keyword_lower)
        })
        .collect();
    
    Ok(filtered)
}

// 删除历史记录
pub fn delete_history(id: &str) -> Result<(), String> {
    let mut store = load_history()?;
    store.records.retain(|r| r.id != id);
    save_history(&store)?;
    Ok(())
}

// 清空历史记录
pub fn clear_all_history() -> Result<(), String> {
    let store = HistoryStore::new();
    save_history(&store)?;
    Ok(())
}

// 导出历史记录到CSV
pub fn export_history_to_csv(export_path: &str) -> Result<String, String> {
    let store = load_history()?;
    
    if store.records.is_empty() {
        return Err("没有历史记录可导出".to_string());
    }
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/query_history_{}.csv", export_path, timestamp);
    
    let mut wtr = csv::Writer::from_path(&file_path)
        .map_err(|e| format!("创建CSV文件失败: {}", e))?;
    
    // 写入表头
    wtr.write_record(&["ID", "平台", "查询语句", "结果数量", "时间", "状态", "错误信息"])
        .map_err(|e| format!("写入CSV表头失败: {}", e))?;
    
    // 写入数据
    for record in &store.records {
        let status = if record.success { "成功".to_string() } else { "失败".to_string() };
        wtr.write_record(&[
            &record.id,
            &record.platform,
            &record.query,
            &record.results_count.to_string(),
            &record.timestamp,
            &status,
            record.error_message.as_deref().unwrap_or(""),
        ])
        .map_err(|e| format!("写入CSV数据失败: {}", e))?;
    }
    
    wtr.flush()
        .map_err(|e| format!("保存CSV文件失败: {}", e))?;
    
    Ok(file_path)
}

