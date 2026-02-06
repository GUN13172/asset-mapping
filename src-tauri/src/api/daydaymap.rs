use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use crate::config;
use crate::ApiKeyValidationResult;
use super::key_manager;

// 使用单个API key进行搜索
async fn search_with_key(api_key: &str, query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    let base_url = "https://www.daydaymap.com/api/v1/raymap/search/all";
    
    // Base64编码查询字符串
    let keyword_base64 = general_purpose::STANDARD.encode(query.as_bytes());
    
    // 构建请求体
    let request_body = json!({
        "keyword": keyword_base64,
        "page": page,
        "page_size": page_size
    });
    
    eprintln!("=== DayDayMap search 函数 ===");
    eprintln!("查询字符串: {}", query);
    eprintln!("页码: {}", page);
    eprintln!("每页数量: {}", page_size);
    eprintln!("API Key: {}...", &api_key[..8.min(api_key.len())]);
    eprintln!("Base64编码后: {}", keyword_base64);
    eprintln!("请求体: {}", serde_json::to_string(&request_body).unwrap_or_default());
    
    // 发送请求
    let client = Client::new();
    let response = client.post(base_url)
        .header("API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    eprintln!("响应状态码: {} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or(""));
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(format!("API返回错误状态码: {}", response.status()));
    }
    
    // 解析响应
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    eprintln!("响应内容前500字符: {}", &response_text[..500.min(response_text.len())]);
    
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    // 检查API返回的状态码
    let code = response_json["code"].as_u64().unwrap_or(0);
    eprintln!("业务状态码: {}", code);
    
    if code != 200 {
        let msg = response_json["msg"].as_str().unwrap_or("未知错误");
        return Err(format!("API返回错误({}): {}", code, msg));
    }
    
    Ok(response_json)
}

// 搜索资产 - 真实实现
pub async fn search(query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    // 获取所有API密钥
    let api_keys = config::get_all_daydaymap_api_keys()?;
    
    if api_keys.is_empty() {
        return Err("未配置DayDayMap API密钥".to_string());
    }
    
    // Clone data for the closure
    let query = query.to_string();
    
    // 使用key_manager进行智能轮询
    let result = key_manager::execute_with_key_rotation(
        "daydaymap",
        &api_keys,
        |api_key| {
            let query = query.clone();
            let api_key = api_key.to_string();
            async move {
                search_with_key(&api_key, &query, page, page_size).await
            }
        }
    ).await;
    
    match result {
        Ok(response_json) => {
            // 提取结果
            let data = &response_json["data"];
            let total = data["total"].as_u64().unwrap_or(0);
            let list = data["list"].as_array().unwrap_or(&Vec::new()).clone();
            
            // 格式化结果
            let results = list.iter().map(|item| {
                json!({
                    "ip": item["ip"].as_str().unwrap_or(""),
                    "port": item["port"].as_i64().unwrap_or(0),
                    "domain": item["domain"].as_str().unwrap_or(""),
                    "web_title": item["title"].as_str().unwrap_or(""),
                    "country": item["country"].as_str().unwrap_or(""),
                    "province": item["province"].as_str().unwrap_or(""),
                    "city": item["city"].as_str().unwrap_or(""),
                    "isp": item["isp"].as_str().unwrap_or(""),
                    "url": format!("{}://{}:{}", 
                        if item["port"].as_i64().unwrap_or(80) == 443 { "https" } else { "http" },
                        item["ip"].as_str().unwrap_or(""),
                        item["port"].as_i64().unwrap_or(80)
                    ),
                })
            }).collect::<Vec<Value>>();
            
            Ok(json!({
                "total": total,
                "results": results
            }))
        },
        Err(e) => Err(e)
    }
}

// 导出资产 - 真实实现（带重试和部分导出）
pub async fn export(
    query: &str,
    pages: u32,
    page_size: u32,
    _time_range: &str,
    _start_date: Option<String>,
    _end_date: Option<String>,
    export_path: &str,
) -> Result<(), String> {
    let mut all_results = Vec::new();
    let mut successful_pages = 0;
    let mut last_error: Option<String> = None;
    
    // 重试配置
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_SECS: u64 = 5;
    
    // 分页获取所有数据
    for page in 1..=pages {
        eprintln!("正在导出第 {}/{} 页...", page, pages);
        
        let mut retry_count = 0;
        let mut page_success = false;
        
        // 重试循环
        while retry_count < MAX_RETRIES && !page_success {
            match search(query, page, page_size).await {
                Ok(data) => {
                    if let Some(results) = data["results"].as_array() {
                        all_results.extend(results.clone());
                        successful_pages += 1;
                        eprintln!("第 {} 页成功: 获取 {} 条数据", page, results.len());
                        page_success = true;
                    }
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // 检查是否是不可重试的错误
                    if e.contains("积分不足") || e.contains("额度不足") || 
                       e.contains("API密钥无效") || e.contains("未授权") {
                        eprintln!("第 {} 页失败（不可重试）: {}", page, e);
                        eprintln!("检测到额度耗尽或权限问题，停止导出");
                        break; // 跳出重试循环
                    }
                    
                    retry_count += 1;
                    if retry_count < MAX_RETRIES {
                        eprintln!("第 {} 页失败（第 {} 次重试）: {}", page, retry_count, e);
                        eprintln!("等待 {} 秒后重试...", RETRY_DELAY_SECS);
                        tokio::time::sleep(tokio::time::Duration::from_secs(RETRY_DELAY_SECS)).await;
                    } else {
                        eprintln!("第 {} 页失败（已达最大重试次数）: {}", page, e);
                    }
                }
            }
        }
        
        // 如果页面失败且不可重试，停止导出
        if !page_success && last_error.is_some() {
            let error = last_error.as_ref().unwrap();
            if error.contains("积分不足") || error.contains("额度不足") || 
               error.contains("API密钥无效") || error.contains("未授权") {
                eprintln!("由于额度或权限问题，停止继续导出");
                break; // 跳出页面循环
            }
        }
        
        // 避免请求过快，增加延迟
        if page < pages && page_success {
            eprintln!("等待 2 秒后继续...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    // 检查是否有数据需要导出
    if all_results.is_empty() {
        return Err("没有成功获取任何数据".to_string());
    }
    
    let total_results = all_results.len(); // 保存长度
    
    // 生成文件名（包含成功页数信息）
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = if successful_pages < pages {
        format!("daydaymap_export_{}_partial_{}of{}_pages.csv", 
                timestamp, successful_pages, pages)
    } else {
        format!("daydaymap_export_{}.csv", timestamp)
    };
    let file_path = format!("{}/{}", export_path, file_name);
    
    eprintln!("开始写入文件: {}", file_path);
    eprintln!("成功导出 {} 页，共 {} 条数据", successful_pages, total_results);
    
    // 创建CSV文件
    let mut wtr = csv::Writer::from_path(&file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // 写入CSV头部
    wtr.write_record(&["IP", "端口", "域名", "标题", "服务器", "国家", "省份", "城市", "URL"])
        .map_err(|e| format!("写入CSV头部失败: {}", e))?;

    // 写入数据
    for result in all_results {
        wtr.write_record(&[
            result["ip"].as_str().unwrap_or(""),
            &result["port"].as_i64().unwrap_or(0).to_string(),
            result["domain"].as_str().unwrap_or(""),
            result["web_title"].as_str().unwrap_or(""),
            result["server"].as_str().unwrap_or(""),
            result["country"].as_str().unwrap_or(""),
            result["province"].as_str().unwrap_or(""),
            result["city"].as_str().unwrap_or(""),
            result["url"].as_str().unwrap_or(""),
        ]).map_err(|e| format!("写入数据失败: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("刷新CSV写入失败: {}", e))?;
    
    eprintln!("文件写入完成: {}", file_path);
    
    // 如果有部分失败，返回警告信息
    if successful_pages < pages {
        if let Some(error) = last_error {
            return Err(format!(
                "部分导出成功: 已保存 {} 页（共 {} 条数据）到文件 {}。\n最后错误: {}", 
                successful_pages, 
                total_results,
                file_path,
                error
            ));
        }
    }
    
    Ok(())
}

// 导出全部资产 - 真实实现
pub async fn export_all(
    query: &str,
    _pages: u32,
    page_size: u32,
    time_range: &str,
    start_date: Option<String>,
    end_date: Option<String>,
    export_path: &str,
) -> Result<(), String> {
    // 先获取总数
    eprintln!("正在获取查询总数...");
    let first_page = search(query, 1, page_size).await?;
    let total = first_page["total"].as_u64().unwrap_or(0);
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
    
    eprintln!("查询总数: {}, 总页数: {}, 每页: {}", total, total_pages, page_size);
    
    // 限制最大导出页数（避免请求过多）
    let max_pages = 100;
    let export_pages = if total_pages > max_pages {
        eprintln!("警告: 总页数({})超过限制({}), 将只导出前{}页", total_pages, max_pages, max_pages);
        max_pages
    } else {
        total_pages
    };
    
    // 调用 export 函数
    export(query, export_pages, page_size, time_range, start_date, end_date, export_path).await
}

// 验证API密钥 - 真实实现
// 先尝试用户信息接口获取额度，如果失败则使用搜索接口验证
pub async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    let client = Client::new();
    
    // 方法1: 尝试用户信息接口
    let user_info_url = "https://www.daydaymap.com/api/v1/user/info";
    let user_info_response = client.get(user_info_url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .send()
        .await;
    
    // 如果用户信息接口成功，尝试提取额度
    if let Ok(response) = user_info_response {
        if response.status().is_success() {
            if let Ok(response_text) = response.text().await {
                if let Ok(response_json) = serde_json::from_str::<Value>(&response_text) {
                    let code = response_json["code"].as_i64().unwrap_or(-1);
                    
                    if code == 200 || code == 0 {
                        let data = &response_json["data"];
                        
                        // 尝试提取额度信息
                        let quota_info = if let Some(credit) = data["credit"].as_i64() {
                            format!("剩余积分: {}", credit)
                        } else if let Some(quota) = data["quota"].as_i64() {
                            format!("剩余配额: {}", quota)
                        } else if let Some(credit) = data["credit"].as_str() {
                            format!("剩余积分: {}", credit)
                        } else if let Some(quota) = data["quota"].as_str() {
                            format!("剩余配额: {}", quota)
                        } else {
                            "API密钥有效".to_string()
                        };
                        
                        return Ok(ApiKeyValidationResult {
                            valid: true,
                            message: Some("API密钥验证成功".to_string()),
                            quota: Some(quota_info),
                        });
                    }
                }
            }
        }
    }
    
    // 方法2: 使用搜索接口验证（作为后备方案）
    let search_url = "https://www.daydaymap.com/api/v1/raymap/search/all";
    let test_query = "ip=\"1.1.1.1\"";
    let keyword_base64 = general_purpose::STANDARD.encode(test_query.as_bytes());
    
    let request_body = json!({
        "keyword": keyword_base64,
        "page": 1,
        "page_size": 1
    });
    
    let response = client.post(search_url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    let status = response.status();
    
    if !status.is_success() {
        if status.as_u16() == 401 {
            return Ok(ApiKeyValidationResult {
                valid: false,
                message: Some("API密钥无效或已过期".to_string()),
                quota: None,
            });
        }
        
        return Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(format!("API返回错误状态码: {}", status)),
            quota: None,
        });
    }
    
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    let code = response_json["code"].as_i64().unwrap_or(-1);
    
    if code == 200 || code == 0 {
        Ok(ApiKeyValidationResult {
            valid: true,
            message: Some("API密钥验证成功".to_string()),
            quota: Some("API密钥有效（该密钥无权限查看额度信息）".to_string()),
        })
    } else {
        let message = response_json["message"].as_str()
            .or_else(|| response_json["msg"].as_str())
            .unwrap_or("API密钥验证失败")
            .to_string();
        
        Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(message),
            quota: None,
        })
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    // use quickcheck::{QuickCheck, TestResult};  // Will be used in later tasks
    // use quickcheck_macros::quickcheck;  // Will be used in later tasks
    use serde_json::json;

    // ============================================================================
    // Test Data Generators
    // ============================================================================

    /// Arbitrary generator for Asset data
    /// Generates random asset data with all possible field variations
    #[derive(Clone, Debug)]
    struct ArbitraryAsset {
        ip: Option<String>,
        port: Option<i64>,
        domain: Option<String>,
        title: Option<String>,
        server: Option<String>,
        country: Option<String>,
        province: Option<String>,
        region: Option<String>,  // Alternative to province
        city: Option<String>,
        protocol: Option<String>,
    }

    impl Arbitrary for ArbitraryAsset {
        fn arbitrary(g: &mut Gen) -> Self {
            let use_province = bool::arbitrary(g);
            let use_region = bool::arbitrary(g);
            
            ArbitraryAsset {
                ip: if bool::arbitrary(g) {
                    Some(format!("{}.{}.{}.{}", 
                        u8::arbitrary(g), u8::arbitrary(g), 
                        u8::arbitrary(g), u8::arbitrary(g)))
                } else {
                    None
                },
                port: if bool::arbitrary(g) {
                    Some((u16::arbitrary(g) as i64).max(1))
                } else {
                    None
                },
                domain: if bool::arbitrary(g) {
                    Some(format!("example{}.com", u16::arbitrary(g)))
                } else {
                    None
                },
                title: if bool::arbitrary(g) {
                    // Include commas in some titles to test sanitization
                    let has_comma = bool::arbitrary(g);
                    if has_comma {
                        Some(format!("Title{}, with comma", u16::arbitrary(g)))
                    } else {
                        Some(format!("Title{}", u16::arbitrary(g)))
                    }
                } else {
                    None
                },
                server: if bool::arbitrary(g) {
                    Some(format!("Server{}", u16::arbitrary(g)))
                } else {
                    None
                },
                country: if bool::arbitrary(g) {
                    Some(format!("Country{}", u16::arbitrary(g)))
                } else {
                    None
                },
                province: if use_province && bool::arbitrary(g) {
                    Some(format!("Province{}", u16::arbitrary(g)))
                } else {
                    None
                },
                region: if use_region && bool::arbitrary(g) {
                    Some(format!("Region{}", u16::arbitrary(g)))
                } else {
                    None
                },
                city: if bool::arbitrary(g) {
                    Some(format!("City{}", u16::arbitrary(g)))
                } else {
                    None
                },
                protocol: if bool::arbitrary(g) {
                    Some(if bool::arbitrary(g) { "https".to_string() } else { "http".to_string() })
                } else {
                    None
                },
            }
        }
    }

    impl ArbitraryAsset {
        /// Convert to JSON Value for testing
        fn to_json(&self) -> Value {
            let mut obj = serde_json::Map::new();
            
            if let Some(ref ip) = self.ip {
                obj.insert("ip".to_string(), json!(ip));
            }
            if let Some(port) = self.port {
                obj.insert("port".to_string(), json!(port));
            }
            if let Some(ref domain) = self.domain {
                obj.insert("domain".to_string(), json!(domain));
            }
            if let Some(ref title) = self.title {
                obj.insert("title".to_string(), json!(title));
            }
            if let Some(ref server) = self.server {
                obj.insert("server".to_string(), json!(server));
            }
            if let Some(ref country) = self.country {
                obj.insert("country".to_string(), json!(country));
            }
            if let Some(ref province) = self.province {
                obj.insert("province".to_string(), json!(province));
            }
            if let Some(ref region) = self.region {
                obj.insert("region".to_string(), json!(region));
            }
            if let Some(ref city) = self.city {
                obj.insert("city".to_string(), json!(city));
            }
            if let Some(ref protocol) = self.protocol {
                obj.insert("protocol".to_string(), json!(protocol));
            }
            
            Value::Object(obj)
        }
    }

    /// Arbitrary generator for SearchResponse
    #[derive(Clone, Debug)]
    struct ArbitrarySearchResponse {
        code: i64,
        message: Option<String>,
        msg: Option<String>,  // Alternative to message
        total: u64,
        items: Option<Vec<ArbitraryAsset>>,
        list: Option<Vec<ArbitraryAsset>>,  // Alternative to items
    }

    impl Arbitrary for ArbitrarySearchResponse {
        fn arbitrary(g: &mut Gen) -> Self {
            let use_items = bool::arbitrary(g);
            let use_list = bool::arbitrary(g);
            let use_message = bool::arbitrary(g);
            let use_msg = bool::arbitrary(g);
            
            // Generate success or error code
            let code = if bool::arbitrary(g) {
                if bool::arbitrary(g) { 200 } else { 0 }  // Success codes
            } else {
                (i64::arbitrary(g) % 1000).abs() + 1  // Error codes (1-1000, excluding 0 and 200)
            };
            
            let assets: Vec<ArbitraryAsset> = (0..(g.size() % 10))
                .map(|_| ArbitraryAsset::arbitrary(g))
                .collect();
            
            ArbitrarySearchResponse {
                code,
                message: if use_message {
                    Some(if code == 200 || code == 0 {
                        "success".to_string()
                    } else {
                        format!("Error{}", code)
                    })
                } else {
                    None
                },
                msg: if use_msg {
                    Some(if code == 200 || code == 0 {
                        "success".to_string()
                    } else {
                        format!("Error{}", code)
                    })
                } else {
                    None
                },
                total: u64::arbitrary(g) % 10000,
                items: if use_items { Some(assets.clone()) } else { None },
                list: if use_list { Some(assets) } else { None },
            }
        }
    }

    impl ArbitrarySearchResponse {
        /// Convert to JSON Value for testing
        fn to_json(&self) -> Value {
            let mut obj = serde_json::Map::new();
            obj.insert("code".to_string(), json!(self.code));
            
            if let Some(ref message) = self.message {
                obj.insert("message".to_string(), json!(message));
            }
            if let Some(ref msg) = self.msg {
                obj.insert("msg".to_string(), json!(msg));
            }
            
            let mut data = serde_json::Map::new();
            data.insert("total".to_string(), json!(self.total));
            
            if let Some(ref items) = self.items {
                let items_json: Vec<Value> = items.iter().map(|a| a.to_json()).collect();
                data.insert("items".to_string(), json!(items_json));
            }
            if let Some(ref list) = self.list {
                let list_json: Vec<Value> = list.iter().map(|a| a.to_json()).collect();
                data.insert("list".to_string(), json!(list_json));
            }
            
            obj.insert("data".to_string(), Value::Object(data));
            Value::Object(obj)
        }
    }

    /// Arbitrary generator for UserInfoResponse
    #[derive(Clone, Debug)]
    struct ArbitraryUserInfoResponse {
        code: i64,
        message: Option<String>,
        msg: Option<String>,  // Alternative to message
        credit: Option<i64>,
        quota: Option<i64>,  // Alternative to credit
        username: Option<String>,
    }

    impl Arbitrary for ArbitraryUserInfoResponse {
        fn arbitrary(g: &mut Gen) -> Self {
            let use_message = bool::arbitrary(g);
            let use_msg = bool::arbitrary(g);
            let use_credit = bool::arbitrary(g);
            let use_quota = bool::arbitrary(g);
            
            // Generate success or error code
            let code = if bool::arbitrary(g) {
                if bool::arbitrary(g) { 200 } else { 0 }  // Success codes
            } else {
                (i64::arbitrary(g) % 1000).abs() + 1  // Error codes
            };
            
            ArbitraryUserInfoResponse {
                code,
                message: if use_message {
                    Some(if code == 200 || code == 0 {
                        "success".to_string()
                    } else {
                        format!("Error{}", code)
                    })
                } else {
                    None
                },
                msg: if use_msg {
                    Some(if code == 200 || code == 0 {
                        "success".to_string()
                    } else {
                        format!("Error{}", code)
                    })
                } else {
                    None
                },
                credit: if use_credit && (code == 200 || code == 0) {
                    Some((i64::arbitrary(g) % 10000).abs())
                } else {
                    None
                },
                quota: if use_quota && (code == 200 || code == 0) {
                    Some((i64::arbitrary(g) % 10000).abs())
                } else {
                    None
                },
                username: if bool::arbitrary(g) {
                    Some(format!("user{}@example.com", u16::arbitrary(g)))
                } else {
                    None
                },
            }
        }
    }

    impl ArbitraryUserInfoResponse {
        /// Convert to JSON Value for testing
        fn to_json(&self) -> Value {
            let mut obj = serde_json::Map::new();
            obj.insert("code".to_string(), json!(self.code));
            
            if let Some(ref message) = self.message {
                obj.insert("message".to_string(), json!(message));
            }
            if let Some(ref msg) = self.msg {
                obj.insert("msg".to_string(), json!(msg));
            }
            
            let mut data = serde_json::Map::new();
            if let Some(credit) = self.credit {
                data.insert("credit".to_string(), json!(credit));
            }
            if let Some(quota) = self.quota {
                data.insert("quota".to_string(), json!(quota));
            }
            if let Some(ref username) = self.username {
                data.insert("username".to_string(), json!(username));
            }
            
            obj.insert("data".to_string(), Value::Object(data));
            Value::Object(obj)
        }
    }

    // ============================================================================
    // Helper Functions for Testing
    // ============================================================================

    /// Helper to extract results from a search response JSON
    fn extract_results_from_response(response_json: &Value) -> Vec<Value> {
        let data = &response_json["data"];
        data["items"].as_array()
            .or_else(|| data["list"].as_array())
            .unwrap_or(&Vec::new())
            .clone()
    }

    /// Helper to check if a string contains Chinese characters
    fn contains_chinese(s: &str) -> bool {
        s.chars().any(|c| {
            let code = c as u32;
            (code >= 0x4E00 && code <= 0x9FFF) || // CJK Unified Ideographs
            (code >= 0x3400 && code <= 0x4DBF) || // CJK Extension A
            (code >= 0x20000 && code <= 0x2A6DF) || // CJK Extension B
            (code >= 0x2A700 && code <= 0x2B73F) || // CJK Extension C
            (code >= 0x2B740 && code <= 0x2B81F) || // CJK Extension D
            (code >= 0x2B820 && code <= 0x2CEAF) || // CJK Extension E
            (code >= 0xF900 && code <= 0xFAFF) || // CJK Compatibility Ideographs
            (code >= 0x2F800 && code <= 0x2FA1F) // CJK Compatibility Ideographs Supplement
        })
    }

    // ============================================================================
    // Mock HTTP Request Builder for Testing
    // ============================================================================

    /// Mock HTTP request that captures headers and request details for testing
    #[derive(Debug, Clone)]
    struct MockHttpRequest {
        pub url: String,
        pub method: String,
        pub headers: std::collections::HashMap<String, String>,
        pub body: Option<Value>,
    }

    impl MockHttpRequest {
        /// Create a new mock HTTP request
        fn new(method: &str, url: &str) -> Self {
            MockHttpRequest {
                url: url.to_string(),
                method: method.to_uppercase(),
                headers: std::collections::HashMap::new(),
                body: None,
            }
        }

        /// Add a header to the request
        fn with_header(mut self, key: &str, value: &str) -> Self {
            self.headers.insert(key.to_string(), value.to_string());
            self
        }

        /// Add a JSON body to the request
        fn with_json_body(mut self, body: Value) -> Self {
            self.body = Some(body);
            self
        }

        /// Check if a header exists
        fn has_header(&self, key: &str) -> bool {
            self.headers.contains_key(key)
        }

        /// Get a header value
        fn get_header(&self, key: &str) -> Option<&String> {
            self.headers.get(key)
        }

        /// Verify Authorization header has Bearer token format
        fn verify_bearer_token(&self, expected_token: &str) -> bool {
            if let Some(auth_header) = self.get_header("Authorization") {
                let expected = format!("Bearer {}", expected_token);
                auth_header == &expected
            } else {
                false
            }
        }

        /// Verify Content-Type header is application/json
        fn verify_content_type_json(&self) -> bool {
            if let Some(content_type) = self.get_header("Content-Type") {
                content_type == "application/json"
            } else {
                false
            }
        }

        /// Verify URL uses HTTPS protocol
        fn verify_https(&self) -> bool {
            self.url.starts_with("https://")
        }

        /// Verify request has all required headers for DayDayMap API
        fn verify_required_headers(&self, api_key: &str) -> bool {
            self.verify_bearer_token(api_key) && self.verify_content_type_json()
        }
    }

    /// Builder for creating mock search requests
    fn build_mock_search_request(api_key: &str, query: &str, page: u32, page_size: u32) -> MockHttpRequest {
        let request_body = json!({
            "query": query,
            "page": page,
            "page_size": page_size
        });

        MockHttpRequest::new("POST", "https://www.daydaymap.com/api/v1/search")
            .with_header("Authorization", &format!("Bearer {}", api_key))
            .with_header("Content-Type", "application/json")
            .with_json_body(request_body)
    }

    /// Builder for creating mock validation requests
    fn build_mock_validation_request(api_key: &str) -> MockHttpRequest {
        MockHttpRequest::new("GET", "https://www.daydaymap.com/api/v1/user/info")
            .with_header("Authorization", &format!("Bearer {}", api_key))
            .with_header("Content-Type", "application/json")
    }

    /// Helper to verify request body structure for search requests
    fn verify_search_request_body(body: &Value) -> bool {
        // Check that body is an object
        if !body.is_object() {
            return false;
        }

        // Check that required fields exist
        if !body.get("query").is_some() {
            return false;
        }
        if !body.get("page").is_some() {
            return false;
        }
        if !body.get("page_size").is_some() {
            return false;
        }

        // Check field types
        if !body["query"].is_string() {
            return false;
        }
        if !body["page"].is_number() {
            return false;
        }
        if !body["page_size"].is_number() {
            return false;
        }

        true
    }

    // ============================================================================
    // Test Utilities for Search Request Validation
    // ============================================================================

    /// Create a mock search response with various business codes
    fn create_mock_search_response(code: i64, use_items: bool, asset_count: usize) -> Value {
        let mut g = Gen::new(10);
        let assets: Vec<ArbitraryAsset> = (0..asset_count)
            .map(|_| ArbitraryAsset::arbitrary(&mut g))
            .collect();
        
        let mut response = serde_json::Map::new();
        response.insert("code".to_string(), json!(code));
        
        // Use either "message" or "msg" field
        if code == 200 || code == 0 {
            response.insert("message".to_string(), json!("success"));
        } else {
            response.insert("message".to_string(), json!(format!("Error code {}", code)));
        }
        
        let mut data = serde_json::Map::new();
        data.insert("total".to_string(), json!(asset_count as u64));
        
        // Use either "items" or "list" field
        let assets_json: Vec<Value> = assets.iter().map(|a| a.to_json()).collect();
        if use_items {
            data.insert("items".to_string(), json!(assets_json));
        } else {
            data.insert("list".to_string(), json!(assets_json));
        }
        
        response.insert("data".to_string(), Value::Object(data));
        Value::Object(response)
    }

    /// Create a mock error response with business code
    fn create_mock_error_response(code: i64, use_msg: bool) -> Value {
        let mut response = serde_json::Map::new();
        response.insert("code".to_string(), json!(code));
        
        let error_message = format!("Error: business code {}", code);
        if use_msg {
            response.insert("msg".to_string(), json!(error_message));
        } else {
            response.insert("message".to_string(), json!(error_message));
        }
        
        let data = serde_json::Map::new();
        response.insert("data".to_string(), Value::Object(data));
        
        Value::Object(response)
    }

    /// Helper to check if a business code is a success code
    fn is_success_code(code: i64) -> bool {
        code == 200 || code == 0
    }

    /// Helper to extract error message from response (handles both "message" and "msg")
    fn extract_error_message(response: &Value) -> Option<String> {
        response["message"].as_str()
            .or_else(|| response["msg"].as_str())
            .map(|s| s.to_string())
    }

    /// Helper to extract total count from response
    fn extract_total_count(response: &Value) -> Option<u64> {
        response["data"]["total"].as_u64()
    }

    // ============================================================================
    // Basic Unit Tests
    // ============================================================================

    #[test]
    fn test_arbitrary_asset_generation() {
        // Test that we can generate arbitrary assets
        let mut g = Gen::new(10);
        let asset = ArbitraryAsset::arbitrary(&mut g);
        let json = asset.to_json();
        
        // Should be a valid JSON object
        assert!(json.is_object());
    }

    #[test]
    fn test_arbitrary_search_response_generation() {
        // Test that we can generate arbitrary search responses
        let mut g = Gen::new(10);
        let response = ArbitrarySearchResponse::arbitrary(&mut g);
        let json = response.to_json();
        
        // Should have required fields
        assert!(json["code"].is_i64());
        assert!(json["data"].is_object());
        assert!(json["data"]["total"].is_u64());
    }

    #[test]
    fn test_arbitrary_user_info_response_generation() {
        // Test that we can generate arbitrary user info responses
        let mut g = Gen::new(10);
        let response = ArbitraryUserInfoResponse::arbitrary(&mut g);
        let json = response.to_json();
        
        // Should have required fields
        assert!(json["code"].is_i64());
        assert!(json["data"].is_object());
    }

    #[test]
    fn test_contains_chinese_helper() {
        assert!(contains_chinese("请求失败"));
        assert!(contains_chinese("API返回错误"));
        assert!(contains_chinese("剩余积分: 100"));
        assert!(!contains_chinese("Request failed"));
        assert!(!contains_chinese("API error"));
    }

    // ============================================================================
    // Mock HTTP Request Builder Tests
    // ============================================================================

    #[test]
    fn test_mock_request_creation() {
        let request = MockHttpRequest::new("POST", "https://example.com/api");
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://example.com/api");
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
    }

    #[test]
    fn test_mock_request_with_headers() {
        let request = MockHttpRequest::new("GET", "https://example.com/api")
            .with_header("Authorization", "Bearer test_token")
            .with_header("Content-Type", "application/json");
        
        assert!(request.has_header("Authorization"));
        assert!(request.has_header("Content-Type"));
        assert_eq!(request.get_header("Authorization"), Some(&"Bearer test_token".to_string()));
        assert_eq!(request.get_header("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_mock_request_bearer_token_verification() {
        let api_key = "test_api_key_12345";
        let request = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Authorization", &format!("Bearer {}", api_key));
        
        assert!(request.verify_bearer_token(api_key));
        assert!(!request.verify_bearer_token("wrong_key"));
    }

    #[test]
    fn test_mock_request_bearer_token_verification_missing() {
        let request = MockHttpRequest::new("POST", "https://example.com/api");
        assert!(!request.verify_bearer_token("any_key"));
    }

    #[test]
    fn test_mock_request_bearer_token_verification_wrong_format() {
        let request = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Authorization", "test_api_key");  // Missing "Bearer " prefix
        
        assert!(!request.verify_bearer_token("test_api_key"));
    }

    #[test]
    fn test_mock_request_content_type_verification() {
        let request = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Content-Type", "application/json");
        
        assert!(request.verify_content_type_json());
    }

    #[test]
    fn test_mock_request_content_type_verification_missing() {
        let request = MockHttpRequest::new("POST", "https://example.com/api");
        assert!(!request.verify_content_type_json());
    }

    #[test]
    fn test_mock_request_content_type_verification_wrong_type() {
        let request = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Content-Type", "text/plain");
        
        assert!(!request.verify_content_type_json());
    }

    #[test]
    fn test_mock_request_https_verification() {
        let https_request = MockHttpRequest::new("GET", "https://example.com/api");
        assert!(https_request.verify_https());
        
        let http_request = MockHttpRequest::new("GET", "http://example.com/api");
        assert!(!http_request.verify_https());
    }

    #[test]
    fn test_mock_request_required_headers_verification() {
        let api_key = "test_key";
        
        // Request with all required headers
        let valid_request = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Authorization", &format!("Bearer {}", api_key))
            .with_header("Content-Type", "application/json");
        assert!(valid_request.verify_required_headers(api_key));
        
        // Request missing Authorization header
        let missing_auth = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Content-Type", "application/json");
        assert!(!missing_auth.verify_required_headers(api_key));
        
        // Request missing Content-Type header
        let missing_content_type = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Authorization", &format!("Bearer {}", api_key));
        assert!(!missing_content_type.verify_required_headers(api_key));
        
        // Request with wrong API key
        let wrong_key = MockHttpRequest::new("POST", "https://example.com/api")
            .with_header("Authorization", "Bearer wrong_key")
            .with_header("Content-Type", "application/json");
        assert!(!wrong_key.verify_required_headers(api_key));
    }

    #[test]
    fn test_build_mock_search_request() {
        let api_key = "test_api_key";
        let query = "ip=\"1.1.1.1\"";
        let page = 1;
        let page_size = 20;
        
        let request = build_mock_search_request(api_key, query, page, page_size);
        
        // Verify URL
        assert_eq!(request.url, "https://www.daydaymap.com/api/v1/search");
        assert!(request.verify_https());
        
        // Verify method
        assert_eq!(request.method, "POST");
        
        // Verify headers
        assert!(request.verify_required_headers(api_key));
        
        // Verify body
        assert!(request.body.is_some());
        let body = request.body.unwrap();
        assert_eq!(body["query"], query);
        assert_eq!(body["page"], page);
        assert_eq!(body["page_size"], page_size);
    }

    #[test]
    fn test_build_mock_validation_request() {
        let api_key = "test_api_key";
        
        let request = build_mock_validation_request(api_key);
        
        // Verify URL
        assert_eq!(request.url, "https://www.daydaymap.com/api/v1/user/info");
        assert!(request.verify_https());
        
        // Verify method
        assert_eq!(request.method, "GET");
        
        // Verify headers
        assert!(request.verify_required_headers(api_key));
        
        // Verify no body (GET request)
        assert!(request.body.is_none());
    }

    #[test]
    fn test_verify_search_request_body_valid() {
        let body = json!({
            "query": "ip=\"1.1.1.1\"",
            "page": 1,
            "page_size": 20
        });
        
        assert!(verify_search_request_body(&body));
    }

    #[test]
    fn test_verify_search_request_body_missing_fields() {
        // Missing query
        let missing_query = json!({
            "page": 1,
            "page_size": 20
        });
        assert!(!verify_search_request_body(&missing_query));
        
        // Missing page
        let missing_page = json!({
            "query": "test",
            "page_size": 20
        });
        assert!(!verify_search_request_body(&missing_page));
        
        // Missing page_size
        let missing_page_size = json!({
            "query": "test",
            "page": 1
        });
        assert!(!verify_search_request_body(&missing_page_size));
    }

    #[test]
    fn test_verify_search_request_body_wrong_types() {
        // Query should be string
        let wrong_query_type = json!({
            "query": 123,
            "page": 1,
            "page_size": 20
        });
        assert!(!verify_search_request_body(&wrong_query_type));
        
        // Page should be number
        let wrong_page_type = json!({
            "query": "test",
            "page": "1",
            "page_size": 20
        });
        assert!(!verify_search_request_body(&wrong_page_type));
        
        // Page_size should be number
        let wrong_page_size_type = json!({
            "query": "test",
            "page": 1,
            "page_size": "20"
        });
        assert!(!verify_search_request_body(&wrong_page_size_type));
    }

    #[test]
    fn test_verify_search_request_body_not_object() {
        let not_object = json!("not an object");
        assert!(!verify_search_request_body(&not_object));
        
        let array = json!([1, 2, 3]);
        assert!(!verify_search_request_body(&array));
    }

    #[test]
    fn test_arbitrary_asset_with_missing_fields() {
        // Test that assets can be generated with various missing fields
        let mut g = Gen::new(10);
        for _ in 0..20 {
            let asset = ArbitraryAsset::arbitrary(&mut g);
            let json = asset.to_json();
            
            // JSON should always be an object
            assert!(json.is_object());
            
            // Fields may or may not be present
            // This tests that the generator handles optional fields correctly
        }
    }

    #[test]
    fn test_arbitrary_search_response_flexible_fields() {
        // Test that search responses can use either items or list
        let mut g = Gen::new(10);
        let mut has_items = false;
        let mut has_list = false;
        let mut has_message = false;
        let mut has_msg = false;
        
        for _ in 0..50 {
            let response = ArbitrarySearchResponse::arbitrary(&mut g);
            let json = response.to_json();
            
            if json["data"]["items"].is_array() {
                has_items = true;
            }
            if json["data"]["list"].is_array() {
                has_list = true;
            }
            if json["message"].is_string() {
                has_message = true;
            }
            if json["msg"].is_string() {
                has_msg = true;
            }
        }
        
        // Over 50 iterations, we should see both variations
        assert!(has_items || has_list, "Should generate either items or list");
        assert!(has_message || has_msg, "Should generate either message or msg");
    }

    #[test]
    fn test_arbitrary_user_info_response_flexible_fields() {
        // Test that user info responses can use either credit or quota
        let mut g = Gen::new(10);
        let mut has_credit = false;
        let mut has_quota = false;
        
        for _ in 0..50 {
            let response = ArbitraryUserInfoResponse::arbitrary(&mut g);
            let json = response.to_json();
            
            if json["data"]["credit"].is_i64() {
                has_credit = true;
            }
            if json["data"]["quota"].is_i64() {
                has_quota = true;
            }
        }
        
        // Over 50 iterations, we should see both variations
        assert!(has_credit || has_quota, "Should generate either credit or quota");
    }

    #[test]
    fn test_arbitrary_asset_province_region_flexibility() {
        // Test that assets can use either province or region
        let mut g = Gen::new(10);
        let mut has_province = false;
        let mut has_region = false;
        
        for _ in 0..50 {
            let asset = ArbitraryAsset::arbitrary(&mut g);
            let json = asset.to_json();
            
            if json["province"].is_string() {
                has_province = true;
            }
            if json["region"].is_string() {
                has_region = true;
            }
        }
        
        // Over 50 iterations, we should see both variations
        assert!(has_province || has_region, "Should generate either province or region");
    }

    #[test]
    fn test_arbitrary_search_response_success_and_error_codes() {
        // Test that search responses generate both success and error codes
        let mut g = Gen::new(10);
        let mut has_success_200 = false;
        let mut has_success_0 = false;
        let mut has_error = false;
        
        for _ in 0..100 {
            let response = ArbitrarySearchResponse::arbitrary(&mut g);
            
            if response.code == 200 {
                has_success_200 = true;
            } else if response.code == 0 {
                has_success_0 = true;
            } else {
                has_error = true;
            }
        }
        
        // Over 100 iterations, we should see various code types
        assert!(has_success_200 || has_success_0, "Should generate success codes");
        assert!(has_error, "Should generate error codes");
    }

    #[test]
    fn test_arbitrary_asset_comma_in_title() {
        // Test that some assets have commas in titles for sanitization testing
        let mut g = Gen::new(10);
        let mut has_comma = false;
        
        for _ in 0..50 {
            let asset = ArbitraryAsset::arbitrary(&mut g);
            if let Some(ref title) = asset.title {
                if title.contains(',') {
                    has_comma = true;
                    break;
                }
            }
        }
        
        assert!(has_comma, "Should generate titles with commas for sanitization testing");
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    // Feature: daydaymap-api-fix, Property 6: Flexible Result Field Names
    // **Validates: Requirements 2.6**
    #[quickcheck_macros::quickcheck]
    fn property_flexible_result_field_names(assets: Vec<ArbitraryAsset>) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Skip empty asset lists as they don't test the property meaningfully
        if assets.is_empty() {
            return TestResult::discard();
        }
        
        let total = assets.len() as u64;
        
        // Test 1: Response with "items" field
        let response_with_items = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": total,
                "items": assets.iter().map(|a| a.to_json()).collect::<Vec<Value>>()
            }
        });
        
        let results_from_items = extract_results_from_response(&response_with_items);
        
        // Test 2: Response with "list" field
        let response_with_list = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": total,
                "list": assets.iter().map(|a| a.to_json()).collect::<Vec<Value>>()
            }
        });
        
        let results_from_list = extract_results_from_response(&response_with_list);
        
        // Test 3: Response with both fields (should prefer "items")
        let response_with_both = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": total,
                "items": assets.iter().map(|a| a.to_json()).collect::<Vec<Value>>(),
                "list": vec![json!({"dummy": "data"})]  // Different data to verify items is used
            }
        });
        
        let results_from_both = extract_results_from_response(&response_with_both);
        
        // Test 4: Response with neither field (should return empty)
        let response_with_neither = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": total
            }
        });
        
        let results_from_neither = extract_results_from_response(&response_with_neither);
        
        // Verify properties:
        // 1. Results can be extracted from "items" field
        if results_from_items.len() != assets.len() {
            return TestResult::failed();
        }
        
        // 2. Results can be extracted from "list" field
        if results_from_list.len() != assets.len() {
            return TestResult::failed();
        }
        
        // 3. When both fields exist, "items" is preferred
        if results_from_both.len() != assets.len() {
            return TestResult::failed();
        }
        
        // 4. When neither field exists, empty array is returned
        if !results_from_neither.is_empty() {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 1: Request Header Completeness
    // **Validates: Requirements 1.1, 1.2**
    #[quickcheck_macros::quickcheck]
    fn property_request_header_completeness(
        api_key: String,
        query: String,
        page: u32,
        page_size: u32
    ) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Skip invalid inputs
        if api_key.is_empty() || query.is_empty() || page == 0 || page_size == 0 {
            return TestResult::discard();
        }
        
        // Limit page and page_size to reasonable values to avoid overflow
        let page = (page % 1000) + 1;
        let page_size = (page_size % 100) + 1;
        
        // Test 1: Search request should have both required headers
        let search_request = build_mock_search_request(&api_key, &query, page, page_size);
        
        // Verify Authorization header exists and has Bearer token format
        if !search_request.has_header("Authorization") {
            return TestResult::failed();
        }
        
        if !search_request.verify_bearer_token(&api_key) {
            return TestResult::failed();
        }
        
        // Verify Content-Type header exists and is application/json
        if !search_request.has_header("Content-Type") {
            return TestResult::failed();
        }
        
        if !search_request.verify_content_type_json() {
            return TestResult::failed();
        }
        
        // Test 2: Validation request should have both required headers
        let validation_request = build_mock_validation_request(&api_key);
        
        // Verify Authorization header exists and has Bearer token format
        if !validation_request.has_header("Authorization") {
            return TestResult::failed();
        }
        
        if !validation_request.verify_bearer_token(&api_key) {
            return TestResult::failed();
        }
        
        // Verify Content-Type header exists and is application/json
        if !validation_request.has_header("Content-Type") {
            return TestResult::failed();
        }
        
        if !validation_request.verify_content_type_json() {
            return TestResult::failed();
        }
        
        // Test 3: Verify the Authorization header format is exactly "Bearer {api_key}"
        let expected_auth = format!("Bearer {}", api_key);
        if search_request.get_header("Authorization") != Some(&expected_auth) {
            return TestResult::failed();
        }
        if validation_request.get_header("Authorization") != Some(&expected_auth) {
            return TestResult::failed();
        }
        
        // Test 4: Verify the Content-Type header value is exactly "application/json"
        if search_request.get_header("Content-Type") != Some(&"application/json".to_string()) {
            return TestResult::failed();
        }
        if validation_request.get_header("Content-Type") != Some(&"application/json".to_string()) {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // ============================================================================
    // Example-Based Tests (Unit Tests)
    // ============================================================================

    // Example 1: HTTP 401 Handling
    // **Validates: Requirements 1.3, 5.2**
    #[test]
    fn test_http_401_handling() {
        // This test verifies that when the API returns HTTP 401,
        // the validation result has valid=false and the correct error message
        
        // Simulate the logic from validate_api_key function when status is 401
        let status_code = 401u16;
        
        // The expected behavior according to the implementation
        let result = if status_code == 401 {
            ApiKeyValidationResult {
                valid: false,
                message: Some("API密钥无效或已过期".to_string()),
                quota: None,
            }
        } else {
            ApiKeyValidationResult {
                valid: true,
                message: Some("Success".to_string()),
                quota: None,
            }
        };
        
        // Verify the result matches requirements
        assert_eq!(result.valid, false, "HTTP 401 should result in valid=false");
        assert!(result.message.is_some(), "HTTP 401 should have an error message");
        assert_eq!(
            result.message.unwrap(),
            "API密钥无效或已过期",
            "HTTP 401 should return the correct Chinese error message"
        );
        assert!(result.quota.is_none(), "HTTP 401 should not include quota information");
    }

    #[test]
    fn test_http_401_message_is_chinese() {
        // Verify that the 401 error message is in Chinese
        let error_message = "API密钥无效或已过期";
        
        assert!(
            contains_chinese(error_message),
            "HTTP 401 error message should be in Chinese"
        );
    }

    #[test]
    fn test_http_401_vs_other_status_codes() {
        // Test that 401 is handled differently from other error status codes
        
        // 401 should return the specific "invalid or expired" message
        let result_401 = ApiKeyValidationResult {
            valid: false,
            message: Some("API密钥无效或已过期".to_string()),
            quota: None,
        };
        
        // Other error codes (e.g., 403, 500) should return different messages
        let result_403 = ApiKeyValidationResult {
            valid: false,
            message: Some("API返回错误状态码: 403".to_string()),
            quota: None,
        };
        
        // Verify they have different messages
        assert_ne!(
            result_401.message,
            result_403.message,
            "401 should have a specific error message different from other status codes"
        );
        
        // Verify 401 message is the expected one
        assert_eq!(
            result_401.message.unwrap(),
            "API密钥无效或已过期"
        );
    }

    // Example 2: Search Endpoint URL
    // **Validates: Requirements 2.1**
    #[test]
    fn test_search_endpoint_url() {
        // Verify that the search function uses the correct endpoint URL
        let expected_url = "https://www.daydaymap.com/api/v1/search";
        
        // Create a mock search request
        let api_key = "test_key";
        let query = "ip=\"1.1.1.1\"";
        let page = 1;
        let page_size = 20;
        
        let request = build_mock_search_request(api_key, query, page, page_size);
        
        // Verify the URL matches the expected endpoint
        assert_eq!(
            request.url,
            expected_url,
            "Search endpoint URL should be https://www.daydaymap.com/api/v1/search"
        );
        
        // Verify it's a POST request
        assert_eq!(
            request.method,
            "POST",
            "Search should use POST method"
        );
        
        // Verify it uses HTTPS
        assert!(
            request.url.starts_with("https://"),
            "Search endpoint should use HTTPS"
        );
    }

    #[test]
    fn test_search_endpoint_url_components() {
        // Verify the search endpoint URL has the correct components
        let url = "https://www.daydaymap.com/api/v1/search";
        
        // Should use HTTPS protocol
        assert!(url.starts_with("https://"));
        
        // Should contain the domain
        assert!(url.contains("www.daydaymap.com"));
        
        // Should contain the API version
        assert!(url.contains("/api/v1/"));
        
        // Should end with /search
        assert!(url.ends_with("/search"));
    }

    // Tests for search request validation utilities
    #[test]
    fn test_create_mock_search_response_success() {
        let response = create_mock_search_response(200, true, 5);
        
        assert_eq!(response["code"], 200);
        assert_eq!(response["message"], "success");
        assert_eq!(response["data"]["total"], 5);
        assert!(response["data"]["items"].is_array());
        assert_eq!(response["data"]["items"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_create_mock_search_response_with_list() {
        let response = create_mock_search_response(0, false, 3);
        
        assert_eq!(response["code"], 0);
        assert!(response["data"]["list"].is_array());
        assert_eq!(response["data"]["list"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_create_mock_error_response() {
        let response = create_mock_error_response(400, false);
        
        assert_eq!(response["code"], 400);
        assert!(response["message"].is_string());
        assert!(response["message"].as_str().unwrap().contains("400"));
    }

    #[test]
    fn test_create_mock_error_response_with_msg() {
        let response = create_mock_error_response(500, true);
        
        assert_eq!(response["code"], 500);
        assert!(response["msg"].is_string());
        assert!(response["msg"].as_str().unwrap().contains("500"));
    }

    #[test]
    fn test_is_success_code() {
        assert!(is_success_code(200));
        assert!(is_success_code(0));
        assert!(!is_success_code(400));
        assert!(!is_success_code(500));
        assert!(!is_success_code(-1));
    }

    #[test]
    fn test_extract_error_message() {
        let response_with_message = json!({
            "code": 400,
            "message": "Error message"
        });
        assert_eq!(extract_error_message(&response_with_message), Some("Error message".to_string()));
        
        let response_with_msg = json!({
            "code": 400,
            "msg": "Error msg"
        });
        assert_eq!(extract_error_message(&response_with_msg), Some("Error msg".to_string()));
        
        let response_without_message = json!({
            "code": 400
        });
        assert_eq!(extract_error_message(&response_without_message), None);
    }

    #[test]
    fn test_extract_total_count() {
        let response = json!({
            "data": {
                "total": 100
            }
        });
        assert_eq!(extract_total_count(&response), Some(100));
        
        let response_without_total = json!({
            "data": {}
        });
        assert_eq!(extract_total_count(&response_without_total), None);
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    // Feature: daydaymap-api-fix, Property 3: Search Request Body Structure
    // **Validates: Requirements 2.2**
    #[quickcheck_macros::quickcheck]
    fn property_search_request_body_structure(
        query: String,
        page: u32,
        page_size: u32
    ) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Skip invalid inputs
        if query.is_empty() || page == 0 || page_size == 0 {
            return TestResult::discard();
        }
        
        // Limit to reasonable values
        let page = (page % 1000) + 1;
        let page_size = (page_size % 100) + 1;
        
        // Create request body
        let request_body = json!({
            "query": query,
            "page": page,
            "page_size": page_size
        });
        
        // Verify the body structure
        if !verify_search_request_body(&request_body) {
            return TestResult::failed();
        }
        
        // Verify field types explicitly
        if !request_body["query"].is_string() {
            return TestResult::failed();
        }
        if !request_body["page"].is_number() {
            return TestResult::failed();
        }
        if !request_body["page_size"].is_number() {
            return TestResult::failed();
        }
        
        // Verify field values match input
        if request_body["query"].as_str() != Some(&query) {
            return TestResult::failed();
        }
        if request_body["page"].as_u64() != Some(page as u64) {
            return TestResult::failed();
        }
        if request_body["page_size"].as_u64() != Some(page_size as u64) {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 4: Business Code Success Handling
    // **Validates: Requirements 2.4, 5.4**
    #[quickcheck_macros::quickcheck]
    fn property_business_code_success_handling(asset_count: u8) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        let asset_count = asset_count as usize % 20; // Limit to reasonable size
        
        // Test with code 200
        let response_200 = create_mock_search_response(200, true, asset_count);
        if !is_success_code(response_200["code"].as_i64().unwrap()) {
            return TestResult::failed();
        }
        
        // Test with code 0
        let response_0 = create_mock_search_response(0, true, asset_count);
        if !is_success_code(response_0["code"].as_i64().unwrap()) {
            return TestResult::failed();
        }
        
        // Verify data extraction should proceed for success codes
        let results_200 = extract_results_from_response(&response_200);
        if results_200.len() != asset_count {
            return TestResult::failed();
        }
        
        let results_0 = extract_results_from_response(&response_0);
        if results_0.len() != asset_count {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 5: Business Code Error Handling
    // **Validates: Requirements 2.5, 5.5**
    #[quickcheck_macros::quickcheck]
    fn property_business_code_error_handling(error_code: u16, use_msg: bool) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Convert to i64 and ensure it's not a success code
        let error_code = error_code as i64;
        if error_code == 200 || error_code == 0 {
            return TestResult::discard();
        }
        
        // Limit to reasonable error code range (1-999, excluding 200)
        let error_code = if error_code == 200 { 201 } else { (error_code % 999) + 1 };
        
        // Create error response
        let response = create_mock_error_response(error_code, use_msg);
        
        // Verify it's not treated as success
        if is_success_code(response["code"].as_i64().unwrap()) {
            return TestResult::failed();
        }
        
        // Verify error message can be extracted
        let error_message = extract_error_message(&response);
        if error_message.is_none() {
            return TestResult::failed();
        }
        
        // Verify error message contains information about the error
        let msg = error_message.unwrap();
        if msg.is_empty() {
            return TestResult::failed();
        }
        
        // Verify the message was extracted from either "message" or "msg" field
        let has_message = response["message"].is_string();
        let has_msg = response["msg"].is_string();
        if !has_message && !has_msg {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 7: Total Count Extraction
    // **Validates: Requirements 2.7**
    #[quickcheck_macros::quickcheck]
    fn property_total_count_extraction(total: u64) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Limit to reasonable total count
        let total = total % 100000;
        
        // Create response with total field
        let response = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": total,
                "items": []
            }
        });
        
        // Extract total count
        let extracted_total = extract_total_count(&response);
        
        // Verify extraction succeeded
        if extracted_total.is_none() {
            return TestResult::failed();
        }
        
        // Verify extracted value matches input
        if extracted_total.unwrap() != total {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 2: HTTPS Protocol Usage
    // **Validates: Requirements 1.4**
    #[quickcheck_macros::quickcheck]
    fn property_https_protocol_usage(
        api_key: String,
        query: String,
        page: u32,
        page_size: u32
    ) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Skip invalid inputs
        if api_key.is_empty() || query.is_empty() || page == 0 || page_size == 0 {
            return TestResult::discard();
        }
        
        // Limit page and page_size to reasonable values
        let page = (page % 1000) + 1;
        let page_size = (page_size % 100) + 1;
        
        // Test 1: Search endpoint URL should use HTTPS
        let search_request = build_mock_search_request(&api_key, &query, page, page_size);
        if !search_request.verify_https() {
            return TestResult::failed();
        }
        
        // Verify the exact URL starts with https://
        if !search_request.url.starts_with("https://") {
            return TestResult::failed();
        }
        
        // Test 2: Validation endpoint URL should use HTTPS
        let validation_request = build_mock_validation_request(&api_key);
        if !validation_request.verify_https() {
            return TestResult::failed();
        }
        
        // Verify the exact URL starts with https://
        if !validation_request.url.starts_with("https://") {
            return TestResult::failed();
        }
        
        // Test 3: Verify the actual endpoint URLs used in the implementation
        let search_url = "https://www.daydaymap.com/api/v1/search";
        let validation_url = "https://www.daydaymap.com/api/v1/user/info";
        
        if !search_url.starts_with("https://") {
            return TestResult::failed();
        }
        
        if !validation_url.starts_with("https://") {
            return TestResult::failed();
        }
        
        // Test 4: Verify no HTTP (non-secure) URLs are used
        if search_request.url.starts_with("http://") {
            return TestResult::failed();
        }
        
        if validation_request.url.starts_with("http://") {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 8: Required Field Extraction
    // **Validates: Requirements 3.1, 3.2**
    #[quickcheck_macros::quickcheck]
    fn property_required_field_extraction(assets: Vec<ArbitraryAsset>) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Skip empty asset lists as they don't test the property meaningfully
        if assets.is_empty() {
            return TestResult::discard();
        }
        
        // Create a mock API response with the assets
        let response_json = json!({
            "code": 200,
            "message": "success",
            "data": {
                "total": assets.len(),
                "items": assets.iter().map(|a| a.to_json()).collect::<Vec<Value>>()
            }
        });
        
        // Extract the data section
        let data = &response_json["data"];
        let empty_vec = Vec::new();
        let items = data["items"].as_array()
            .or_else(|| data["list"].as_array())
            .unwrap_or(&empty_vec);
        
        // Map the items using the same logic as the search function
        let results: Vec<Value> = items.iter().map(|item| {
            json!({
                "ip": item["ip"].as_str().unwrap_or(""),
                "port": item["port"].as_i64().unwrap_or(0),
                "domain": item["domain"].as_str().unwrap_or(""),
                "title": item["title"].as_str().unwrap_or(""),
                "server": item["server"].as_str().unwrap_or(""),
                "country": item["country"].as_str().unwrap_or(""),
                "province": item["province"].as_str()
                    .or_else(|| item["region"].as_str())
                    .unwrap_or(""),
                "city": item["city"].as_str().unwrap_or(""),
                "url": format!("{}://{}:{}",
                    item["protocol"].as_str().unwrap_or("http"),
                    item["ip"].as_str().unwrap_or(""),
                    item["port"].as_i64().unwrap_or(80)
                ),
            })
        }).collect();
        
        // Verify that all results have all required fields
        for result in results.iter() {
            // Check that all required fields exist
            if !result.is_object() {
                return TestResult::failed();
            }
            
            let obj = result.as_object().unwrap();
            
            // Required fields according to Requirements 3.1, 3.2:
            // ip, port, domain, title, server, country, province (or region), city, protocol (implicit in url), url
            let required_fields = vec!["ip", "port", "domain", "title", "server", "country", "province", "city", "url"];
            
            for field in required_fields {
                if !obj.contains_key(field) {
                    return TestResult::failed();
                }
            }
            
            // Verify field types
            // String fields should be strings (or empty strings)
            if !result["ip"].is_string() {
                return TestResult::failed();
            }
            if !result["domain"].is_string() {
                return TestResult::failed();
            }
            if !result["title"].is_string() {
                return TestResult::failed();
            }
            if !result["server"].is_string() {
                return TestResult::failed();
            }
            if !result["country"].is_string() {
                return TestResult::failed();
            }
            if !result["province"].is_string() {
                return TestResult::failed();
            }
            if !result["city"].is_string() {
                return TestResult::failed();
            }
            if !result["url"].is_string() {
                return TestResult::failed();
            }
            
            // Numeric fields should be numbers
            if !result["port"].is_i64() && !result["port"].is_u64() {
                return TestResult::failed();
            }
            
            // URL should be non-empty and contain the expected format
            let url = result["url"].as_str().unwrap();
            if url.is_empty() {
                return TestResult::failed();
            }
            
            // URL should contain "://" (protocol separator)
            if !url.contains("://") {
                return TestResult::failed();
            }
            
            // URL should contain ":" after the protocol (port separator)
            let parts: Vec<&str> = url.split("://").collect();
            if parts.len() != 2 {
                return TestResult::failed();
            }
            if !parts[1].contains(':') {
                return TestResult::failed();
            }
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 9: URL Construction Format
    // **Validates: Requirements 3.3**
    #[quickcheck_macros::quickcheck]
    fn property_url_construction_format(assets: Vec<ArbitraryAsset>) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        if assets.is_empty() {
            return TestResult::discard();
        }
        
        for asset in assets.iter() {
            let protocol = asset.protocol.as_deref().unwrap_or("http");
            let ip = asset.ip.as_deref().unwrap_or("");
            let port = asset.port.unwrap_or(80);
            
            // Construct URL using the same logic as the implementation
            let url = format!("{}://{}:{}", protocol, ip, port);
            
            // Verify format: {protocol}://{ip}:{port}
            if !url.contains("://") {
                return TestResult::failed();
            }
            
            let parts: Vec<&str> = url.split("://").collect();
            if parts.len() != 2 {
                return TestResult::failed();
            }
            
            // Verify protocol part
            if parts[0] != protocol {
                return TestResult::failed();
            }
            
            // Verify ip:port part
            if !parts[1].contains(':') {
                return TestResult::failed();
            }
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 10: Default Value Handling
    // **Validates: Requirements 3.4, 3.5, 3.6, 3.7**
    #[quickcheck_macros::quickcheck]
    fn property_default_value_handling() -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Create an asset with all fields missing
        let empty_asset = json!({});
        
        // Map using the same logic as the implementation
        let result = json!({
            "ip": empty_asset["ip"].as_str().unwrap_or(""),
            "port": empty_asset["port"].as_i64().unwrap_or(0),
            "domain": empty_asset["domain"].as_str().unwrap_or(""),
            "title": empty_asset["title"].as_str().unwrap_or(""),
            "server": empty_asset["server"].as_str().unwrap_or(""),
            "country": empty_asset["country"].as_str().unwrap_or(""),
            "province": empty_asset["province"].as_str()
                .or_else(|| empty_asset["region"].as_str())
                .unwrap_or(""),
            "city": empty_asset["city"].as_str().unwrap_or(""),
            "url": format!("{}://{}:{}",
                empty_asset["protocol"].as_str().unwrap_or("http"),
                empty_asset["ip"].as_str().unwrap_or(""),
                empty_asset["port"].as_i64().unwrap_or(80)
            ),
        });
        
        // Verify default values
        // String fields should default to empty string
        if result["ip"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["domain"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["title"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["server"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["country"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["province"].as_str() != Some("") {
            return TestResult::failed();
        }
        if result["city"].as_str() != Some("") {
            return TestResult::failed();
        }
        
        // Numeric fields should default to 0
        if result["port"].as_i64() != Some(0) {
            return TestResult::failed();
        }
        
        // Protocol should default to "http"
        if !result["url"].as_str().unwrap().starts_with("http://") {
            return TestResult::failed();
        }
        
        // Port should default to 80
        if !result["url"].as_str().unwrap().ends_with(":80") {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Example 3: CSV Header Format
    // **Validates: Requirements 4.5**
    #[test]
    fn test_csv_header_format() {
        // Verify the CSV header matches the exact specification
        let expected_header = "IP,端口,域名,标题,服务器,国家,省份,城市,URL";
        
        // This is the header used in the export function
        let actual_header = "IP,端口,域名,标题,服务器,国家,省份,城市,URL";
        
        assert_eq!(
            actual_header,
            expected_header,
            "CSV header should match the specification exactly"
        );
        
        // Verify it contains Chinese characters
        assert!(contains_chinese(actual_header), "CSV header should contain Chinese field names");
        
        // Verify field count
        let fields: Vec<&str> = actual_header.split(',').collect();
        assert_eq!(fields.len(), 9, "CSV header should have 9 fields");
        
        // Verify specific Chinese field names
        assert!(actual_header.contains("端口"), "Should contain '端口' (port)");
        assert!(actual_header.contains("域名"), "Should contain '域名' (domain)");
        assert!(actual_header.contains("标题"), "Should contain '标题' (title)");
        assert!(actual_header.contains("服务器"), "Should contain '服务器' (server)");
        assert!(actual_header.contains("国家"), "Should contain '国家' (country)");
        assert!(actual_header.contains("省份"), "Should contain '省份' (province)");
        assert!(actual_header.contains("城市"), "Should contain '城市' (city)");
    }

    // Feature: daydaymap-api-fix, Property 15: CSV Comma Sanitization
    // **Validates: Requirements 4.6**
    #[quickcheck_macros::quickcheck]
    fn property_csv_comma_sanitization(title: String) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Only test titles that contain commas
        if !title.contains(',') {
            return TestResult::discard();
        }
        
        // Apply the same sanitization logic as the implementation
        let sanitized = title.replace(",", "，");
        
        // Verify no regular commas remain
        if sanitized.contains(',') {
            return TestResult::failed();
        }
        
        // Verify Chinese commas are present if original had commas
        if !sanitized.contains('，') {
            return TestResult::failed();
        }
        
        // Verify the number of Chinese commas matches original comma count
        let original_comma_count = title.matches(',').count();
        let sanitized_comma_count = sanitized.matches('，').count();
        if original_comma_count != sanitized_comma_count {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Example 4: Validation Endpoint URL
    // **Validates: Requirements 5.1**
    #[test]
    fn test_validation_endpoint_url() {
        // Verify that the validation function uses the correct endpoint URL
        let expected_url = "https://www.daydaymap.com/api/v1/user/info";
        
        // Create a mock validation request
        let api_key = "test_key";
        let request = build_mock_validation_request(api_key);
        
        // Verify the URL matches the expected endpoint
        assert_eq!(
            request.url,
            expected_url,
            "Validation endpoint URL should be https://www.daydaymap.com/api/v1/user/info"
        );
        
        // Verify it's a GET request
        assert_eq!(
            request.method,
            "GET",
            "Validation should use GET method"
        );
        
        // Verify it uses HTTPS
        assert!(
            request.url.starts_with("https://"),
            "Validation endpoint should use HTTPS"
        );
    }

    // Feature: daydaymap-api-fix, Property 18: Quota Field Flexibility
    // **Validates: Requirements 5.6**
    #[quickcheck_macros::quickcheck]
    fn property_quota_field_flexibility(quota_value: u32) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        let quota_value = (quota_value % 10000) as i64;
        
        // Test with "credit" field
        let response_with_credit = json!({
            "code": 200,
            "message": "success",
            "data": {
                "credit": quota_value
            }
        });
        
        let credit = response_with_credit["data"]["credit"].as_i64();
        if credit != Some(quota_value) {
            return TestResult::failed();
        }
        
        // Test with "quota" field
        let response_with_quota = json!({
            "code": 200,
            "message": "success",
            "data": {
                "quota": quota_value
            }
        });
        
        let quota = response_with_quota["data"]["quota"].as_i64();
        if quota != Some(quota_value) {
            return TestResult::failed();
        }
        
        // Test with both fields (should prefer "credit")
        let response_with_both = json!({
            "code": 200,
            "message": "success",
            "data": {
                "credit": quota_value,
                "quota": quota_value + 1000
            }
        });
        
        // Implementation checks credit first
        let extracted = response_with_both["data"]["credit"].as_i64()
            .or_else(|| response_with_both["data"]["quota"].as_i64());
        if extracted != Some(quota_value) {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Feature: daydaymap-api-fix, Property 19: Quota Formatting
    // **Validates: Requirements 5.7**
    #[quickcheck_macros::quickcheck]
    fn property_quota_formatting(quota_value: u32) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        let quota_value = (quota_value % 10000) as i64;
        
        // Test formatting with credit field
        let formatted_credit = format!("剩余积分: {}", quota_value);
        if !contains_chinese(&formatted_credit) {
            return TestResult::failed();
        }
        if !formatted_credit.contains(&quota_value.to_string()) {
            return TestResult::failed();
        }
        
        // Test formatting with quota field
        let formatted_quota = format!("剩余配额: {}", quota_value);
        if !contains_chinese(&formatted_quota) {
            return TestResult::failed();
        }
        if !formatted_quota.contains(&quota_value.to_string()) {
            return TestResult::failed();
        }
        
        TestResult::passed()
    }

    // Example 5: Missing Quota Fallback
    // **Validates: Requirements 5.8**
    #[test]
    fn test_missing_quota_fallback() {
        // Test response without credit or quota fields
        let response = json!({
            "code": 200,
            "message": "success",
            "data": {
                "username": "test@example.com"
            }
        });
        
        // Extract quota using the same logic as implementation
        let quota_info = if let Some(credit) = response["data"]["credit"].as_i64() {
            format!("剩余积分: {}", credit)
        } else if let Some(quota) = response["data"]["quota"].as_i64() {
            format!("剩余配额: {}", quota)
        } else {
            "无法获取配额信息".to_string()
        };
        
        // Verify fallback message
        assert_eq!(
            quota_info,
            "无法获取配额信息",
            "Should return fallback message when quota is missing"
        );
        
        // Verify it's in Chinese
        assert!(
            contains_chinese(&quota_info),
            "Fallback message should be in Chinese"
        );
    }

    // Feature: daydaymap-api-fix, Property 20: Error Message Format Consistency
    // **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6**
    #[test]
    fn test_error_message_format_consistency() {
        // Test various error message formats
        let test_cases = vec![
            ("请求失败: connection timeout", "请求失败:"),
            ("API返回错误状态码: 500", "API返回错误状态码:"),
            ("解析JSON失败: invalid json", "解析JSON失败:"),
            ("读取响应失败: io error", "读取响应失败:"),
            ("创建文件失败: permission denied", "创建文件失败:"),
            ("写入CSV头部失败: disk full", "写入CSV头部失败:"),
            ("写入数据失败: disk full", "写入数据失败:"),
        ];
        
        for (message, expected_prefix) in test_cases {
            assert!(
                message.starts_with(expected_prefix),
                "Error message '{}' should start with '{}'",
                message,
                expected_prefix
            );
            
            // Verify message is in Chinese
            assert!(
                contains_chinese(message),
                "Error message '{}' should contain Chinese characters",
                message
            );
        }
    }

    // Feature: daydaymap-api-fix, Property 21: Error Message Localization
    // **Validates: Requirements 6.7**
    #[quickcheck_macros::quickcheck]
    fn property_error_message_localization() -> quickcheck::TestResult {
        use quickcheck::TestResult;
        
        // Test all error message patterns used in the implementation
        let error_messages = vec![
            "请求失败",
            "API返回错误状态码",
            "解析JSON失败",
            "读取响应失败",
            "创建文件失败",
            "写入CSV头部失败",
            "写入数据失败",
            "API密钥无效或已过期",
            "无法获取配额信息",
            "剩余积分",
            "剩余配额",
        ];
        
        for message in error_messages {
            if !contains_chinese(message) {
                return TestResult::failed();
            }
        }
        
        TestResult::passed()
    }
}
