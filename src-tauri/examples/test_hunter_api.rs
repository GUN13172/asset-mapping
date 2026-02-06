use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() {
    println!("=== Hunter API 测试 ===\n");
    
    // 从环境变量或配置文件读取API密钥
    let api_key = std::env::var("HUNTER_API_KEY")
        .unwrap_or_else(|_| {
            println!("警告: 未设置HUNTER_API_KEY环境变量，使用测试密钥");
            "test_key".to_string()
        });
    
    println!("API Key: {}...\n", &api_key[..8.min(api_key.len())]);
    
    // 测试1: 验证API密钥
    println!("【测试1】验证API密钥");
    test_validate_api_key(&api_key).await;
    println!();
    
    // 测试2: 基础查询
    println!("【测试2】基础查询");
    test_basic_search(&api_key).await;
    println!();
    
    // 测试3: 带状态码过滤的查询
    println!("【测试3】带状态码过滤的查询");
    test_search_with_status_code(&api_key).await;
    println!();
    
    // 测试4: 带时间范围的查询
    println!("【测试4】带时间范围的查询");
    test_search_with_time_range(&api_key).await;
    println!();
}

// 测试API密钥验证
async fn test_validate_api_key(api_key: &str) {
    // Hunter API 没有单独的用户信息接口，使用搜索接口验证
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    // 使用一个简单的查询来验证API密钥
    let test_query = "domain=\"test.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(test_query.as_bytes());
    
    println!("测试查询: {}", test_query);
    
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "1"),
        ("is_web", "3"),
    ];
    
    let client = Client::new();
    match client.get(base_url)
        .query(&params)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            
            match response.text().await {
                Ok(text) => {
                    println!("响应内容: {}", text);
                    
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            let code = json["code"].as_u64().unwrap_or(0);
                            println!("业务状态码: {}", code);
                            
                            if code == 200 {
                                println!("✓ API密钥有效");
                                if let Some(data) = json["data"].as_object() {
                                    if let Some(rest_quota) = data.get("rest_quota") {
                                        println!("剩余积分: {}", rest_quota);
                                    }
                                    if let Some(consume_quota) = data.get("consume_quota") {
                                        println!("消耗积分: {}", consume_quota);
                                    }
                                    if let Some(account_type) = data.get("account_type") {
                                        println!("账户类型: {}", account_type);
                                    }
                                }
                            } else {
                                println!("✗ API密钥无效");
                                if let Some(msg) = json["message"].as_str().or_else(|| json["msg"].as_str()) {
                                    println!("错误信息: {}", msg);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ 解析JSON失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
}

// 测试基础查询
async fn test_basic_search(api_key: &str) {
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    // 查询语法: domain="baidu.com"
    let query = "domain=\"baidu.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(query.as_bytes());
    
    println!("查询语法: {}", query);
    println!("Base64编码: {}", encoded_query);
    
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "10"),
        ("is_web", "3"),
    ];
    
    let client = Client::new();
    match client.get(base_url)
        .query(&params)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            
            match response.text().await {
                Ok(text) => {
                    println!("响应内容前500字符: {}", &text[..500.min(text.len())]);
                    
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            let code = json["code"].as_u64().unwrap_or(0);
                            println!("业务状态码: {}", code);
                            
                            if code == 200 {
                                println!("✓ 查询成功");
                                if let Some(data) = json["data"].as_object() {
                                    let total = data.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                                    println!("总结果数: {}", total);
                                    
                                    if let Some(arr) = data.get("arr").and_then(|v| v.as_array()) {
                                        println!("返回结果数: {}", arr.len());
                                        if let Some(first) = arr.first() {
                                            println!("第一条结果: {}", serde_json::to_string_pretty(first).unwrap());
                                        }
                                    }
                                    
                                    if let Some(consume) = data.get("consume_quota").and_then(|v| v.as_str()) {
                                        println!("消耗积分: {}", consume);
                                    }
                                    if let Some(rest) = data.get("rest_quota").and_then(|v| v.as_str()) {
                                        println!("剩余积分: {}", rest);
                                    }
                                }
                            } else {
                                println!("✗ 查询失败");
                                if let Some(msg) = json["message"].as_str().or_else(|| json["msg"].as_str()) {
                                    println!("错误信息: {}", msg);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ 解析JSON失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
}

// 测试带状态码过滤的查询
async fn test_search_with_status_code(api_key: &str) {
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    let query = "domain=\"baidu.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(query.as_bytes());
    
    println!("查询语法: {}", query);
    println!("状态码过滤: 200");
    
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "10"),
        ("is_web", "3"),
        ("status_code", "200"),
    ];
    
    let client = Client::new();
    match client.get(base_url)
        .query(&params)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            
            match response.text().await {
                Ok(text) => {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            let code = json["code"].as_u64().unwrap_or(0);
                            println!("业务状态码: {}", code);
                            
                            if code == 200 {
                                println!("✓ 查询成功");
                                if let Some(data) = json["data"].as_object() {
                                    let total = data.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                                    println!("总结果数: {}", total);
                                }
                            } else {
                                println!("✗ 查询失败");
                                if let Some(msg) = json["message"].as_str().or_else(|| json["msg"].as_str()) {
                                    println!("错误信息: {}", msg);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ 解析JSON失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
}

// 测试带时间范围的查询
async fn test_search_with_time_range(api_key: &str) {
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    let query = "domain=\"baidu.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(query.as_bytes());
    
    println!("查询语法: {}", query);
    println!("时间范围: 2023-01-01 00:00:00 ~ 2023-12-31 23:59:59");
    
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "10"),
        ("is_web", "3"),
        ("start_time", "2023-01-01 00:00:00"),
        ("end_time", "2023-12-31 23:59:59"),
    ];
    
    let client = Client::new();
    match client.get(base_url)
        .query(&params)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("响应状态码: {}", status);
            
            match response.text().await {
                Ok(text) => {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            let code = json["code"].as_u64().unwrap_or(0);
                            println!("业务状态码: {}", code);
                            
                            if code == 200 {
                                println!("✓ 查询成功");
                                if let Some(data) = json["data"].as_object() {
                                    let total = data.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                                    println!("总结果数: {}", total);
                                }
                            } else {
                                println!("✗ 查询失败");
                                if let Some(msg) = json["message"].as_str().or_else(|| json["msg"].as_str()) {
                                    println!("错误信息: {}", msg);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ 解析JSON失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ 读取响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
        }
    }
}
