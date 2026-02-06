// 测试多个查询
use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("=== DayDayMap API 多查询测试 ===\n");
    
    let queries = vec![
        ("port=\"443\"", "查询HTTPS端口"),
        ("ip=\"1.1.1.1\"", "查询特定IP"),
        ("country=\"中国\"", "查询中国资产"),
    ];
    
    for (query, description) in queries {
        println!("测试: {} ({})", description, query);
        match search(api_key, query, 1, 5).await {
            Ok(response) => {
                if let Some(total) = response["total"].as_u64() {
                    println!("  ✓ 总结果数: {}", total);
                }
                if let Some(results) = response["results"].as_array() {
                    println!("  ✓ 返回结果数: {}", results.len());
                }
            }
            Err(e) => {
                println!("  ✗ 失败: {}", e);
            }
        }
        println!();
        
        // 避免请求过快
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    println!("=== 测试完成 ===");
    Ok(())
}

async fn search(api_key: &str, query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    let base_url = "https://www.daydaymap.com/api/v1/raymap/search/all";
    
    let keyword_base64 = general_purpose::STANDARD.encode(query.as_bytes());
    
    let request_body = json!({
        "keyword": keyword_base64,
        "page": page,
        "page_size": page_size
    });
    
    let client = Client::new();
    let response = client.post(base_url)
        .header("api-key", api_key)
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
            "country": item["country"].as_str().unwrap_or(""),
        })
    }).collect::<Vec<Value>>();
    
    Ok(json!({
        "total": total,
        "results": results
    }))
}
