// 测试修复后的API密钥验证功能
use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug)]
struct ApiKeyValidationResult {
    valid: bool,
    message: Option<String>,
    quota: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("=== 测试修复后的API密钥验证 ===\n");
    println!("使用搜索接口验证密钥 (查询: ip=\"1.1.1.1\")");
    println!("如果能成功查询到数据，则证明密钥有效\n");
    
    match validate_api_key(api_key).await {
        Ok(result) => {
            println!("验证结果:");
            println!("  - 有效: {}", result.valid);
            if let Some(msg) = result.message {
                println!("  - 消息: {}", msg);
            }
            if let Some(quota) = result.quota {
                println!("  - 信息: {}", quota);
            }
            
            if result.valid {
                println!("\n✓ API密钥验证成功！");
            } else {
                println!("\n✗ API密钥验证失败");
            }
        }
        Err(e) => {
            println!("✗ 验证过程出错: {}", e);
        }
    }
    
    println!("\n=== 测试完成 ===");
    Ok(())
}

// 验证API密钥 - 使用搜索接口
async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    let base_url = "https://www.daydaymap.com/api/v1/raymap/search/all";
    
    // Base64编码测试查询
    let test_query = "ip=\"1.1.1.1\"";
    let keyword_base64 = general_purpose::STANDARD.encode(test_query.as_bytes());
    
    println!("发送验证请求...");
    println!("  - 测试查询: {}", test_query);
    println!("  - Base64编码: {}", keyword_base64);
    
    // 构建请求体
    let request_body = json!({
        "keyword": keyword_base64,
        "page": 1,
        "page_size": 1
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
    
    let status = response.status();
    println!("  - HTTP状态码: {}", status);
    
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
    println!("  - 业务状态码: {}", code);
    
    if code == 200 || code == 0 {
        let data = &response_json["data"];
        let total = data["total"].as_u64().unwrap_or(0);
        
        let quota_info = format!("API密钥有效，测试查询返回 {} 条结果", total);
        
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
