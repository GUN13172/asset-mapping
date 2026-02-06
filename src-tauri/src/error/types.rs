use std::fmt;

/// Errors that can occur during query conversion
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Platform is not supported
    UnsupportedPlatform(String),
    /// Syntax validation failed
    SyntaxValidationFailed(String),
    /// Field is not supported by the platform
    UnsupportedField { platform: String, field: String },
    /// Operator is not supported by the platform
    UnsupportedOperator { platform: String, operator: String, suggestion: String },
    /// Configuration loading failed
    ConfigurationError(String),
    /// Internal conversion error
    #[allow(dead_code)]
    InternalError(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::UnsupportedPlatform(platform) => {
                write!(f, "Unsupported platform: {}", platform)
            }
            ConversionError::SyntaxValidationFailed(msg) => {
                write!(f, "{}", msg)
            }
            ConversionError::UnsupportedField { platform, field } => {
                write!(f, "{} platform does not support field '{}'", platform.to_uppercase(), field)
            }
            ConversionError::UnsupportedOperator { platform, operator, suggestion } => {
                write!(f, "{} platform does not support '{}' operator, please use '{}' instead",
                    platform.to_uppercase(), operator, suggestion)
            }
            ConversionError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            ConversionError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;
