// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod config;
mod utils;
mod converter;
mod error;
mod history;

use serde::{Deserialize, Serialize};
use tauri::api::dialog;
use tauri::{AppHandle, Window};
use converter::QueryConverter;
use config::ConfigManager;
use std::path::PathBuf;

// 获取配置文件路径的辅助函数
fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    // 在开发模式下，使用src-tauri目录下的config.json
    // 在生产模式下，使用resource目录下的config.json
    let resource_path = app_handle
        .path_resolver()
        .resolve_resource("config.json")
        .ok_or_else(|| "无法找到配置文件".to_string())?;
    
    Ok(resource_path)
}

// 设置结构体
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Settings {
    export_path: String,
    default_platform: String,
    page_size: u32,
    auto_validate_api_keys: bool,
    theme: String,
    language: String,
}

// API密钥验证结果
#[derive(Serialize, Deserialize)]
pub struct ApiKeyValidationResult {
    pub valid: bool,
    pub message: Option<String>,
    pub quota: Option<String>,
}

// 进度事件结构体
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub task_id: String,
    pub percent: f64,
    pub status: String,       // "running" | "success" | "error" | "cancelled"
    pub status_text: String,
    pub log_message: Option<String>,
    pub log_type: Option<String>, // "info" | "success" | "error" | "warning"
    pub current_page: Option<u32>,
    pub total_pages: Option<u32>,
    pub total_results: Option<u64>,
    pub fetched_results: Option<u64>,
}

// 发送进度事件的辅助函数
fn emit_progress(window: &Window, event: &ProgressEvent) {
    let _ = window.emit("export-progress", event);
}

// 搜索资产
#[tauri::command]
async fn search_assets(
    platform: String,
    query: String,
    page: u32,
    page_size: u32,
) -> Result<serde_json::Value, String> {
    // 添加调试日志
    eprintln!("=== search_assets 调用 ===");
    eprintln!("平台: {}", platform);
    eprintln!("查询: {}", query);
    eprintln!("页码: {}", page);
    eprintln!("每页数量: {}", page_size);
    
    let result = match platform.as_str() {
        "hunter" => api::hunter::search(&query, page, page_size).await,
        "fofa" => api::fofa::search(&query, page, page_size).await,
        "quake" => api::quake::search(&query, page, page_size).await,
        "daydaymap" => api::daydaymap::search(&query, page, page_size).await,
        _ => Err("不支持的平台".to_string()),
    };
    
    // 保存历史记录
    match &result {
        Ok(data) => {
            // 提取结果数量
            let results_count = data["total"].as_u64().unwrap_or(0);
            
            // 保存成功的查询记录
            if let Err(e) = history::add_history(
                platform.clone(),
                query.clone(),
                results_count,
                true,
                None,
            ) {
                eprintln!("保存历史记录失败: {}", e);
            }
        }
        Err(error_msg) => {
            // 保存失败的查询记录
            if let Err(e) = history::add_history(
                platform.clone(),
                query.clone(),
                0,
                false,
                Some(error_msg.clone()),
            ) {
                eprintln!("保存历史记录失败: {}", e);
            }
        }
    }
    
    result
}

// 导出当前查询结果（带进度事件）
#[tauri::command]
async fn export_results_with_progress(
    window: Window,
    task_id: String,
    platform: String,
    query: String,
    pages: u32,
    page_size: u32,
    _time_range: String,
    _start_date: Option<String>,
    _end_date: Option<String>,
) -> Result<String, String> {
    let export_path = config::get_export_path()?;

    // 发送开始事件
    emit_progress(&window, &ProgressEvent {
        task_id: task_id.clone(),
        percent: 0.0,
        status: "running".to_string(),
        status_text: format!("正在准备导出 [{}] ...", platform),
        log_message: Some(format!("开始导出: 平台={}, 页数={}, 每页={}", platform, pages, page_size)),
        log_type: Some("info".to_string()),
        current_page: Some(0),
        total_pages: Some(pages),
        total_results: None,
        fetched_results: Some(0),
    });

    let mut all_results = Vec::new();
    let max_retries = 3;
    let retry_delay_secs = 5;

    for page in 1..=pages {
        let pct = ((page - 1) as f64 / pages as f64) * 100.0;
        emit_progress(&window, &ProgressEvent {
            task_id: task_id.clone(),
            percent: pct,
            status: "running".to_string(),
            status_text: format!("正在获取第 {}/{} 页...", page, pages),
            log_message: Some(format!("请求第 {} 页数据...", page)),
            log_type: Some("info".to_string()),
            current_page: Some(page),
            total_pages: Some(pages),
            total_results: None,
            fetched_results: Some(all_results.len() as u64),
        });

        let mut retry_count = 0;
        let mut page_success = false;

        while retry_count < max_retries && !page_success {
            let result = match platform.as_str() {
                "hunter" => api::hunter::search(&query, page, page_size).await,
                "fofa" => api::fofa::search(&query, page, page_size).await,
                "quake" => api::quake::search(&query, page, page_size).await,
                "daydaymap" => api::daydaymap::search(&query, page, page_size).await,
                _ => Err("不支持的平台".to_string()),
            };

            match result {
                Ok(data) => {
                    if let Some(results) = data["results"].as_array() {
                        all_results.extend(results.clone());
                        emit_progress(&window, &ProgressEvent {
                            task_id: task_id.clone(),
                            percent: (page as f64 / pages as f64) * 100.0,
                            status: "running".to_string(),
                            status_text: format!("第 {}/{} 页完成，已获取 {} 条数据", page, pages, all_results.len()),
                            log_message: Some(format!("✓ 第 {} 页成功: {} 条", page, results.len())),
                            log_type: Some("success".to_string()),
                            current_page: Some(page),
                            total_pages: Some(pages),
                            total_results: data["total"].as_u64(),
                            fetched_results: Some(all_results.len() as u64),
                        });
                        page_success = true;
                    }
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count < max_retries {
                        emit_progress(&window, &ProgressEvent {
                            task_id: task_id.clone(),
                            percent: pct,
                            status: "running".to_string(),
                            status_text: format!("第 {} 页失败，正在重试 ({}/{})...", page, retry_count, max_retries),
                            log_message: Some(format!("⚠ 第 {} 页失败: {}，重试中...", page, e)),
                            log_type: Some("warning".to_string()),
                            current_page: Some(page),
                            total_pages: Some(pages),
                            total_results: None,
                            fetched_results: Some(all_results.len() as u64),
                        });
                        tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                    } else {
                        emit_progress(&window, &ProgressEvent {
                            task_id: task_id.clone(),
                            percent: pct,
                            status: "error".to_string(),
                            status_text: format!("第 {} 页失败，已达最大重试次数", page),
                            log_message: Some(format!("✗ 第 {} 页最终失败: {}", page, e)),
                            log_type: Some("error".to_string()),
                            current_page: Some(page),
                            total_pages: Some(pages),
                            total_results: None,
                            fetched_results: Some(all_results.len() as u64),
                        });
                        // 如果有部分数据，仍然保存
                        if !all_results.is_empty() {
                            break;
                        }
                        return Err(format!("导出失败: {}", e));
                    }
                }
            }
        }

        // 页间延迟
        if page < pages && page_success {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    if all_results.is_empty() {
        emit_progress(&window, &ProgressEvent {
            task_id: task_id.clone(),
            percent: 100.0,
            status: "error".to_string(),
            status_text: "未获取到任何数据".to_string(),
            log_message: Some("✗ 导出失败: 无数据".to_string()),
            log_type: Some("error".to_string()),
            current_page: None,
            total_pages: Some(pages),
            total_results: Some(0),
            fetched_results: Some(0),
        });
        return Err("未获取到任何数据".to_string());
    }

    // 保存CSV
    emit_progress(&window, &ProgressEvent {
        task_id: task_id.clone(),
        percent: 95.0,
        status: "running".to_string(),
        status_text: "正在保存CSV文件...".to_string(),
        log_message: Some(format!("正在写入 {} 条数据到CSV...", all_results.len())),
        log_type: Some("info".to_string()),
        current_page: None,
        total_pages: Some(pages),
        total_results: Some(all_results.len() as u64),
        fetched_results: Some(all_results.len() as u64),
    });

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/{}_export_{}.csv", export_path, platform, timestamp);

    // 使用各平台的 save_to_csv 或通用方式
    save_results_to_csv(&file_path, &all_results)?;

    emit_progress(&window, &ProgressEvent {
        task_id: task_id.clone(),
        percent: 100.0,
        status: "success".to_string(),
        status_text: format!("导出完成！共 {} 条数据", all_results.len()),
        log_message: Some(format!("✓ 文件已保存: {}", file_path)),
        log_type: Some("success".to_string()),
        current_page: Some(pages),
        total_pages: Some(pages),
        total_results: Some(all_results.len() as u64),
        fetched_results: Some(all_results.len() as u64),
    });

    Ok(file_path)
}

// 通用CSV保存函数
fn save_results_to_csv(file_path: &str, results: &[serde_json::Value]) -> Result<(), String> {
    let export_dir = std::path::Path::new(file_path).parent()
        .ok_or_else(|| "无效的文件路径".to_string())?;
    if !export_dir.exists() {
        std::fs::create_dir_all(export_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut wtr = csv::Writer::from_path(file_path)
        .map_err(|e| format!("创建CSV文件失败: {}", e))?;

    // 收集所有字段
    let mut fields = Vec::new();
    if let Some(first) = results.first() {
        if let Some(obj) = first.as_object() {
            fields = obj.keys().cloned().collect();
        }
    }

    wtr.write_record(&fields).map_err(|e| format!("写入CSV头失败: {}", e))?;

    for result in results {
        let record: Vec<String> = fields.iter().map(|f| {
            if let Some(v) = result.get(f) {
                if v.is_string() { v.as_str().unwrap_or("").to_string() }
                else { v.to_string() }
            } else { String::new() }
        }).collect();
        wtr.write_record(&record).map_err(|e| format!("写入数据失败: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("保存CSV失败: {}", e))?;
    Ok(())
}

// 导出当前查询结果
#[tauri::command]
async fn export_results(
    platform: String,
    query: String,
    pages: u32,
    page_size: u32,
    time_range: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<(), String> {
    let export_path = config::get_export_path()?;
    
    match platform.as_str() {
        "hunter" => {
            api::hunter::export(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "fofa" => {
            api::fofa::export(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "quake" => {
            api::quake::export(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "daydaymap" => {
            api::daydaymap::export(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        _ => Err("不支持的平台".to_string()),
    }
}

// 导出平台全部资产
#[tauri::command]
async fn export_platform_all(
    platform: String,
    query: String,
    pages: u32,
    page_size: u32,
    time_range: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<(), String> {
    let export_path = config::get_export_path()?;
    
    match platform.as_str() {
        "hunter" => {
            api::hunter::export_all(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "fofa" => {
            api::fofa::export_all(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "quake" => {
            api::quake::export_all(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        "daydaymap" => {
            api::daydaymap::export_all(&query, pages, page_size, &time_range, start_date, end_date, &export_path).await
        }
        _ => Err("不支持的平台".to_string()),
    }
}

// 导出所有平台资产
#[tauri::command]
async fn export_all_platforms(
    query: String,
    pages: u32,
    page_size: u32,
    time_range: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<(), String> {
    let export_path = config::get_export_path()?;
    
    api::export_all_platforms(
        &query, 
        pages, 
        page_size, 
        &time_range, 
        start_date, 
        end_date, 
        &export_path
    ).await
}

// 获取API密钥
#[tauri::command]
fn get_api_keys(platform: String) -> Result<serde_json::Value, String> {
    match platform.as_str() {
        "hunter" => config::get_hunter_api_keys(),
        "fofa" => config::get_fofa_api_keys(),
        "quake" => config::get_quake_api_keys(),
        "daydaymap" => config::get_daydaymap_api_keys(),
        _ => Err("不支持的平台".to_string()),
    }
}

// 添加API密钥
#[tauri::command]
async fn add_api_key(platform: String, api_key: String, email: Option<String>) -> Result<(), String> {
    match platform.as_str() {
        "hunter" => config::add_hunter_api_key(&api_key),
        "fofa" => {
            let email = email.ok_or("FOFA平台需要提供邮箱")?;
            config::add_fofa_api_key(&api_key, &email)
        }
        "quake" => config::add_quake_api_key(&api_key),
        "daydaymap" => config::add_daydaymap_api_key(&api_key),
        _ => Err("不支持的平台".to_string()),
    }
}

// 删除API密钥
#[tauri::command]
fn delete_api_key(platform: String, api_key: String, email: Option<String>) -> Result<(), String> {
    match platform.as_str() {
        "hunter" => config::delete_hunter_api_key(&api_key),
        "fofa" => {
            let email = email.ok_or("FOFA平台需要提供邮箱")?;
            config::delete_fofa_api_key(&api_key, &email)
        }
        "quake" => config::delete_quake_api_key(&api_key),
        "daydaymap" => config::delete_daydaymap_api_key(&api_key),
        _ => Err("不支持的平台".to_string()),
    }
}

// 验证API密钥
#[tauri::command]
async fn validate_api_key(platform: String, api_key: String, email: Option<String>) -> Result<ApiKeyValidationResult, String> {
    match platform.as_str() {
        "hunter" => api::hunter::validate_api_key(&api_key).await,
        "fofa" => {
            let email = email.ok_or("FOFA平台需要提供邮箱")?;
            api::fofa::validate_api_key(&api_key, &email).await
        }
        "quake" => api::quake::validate_api_key(&api_key).await,
        "daydaymap" => api::daydaymap::validate_api_key(&api_key).await,
        _ => Err("不支持的平台".to_string()),
    }
}

// 获取设置
#[tauri::command]
fn get_settings() -> Result<Settings, String> {
    config::get_settings()
}

// 保存设置
#[tauri::command]
fn save_settings(settings: Settings) -> Result<(), String> {
    config::save_settings(&settings)
}

// 选择目录
#[tauri::command]
async fn select_directory() -> Result<String, String> {
    let path = dialog::blocking::FileDialogBuilder::default()
        .set_title("选择导出目录")
        .set_directory("~")
        .pick_folder();
    
    match path {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("未选择目录".to_string()),
    }
}

// 转换结果结构体
#[derive(Serialize, Deserialize)]
struct ConversionResult {
    platform: String,
    query: String,
}

// 获取支持的平台列表
#[tauri::command]
fn get_supported_platforms(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let config_manager = ConfigManager::from_file(&config_path)
        .map_err(|e| format!("加载配置文件失败: {}", e))?;
    
    let converter = QueryConverter::new(config_manager);
    Ok(converter.get_supported_platforms())
}

// 转换查询语句
#[tauri::command]
fn convert_query(
    app_handle: AppHandle,
    query: String,
    from_platform: String,
    to_platform: String,
) -> Result<String, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let config_manager = ConfigManager::from_file(&config_path)
        .map_err(|e| format!("加载配置文件失败: {}", e))?;
    
    let converter = QueryConverter::new(config_manager);
    
    // 验证查询语法
    converter.validate_query_syntax(&query, &from_platform)
        .map_err(|e| format!("{}", e))?;
    
    // 执行转换
    converter.convert(&query, &from_platform, &to_platform)
        .map_err(|e| format!("{}", e))
}

// 转换查询语句到所有平台
#[tauri::command]
fn convert_query_to_all(
    app_handle: AppHandle,
    query: String,
    from_platform: String,
) -> Result<Vec<ConversionResult>, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let config_manager = ConfigManager::from_file(&config_path)
        .map_err(|e| format!("加载配置文件失败: {}", e))?;
    
    let converter = QueryConverter::new(config_manager);
    
    // 验证查询语法
    converter.validate_query_syntax(&query, &from_platform)
        .map_err(|e| format!("{}", e))?;
    
    let supported_platforms = converter.get_supported_platforms();
    let mut results = Vec::new();
    
    for platform in supported_platforms {
        if platform != from_platform {
            match converter.convert(&query, &from_platform, &platform) {
                Ok(converted_query) => {
                    results.push(ConversionResult {
                        platform: platform.clone(),
                        query: converted_query,
                    });
                }
                Err(e) => {
                    return Err(format!("转换到 {} 平台失败: {}", platform, e));
                }
            }
        }
    }
    
    Ok(results)
}

// 验证查询语法
#[tauri::command]
fn validate_query_syntax(
    app_handle: AppHandle,
    query: String,
    platform: String,
) -> Result<bool, String> {
    let config_path = get_config_path(&app_handle)?;
    
    let config_manager = ConfigManager::from_file(&config_path)
        .map_err(|e| format!("加载配置文件失败: {}", e))?;
    
    let converter = QueryConverter::new(config_manager);
    
    match converter.validate_query_syntax(&query, &platform) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("{}", e)),
    }
}

// 添加查询历史记录
#[tauri::command]
fn add_query_history(
    platform: String,
    query: String,
    results_count: u64,
    success: bool,
    error_message: Option<String>,
) -> Result<(), String> {
    history::add_history(platform, query, results_count, success, error_message)
}

// 获取所有历史记录
#[tauri::command]
fn get_query_history() -> Result<Vec<history::QueryHistory>, String> {
    history::get_all_history()
}

// 根据平台筛选历史记录
#[tauri::command]
fn get_history_by_platform(platform: String) -> Result<Vec<history::QueryHistory>, String> {
    history::get_history_by_platform(&platform)
}

// 搜索历史记录
#[tauri::command]
fn search_query_history(keyword: String) -> Result<Vec<history::QueryHistory>, String> {
    history::search_history(&keyword)
}

// 删除历史记录
#[tauri::command]
fn delete_query_history(id: String) -> Result<(), String> {
    history::delete_history(&id)
}

// 清空所有历史记录
#[tauri::command]
fn clear_all_history() -> Result<(), String> {
    history::clear_all_history()
}

// 导出历史记录
#[tauri::command]
fn export_query_history(export_path: String) -> Result<String, String> {
    history::export_history_to_csv(&export_path)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_assets,
            export_results,
            export_results_with_progress,
            export_platform_all,
            export_all_platforms,
            get_api_keys,
            add_api_key,
            delete_api_key,
            validate_api_key,
            get_settings,
            save_settings,
            select_directory,
            get_supported_platforms,
            convert_query,
            convert_query_to_all,
            validate_query_syntax,
            add_query_history,
            get_query_history,
            get_history_by_platform,
            search_query_history,
            delete_query_history,
            clear_all_history,
            export_query_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
