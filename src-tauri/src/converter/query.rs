use crate::config::ConfigManager;
use crate::converter::{FieldConverter, OperatorConverter, SyntaxValidator};
use crate::error::ConversionResult;

/// Main query converter
pub struct QueryConverter {
    config_manager: ConfigManager,
}

impl QueryConverter {
    /// Create a new query converter with configuration manager
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    /// Validate query syntax for the source platform
    pub fn validate_query_syntax(&self, query: &str, platform_name: &str) -> ConversionResult<()> {
        let from_config = self.config_manager.get_platform_config(platform_name)?;
        SyntaxValidator::validate_query_syntax(query, from_config, platform_name)
    }

    /// Convert query from one platform to another
    pub fn convert(
        &self,
        query: &str,
        from_platform: &str,
        to_platform: &str,
    ) -> ConversionResult<String> {
        let from_config = self.config_manager.get_platform_config(from_platform)?;
        let to_config = self.config_manager.get_platform_config(to_platform)?;

        // If same platform, return directly
        if from_platform == to_platform {
            return Ok(query.to_string());
        }

        // Normalize the query first (convert logical operators to uppercase)
        let mut result = SyntaxValidator::normalize_query(query);

        // Convert not equal operators first (needs to be done before field conversion)
        result = OperatorConverter::convert_not_equal_operator(&result, from_config, to_config);

        // Convert field prefixes
        result = FieldConverter::convert_fields(&result, from_config, to_config);

        // Convert other logical operators
        result = OperatorConverter::convert_other_operators(&result, from_config, to_config);

        Ok(result)
    }

    /// Get list of supported platforms
    pub fn get_supported_platforms(&self) -> Vec<String> {
        self.config_manager.get_supported_platforms()
    }

    /// Check if a platform is supported
    #[allow(dead_code)]
    pub fn is_platform_supported(&self, platform: &str) -> bool {
        self.config_manager.is_platform_supported(platform)
    }
}
