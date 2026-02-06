// DayDayMap API 详细测试
// 显示完整的 API 响应以便调试

use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    println!("=== DayDayMap API 详细测试 ===\n");
    
    // 使用提供的 API 密钥
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("API 密钥: {}", api_key);
    println!("密钥长度: {} 字符\n", api_key.len());
    
    // 测试 1: 验证 API 密钥（显示完整响应）
    println!("测试 1: 验证 API 密钥");
    println!("===================");
    
    let validation_url = "https://www.daydaymap.com/api/v1/user/info";
    println!("请求 URL: {}", validation_url);
    println!("请求方法: GET");
    println!("请求头:");
    println!("  Authorization: Bearer {}", api_key);
    println!("  Content-Type: application/json\n");
    
    let client = Client::new();
    match client.get(validation_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            println!("响应状态: {}\n", if status.is_success() { "成功" } else { "失败" });
            
            match response.text().await {
                Ok(body) => {
                    println!("响应体:");
                    println!("{}\n", body);
                    
                    // 尝试解析 JSON
                    match serde_json::from_str::<Value>(&body) {
                        Ok(json) => {
                            println!("解析后的 JSON:");
                            println!("{}\n", serde_json::to_string_pretty(&json).unwrap());
                            
                            // 分析响应
                            if let Some(code) = json["code"].as_i64() {
                                println!("业务代码: {}", code);
                                
                                if code == 200 || code == 0 {
                                    println!("✓ 业务代码表示成功");
                                    
                                    if let Some(data) = json["data"].as_object() {
                                        println!("\n用户数据:");
                                        for (key, value) in data {
                                            println!("  {}: {}", key, value);
                                        }
                                    }
                                } else {
                                    println!("✗ 业务代码表示失败");
                                    
                                    let message = json["message"].as_str()
                                        .or_else(|| json["msg"].as_str())
                                        .unwrap_or("无错误消息");
                                    println!("错误消息: {}", message);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ JSON 解析失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应体失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
    
    println!("\n");
    
    // 测试 2: 搜索请求（显示完整响应）
    println!("测试 2: 搜索资产");
    println!("===================");
    
    let search_url = "https://www.daydaymap.com/api/v1/search";
    let query = "port=\"80\"";
    let page = 1;
    let page_size = 5;
    
    let request_body = json!({
        "query": query,
        "page": page,
        "page_size": page_size
    });
    
    println!("请求 URL: {}", search_url);
    println!("请求方法: POST");
    println!("请求头:");
    println!("  Authorization: Bearer {}", api_key);
    println!("  Content-Type: application/json");
    println!("\n请求体:");
    println!("{}\n", serde_json::to_string_pretty(&request_body).unwrap());
    
    match client.post(search_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            println!("响应状态: {}\n", if status.is_success() { "成功" } else { "失败" });
            
            match response.text().await {
                Ok(body) => {
                    println!("响应体:");
                    println!("{}\n", body);
                    
                    // 尝试解析 JSON
                    match serde_json::from_str::<Value>(&body) {
                        Ok(json) => {
                            println!("解析后的 JSON:");
                            println!("{}\n", serde_json::to_string_pretty(&json).unwrap());
                            
                            // 分析响应
                            if let Some(code) = json["code"].as_i64() {
                                println!("业务代码: {}", code);
                                
                                if code == 200 || code == 0 {
                                    println!("✓ 业务代码表示成功");
                                    
                                    if let Some(data) = json["data"].as_object() {
                                        if let Some(total) = data.get("total") {
                                            println!("总结果数: {}", total);
                                        }
                                        
                                        // 检查结果字段
                                        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
                                            println!("结果字段: items");
                                            println!("返回结果数: {}", items.len());
                                        } else if let Some(list) = data.get("list").and_then(|v| v.as_array()) {
                                            println!("结果字段: list");
                                            println!("返回结果数: {}", list.len());
                                        } else {
                                            println!("✗ 未找到结果数组（items 或 list）");
                                        }
                                    }
                                } else {
                                    println!("✗ 业务代码表示失败");
                                    
                                    let message = json["message"].as_str()
                                        .or_else(|| json["msg"].as_str())
                                        .unwrap_or("无错误消息");
                                    println!("错误消息: {}", message);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ JSON 解析失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应体失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
    
    println!("\n=== 测试完成 ===");
    println!("\n总结:");
    println!("------");
    println!("如果看到 HTTP 401 状态码，说明 API 密钥无效或已过期。");
    println!("如果看到其他错误，请检查网络连接和 API 端点是否正确。");
    println!("如果需要新的 API 密钥，请访问 DayDayMap 官网获取。");
}
