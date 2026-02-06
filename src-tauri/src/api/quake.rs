use reqwest::Client;
use serde_json::{json, Value};
use crate::config;
use crate::ApiKeyValidationResult;
use super::key_manager;

// 使用单个API key进行搜索
async fn search_with_key(api_key: &str, query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    let base_url = "https://quake.360.net/api/v3/search/quake_service";
    
    // 构建请求体
    let request_body = json!({
        "query": query,
        "start": (page - 1) * page_size,
        "size": page_size,
        "include": ["ip", "port", "hostname", "domain", "title", "country", "province", "city", "service"]
    });
    
    // 发送请求
    let client = Client::new();
    let response = client.post(base_url)
        .header("X-QuakeToken", api_key)
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
    if response_json["code"].as_i64() != Some(0) {
        return Err(format!("API返回错误: {}", 
            response_json["message"].as_str().unwrap_or("未知错误")));
    }
    
    Ok(response_json)
}

// 搜索资产 - 真实实现
pub async fn search(query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    // 获取所有API密钥
    let api_keys = config::get_all_quake_api_keys()?;
    
    if api_keys.is_empty() {
        return Err("未配置Quake API密钥".to_string());
    }
    
    // Clone data for the closure
    let query = query.to_string();
    
    // 使用key_manager进行智能轮询
    let result = key_manager::execute_with_key_rotation(
        "quake",
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
            let meta = &response_json["meta"];
            let total = meta["pagination"]["total"].as_u64().unwrap_or(0);
            let data = response_json["data"].as_array().unwrap_or(&Vec::new()).clone();
            
            // 格式化结果
            let results = data.iter().map(|item| {
                let service = &item["service"];
                let location = &item["location"];
                
                json!({
                    "ip": item["ip"].as_str().unwrap_or(""),
                    "port": service["port"].as_i64().unwrap_or(0),
                    "hostname": item["hostname"].as_str().unwrap_or(""),
                    "domain": item["domain"].as_str().unwrap_or(""),
                    "web_title": service["http"]["title"].as_str().unwrap_or(""),
                    "server": service["http"]["server"].as_str().unwrap_or(""),
                    "country": location["country_cn"].as_str().unwrap_or(""),
                    "province": location["province_cn"].as_str().unwrap_or(""),
                    "city": location["city_cn"].as_str().unwrap_or(""),
                    "service_name": service["name"].as_str().unwrap_or(""),
                    "url": format!("{}://{}:{}", 
                        service["name"].as_str().unwrap_or("http"),
                        item["ip"].as_str().unwrap_or(""),
                        service["port"].as_i64().unwrap_or(80)
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

// 导出资产 - 真实实现
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
                    if e.contains("积分") || e.contains("quota") || e.contains("所有API Key都无法使用") {
                        eprintln!("Quake: 配额耗尽，停止导出");
                        
                        // 保存部分导出的数据
                        if !all_results.is_empty() {
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let file_path = format!("{}/quake_export_{}_partial_{}of{}_pages.csv", 
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
                            let file_path = format!("{}/quake_export_{}_partial_{}of{}_pages.csv", 
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
    let file_path = format!("{}/quake_export_{}.csv", export_path, timestamp);
    
    save_to_csv(&file_path, &all_results)?;
    eprintln!("导出完成: {}", file_path);
    
    Ok(())
}

// 保存数据到CSV文件
fn save_to_csv(file_path: &str, results: &[Value]) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // 写入CSV头部
    wtr.write_record(&["IP", "端口", "域名", "标题", "服务", "国家", "省份", "城市", "URL"])
        .map_err(|e| format!("写入CSV头部失败: {}", e))?;

    // 写入数据
    for result in results {
        wtr.write_record(&[
            result["ip"].as_str().unwrap_or(""),
            &result["port"].as_i64().unwrap_or(0).to_string(),
            result["domain"].as_str().unwrap_or(""),
            result["web_title"].as_str().unwrap_or(""),
            result["service_name"].as_str().unwrap_or(""),
            result["country"].as_str().unwrap_or(""),
            result["province"].as_str().unwrap_or(""),
            result["city"].as_str().unwrap_or(""),
            result["url"].as_str().unwrap_or(""),
        ]).map_err(|e| format!("写入数据失败: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("刷新CSV写入失败: {}", e))?;
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
pub async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    // Quake API 用户信息接口
    let base_url = "https://quake.360.net/api/v3/user/info";
    
    // 发送请求
    let client = Client::new();
    let response = client.get(base_url)
        .header("X-QuakeToken", api_key)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(format!("API返回错误状态码: {}", response.status())),
            quota: None,
        });
    }
    
    // 解析响应
    let response_text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    // 检查API返回的状态码
    let code = response_json["code"].as_i64().unwrap_or(-1);
    
    if code == 0 {
        // API密钥有效
        let data = &response_json["data"];
        
        // 提取用户信息
        let user = data["user"].as_object();
        let credit = data["credit"].as_object();
        
        // 构建配额信息
        let quota_info = if let (Some(_user_info), Some(credit_info)) = (user, credit) {
            let month_remaining = credit_info.get("month_remaining_credit")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let constant = credit_info.get("constant_credit")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let total = month_remaining + constant;
            
            format!("剩余积分: {}", total)
        } else {
            "无法获取配额信息".to_string()
        };
        
        Ok(ApiKeyValidationResult {
            valid: true,
            message: Some("API密钥验证成功".to_string()),
            quota: Some(quota_info),
        })
    } else {
        // API密钥无效
        let message = response_json["message"].as_str()
            .unwrap_or("API密钥无效")
            .to_string();
        
        Ok(ApiKeyValidationResult {
            valid: false,
            message: Some(message),
            quota: None,
        })
    }
} 