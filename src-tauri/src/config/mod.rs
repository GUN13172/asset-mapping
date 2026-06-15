use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub mod platform;
pub use platform::*;

// 配置文件路径
const CONFIG_DIR: &str = "asset-mapping";
const HUNTER_CONFIG_FILE: &str = "hunter_api.json";
const FOFA_CONFIG_FILE: &str = "fofa_api.json";
const QUAKE_CONFIG_FILE: &str = "quake_api.json";
const DAYDAYMAP_CONFIG_FILE: &str = "daydaymap_api.json";
const SETTINGS_FILE: &str = "settings.json";

// 设置结构体（供内部兼容使用，实际序列化使用 crate::Settings）
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub export_path: String,
    pub default_platform: String,
    pub page_size: u32,
    pub auto_validate_api_keys: bool,
    pub theme: String,
    pub language: String,
    #[serde(default)]
    pub allow_insecure_tls: bool,
    // 代理配置
    #[serde(default)]
    pub proxy_enabled: bool,
    #[serde(default)]
    pub proxy_url: String,
    #[serde(default)]
    pub proxy_username: String,
    #[serde(default)]
    pub proxy_password: String,
    // 请求超时（秒）
    #[serde(default = "crate::default_timeout")]
    pub request_timeout: u32,
}

// 获取配置目录
fn get_config_dir() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取配置目录".to_string())?
        .join(CONFIG_DIR);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    Ok(config_dir)
}

// 获取导出路径
pub fn get_export_path() -> Result<String, String> {
    let settings = get_settings()?;
    if settings.export_path.is_empty() {
        let download_dir = dirs::download_dir().ok_or_else(|| "无法获取下载目录".to_string())?;
        Ok(download_dir.to_string_lossy().to_string())
    } else {
        Ok(settings.export_path)
    }
}

// ===== API 密钥混淆编解码 =====
const KEY_MASK: &[u8] = b"asset-mapping-secret-2024";

/// 将明文密钥编码为混淆字符串（XOR + Base64）
pub fn encode_key(plain: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    let masked: Vec<u8> = plain
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ KEY_MASK[i % KEY_MASK.len()])
        .collect();
    general_purpose::STANDARD.encode(&masked)
}

/// 将混淆字符串解码为明文密钥，如果解码失败则当作明文返回（兼容旧配置）
pub fn decode_key(encoded: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    match general_purpose::STANDARD.decode(encoded) {
        Ok(masked) => {
            let plain: Vec<u8> = masked
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ KEY_MASK[i % KEY_MASK.len()])
                .collect();
            String::from_utf8(plain).unwrap_or_else(|_| encoded.to_string())
        }
        Err(_) => encoded.to_string(), // base64 解码失败，当作明文
    }
}

// 获取Hunter API密钥
#[allow(dead_code)]
pub fn get_hunter_api_key() -> Result<String, String> {
    let api_keys = get_hunter_api_keys_internal()?;

    if api_keys.is_empty() {
        return Err("未配置Hunter API密钥".to_string());
    }

    Ok(api_keys[0].clone())
}

// 获取Hunter API密钥列表
fn get_hunter_api_keys_internal() -> Result<Vec<String>, String> {
    let config_file = get_config_dir()?.join(HUNTER_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let api_keys = config["api_keys"]
        .as_array()
        .ok_or_else(|| "配置文件格式错误".to_string())?
        .iter()
        .filter_map(|v| v.as_str())
        .map(decode_key)
        .collect();

    Ok(api_keys)
}

// 获取Hunter API密钥列表（供前端使用）
pub fn get_hunter_api_keys() -> Result<Value, String> {
    let api_keys = get_hunter_api_keys_internal()?;
    Ok(json!({ "api_keys": api_keys }))
}

// 添加Hunter API密钥
pub fn add_hunter_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(HUNTER_CONFIG_FILE);

    // 读取已有密钥（解码后的明文）
    let existing_keys = if config_file.exists() {
        get_hunter_api_keys_internal().unwrap_or_default()
    } else {
        Vec::new()
    };

    // 检查是否已存在
    if existing_keys.contains(&api_key.to_string()) {
        return Ok(());
    }

    // 所有密钥重新编码后保存
    let mut all_keys = existing_keys;
    all_keys.push(api_key.to_string());
    let encoded: Vec<String> = all_keys.iter().map(|k| encode_key(k)).collect();

    let config = json!({
        "api_keys": encoded
    });

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 删除Hunter API密钥
pub fn delete_hunter_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(HUNTER_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(());
    }

    // 读取现有密钥（解码后），过滤掉要删除的，重新编码保存
    let remaining: Vec<String> = get_hunter_api_keys_internal()?
        .into_iter()
        .filter(|k| k != api_key)
        .collect();
    let encoded: Vec<String> = remaining.iter().map(|k| encode_key(k)).collect();

    let new_config = json!({ "api_keys": encoded });

    let new_content =
        serde_json::to_string_pretty(&new_config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, new_content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 获取FOFA API密钥
#[allow(dead_code)]
pub fn get_fofa_api_key() -> Result<(String, String), String> {
    let (api_keys, emails) = get_fofa_api_keys_internal()?;

    if api_keys.is_empty() || emails.is_empty() {
        return Err("未配置FOFA API密钥或邮箱".to_string());
    }

    Ok((api_keys[0].clone(), emails[0].clone()))
}

// 获取FOFA API密钥列表
fn get_fofa_api_keys_internal() -> Result<(Vec<String>, Vec<String>), String> {
    let config_file = get_config_dir()?.join(FOFA_CONFIG_FILE);

    if !config_file.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let api_keys = config["api_keys"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(decode_key)
        .collect();

    let emails = config["emails"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(decode_key)
        .collect();

    Ok((api_keys, emails))
}

// 获取FOFA API密钥列表（供前端使用）
pub fn get_fofa_api_keys() -> Result<Value, String> {
    let (api_keys, emails) = get_fofa_api_keys_internal()?;
    Ok(json!({
        "api_keys": api_keys,
        "emails": emails
    }))
}

// 添加FOFA API密钥
pub fn add_fofa_api_key(api_key: &str, email: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(FOFA_CONFIG_FILE);

    // 读取已有密钥（解码后的明文）
    let (existing_keys, existing_emails) = if config_file.exists() {
        get_fofa_api_keys_internal().unwrap_or_default()
    } else {
        (Vec::new(), Vec::new())
    };

    // 检查是否已存在
    if existing_keys.contains(&api_key.to_string()) {
        return Ok(());
    }

    let mut all_keys = existing_keys;
    let mut all_emails = existing_emails;
    all_keys.push(api_key.to_string());
    all_emails.push(email.to_string());

    let encoded_keys: Vec<String> = all_keys.iter().map(|k| encode_key(k)).collect();
    let encoded_emails: Vec<String> = all_emails.iter().map(|e| encode_key(e)).collect();

    let config = json!({
        "api_keys": encoded_keys,
        "emails": encoded_emails
    });

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 删除FOFA API密钥
pub fn delete_fofa_api_key(api_key: &str, email: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(FOFA_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(());
    }

    // 读取现有密钥（解码后），过滤掉要删除的
    let (existing_keys, existing_emails) = get_fofa_api_keys_internal()?;
    let mut remaining_keys = Vec::new();
    let mut remaining_emails = Vec::new();

    for (k, e) in existing_keys.iter().zip(existing_emails.iter()) {
        if k != api_key || e != email {
            remaining_keys.push(encode_key(k));
            remaining_emails.push(encode_key(e));
        }
    }

    let new_config = json!({
        "api_keys": remaining_keys,
        "emails": remaining_emails
    });

    let new_content =
        serde_json::to_string_pretty(&new_config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, new_content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 获取Quake API密钥
#[allow(dead_code)]
pub fn get_quake_api_key() -> Result<String, String> {
    let api_keys = get_quake_api_keys_internal()?;

    if api_keys.is_empty() {
        return Err("未配置Quake API密钥".to_string());
    }

    Ok(api_keys[0].clone())
}

// 获取Quake API密钥列表
fn get_quake_api_keys_internal() -> Result<Vec<String>, String> {
    let config_file = get_config_dir()?.join(QUAKE_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let api_keys = config["api_keys"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(decode_key)
        .collect();

    Ok(api_keys)
}

// 获取Quake API密钥列表（供前端使用）
pub fn get_quake_api_keys() -> Result<Value, String> {
    let api_keys = get_quake_api_keys_internal()?;
    Ok(json!({ "api_keys": api_keys }))
}

// 添加Quake API密钥
pub fn add_quake_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(QUAKE_CONFIG_FILE);

    let existing_keys = if config_file.exists() {
        get_quake_api_keys_internal().unwrap_or_default()
    } else {
        Vec::new()
    };

    if existing_keys.contains(&api_key.to_string()) {
        return Ok(());
    }

    let mut all_keys = existing_keys;
    all_keys.push(api_key.to_string());
    let encoded: Vec<String> = all_keys.iter().map(|k| encode_key(k)).collect();

    let config = json!({
        "api_keys": encoded
    });

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 删除Quake API密钥
pub fn delete_quake_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(QUAKE_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(());
    }

    let remaining: Vec<String> = get_quake_api_keys_internal()?
        .into_iter()
        .filter(|k| k != api_key)
        .collect();
    let encoded: Vec<String> = remaining.iter().map(|k| encode_key(k)).collect();

    let new_config = json!({ "api_keys": encoded });

    let new_content =
        serde_json::to_string_pretty(&new_config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, new_content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 获取DayDayMap API密钥
#[allow(dead_code)]
pub fn get_daydaymap_api_key() -> Result<String, String> {
    let api_keys = get_daydaymap_api_keys_internal()?;

    if api_keys.is_empty() {
        return Err("未配置DayDayMap API密钥".to_string());
    }

    Ok(api_keys[0].clone())
}

// 获取所有DayDayMap API密钥（用于轮询）
pub fn get_all_daydaymap_api_keys() -> Result<Vec<String>, String> {
    get_daydaymap_api_keys_internal()
}

// 获取所有Hunter API密钥（用于轮询）
pub fn get_all_hunter_api_keys() -> Result<Vec<String>, String> {
    get_hunter_api_keys_internal()
}

// 获取所有FOFA API密钥（用于轮询）
pub fn get_all_fofa_api_keys() -> Result<Vec<(String, String)>, String> {
    let (api_keys, emails) = get_fofa_api_keys_internal()?;

    // 将API密钥和邮箱配对
    let paired_keys: Vec<(String, String)> = api_keys.into_iter().zip(emails).collect();

    Ok(paired_keys)
}

// 获取所有Quake API密钥（用于轮询）
pub fn get_all_quake_api_keys() -> Result<Vec<String>, String> {
    get_quake_api_keys_internal()
}

// 获取DayDayMap API密钥列表
fn get_daydaymap_api_keys_internal() -> Result<Vec<String>, String> {
    let config_file = get_config_dir()?.join(DAYDAYMAP_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let api_keys = config["api_keys"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(decode_key)
        .collect();

    Ok(api_keys)
}

// 获取DayDayMap API密钥列表（供前端使用）
pub fn get_daydaymap_api_keys() -> Result<Value, String> {
    let api_keys = get_daydaymap_api_keys_internal()?;
    Ok(json!({ "api_keys": api_keys }))
}

// 添加DayDayMap API密钥
pub fn add_daydaymap_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(DAYDAYMAP_CONFIG_FILE);

    let existing_keys = if config_file.exists() {
        get_daydaymap_api_keys_internal().unwrap_or_default()
    } else {
        Vec::new()
    };

    if existing_keys.contains(&api_key.to_string()) {
        return Ok(());
    }

    let mut all_keys = existing_keys;
    all_keys.push(api_key.to_string());
    let encoded: Vec<String> = all_keys.iter().map(|k| encode_key(k)).collect();

    let config = json!({
        "api_keys": encoded
    });

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 删除DayDayMap API密钥
pub fn delete_daydaymap_api_key(api_key: &str) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(DAYDAYMAP_CONFIG_FILE);

    if !config_file.exists() {
        return Ok(());
    }

    let remaining: Vec<String> = get_daydaymap_api_keys_internal()?
        .into_iter()
        .filter(|k| k != api_key)
        .collect();
    let encoded: Vec<String> = remaining.iter().map(|k| encode_key(k)).collect();

    let new_config = json!({ "api_keys": encoded });

    let new_content =
        serde_json::to_string_pretty(&new_config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, new_content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

// 获取设置
pub fn get_settings() -> Result<crate::Settings, String> {
    let config_file = get_config_dir()?.join(SETTINGS_FILE);

    if !config_file.exists() {
        // 返回默认设置
        return Ok(crate::Settings {
            export_path: String::new(),
            default_platform: "hunter".to_string(),
            page_size: 20,
            auto_validate_api_keys: true,
            theme: "dark".to_string(),
            language: "zh_CN".to_string(),
            allow_insecure_tls: false,
            proxy_enabled: false,
            proxy_url: String::new(),
            proxy_username: String::new(),
            proxy_password: String::new(),
            request_timeout: 30,
        });
    }

    let content =
        fs::read_to_string(&config_file).map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 尝试用 camelCase 解析（前端传来的格式）
    let settings: crate::Settings = serde_json::from_str(&content)
        .or_else(|_| {
            // 如果失败，尝试用 snake_case 解析（旧格式兼容）
            let value: Value =
                serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;
            Ok::<crate::Settings, String>(crate::Settings {
                export_path: value["export_path"]
                    .as_str()
                    .or_else(|| value["exportPath"].as_str())
                    .unwrap_or("")
                    .to_string(),
                default_platform: value["default_platform"]
                    .as_str()
                    .or_else(|| value["defaultPlatform"].as_str())
                    .unwrap_or("hunter")
                    .to_string(),
                page_size: value["page_size"]
                    .as_u64()
                    .or_else(|| value["pageSize"].as_u64())
                    .unwrap_or(20) as u32,
                auto_validate_api_keys: value["auto_validate_api_keys"]
                    .as_bool()
                    .or_else(|| value["autoValidateApiKeys"].as_bool())
                    .unwrap_or(true),
                theme: value["theme"].as_str().unwrap_or("dark").to_string(),
                language: value["language"].as_str().unwrap_or("zh_CN").to_string(),
                allow_insecure_tls: value["allow_insecure_tls"]
                    .as_bool()
                    .or_else(|| value["allowInsecureTls"].as_bool())
                    .unwrap_or(false),
                proxy_enabled: value["proxy_enabled"]
                    .as_bool()
                    .or_else(|| value["proxyEnabled"].as_bool())
                    .unwrap_or(false),
                proxy_url: value["proxy_url"]
                    .as_str()
                    .or_else(|| value["proxyUrl"].as_str())
                    .unwrap_or("")
                    .to_string(),
                proxy_username: value["proxy_username"]
                    .as_str()
                    .or_else(|| value["proxyUsername"].as_str())
                    .unwrap_or("")
                    .to_string(),
                proxy_password: value["proxy_password"]
                    .as_str()
                    .or_else(|| value["proxyPassword"].as_str())
                    .unwrap_or("")
                    .to_string(),
                request_timeout: value["request_timeout"]
                    .as_u64()
                    .or_else(|| value["requestTimeout"].as_u64())
                    .unwrap_or(30) as u32,
            })
        })
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(settings)
}

// 保存设置
pub fn save_settings(settings: &crate::Settings) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join(SETTINGS_FILE);

    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_file, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}
