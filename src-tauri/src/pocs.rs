use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: String,
    pub author: String,
    pub tags: Vec<String>,
    pub content: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct NucleiInfo {
    name: Option<String>,
    author: Option<String>,
    severity: Option<String>,
    description: Option<String>,
    tags: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NucleiPoc {
    id: String,
    info: NucleiInfo,
}

pub fn scan_pocs(dir_path: &Path) -> Vec<PocTemplate> {
    let mut pocs = Vec::new();

    if !dir_path.exists() {
        return pocs;
    }

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(nuclei_poc) = serde_yaml::from_str::<NucleiPoc>(&content) {
                    let tags = nuclei_poc
                        .info
                        .tags
                        .unwrap_or_default()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    pocs.push(PocTemplate {
                        id: nuclei_poc.id,
                        name: nuclei_poc
                            .info
                            .name
                            .unwrap_or_else(|| "Unknown".to_string()),
                        description: nuclei_poc.info.description.unwrap_or_default(),
                        severity: nuclei_poc
                            .info
                            .severity
                            .unwrap_or_else(|| "info".to_string()),
                        author: nuclei_poc
                            .info
                            .author
                            .unwrap_or_else(|| "anonymous".to_string()),
                        tags,
                        content,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    pocs
}

pub fn get_default_pocs_dir() -> PathBuf {
    // 优先检查用户 HOME 目录下的 nuclei-templates
    if let Some(home) = dirs::home_dir() {
        let n_path = home.join("nuclei-templates");
        if n_path.exists() {
            return n_path;
        }
    }

    // 如果没有，返回配置目录下的 pocs
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("asset-mapping")
        .join("pocs")
}
