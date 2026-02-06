pub mod hunter;
pub mod fofa;
pub mod quake;
pub mod daydaymap;
pub mod key_manager;

use serde_json::Value;
use std::path::Path;

// 导出所有平台的资产
pub async fn export_all_platforms(
    query: &str,
    pages: u32,
    page_size: u32,
    time_range: &str,
    start_date: Option<String>,
    end_date: Option<String>,
    export_path: &str,
) -> Result<(), String> {
    // 为每个平台创建适配的查询语句
    let hunter_query = adapt_query_for_platform(query, "hunter", time_range, &start_date, &end_date)?;
    let fofa_query = adapt_query_for_platform(query, "fofa", time_range, &start_date, &end_date)?;
    let quake_query = adapt_query_for_platform(query, "quake", time_range, &start_date, &end_date)?;
    let daydaymap_query = adapt_query_for_platform(query, "daydaymap", time_range, &start_date, &end_date)?;
    
    // 并行查询所有平台
    let mut all_results: Vec<Value> = Vec::new();
    
    // Hunter查询
    let hunter_results = match hunter::search(&hunter_query, 1, pages * page_size).await {
        Ok(results) => {
            if let Some(results_array) = results["results"].as_array() {
                results_array.iter().map(|r| {
                    let mut result = r.clone();
                    if let Value::Object(obj) = &mut result {
                        obj.insert("platform".to_string(), Value::String("hunter".to_string()));
                    }
                    result
                }).collect()
            } else {
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };
    all_results.extend(hunter_results);
    
    // FOFA查询
    let fofa_results = match fofa::search(&fofa_query, 1, pages * page_size).await {
        Ok(results) => {
            if let Some(results_array) = results["results"].as_array() {
                results_array.iter().map(|r| {
                    let mut result = r.clone();
                    if let Value::Object(obj) = &mut result {
                        obj.insert("platform".to_string(), Value::String("fofa".to_string()));
                    }
                    result
                }).collect()
            } else {
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };
    all_results.extend(fofa_results);
    
    // Quake查询
    let quake_results = match quake::search(&quake_query, 1, pages * page_size).await {
        Ok(results) => {
            if let Some(results_array) = results["results"].as_array() {
                results_array.iter().map(|r| {
                    let mut result = r.clone();
                    if let Value::Object(obj) = &mut result {
                        obj.insert("platform".to_string(), Value::String("quake".to_string()));
                    }
                    result
                }).collect()
            } else {
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };
    all_results.extend(quake_results);
    
    // DayDayMap查询
    let daydaymap_results = match daydaymap::search(&daydaymap_query, 1, pages * page_size).await {
        Ok(results) => {
            if let Some(results_array) = results["results"].as_array() {
                results_array.iter().map(|r| {
                    let mut result = r.clone();
                    if let Value::Object(obj) = &mut result {
                        obj.insert("platform".to_string(), Value::String("daydaymap".to_string()));
                    }
                    result
                }).collect()
            } else {
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    };
    all_results.extend(daydaymap_results);
    
    // 导出结果到CSV
    if all_results.is_empty() {
        return Err("未找到任何结果".to_string());
    }
    
    // 确保导出目录存在
    let export_dir = Path::new(export_path);
    if !export_dir.exists() {
        std::fs::create_dir_all(export_dir).map_err(|e| e.to_string())?;
    }
    
    // 生成导出文件名
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let export_file = export_dir.join(format!("all_platforms_export_{}.csv", timestamp));
    
    // 写入CSV文件
    let mut writer = csv::Writer::from_path(export_file).map_err(|e| e.to_string())?;
    
    // 获取所有可能的字段
    let mut all_fields = std::collections::HashSet::new();
    for result in &all_results {
        if let Some(obj) = result.as_object() {
            for key in obj.keys() {
                all_fields.insert(key.clone());
            }
        }
    }
    
    // 写入CSV头
    let fields: Vec<String> = all_fields.into_iter().collect();
    writer.write_record(&fields).map_err(|e| e.to_string())?;
    
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
        writer.write_record(&record).map_err(|e| e.to_string())?;
    }
    
    writer.flush().map_err(|e| e.to_string())?;
    
    Ok(())
}

// 为不同平台适配查询语句
fn adapt_query_for_platform(
    query: &str,
    platform: &str,
    time_range: &str,
    start_date: &Option<String>,
    end_date: &Option<String>,
) -> Result<String, String> {
    // 解析查询语句中的条件
    let mut conditions = parse_query_conditions(query, platform)?;
    
    // 添加时间范围条件
    if time_range != "all" {
        match platform {
            "hunter" => {
                if time_range != "custom" {
                    let days = time_range.replace("d", "");
                    conditions.push(format!("time<=\"{}d\"", days));
                } else if let (Some(start), Some(end)) = (start_date, end_date) {
                    conditions.push(format!("time>=\"{}\" && time<=\"{}\"", start, end));
                }
            },
            "fofa" => {
                if time_range != "custom" {
                    let days = time_range.replace("d", "");
                    conditions.push(format!("before=\"{}d\"", days));
                } else if let (Some(start), Some(end)) = (start_date, end_date) {
                    conditions.push(format!("before=\"{}\" && after=\"{}\"", end, start));
                }
            },
            "quake" => {
                if time_range != "custom" {
                    let days = time_range.replace("d", "");
                    conditions.push(format!("time: [now-{}d TO now]", days));
                } else if let (Some(start), Some(end)) = (start_date, end_date) {
                    conditions.push(format!("time: [\"{start}\" TO \"{end}\"]"));
                }
            },
            "daydaymap" => {
                if time_range != "custom" {
                    let days = time_range.replace("d", "");
                    conditions.push(format!("time:\"{}d\"", days));
                } else if let (Some(start), Some(end)) = (start_date, end_date) {
                    conditions.push(format!("time:[{start} TO {end}]"));
                }
            },
            _ => return Err("不支持的平台".to_string()),
        }
    }
    
    // 根据平台组合条件
    let operator = match platform {
        "hunter" => " && ",
        "fofa" => " && ",
        "quake" => " AND ",
        "daydaymap" => " AND ",
        _ => return Err("不支持的平台".to_string()),
    };
    
    Ok(conditions.join(operator))
}

// 解析查询语句中的条件
fn parse_query_conditions(query: &str, target_platform: &str) -> Result<Vec<String>, String> {
    // 识别当前查询语句的平台
    let source_platform = detect_query_platform(query)?;
    
    // 如果源平台和目标平台相同，直接返回原始查询
    if source_platform == target_platform {
        // 分割条件
        let operator = match source_platform {
            "hunter" => " && ",
            "fofa" => " && ",
            "quake" => " AND ",
            "daydaymap" => " AND ",
            _ => return Err("不支持的平台".to_string()),
        };
        
        return Ok(query.split(operator).map(|s| s.trim().to_string()).collect());
    }
    
    // 解析查询条件
    let mut conditions = Vec::new();
    
    // 根据源平台的语法分割条件
    let operator = match source_platform {
        "hunter" => " && ",
        "fofa" => " && ",
        "quake" => " AND ",
        "daydaymap" => " AND ",
        _ => return Err("不支持的平台".to_string()),
    };
    
    for condition in query.split(operator) {
        let condition = condition.trim();
        
        // 转换条件到目标平台的语法
        let converted = convert_condition(condition, source_platform, target_platform)?;
        if !converted.is_empty() {
            conditions.push(converted);
        }
    }
    
    Ok(conditions)
}

// 检测查询语句的平台
fn detect_query_platform(query: &str) -> Result<&str, String> {
    if query.contains("domain.suffix=") || query.contains("web.title=") {
        Ok("hunter")
    } else if query.contains("domain=") && query.contains("&&") {
        Ok("fofa")
    } else if query.contains("domain:") && query.contains("AND") {
        Ok("quake")
    } else if query.contains("domain:\"") && query.contains("AND") {
        Ok("daydaymap")
    } else {
        // 默认假设为Hunter语法
        Ok("hunter")
    }
}

// 转换条件到目标平台的语法
fn convert_condition(condition: &str, from_platform: &str, to_platform: &str) -> Result<String, String> {
    // 解析条件类型
    if condition.is_empty() {
        return Ok(String::new());
    }
    
    // 域名条件转换
    if condition.contains("domain") {
        match (from_platform, to_platform) {
            ("hunter", "fofa") => {
                if condition.starts_with("domain.suffix=") {
                    return Ok(condition.replace("domain.suffix=", "domain="));
                }
            },
            ("hunter", "quake") => {
                if condition.starts_with("domain.suffix=") {
                    return Ok(condition.replace("domain.suffix=", "domain: "));
                }
            },
            ("hunter", "daydaymap") => {
                if condition.starts_with("domain.suffix=") {
                    return Ok(condition.replace("domain.suffix=", "domain:"));
                }
            },
            ("fofa", "hunter") => {
                if condition.starts_with("domain=") {
                    return Ok(condition.replace("domain=", "domain.suffix="));
                }
            },
            ("fofa", "quake") => {
                if condition.starts_with("domain=") {
                    return Ok(condition.replace("domain=", "domain: "));
                }
            },
            ("fofa", "daydaymap") => {
                if condition.starts_with("domain=") {
                    return Ok(condition.replace("domain=", "domain:"));
                }
            },
            ("quake", "hunter") => {
                if condition.starts_with("domain:") {
                    return Ok(condition.replace("domain:", "domain.suffix="));
                }
            },
            ("quake", "fofa") => {
                if condition.starts_with("domain:") {
                    return Ok(condition.replace("domain:", "domain="));
                }
            },
            ("quake", "daydaymap") => {
                if condition.starts_with("domain:") {
                    // 已经是相似的语法，只需要调整引号
                    return Ok(condition.replace("domain: ", "domain:"));
                }
            },
            ("daydaymap", "hunter") => {
                if condition.starts_with("domain:") {
                    return Ok(condition.replace("domain:", "domain.suffix="));
                }
            },
            ("daydaymap", "fofa") => {
                if condition.starts_with("domain:") {
                    return Ok(condition.replace("domain:", "domain="));
                }
            },
            ("daydaymap", "quake") => {
                if condition.starts_with("domain:") {
                    return Ok(condition.replace("domain:", "domain: "));
                }
            },
            _ => {}
        }
    }
    
    // IP条件转换
    if condition.contains("ip") {
        match (from_platform, to_platform) {
            ("hunter", "fofa") => {
                if condition.starts_with("ip=") {
                    return Ok(condition.to_string());
                } else if condition.starts_with("ip.province=") {
                    return Ok(condition.replace("ip.province=", "region="));
                } else if condition.starts_with("ip.city=") {
                    return Ok(condition.replace("ip.city=", "city="));
                }
            },
            ("hunter", "quake") => {
                if condition.starts_with("ip=") {
                    return Ok(condition.replace("ip=", "ip: "));
                } else if condition.starts_with("ip.province=") {
                    return Ok(condition.replace("ip.province=", "province: "));
                } else if condition.starts_with("ip.city=") {
                    return Ok(condition.replace("ip.city=", "city: "));
                }
            },
            ("hunter", "daydaymap") => {
                if condition.starts_with("ip=") {
                    return Ok(condition.replace("ip=", "ip:"));
                } else if condition.starts_with("ip.province=") {
                    return Ok(condition.replace("ip.province=", "province:"));
                } else if condition.starts_with("ip.city=") {
                    return Ok(condition.replace("ip.city=", "city:"));
                }
            },
            // 其他平台的转换类似...
            _ => {}
        }
    }
    
    // 标题条件转换
    if condition.contains("title") {
        match (from_platform, to_platform) {
            ("hunter", "fofa") => {
                if condition.starts_with("web.title=") {
                    return Ok(condition.replace("web.title=", "title="));
                }
            },
            ("hunter", "quake") => {
                if condition.starts_with("web.title=") {
                    return Ok(condition.replace("web.title=", "title: "));
                }
            },
            ("hunter", "daydaymap") => {
                if condition.starts_with("web.title=") {
                    return Ok(condition.replace("web.title=", "title:"));
                }
            },
            // 其他平台的转换类似...
            _ => {}
        }
    }
    
    // 端口条件转换
    if condition.contains("port") {
        match (from_platform, to_platform) {
            ("hunter", "fofa") => {
                if condition.starts_with("port=") {
                    return Ok(condition.to_string());
                }
            },
            ("hunter", "quake") => {
                if condition.starts_with("port=") {
                    let port = condition.replace("port=", "").replace("\"", "");
                    return Ok(format!("port: {}", port));
                }
            },
            ("hunter", "daydaymap") => {
                if condition.starts_with("port=") {
                    return Ok(condition.replace("port=", "port:"));
                }
            },
            // 其他平台的转换类似...
            _ => {}
        }
    }
    
    // 如果无法转换，返回空字符串
    Ok(String::new())
} 