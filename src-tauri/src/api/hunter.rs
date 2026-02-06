use reqwest::Client;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use crate::config;
use crate::ApiKeyValidationResult;
use super::key_manager;

// 使用单个API key进行搜索
async fn search_with_key(
    api_key: &str, 
    query: &str, 
    page: u32, 
    page_size: u32,
    status_code: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Value, String> {
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    // 对查询字符串进行Base64编码
    let encoded_query = general_purpose::URL_SAFE.encode(query.as_bytes());
    
    // 构建请求参数
    let mut params = vec![
        ("api-key", api_key.to_string()),
        ("search", encoded_query),
        ("page", page.to_string()),
        ("page_size", page_size.to_string()),
        ("is_web", "3".to_string()), // 3=全部资产，1=仅web资产，2=非web资产
    ];
    
    // 添加可选参数
    if let Some(code) = status_code {
        params.push(("status_code", code.to_string()));
    }
    if let Some(start) = start_time {
        params.push(("start_time", start.to_string()));
    }
    if let Some(end) = end_time {
        params.push(("end_time", end.to_string()));
    }
    
    // 发送请求
    let client = Client::new();
    let response = client.get(base_url)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(format!("API返回错误状态码: {}", response.status()));
    }
    
    // 解析响应
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let response_json: Value = serde_json::from_str(&response_text).map_err(|e| format!("解析JSON失败: {}", e))?;
    
    // 检查API返回的状态码
    if response_json["code"].as_u64() != Some(200) {
        return Err(format!("API返回错误: {}", response_json["message"].as_str().unwrap_or("未知错误")));
    }
    
    Ok(response_json)
}

// 搜索资产 - 支持自动轮询多个API Key
pub async fn search(query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    search_with_options(query, page, page_size, None, None, None).await
}

// 搜索资产（带可选参数）
pub async fn search_with_options(
    query: &str, 
    page: u32, 
    page_size: u32,
    status_code: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Value, String> {
    // 获取所有API密钥
    let api_keys = config::get_all_hunter_api_keys()?;
    
    if api_keys.is_empty() {
        return Err("未配置Hunter API密钥".to_string());
    }
    
    // Clone data for the closure
    let query = query.to_string();
    let status_code = status_code.map(|s| s.to_string());
    let start_time = start_time.map(|s| s.to_string());
    let end_time = end_time.map(|s| s.to_string());
    
    // 使用key_manager进行智能轮询
    let result = key_manager::execute_with_key_rotation(
        "hunter",
        &api_keys,
        |api_key| {
            let query = query.clone();
            let api_key = api_key.to_string();
            let status_code = status_code.clone();
            let start_time = start_time.clone();
            let end_time = end_time.clone();
            async move {
                search_with_key(
                    &api_key, 
                    &query, 
                    page, 
                    page_size,
                    status_code.as_deref(),
                    start_time.as_deref(),
                    end_time.as_deref(),
                ).await
            }
        }
    ).await;
    
    match result {
        Ok(response_json) => {
            // 提取结果
            let total = response_json["data"]["total"].as_u64().unwrap_or(0);
            let arr = response_json["data"]["arr"].as_array().unwrap_or(&Vec::new()).clone();
            
            // 格式化结果
            let results = arr.iter().map(|item| {
                // 构建URL，优先使用domain，否则使用IP
                let host = if let Some(domain) = item["domain"].as_str() {
                    if !domain.is_empty() {
                        domain.to_string()
                    } else {
                        item["ip"].as_str().unwrap_or("").to_string()
                    }
                } else {
                    item["ip"].as_str().unwrap_or("").to_string()
                };
                
                let protocol = item["protocol"].as_str().unwrap_or("http");
                
                // 正确获取端口号，可能是字符串或数字
                let port = if let Some(port_str) = item["port"].as_str() {
                    port_str.to_string()
                } else if let Some(port_num) = item["port"].as_u64() {
                    port_num.to_string()
                } else if let Some(port_num) = item["port"].as_i64() {
                    port_num.to_string()
                } else {
                    "80".to_string()
                };
                
                let url = format!("{}://{}:{}", protocol, host, port);
                
                // 提取组件信息
                let components = if let Some(comp_array) = item["component"].as_array() {
                    comp_array.iter()
                        .filter_map(|c| {
                            let name = c["name"].as_str().unwrap_or("");
                            let version = c["version"].as_str().unwrap_or("");
                            if !name.is_empty() {
                                if !version.is_empty() {
                                    Some(format!("{}:{}", name, version))
                                } else {
                                    Some(name.to_string())
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                } else {
                    String::new()
                };
                
                json!({
                    "url": url,
                    "ip": item["ip"].as_str().unwrap_or(""),
                    "port": port,
                    "domain": item["domain"].as_str().unwrap_or(""),
                    "web_title": item["web_title"].as_str().unwrap_or(""),
                    "status_code": item["status_code"].as_i64().unwrap_or(0),
                    "country": item["country"].as_str().unwrap_or(""),
                    "province": item["province"].as_str().unwrap_or(""),
                    "city": item["city"].as_str().unwrap_or(""),
                    "server": item["banner"].as_str().unwrap_or(""),
                    "protocol": protocol,
                    "base_protocol": item["base_protocol"].as_str().unwrap_or(""),
                    "os": item["os"].as_str().unwrap_or(""),
                    "company": item["company"].as_str().unwrap_or(""),
                    "number": item["number"].as_str().unwrap_or(""),
                    "isp": item["isp"].as_str().unwrap_or(""),
                    "as_org": item["as_org"].as_str().unwrap_or(""),
                    "component": components,
                    "updated_at": item["updated_at"].as_str().unwrap_or(""),
                })
            }).collect::<Vec<Value>>();
            
            Ok(json!({
                "total": total,
                "results": results,
                "consume_quota": response_json["data"]["consume_quota"].as_str().unwrap_or(""),
                "rest_quota": response_json["data"]["rest_quota"].as_str().unwrap_or(""),
            }))
        },
        Err(e) => Err(e)
    }
}

// 导出资产
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
    let mut last_successful_page = 0;
    let max_retries = 3;
    let retry_delay_secs = 5;
    
    // 分页获取所有数据
    for page in 1..=pages {
        eprintln!("正在导出第 {}/{} 页...", page, pages);
        
        let mut retry_count = 0;
        let mut page_success = false;
        
        while retry_count < max_retries && !page_success {
            match search(query, page, page_size).await {
                Ok(data) => {
                    if let Some(results) = data["results"].as_array() {
                        all_results.extend(results.clone());
                        eprintln!("第 {} 页成功: 获取 {} 条数据", page, results.len());
                        last_successful_page = page;
                        page_success = true;
                    }
                }
                Err(e) => {
                    // 检查是否是配额耗尽错误
                    if e.contains("积分用完") || e.contains("次牛") || e.contains("quota") || e.contains("所有API Key都无法使用") {
                        eprintln!("Hunter: 配额耗尽，停止导出");
                        
                        // 保存部分导出的数据
                        if !all_results.is_empty() {
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let file_path = format!("{}/hunter_export_{}_partial_{}of{}_pages.csv", 
                                export_path, timestamp, last_successful_page, pages);
                            
                            save_to_csv(&file_path, &all_results)?;
                            eprintln!("已保存部分数据到: {}", file_path);
                        }
                        
                        return Err(format!("第{}页查询失败(配额耗尽): {}。已保存前{}页数据", page, e, last_successful_page));
                    }
                    
                    // 其他错误，尝试重试
                    retry_count += 1;
                    if retry_count < max_retries {
                        eprintln!("第 {} 页失败，{} 秒后重试 ({}/{})...", page, retry_delay_secs, retry_count, max_retries);
                        tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                    } else {
                        eprintln!("第 {} 页失败，已达到最大重试次数", page);
                        
                        // 保存部分导出的数据
                        if !all_results.is_empty() {
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let file_path = format!("{}/hunter_export_{}_partial_{}of{}_pages.csv", 
                                export_path, timestamp, last_successful_page, pages);
                            
                            save_to_csv(&file_path, &all_results)?;
                            eprintln!("已保存部分数据到: {}", file_path);
                        }
                        
                        return Err(format!("第{}页查询失败: {}。已保存前{}页数据", page, e, last_successful_page));
                    }
                }
            }
        }
        
        // 避免请求过快，增加延迟到2秒
        if page < pages {
            eprintln!("等待 2 秒后继续...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    // 生成文件名
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/hunter_export_{}.csv", export_path, timestamp);
    
    save_to_csv(&file_path, &all_results)?;
    eprintln!("导出完成: {}", file_path);
    
    Ok(())
}

// 保存数据到CSV文件
fn save_to_csv(file_path: &str, results: &[Value]) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // 写入CSV头部
    wtr.write_record(&[
        "IP", "端口", "域名", "标题", "状态码", "服务器", "协议", "基础协议",
        "操作系统", "国家", "省份", "城市", "公司", "备案号", "ISP", "AS组织",
        "组件", "更新时间", "URL"
    ]).map_err(|e| format!("写入CSV头部失败: {}", e))?;

    // 写入数据
    for result in results {
        wtr.write_record(&[
            result["ip"].as_str().unwrap_or(""),
            &result["port"].as_str().map(|s| s.to_string())
                .unwrap_or_else(|| result["port"].as_i64().unwrap_or(0).to_string()),
            result["domain"].as_str().unwrap_or(""),
            result["web_title"].as_str().unwrap_or(""),
            &result["status_code"].as_i64().unwrap_or(0).to_string(),
            result["server"].as_str().unwrap_or(""),
            result["protocol"].as_str().unwrap_or(""),
            result["base_protocol"].as_str().unwrap_or(""),
            result["os"].as_str().unwrap_or(""),
            result["country"].as_str().unwrap_or(""),
            result["province"].as_str().unwrap_or(""),
            result["city"].as_str().unwrap_or(""),
            result["company"].as_str().unwrap_or(""),
            result["number"].as_str().unwrap_or(""),
            result["isp"].as_str().unwrap_or(""),
            result["as_org"].as_str().unwrap_or(""),
            result["component"].as_str().unwrap_or(""),
            result["updated_at"].as_str().unwrap_or(""),
            result["url"].as_str().unwrap_or(""),
        ]).map_err(|e| format!("写入数据失败: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("刷新CSV写入失败: {}", e))?;
    Ok(())
}

// 导出全部资产
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
    
    // 导出所有页
    export(query, export_pages, page_size, time_range, start_date, end_date, export_path).await
}

// 验证API密钥
pub async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    eprintln!("=== Hunter API密钥验证 ===");
    eprintln!("API Key: {}...", &api_key[..8.min(api_key.len())]);
    
    // Hunter API 没有单独的用户信息接口，使用搜索接口验证
    // 使用一个简单的查询来验证API密钥并获取配额信息
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    // 使用一个简单的查询：domain="test.com"
    let test_query = "domain=\"test.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(test_query.as_bytes());
    
    // 构建请求参数
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "1"),
        ("is_web", "3"),
    ];
    
    eprintln!("请求URL: {}", base_url);
    eprintln!("测试查询: {}", test_query);
    
    // 发送请求
    let client = Client::new();
    let response = client.get(base_url)
        .query(&params)
        .send()
        .await
        .map_err(|e| {
            eprintln!("✗ 网络请求失败: {}", e);
            format!("请求失败: {}", e)
        })?;
    
    // 检查响应状态
    let status = response.status();
    eprintln!("HTTP状态码: {}", status);
    
    if !status.is_success() {
        let error_msg = format!("API返回错误状态码: {}", status);
        eprintln!("✗ {}", error_msg);
        
        // 尝试读取错误响应内容
        if let Ok(text) = response.text().await {
            eprintln!("错误响应: {}", text);
        }
        
        return Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(error_msg),
            quota: None,
        });
    }
    
    // 解析响应
    let response_text = response.text().await
        .map_err(|e| {
            eprintln!("✗ 读取响应失败: {}", e);
            format!("读取响应失败: {}", e)
        })?;
    
    eprintln!("响应内容: {}", &response_text[..500.min(response_text.len())]);
    
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            eprintln!("✗ 解析JSON失败: {}", e);
            format!("解析JSON失败: {}", e)
        })?;
    
    // 检查API返回的状态码
    let code = response_json["code"].as_u64().unwrap_or(0);
    eprintln!("业务状态码: {}", code);
    
    if code == 200 {
        // API密钥有效
        eprintln!("✓ API密钥验证成功");
        let data = &response_json["data"];
        
        // 提取配额信息
        let rest_quota = data["rest_quota"].as_str().unwrap_or("未知");
        let consume_quota = data["consume_quota"].as_str().unwrap_or("");
        
        eprintln!("剩余积分: {}", rest_quota);
        eprintln!("消耗积分: {}", consume_quota);
        
        // 提取账户类型（如果有）
        if let Some(account_type) = data["account_type"].as_str() {
            eprintln!("账户类型: {}", account_type);
        }
        
        Ok(ApiKeyValidationResult {
            valid: true,
            message: Some("API密钥验证成功".to_string()),
            quota: Some(rest_quota.to_string()),
        })
    } else {
        // API密钥无效
        eprintln!("✗ API密钥验证失败");
        let message = response_json["message"].as_str()
            .or_else(|| response_json["msg"].as_str())
            .unwrap_or("API密钥验证失败")
            .to_string();
        
        eprintln!("错误信息: {}", message);
        
        Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(message),
            quota: None,
        })
    }
}
