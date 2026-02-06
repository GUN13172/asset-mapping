// DayDayMap API 实际测试
// 使用真实的 API 密钥测试 DayDayMap 集成

use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    println!("=== DayDayMap API 测试 ===\n");
    
    // 使用提供的 API 密钥
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    // 测试 1: 验证 API 密钥
    println!("测试 1: 验证 API 密钥");
    println!("-------------------");
    match validate_api_key(api_key).await {
        Ok(result) => {
            println!("✓ 验证成功");
            println!("  有效性: {}", result.valid);
            if let Some(message) = result.message {
                println!("  消息: {}", message);
            }
            if let Some(quota) = result.quota {
                println!("  配额: {}", quota);
            }
        }
        Err(e) => {
            println!("✗ 验证失败: {}", e);
        }
    }
    
    println!("\n");
    
    // 测试 2: 执行搜索
    println!("测试 2: 搜索资产");
    println!("-------------------");
    let query = "port=\"80\"";  // 简单的搜索查询
    let page = 1;
    let page_size = 5;
    
    println!("查询: {}", query);
    println!("页码: {}, 每页: {}", page, page_size);
    
    match search(api_key, query, page, page_size).await {
        Ok(result) => {
            println!("✓ 搜索成功");
            println!("  总数: {}", result["total"]);
            
            if let Some(results) = result["results"].as_array() {
                println!("  返回结果数: {}", results.len());
                
                // 显示前几个结果
                for (i, item) in results.iter().take(3).enumerate() {
                    println!("\n  结果 {}:", i + 1);
                    println!("    IP: {}", item["ip"].as_str().unwrap_or("N/A"));
                    println!("    端口: {}", item["port"]);
                    println!("    域名: {}", item["domain"].as_str().unwrap_or("N/A"));
                    println!("    标题: {}", item["title"].as_str().unwrap_or("N/A"));
                    println!("    URL: {}", item["url"].as_str().unwrap_or("N/A"));
                }
                
                if results.len() > 3 {
                    println!("\n  ... 还有 {} 个结果", results.len() - 3);
                }
            }
        }
        Err(e) => {
            println!("✗ 搜索失败: {}", e);
        }
    }
    
    println!("\n=== 测试完成 ===");
}

// 验证 API 密钥
async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    let base_url = "https://www.daydaymap.com/api/v1/user/info";
    
    let client = Client::new();
    let response = client.get(base_url)
        .header("Authorization", format!("Bearer {}", api_key))
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
    let base_url = "https://www.daydaymap.com/api/v1/search";
    
    let request_body = json!({
        "query": query,
        "page": page,
        "page_size": page_size
    });
    
    let client = Client::new();
    let response = client.post(base_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API返回错误状态码: {}", response.status()));
    }
    
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    let code = response_json["code"].as_i64().unwrap_or(-1);
    if code != 200 && code != 0 {
        return Err(format!("API返回错误: {}", 
            response_json["message"].as_str()
                .or_else(|| response_json["msg"].as_str())
                .unwrap_or("未知错误")));
    }
    
    let data = &response_json["data"];
    let total = data["total"].as_u64().unwrap_or(0);
    let items = data["items"].as_array()
        .or_else(|| data["list"].as_array())
        .unwrap_or(&Vec::new())
        .clone();
    
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

// API 密钥验证结果
#[derive(Debug)]
struct ApiKeyValidationResult {
    valid: bool,
    message: Option<String>,
    quota: Option<String>,
}
