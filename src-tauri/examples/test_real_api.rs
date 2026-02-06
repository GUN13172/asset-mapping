// 使用真实API密钥测试DayDayMap API
use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("=== 测试 DayDayMap API (真实密钥) ===\n");
    
    // 测试1: 验证API密钥
    println!("1. 测试API密钥验证...");
    match validate_api_key(api_key).await {
        Ok(result) => {
            println!("   ✓ API密钥验证成功");
            println!("   - 有效: {}", result.valid);
            if let Some(msg) = result.message {
                println!("   - 消息: {}", msg);
            }
            if let Some(quota) = result.quota {
                println!("   - 配额: {}", quota);
            }
        }
        Err(e) => {
            println!("   ✗ API密钥验证失败: {}", e);
        }
    }
    
    println!();
    
    // 测试2: 搜索测试
    println!("2. 测试搜索功能 (查询: port=\"80\")...");
    match search(api_key, "port=\"80\"", 1, 10).await {
        Ok(response) => {
            println!("   ✓ 搜索成功");
            if let Some(total) = response["total"].as_u64() {
                println!("   - 总结果数: {}", total);
            }
            if let Some(results) = response["results"].as_array() {
                println!("   - 本页结果数: {}", results.len());
                if !results.is_empty() {
                    println!("   - 第一条结果:");
                    if let Some(first) = results.first() {
                        if let Some(ip) = first["ip"].as_str() {
                            println!("     IP: {}", ip);
                        }
                        if let Some(port) = first["port"].as_i64() {
                            println!("     端口: {}", port);
                        }
                        if let Some(country) = first["country"].as_str() {
                            println!("     国家: {}", country);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("   ✗ 搜索失败: {}", e);
        }
    }
    
    println!("\n=== 测试完成 ===");
    Ok(())
}

// 验证API密钥
async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    let base_url = "https://www.daydaymap.com/api/v1/user/info";
    
    let client = Client::new();
    let response = client.get(base_url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
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
        let data = &response_json["data"];
        
        let quota_info = if let Some(credit) = data["credit"].as_i64() {
            format!("剩余积分: {}", credit)
        } else if let Some(quota) = data["quota"].as_i64() {
            format!("剩余配额: {}", quota)
        } else {
            "无法获取配额信息".to_string()
        };
        
        Ok(ApiKeyValidationResult {
            valid: true,
            message: Some("API密钥验证成功".to_string()),
            quota: Some(quota_info),
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

// 搜索资产
async fn search(api_key: &str, query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    let base_url = "https://www.daydaymap.com/api/v1/raymap/search/all";
    
    // Base64编码查询字符串
    let keyword_base64 = general_purpose::STANDARD.encode(query.as_bytes());
    
    println!("   - 查询字符串: {}", query);
    println!("   - Base64编码: {}", keyword_base64);
    
    // 构建请求体
    let request_body = json!({
        "keyword": keyword_base64,
        "page": page,
        "page_size": page_size
    });
    
    // 发送请求
    let client = Client::new();
    let response = client.post(base_url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(format!("API返回错误状态码: {}", response.status()));
    }
    
    // 解析响应
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    // 检查API返回的状态码
    let code = response_json["code"].as_i64().unwrap_or(-1);
    if code != 200 && code != 0 {
        return Err(format!("API返回错误: {}", 
            response_json["message"].as_str()
                .or_else(|| response_json["msg"].as_str())
                .unwrap_or("未知错误")));
    }
    
    // 提取结果
    let data = &response_json["data"];
    let total = data["total"].as_u64().unwrap_or(0);
    let items = data["items"].as_array()
        .or_else(|| data["list"].as_array())
        .unwrap_or(&Vec::new())
        .clone();
    
    // 格式化结果
    let results = items.iter().map(|item| {
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
    }).collect::<Vec<Value>>();
    
    Ok(json!({
        "total": total,
        "results": results
    }))
}

#[derive(Debug)]
struct ApiKeyValidationResult {
    valid: bool,
    message: Option<String>,
    quota: Option<String>,
}
