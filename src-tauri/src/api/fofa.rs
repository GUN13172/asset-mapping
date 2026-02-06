use reqwest::Client;
use serde_json::{json, Value};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use crate::config;
use crate::ApiKeyValidationResult;
use super::key_manager;

// 使用单个API key进行搜索
async fn search_with_key(api_key: &str, email: &str, query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    let base_url = "https://fofa.info/api/v1/search/all";
    
    // 对查询字符串进行Base64编码
    let encoded_query = general_purpose::URL_SAFE.encode(query.as_bytes());
    
    // 构建请求参数
    let params = [
        ("key", api_key.to_string()),
        ("email", email.to_string()),
        ("qbase64", encoded_query),
        ("page", page.to_string()),
        ("size", page_size.to_string()),
        ("fields", "host,ip,port,title,country,province,city,server".to_string()),
    ];
    
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
    
    // 检查API返回的错误
    if let Some(error) = response_json["error"].as_bool() {
        if error {
            return Err(format!("API返回错误: {}", response_json["errmsg"].as_str().unwrap_or("未知错误")));
        }
    }
    
    Ok(response_json)
}

// 搜索资产
pub async fn search(query: &str, page: u32, page_size: u32) -> Result<Value, String> {
    // 获取所有API密钥（包含email）
    let api_key_pairs = config::get_all_fofa_api_keys()?;
    
    if api_key_pairs.is_empty() {
        return Err("未配置FOFA API密钥".to_string());
    }
    
    // 将(key, email)对转换为字符串格式，用于key_manager
    let api_keys: Vec<String> = api_key_pairs.iter()
        .map(|(key, email)| format!("{}:{}", key, email))
        .collect();
    
    // Clone data for the closure
    let query = query.to_string();
    
    // 使用key_manager进行智能轮询
    let result = key_manager::execute_with_key_rotation(
        "fofa",
        &api_keys,
        |combined_key| {
            let query = query.clone();
            let combined_key = combined_key.to_string();
            async move {
                // 分离key和email
                let parts: Vec<&str> = combined_key.split(':').collect();
                if parts.len() != 2 {
                    return Err("API密钥格式错误".to_string());
                }
                let api_key = parts[0];
                let email = parts[1];
                
                search_with_key(api_key, email, &query, page, page_size).await
            }
        }
    ).await;
    
    match result {
        Ok(response_json) => {
            // 提取结果
            let total = response_json["size"].as_u64().unwrap_or(0);
            let results = response_json["results"].as_array().unwrap_or(&Vec::new()).clone();
            
            // 格式化结果
            let fields = ["host", "ip", "port", "web_title", "country", "province", "city", "server"];
            let formatted_results = results.iter().map(|item| {
                let empty_vec = Vec::new();
                let array = item.as_array().unwrap_or(&empty_vec);
                let mut result = json!({});
                
                for (i, field) in fields.iter().enumerate() {
                    if i < array.len() {
                        result[field] = array[i].clone();
                    }
                }
                
                // 构建URL
                if let Some(host) = result["host"].as_str() {
                    result["url"] = Value::String(host.to_string());
                }
                
                result
            }).collect::<Vec<Value>>();
            
            Ok(json!({
                "total": total,
                "results": formatted_results
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
    time_range: &str,
    start_date: Option<String>,
    end_date: Option<String>,
    export_path: &str,
) -> Result<(), String> {
    // 处理时间范围
    let mut final_query = query.to_string();
    
    if time_range != "all" {
        if time_range != "custom" {
            let days = time_range.replace("d", "");
            final_query = format!("{} && before=\"{}d\"", final_query, days);
        } else if let (Some(start), Some(end)) = (start_date, end_date) {
            final_query = format!("{} && before=\"{}\" && after=\"{}\"", final_query, end, start);
        }
    }
    
    // 查询数据
    let mut all_results = Vec::new();
    let mut last_successful_page = 0;
    let max_retries = 3;
    let retry_delay_secs = 5;
    
    for page in 1..=pages {
        eprintln!("正在导出第 {}/{} 页...", page, pages);
        
        let mut retry_count = 0;
        let mut page_success = false;
        
        while retry_count < max_retries && !page_success {
            match search(&final_query, page, page_size).await {
                Ok(result) => {
                    if let Some(results) = result["results"].as_array() {
                        all_results.extend(results.clone());
                        eprintln!("第 {} 页成功: 获取 {} 条数据", page, results.len());
                        last_successful_page = page;
                        page_success = true;
                    }
                }
                Err(e) => {
                    // 检查是否是配额耗尽错误
                    if e.contains("F币") || e.contains("quota") || e.contains("所有API Key都无法使用") {
                        eprintln!("FOFA: 配额耗尽，停止导出");
                        
                        // 保存部分导出的数据
                        if !all_results.is_empty() {
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let file_path = format!("{}/fofa_export_{}_partial_{}of{}_pages.csv", 
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
                            let file_path = format!("{}/fofa_export_{}_partial_{}of{}_pages.csv", 
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
    
    // 导出到CSV
    if all_results.is_empty() {
        return Err("未找到结果".to_string());
    }
    
    // 生成导出文件名
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/fofa_export_{}.csv", export_path, timestamp);
    
    save_to_csv(&file_path, &all_results)?;
    eprintln!("导出完成: {}", file_path);
    
    Ok(())
}

// 保存数据到CSV文件
fn save_to_csv(file_path: &str, results: &[Value]) -> Result<(), String> {
    // 确保导出目录存在
    let export_dir = Path::new(file_path).parent().unwrap();
    if !export_dir.exists() {
        std::fs::create_dir_all(export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    }
    
    // 写入CSV文件
    let mut writer = csv::Writer::from_path(file_path).map_err(|e| format!("创建CSV文件失败: {}", e))?;
    
    // 获取所有字段
    let mut fields = Vec::new();
    if let Some(first) = results.first() {
        if let Some(obj) = first.as_object() {
            fields = obj.keys().map(|k| k.clone()).collect();
        }
    }
    
    // 写入CSV头
    writer.write_record(&fields).map_err(|e| format!("写入CSV头失败: {}", e))?;
    
    // 写入数据
    for result in results {
        let mut record = Vec::new();
        for field in &fields {
            if let Some(value) = result.get(field) {
                if value.is_string() {
                    record.push(value.as_str().unwrap_or("").to_string());
                } else {
                    record.push(value.to_string());
                }
            } else {
                record.push(String::new());
            }
        }
        writer.write_record(&record).map_err(|e| format!("写入CSV数据失败: {}", e))?;
    }
    
    writer.flush().map_err(|e| format!("保存CSV文件失败: {}", e))?;
    
    Ok(())
}

// 导出全部资产
pub async fn export_all(
    query: &str,
    pages: u32,
    page_size: u32,
    time_range: &str,
    start_date: Option<String>,
    end_date: Option<String>,
    export_path: &str,
) -> Result<(), String> {
    // 处理时间范围
    let mut final_query = query.to_string();
    
    if time_range != "all" {
        if time_range != "custom" {
            let days = time_range.replace("d", "");
            final_query = format!("{} && before=\"{}d\"", final_query, days);
        } else if let (Some(start), Some(end)) = (start_date, end_date) {
            final_query = format!("{} && before=\"{}\" && after=\"{}\"", final_query, end, start);
        }
    }
    
    // 先获取总数
    let initial_result = search(&final_query, 1, 1).await?;
    let total = initial_result["total"].as_u64().unwrap_or(0) as u32;
    
    // 计算实际需要的页数
    let actual_pages = if total > 0 {
        (total + page_size - 1) / page_size
    } else {
        0
    };
    
    // 限制最大页数
    let pages_to_fetch = std::cmp::min(actual_pages, pages);
    
    // 查询数据
    let mut all_results = Vec::new();
    
    for page in 1..=pages_to_fetch {
        let result = search(&final_query, page, page_size).await?;
        
        if let Some(results) = result["results"].as_array() {
            all_results.extend(results.clone());
        }
    }
    
    // 导出到CSV
    if all_results.is_empty() {
        return Err("未找到结果".to_string());
    }
    
    // 确保导出目录存在
    let export_dir = Path::new(export_path);
    if !export_dir.exists() {
        std::fs::create_dir_all(export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    }
    
    // 生成导出文件名
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let export_file = export_dir.join(format!("fofa_all_export_{}.csv", timestamp));
    
    // 写入CSV文件
    let mut writer = csv::Writer::from_path(export_file).map_err(|e| format!("创建CSV文件失败: {}", e))?;
    
    // 获取所有字段
    let mut fields = Vec::new();
    if let Some(first) = all_results.first() {
        if let Some(obj) = first.as_object() {
            fields = obj.keys().map(|k| k.clone()).collect();
        }
    }
    
    // 写入CSV头
    writer.write_record(&fields).map_err(|e| format!("写入CSV头失败: {}", e))?;
    
    // 写入数据
    for result in all_results {
        let mut record = Vec::new();
        for field in &fields {
            if let Some(value) = result.get(field) {
                if value.is_string() {
                    record.push(value.as_str().unwrap_or("").to_string());
                } else {
                    record.push(value.to_string());
                }
            } else {
                record.push(String::new());
            }
        }
        writer.write_record(&record).map_err(|e| format!("写入CSV数据失败: {}", e))?;
    }
    
    writer.flush().map_err(|e| format!("保存CSV文件失败: {}", e))?;
    
    Ok(())
}

// 验证API密钥
pub async fn validate_api_key(api_key: &str, email: &str) -> Result<ApiKeyValidationResult, String> {
    let base_url = "https://fofa.info/api/v1/info/my";
    
    // 构建请求参数
    let params = [
        ("key", api_key),
        ("email", email),
    ];
    
    // 发送请求
    let client = Client::new();
    let response = client.get(base_url)
        .query(&params)
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
    let response_json: Value = serde_json::from_str(&response_text).map_err(|e| format!("解析JSON失败: {}", e))?;
    
    // 检查API返回的错误
    if let Some(error) = response_json["error"].as_bool() {
        if error {
            return Ok(ApiKeyValidationResult {
                valid: false,
                message: Some(response_json["errmsg"].as_str().unwrap_or("未知错误").to_string()),
                quota: None,
            });
        }
    }
    
    // 提取配额信息
    let fcoin = response_json["fcoin"].as_i64().unwrap_or(0).to_string();
    
    Ok(ApiKeyValidationResult {
        valid: true,
        message: None,
        quota: Some(format!("F币: {}", fcoin)),
    })
} 