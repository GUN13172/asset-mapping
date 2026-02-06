use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::error::{ConversionError, ConversionResult};

/// Platform operators configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operators {
    pub equal: String,
    pub and: String,
    pub or: String,
    pub not_equal: String,
    pub left_paren: String,
    pub right_paren: String,
}

/// Platform configuration containing operators and field mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub operators: Operators,
    pub fields: HashMap<String, String>,
}

/// Configuration manager for all platforms
#[derive(Debug, Clone)]
pub struct ConfigManager {
    configs: HashMap<String, PlatformConfig>,
}

impl ConfigManager {
    /// Load configuration from JSON file
    pub fn from_file<P: AsRef<Path>>(config_path: P) -> ConversionResult<Self> {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| ConversionError::ConfigurationError(format!("Failed to read config file: {}", e)))?;
        
        let configs: HashMap<String, PlatformConfig> = serde_json::from_str(&config_content)
            .map_err(|e| ConversionError::ConfigurationError(format!("Failed to parse config file: {}", e)))?;
        
        Ok(Self { configs })
    }

    /// Get configuration for a specific platform
    pub fn get_platform_config(&self, platform: &str) -> ConversionResult<&PlatformConfig> {
        self.configs.get(platform)
            .ok_or_else(|| ConversionError::UnsupportedPlatform(platform.to_string()))
    }

    /// Get list of supported platforms
    pub fn get_supported_platforms(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Check if a platform is supported
    #[allow(dead_code)]
    pub fn is_platform_supported(&self, platform: &str) -> bool {
        self.configs.contains_key(platform)
    }
}
