// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod config;
mod converter;
mod error;
mod history;
mod pocs;
mod utils;

use config::ConfigManager;
use converter::QueryConverter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::api::dialog;
use tauri::{AppHandle, Window};

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
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub task_id: String,
    pub percent: f64,
    pub status: String, // "running" | "success" | "error" | "cancelled"
    pub status_text: String,
    pub log_message: Option<String>,
    pub log_type: Option<String>, // "info" | "success" | "error" | "warning"
    pub current_page: Option<u32>,
    pub total_pages: Option<u32>,
    pub total_results: Option<u64>,
    pub fetched_results: Option<u64>,
}

// 扫描配置结构体
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub targets: Vec<String>,
    pub threads: u32,
    pub timeout: u32,
    pub pcas: Option<Vec<String>>,
}

// 漏洞发现结果结构体
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityResult {
    pub target: String,
    pub poc_name: String,
    pub severity: String,
    pub matched_at: String,
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
            if let Err(e) =
                history::add_history(platform.clone(), query.clone(), results_count, true, None)
            {
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
    emit_progress(
        &window,
        &ProgressEvent {
            task_id: task_id.clone(),
            percent: 0.0,
            status: "running".to_string(),
            status_text: format!("正在准备导出 [{}] ...", platform),
            log_message: Some(format!(
                "开始导出: 平台={}, 页数={}, 每页={}",
                platform, pages, page_size
            )),
            log_type: Some("info".to_string()),
            current_page: Some(0),
            total_pages: Some(pages),
            total_results: None,
            fetched_results: Some(0),
        },
    );

    let mut all_results = Vec::new();
    let max_retries = 3;
    let retry_delay_secs = 5;

    for page in 1..=pages {
        let pct = ((page - 1) as f64 / pages as f64) * 100.0;
        emit_progress(
            &window,
            &ProgressEvent {
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
            },
        );

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
                        emit_progress(
                            &window,
                            &ProgressEvent {
                                task_id: task_id.clone(),
                                percent: (page as f64 / pages as f64) * 100.0,
                                status: "running".to_string(),
                                status_text: format!(
                                    "第 {}/{} 页完成，已获取 {} 条数据",
                                    page,
                                    pages,
                                    all_results.len()
                                ),
                                log_message: Some(format!(
                                    "✓ 第 {} 页成功: {} 条",
                                    page,
                                    results.len()
                                )),
                                log_type: Some("success".to_string()),
                                current_page: Some(page),
                                total_pages: Some(pages),
                                total_results: data["total"].as_u64(),
                                fetched_results: Some(all_results.len() as u64),
                            },
                        );
                        page_success = true;
                    }
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count < max_retries {
                        emit_progress(
                            &window,
                            &ProgressEvent {
                                task_id: task_id.clone(),
                                percent: pct,
                                status: "running".to_string(),
                                status_text: format!(
                                    "第 {} 页失败，正在重试 ({}/{})...",
                                    page, retry_count, max_retries
                                ),
                                log_message: Some(format!(
                                    "⚠ 第 {} 页失败: {}，重试中...",
                                    page, e
                                )),
                                log_type: Some("warning".to_string()),
                                current_page: Some(page),
                                total_pages: Some(pages),
                                total_results: None,
                                fetched_results: Some(all_results.len() as u64),
                            },
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs))
                            .await;
                    } else {
                        emit_progress(
                            &window,
                            &ProgressEvent {
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
                            },
                        );
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
        emit_progress(
            &window,
            &ProgressEvent {
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
            },
        );
        return Err("未获取到任何数据".to_string());
    }

    // 保存CSV
    emit_progress(
        &window,
        &ProgressEvent {
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
        },
    );

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/{}_export_{}.csv", export_path, platform, timestamp);

    // 使用各平台的 save_to_csv 或通用方式
    save_results_to_csv(&file_path, &all_results)?;

    emit_progress(
        &window,
        &ProgressEvent {
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
        },
    );

    Ok(file_path)
}

// 通用CSV保存函数
fn save_results_to_csv(file_path: &str, results: &[serde_json::Value]) -> Result<(), String> {
    let export_dir = std::path::Path::new(file_path)
        .parent()
        .ok_or_else(|| "无效的文件路径".to_string())?;
    if !export_dir.exists() {
        std::fs::create_dir_all(export_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut wtr =
        csv::Writer::from_path(file_path).map_err(|e| format!("创建CSV文件失败: {}", e))?;

    // 收集所有字段
    let mut fields = Vec::new();
    if let Some(first) = results.first() {
        if let Some(obj) = first.as_object() {
            fields = obj.keys().cloned().collect();
        }
    }

    wtr.write_record(&fields)
        .map_err(|e| format!("写入CSV头失败: {}", e))?;

    for result in results {
        let record: Vec<String> = fields
            .iter()
            .map(|f| {
                if let Some(v) = result.get(f) {
                    if v.is_string() {
                        v.as_str().unwrap_or("").to_string()
                    } else {
                        v.to_string()
                    }
                } else {
                    String::new()
                }
            })
            .collect();
        wtr.write_record(&record)
            .map_err(|e| format!("写入数据失败: {}", e))?;
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
            api::hunter::export(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "fofa" => {
            api::fofa::export(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "quake" => {
            api::quake::export(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "daydaymap" => {
            api::daydaymap::export(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
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
            api::hunter::export_all(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "fofa" => {
            api::fofa::export_all(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "quake" => {
            api::quake::export_all(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
        }
        "daydaymap" => {
            api::daydaymap::export_all(
                &query,
                pages,
                page_size,
                &time_range,
                start_date,
                end_date,
                &export_path,
            )
            .await
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
        &export_path,
    )
    .await
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
async fn add_api_key(
    platform: String,
    api_key: String,
    email: Option<String>,
) -> Result<(), String> {
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
async fn validate_api_key(
    platform: String,
    api_key: String,
    email: Option<String>,
) -> Result<ApiKeyValidationResult, String> {
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

    let config_manager =
        ConfigManager::from_file(&config_path).map_err(|e| format!("加载配置文件失败: {}", e))?;

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

    let config_manager =
        ConfigManager::from_file(&config_path).map_err(|e| format!("加载配置文件失败: {}", e))?;

    let converter = QueryConverter::new(config_manager);

    // 验证查询语法
    converter
        .validate_query_syntax(&query, &from_platform)
        .map_err(|e| format!("{}", e))?;

    // 执行转换
    converter
        .convert(&query, &from_platform, &to_platform)
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

    let config_manager =
        ConfigManager::from_file(&config_path).map_err(|e| format!("加载配置文件失败: {}", e))?;

    let converter = QueryConverter::new(config_manager);

    // 验证查询语法
    converter
        .validate_query_syntax(&query, &from_platform)
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

    let config_manager =
        ConfigManager::from_file(&config_path).map_err(|e| format!("加载配置文件失败: {}", e))?;

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

// 漏洞扫描
#[tauri::command]
async fn run_vulnerability_scan(window: Window, config: ScanConfig) -> Result<(), String> {
    let task_id = format!("scan_{}", chrono::Utc::now().timestamp());
    let targets_count = config.targets.len();

    tauri::async_runtime::spawn(async move {
        // 1. 启动阶段
        emit_progress(
            &window,
            &ProgressEvent {
                task_id: task_id.clone(),
                percent: 0.0,
                status: "running".to_string(),
                status_text: "准备启动 Nuclei 引擎...".to_string(),
                log_message: Some("正在初始化扫描环境...".to_string()),
                log_type: Some("info".to_string()),
                current_page: None,
                total_pages: None,
                total_results: None,
                fetched_results: None,
            },
        );

        for (i, target) in config.targets.iter().enumerate() {
            let progress = (i as f64 / targets_count as f64) * 100.0;

            emit_progress(
                &window,
                &ProgressEvent {
                    task_id: task_id.clone(),
                    percent: progress,
                    status: "running".to_string(),
                    status_text: format!("正在对 {} 进行漏洞探测...", target),
                    log_message: Some(format!("启动核心引擎探测目标: {}", target)),
                    log_type: Some("info".to_string()),
                    current_page: None,
                    total_pages: None,
                    total_results: None,
                    fetched_results: None,
                },
            );

            // 构造命令: nuclei -u <target> -json -timeout <timeout>
            let mut cmd = tokio::process::Command::new("nuclei");
            cmd.arg("-u").arg(&target);
            cmd.arg("-json");
            cmd.arg("-timeout").arg(config.timeout.to_string());
            cmd.arg("-c").arg(config.threads.to_string());

            // 如果指定了 POC 列表
            if let Some(pcas) = &config.pcas {
                if !pcas.is_empty() {
                    cmd.arg("-t").arg(pcas.join(","));
                }
            }

            // Debug log: Print the command
            println!("Executing command: {:?}", cmd);
            emit_progress(
                &window,
                &ProgressEvent {
                    task_id: task_id.clone(),
                    percent: progress,
                    status: "running".to_string(),
                    status_text: "正在执行...".to_string(),
                    log_message: Some(format!(
                        "执行命令: nuclei -u {} -t [{} templates] ...",
                        target,
                        config.pcas.as_ref().map(|v| v.len()).unwrap_or(0)
                    )),
                    log_type: Some("info".to_string()),
                    ..Default::default()
                },
            );

            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    emit_progress(&window, &ProgressEvent {
                        task_id: task_id.clone(),
                        percent: progress,
                        status: "running".to_string(),
                        status_text: "引擎启动失败".to_string(),
                        log_message: Some(format!("发生错误: 无法运行 nuclei 二进制文件 ({})，请确保已安装并加入 PATH。", e)),
                        log_type: Some("error".to_string()),
                        ..Default::default()
                    });
                    continue;
                }
            };

            use tokio::io::{AsyncBufReadExt, BufReader};

            // 处理 stdout
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout).lines();

            // 处理 stderr (在一个单独的任务中读取，防止阻塞)
            let stderr = child.stderr.take().unwrap();
            let mut stderr_reader = BufReader::new(stderr).lines();
            let window_clone = window.clone();
            let task_id_clone = task_id.clone();

            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    // Nuclei 的 stderr 通常包含进度信息，但也可能包含严重错误
                    // 我们将其记录为 info 或 warn，取决于内容
                    let log_type = if line.contains("error") || line.contains("Error") {
                        "error"
                    } else {
                        "info"
                    };
                    emit_progress(
                        &window_clone,
                        &ProgressEvent {
                            task_id: task_id_clone.clone(),
                            percent: 0.0, // Indeterminate
                            status: "running".to_string(),
                            status_text: "正在执行...".to_string(),
                            log_message: Some(format!("[Nuclei stderr]: {}", line)),
                            log_type: Some(log_type.to_string()),
                            ..Default::default()
                        },
                    );
                }
            });

            while let Ok(Some(line)) = reader.next_line().await {
                // 解析 Nuclei 的 JSON 输出
                if let Ok(vul_entry) = serde_json::from_str::<serde_json::Value>(&line) {
                    let res = VulnerabilityResult {
                        target: vul_entry["matched-at"]
                            .as_str()
                            .unwrap_or(&target)
                            .to_string(),
                        poc_name: vul_entry["info"]["name"]
                            .as_str()
                            .unwrap_or("Unknown")
                            .to_string(),
                        severity: vul_entry["info"]["severity"]
                            .as_str()
                            .unwrap_or("info")
                            .to_string(),
                        matched_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    let _ = window.emit("vulnerability-found", res);
                } else {
                    // 非 JSON 输出（可能是旧版本或普通日志），打印出来
                    // println!("Nuclei stdout: {}", line);
                }
            }

            let _ = child.wait().await;
        }

        // 3. 完成阶段
        let _ = history::add_scan_history(config.targets.clone(), 0, "success".to_string()); // vul_count can be further refined
        emit_progress(
            &window,
            &ProgressEvent {
                task_id: task_id.clone(),
                percent: 100.0,
                status: "success".to_string(),
                status_text: "扫描任务已完成".to_string(),
                log_message: Some("Nuclei 扫描任务已结束。".to_string()),
                log_type: Some("success".to_string()),
                current_page: None,
                total_pages: None,
                total_results: None,
                fetched_results: None,
            },
        );
    });

    Ok(())
}

#[tauri::command]
async fn get_scan_history() -> Result<Vec<history::ScanHistory>, String> {
    history::get_scan_history()
}

#[tauri::command]
async fn export_scan_results(results: Vec<VulnerabilityResult>) -> Result<String, String> {
    if results.is_empty() {
        return Err("没有结果可导出".to_string());
    }

    let export_path = config::get_export_path()?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = format!("{}/scan_results_{}.csv", export_path, timestamp);

    let mut wtr = csv::Writer::from_path(&file_path).map_err(|e| e.to_string())?;
    wtr.write_record(&["目标", "POC名称", "等级", "发现时间"])
        .map_err(|e| e.to_string())?;

    for res in results {
        wtr.write_record(&[&res.target, &res.poc_name, &res.severity, &res.matched_at])
            .map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(file_path)
}

// HTTP 响应结构体
#[derive(Serialize, Deserialize)]
pub struct RawHttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[tauri::command]
async fn send_raw_http(raw_request: String) -> Result<RawHttpResponse, String> {
    let mut lines = raw_request.lines();

    // 1. 解析请求行 (e.g., GET / HTTP/1.1)
    let first_line = lines.next().ok_or("请求格式错误: 缺少首行")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("请求行格式错误，应包含 Method Path Version".to_string());
    }
    let method = parts[0].to_uppercase();
    let path = parts[1];

    // 2. 解析 Headers
    let mut headers = reqwest::header::HeaderMap::new();
    let mut host = String::new();
    let mut has_body = false;

    for line in lines.by_ref() {
        if line.is_empty() {
            has_body = true;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if key.to_lowercase() == "host" {
                host = value.to_string();
            }
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(val) = reqwest::header::HeaderValue::from_str(value) {
                    headers.insert(name, val);
                }
            }
        }
    }

    // 3. 构建 URL
    if host.is_empty() {
        return Err("请求缺少 Host 头".to_string());
    }
    let protocol = if headers.contains_key("referer")
        && headers["referer"]
            .to_str()
            .unwrap_or("")
            .starts_with("https")
    {
        "https"
    } else {
        "http"
    };

    let full_url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}://{}{}", protocol, host, path)
    };

    // 4. 解析 Body
    let mut body_content = String::new();
    if has_body {
        body_content = lines.collect::<Vec<&str>>().join("\n");
    }

    // 5. 发送请求
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let req_method = match method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request_builder = client.request(req_method, &full_url).headers(headers);
    if !body_content.is_empty() {
        request_builder = request_builder.body(body_content);
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    // 6. 构造回包
    let status = response.status();
    let mut resp_headers = std::collections::HashMap::new();
    for (name, value) in response.headers().iter() {
        resp_headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
    }

    let resp_body = response.text().await.unwrap_or_default();

    Ok(RawHttpResponse {
        status: status.as_u16(),
        status_text: status.to_string(),
        headers: resp_headers,
        body: resp_body,
    })
}

#[tauri::command]
async fn list_pocs() -> Result<Vec<pocs::PocTemplate>, String> {
    let dir = pocs::get_default_pocs_dir();
    Ok(pocs::scan_pocs(&dir))
}

#[tauri::command]
async fn pull_latest_pocs() -> Result<String, String> {
    let output = std::process::Command::new("nuclei")
        .arg("-ut")
        .output()
        .map_err(|e| format!("运行更新命令失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn import_local_pocs(path: Option<String>) -> Result<Vec<pocs::PocTemplate>, String> {
    let dir = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        return Err("未指定路径".to_string());
    };

    if !dir.exists() || !dir.is_dir() {
        return Err("路径不存在或不是目录".to_string());
    }

    Ok(pocs::scan_pocs(&dir))
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
            run_vulnerability_scan,
            send_raw_http,
            get_scan_history,
            export_scan_results,
            list_pocs,
            pull_latest_pocs,
            import_local_pocs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
