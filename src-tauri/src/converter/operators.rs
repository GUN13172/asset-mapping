use crate::config::PlatformConfig;
use regex::Regex;

/// Operator converter for transforming operators between platforms
pub struct OperatorConverter;

impl OperatorConverter {
    /// Convert not equal operators
    pub fn convert_not_equal_operator(
        query: &str,
        from_config: &PlatformConfig,
        to_config: &PlatformConfig,
    ) -> String {
        let mut result = query.to_string();

        // If source and target not equal operators are the same, return directly
        if from_config.operators.not_equal == to_config.operators.not_equal {
            return result;
        }

        // Handle conversion from != format
        if from_config.operators.not_equal == "!=" {
            // Match field!="value" format
            let re = Regex::new(r#"(\w+)!="([^"]*)""#).unwrap();
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let field = &caps[1];
                    let value = &caps[2];

                    // Convert based on target platform's not equal operator format
                    if to_config.operators.not_equal.trim() == "NOT" {
                        // Convert to NOT field:"value" format
                        format!("{} {}:\"{}\"", to_config.operators.not_equal, field, value)
                    } else {
                        // Other formats, directly replace operator
                        format!("{}{}\"{}\"", field, to_config.operators.not_equal, value)
                    }
                })
                .to_string();
        }
        // Handle conversion from NOT format
        else if from_config.operators.not_equal.trim() == "NOT" {
            // Handle NOT field="value" format (FOFA style)
            let not_pattern = regex::escape(&from_config.operators.not_equal);
            let re_pattern = format!(r#"{}\s+(\w+)="([^"]*)""#, not_pattern);
            let re = Regex::new(&re_pattern).unwrap();

            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let field = &caps[1];
                    let value = &caps[2];

                    // Convert based on target platform's not equal operator format
                    if to_config.operators.not_equal == "!=" {
                        // Convert to field!="value" format
                        format!("{}!=\"{}\"", field, value)
                    } else if to_config.operators.not_equal.trim() == "NOT" {
                        // Convert to NOT field:"value" format (QUAKE style)
                        format!("{} {}:\"{}\"", to_config.operators.not_equal, field, value)
                    } else {
                        // Other formats
                        format!("{}{}\"{}\"", field, to_config.operators.not_equal, value)
                    }
                })
                .to_string();

            // Handle NOT field:"value" format (QUAKE style)
            let re_pattern2 = format!(r#"{}\s+(\w+):"([^"]*)""#, not_pattern);
            let re2 = Regex::new(&re_pattern2).unwrap();

            result = re2
                .replace_all(&result, |caps: &regex::Captures| {
                    let field = &caps[1];
                    let value = &caps[2];

                    // Convert based on target platform's not equal operator format
                    if to_config.operators.not_equal == "!=" {
                        // Convert to field!="value" format
                        format!("{}!=\"{}\"", field, value)
                    } else {
                        // Other formats
                        format!("{}{}\"{}\"", field, to_config.operators.not_equal, value)
                    }
                })
                .to_string();
        }

        result
    }

    /// Convert other logical operators
    pub fn convert_other_operators(
        query: &str,
        from_config: &PlatformConfig,
        to_config: &PlatformConfig,
    ) -> String {
        let mut result = query.to_string();

        // Handle other operators
        let operator_mappings = [
            (&from_config.operators.equal, &to_config.operators.equal),
            (&from_config.operators.and, &to_config.operators.and),
            (&from_config.operators.or, &to_config.operators.or),
            (&from_config.operators.left_paren, &to_config.operators.left_paren),
            (&from_config.operators.right_paren, &to_config.operators.right_paren),
        ];

        for (from_op, to_op) in operator_mappings {
            if from_op != to_op {
                result = result.replace(from_op, to_op);
            }
        }

        result
    }
}
