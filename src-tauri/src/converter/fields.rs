use crate::config::PlatformConfig;
use regex::Regex;

/// Field converter for transforming field names between platforms
pub struct FieldConverter;

impl FieldConverter {
    /// Convert field prefixes between platforms
    pub fn convert_fields(
        query: &str,
        from_config: &PlatformConfig,
        to_config: &PlatformConfig,
    ) -> String {
        let mut result = query.to_string();

        // Sort field pairs by field name length, process longer names first to avoid partial matching
        let mut field_pairs: Vec<_> = from_config.fields.iter().collect();
        field_pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (field_name, from_field_prefix) in field_pairs {
            if let Some(to_field_prefix) = to_config.fields.get(field_name) {
                if from_field_prefix != to_field_prefix {
                    // Create patterns for both = and : operators
                    let from_patterns = [
                        format!("{}=", from_field_prefix),
                        format!("{}:", from_field_prefix),
                    ];

                    for from_pattern in from_patterns {
                        // Use regex for precise matching with word boundaries
                        let from_pattern_escaped = regex::escape(&from_pattern);
                        let pattern = format!(r"\b{}", from_pattern_escaped);

                        if let Ok(re) = Regex::new(&pattern) {
                            // Determine the target operator based on the original operator
                            let replacement = if from_pattern.ends_with('=') {
                                format!("{}=", to_field_prefix)
                            } else {
                                format!("{}:", to_field_prefix)
                            };
                            result = re.replace_all(&result, replacement.as_str()).to_string();
                        }
                    }
                }
            }
        }

        result
    }
}
