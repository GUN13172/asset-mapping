use crate::config::PlatformConfig;
use crate::error::{ConversionError, ConversionResult};
use regex::Regex;

/// Syntax validator for query statements
pub struct SyntaxValidator;

impl SyntaxValidator {
    /// Normalize query by converting logical operators to uppercase
    pub fn normalize_query(query: &str) -> String {
        let mut normalized = query.to_string();

        // Convert logical operators to uppercase using word boundaries
        let replacements = [
            (r"\band\b", "AND"),
            (r"\bor\b", "OR"),
            (r"\bnot\b", "NOT"),
        ];

        for (pattern, replacement) in replacements {
            let re = Regex::new(pattern).unwrap();
            normalized = re.replace_all(&normalized, replacement).to_string();
        }

        normalized
    }

    /// Validate query syntax against platform configuration
    pub fn validate_query_syntax(
        query: &str,
        from_config: &PlatformConfig,
        platform_name: &str,
    ) -> ConversionResult<()> {
        // First normalize the query
        let normalized_query = Self::normalize_query(query);

        // Check operator consistency (all fields should use the same operator type)
        Self::validate_operator_consistency(query, from_config, platform_name)?;

        // Check operator support using normalized query
        Self::validate_operators(&normalized_query, from_config, platform_name)?;

        // Check field support using original query (fields are case-sensitive)
        Self::validate_fields(query, from_config, platform_name)?;

        Ok(())
    }

    /// Validate operator consistency (all fields should use the same operator type)
    fn validate_operator_consistency(
        query: &str,
        from_config: &PlatformConfig,
        platform_name: &str,
    ) -> ConversionResult<()> {
        let mut used_operators = std::collections::HashSet::new();

        // Extract all field operators from the query
        let re = Regex::new(r"\w+(?:\.\w+)*([=:])").unwrap();

        for caps in re.captures_iter(query) {
            if let Some(op_match) = caps.get(1) {
                used_operators.insert(op_match.as_str());
            }
        }

        // Check if multiple different operators are used
        if used_operators.len() > 1 {
            let operators_list: Vec<&str> = used_operators.iter().cloned().collect();
            return Err(ConversionError::SyntaxValidationFailed(
                format!("Inconsistent field operators in query. Found: '{}'. {} platform expects consistent use of '{}'",
                    operators_list.join(", "),
                    platform_name.to_uppercase(),
                    from_config.operators.equal)
            ));
        }

        // Check if the used operator matches the platform's expected operator
        if let Some(&used_op) = used_operators.iter().next() {
            if used_op != from_config.operators.equal {
                return Err(ConversionError::UnsupportedOperator {
                    platform: platform_name.to_string(),
                    operator: format!("field{}", used_op),
                    suggestion: format!("field{}", from_config.operators.equal),
                });
            }
        }

        Ok(())
    }

    /// Validate operators used in the query (simplified for normalized input)
    fn validate_operators(
        query: &str,
        from_config: &PlatformConfig,
        platform_name: &str,
    ) -> ConversionResult<()> {
        // Check AND operator
        if query.contains("AND") && from_config.operators.and != "AND" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "AND".to_string(),
                suggestion: from_config.operators.and.clone(),
            });
        }

        // Check OR operator
        if query.contains("OR") && from_config.operators.or != "OR" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "OR".to_string(),
                suggestion: from_config.operators.or.clone(),
            });
        }

        // Check && operator
        if query.contains("&&") && from_config.operators.and != "&&" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "&&".to_string(),
                suggestion: from_config.operators.and.clone(),
            });
        }

        // Check || operator
        if query.contains("||") && from_config.operators.or != "||" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "||".to_string(),
                suggestion: from_config.operators.or.clone(),
            });
        }

        // Check NOT operator
        let not_re = Regex::new(r"\bNOT\s+\w+").unwrap();
        if not_re.is_match(query) && from_config.operators.not_equal.trim() != "NOT" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "NOT".to_string(),
                suggestion: from_config.operators.not_equal.clone(),
            });
        }

        // Check != operator
        if query.contains("!=") && from_config.operators.not_equal != "!=" {
            return Err(ConversionError::UnsupportedOperator {
                platform: platform_name.to_string(),
                operator: "!=".to_string(),
                suggestion: from_config.operators.not_equal.clone(),
            });
        }

        Ok(())
    }



    /// Validate fields used in the query
    fn validate_fields(
        query: &str,
        from_config: &PlatformConfig,
        platform_name: &str,
    ) -> ConversionResult<()> {
        let used_field_names = Self::extract_field_names_from_query(query);

        for field_name in used_field_names {
            // Check if this field name exists in the platform configuration
            // First try exact match
            if from_config.fields.contains_key(&field_name) {
                continue;
            }

            // Then try to match by base field name (for compound fields like response.title)
            let base_field = if field_name.contains('.') {
                field_name.split('.').last().unwrap_or(&field_name)
            } else {
                &field_name
            };

            // Check if the base field exists in config
            let is_supported = from_config.fields.iter().any(|(config_field, _)| {
                config_field == base_field
            });

            if !is_supported {
                return Err(ConversionError::UnsupportedField {
                    platform: platform_name.to_string(),
                    field: base_field.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Extract field names from query (without operators)
    fn extract_field_names_from_query(query: &str) -> Vec<String> {
        let mut field_names = Vec::new();

        // Use regex to find all field names
        let re = Regex::new(r"(\w+(?:\.\w+)*)(?:[:=]|!=)").unwrap();

        for caps in re.captures_iter(query) {
            if let Some(field_match) = caps.get(1) {
                let field_str = field_match.as_str();

                if !field_names.contains(&field_str.to_string()) {
                    field_names.push(field_str.to_string());
                }
            }
        }

        field_names
    }


}
