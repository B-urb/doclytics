use crate::types;
use std::env;
use substring::Substring;

pub fn normalize_string(s: &str) -> String {
    s.replace("-", "").replace("_", "").to_lowercase()
}

pub fn extract_json_object(input: &str) -> Result<String, String> {
    let mut brace_count = 0;
    let mut json_start = None;
    let mut json_end = None;

    for (i, c) in input.chars().enumerate() {
        match c {
            '{' | '[' => {
                if brace_count == 0 {
                    json_start = Some(i);
                }
                brace_count += 1;
            }
            '}' | ']' => {
                brace_count -= 1;
                if brace_count == 0 {
                    json_end = Some(i);
                    break; // Found the complete JSON object
                }
            }
            _ => {}
        }
    }

    if let (Some(start), Some(end)) = (json_start, json_end) {
        slog_scope::debug!("{}", input.substring(start, end + 1));
        Ok(input.substring(start, end + 1).to_string())
    } else {
        let error_msg = "No JSON object found in the response!".to_string();
        slog_scope::debug!("{}", error_msg);
        Err(error_msg)
    }
}

pub fn create_mode_from_env(env_key: &str) -> types::Mode {
    let mode_env = env::var(env_key).unwrap_or_else(|_| "1".to_string());
    let mode_int = mode_env.parse::<i32>().unwrap_or(1);
    types::Mode::from_int(mode_int)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_object() {
        let json_str = "Some text before JSON object {\"key\": \"value\"} Some text after";
        assert_eq!(
            extract_json_object(json_str).unwrap(),
            "{\"key\": \"value\"}"
        );

        let json_array_str = "Some text before JSON array [1,2,3] Some text after";
        assert_eq!(extract_json_object(json_array_str).unwrap(), "[1,2,3]");

        let empty_json_str = "No JSON object or array here";
        assert!(extract_json_object(empty_json_str).is_err());
    }
}
