#![allow(dead_code)]

use chrono::{DateTime, Local};

// 格式化时间戳
pub fn format_timestamp(timestamp: i64) -> String {
    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Local::now().into());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

// 检查字符串是否为有效的IP地址
pub fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    
    if parts.len() != 4 {
        return false;
    }
    
    for part in parts {
        // parse::<u8>() 已经保证了 0-255 范围，解析失败即非法
        if part.parse::<u8>().is_err() {
            return false;
        }
    }
    
    true
}

// 检查字符串是否为有效的端口
pub fn is_valid_port(port: &str) -> bool {
    // parse::<u16>() 已经保证了 0-65535 范围，只需排除 0
    if let Ok(num) = port.parse::<u16>() {
        num > 0
    } else {
        false
    }
}

// 检查字符串是否为有效的域名
pub fn is_valid_domain(domain: &str) -> bool {
    // 简单检查，实际应用中可能需要更复杂的验证
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }
    
    let parts: Vec<&str> = domain.split('.').collect();
    
    if parts.len() < 2 {
        return false;
    }
    
    for part in parts {
        if part.is_empty() || part.len() > 63 {
            return false;
        }
        
        if !part.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return false;
        }
        
        if part.starts_with('-') || part.ends_with('-') {
            return false;
        }
    }
    
    true
}

// 检查字符串是否为有效的URL
pub fn is_valid_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    
    url.starts_with("http://") || url.starts_with("https://")
}

// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
} 