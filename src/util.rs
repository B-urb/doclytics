use serde_json::Value;
use chrono::NaiveDate;

#[derive(Debug)]
pub enum InferredType {
    String,
    Integer,
    Date,
    Unknown,
}

pub fn infer_type(value: &Value) -> InferredType {
    if value.is_string() {
        let string_value = value.as_str().unwrap();
        if string_value.parse::<i64>().is_ok() {
            InferredType::Integer
        } else if NaiveDate::parse_from_str(string_value, "%Y-%m-%d").is_ok() {
            InferredType::Date
        } else {
            InferredType::String
        }
    } else if value.is_number() {
        InferredType::Integer
    } else {
        InferredType::Unknown
    }
}

pub fn infer_types_from_value(value: &Value) -> Vec<(String, InferredType)> {
    let mut types = Vec::new();

    if let Value::Object(map) = value {
        for (key, value) in map.iter() {
            let inferred_type = infer_type(value);
            types.push((key.clone(), inferred_type));
        }
    }

    types
}
