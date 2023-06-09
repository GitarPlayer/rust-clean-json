use std::io::Read;
use serde_json::{Value, Map};

// Function to clean a JSON Value
fn clean_value(val: &Value) -> Option<Value> {
    match val {
        Value::Null => None,  // Remove null values
        Value::String(s) => {
            let trimmed = s.trim().to_owned();
            Some(Value::String(trimmed))  // Trim string values
        },
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr.iter()
                .filter_map(clean_value)
                .collect();
            if cleaned.is_empty() || cleaned.iter().all(|v| v.is_null()) {
                Some(Value::Null)  // Convert empty arrays or arrays with only null values to null
            } else {
                Some(Value::Array(cleaned))
            }
        },
        Value::Object(map) => {
            let cleaned: Map<String, Value> = map.iter()
                .filter_map(|(k, v)| clean_value(v).map(|v| (k.trim().to_owned(), v)))
                .collect();
            if cleaned.is_empty() {
                Some(Value::Object(vec![("value".to_owned(), Value::Null)].into_iter().collect()))  // Convert empty objects to {"value": null}
            } else {
                Some(Value::Object(cleaned))
            }
        },
        _ => Some(val.clone()),
    }
}

// Function to clean a JSON string
fn clean_json(json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(json)?;

    if value.is_object() && value.as_object().unwrap().is_empty() {
        return Err("JSON is an empty object".into());
    }

    let cleaned = clean_value(&value);
    match cleaned {
        Some(v) => Ok(serde_json::to_string(&v)?),
        None => Err("Cleaned JSON is empty".into()),
    }
}

fn main() {
    let mut buffer = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
        eprintln!("Error reading input: {}", e);
        std::process::exit(1);
    }
    match clean_json(&buffer) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error cleaning json: {}", e);
            std::process::exit(1);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let input = r#"
        {
            "  key  ": "  true  ",
            "  empty array  ": [],
            "  empty object  ": {},
            "  array with null  ": [null],
            "  empty string  ": "   ",
            "  null  ": null,
            "  nested  ": {
                "  key  ": "  false  ",
                "  empty array  ": [],
                "  array with null  ": [null],
                "  empty object  ": {},
                "  empty string  ": "   ",
                "  null  ": null
            },
            "  nested array  ": [[], [null], [null, null]]
        }
        "#;
        let expected = r#"{"key":"true","empty array":null,"empty object":{"value":null},"array with null":null,"empty string":"","nested":{"key":"false","empty array":null,"array with null":null,"empty object":{"value":null},"empty string":""},"nested array":null}"#;
        let cleaned = clean_json(input).unwrap();

        let cleaned_value: Value = serde_json::from_str(&cleaned).unwrap();
        let expected_value: Value = serde_json::from_str(expected).unwrap();

        assert_eq!(cleaned_value, expected_value);
    }

    #[test]
    fn it_errors_on_invalid_json() {
        let input = r#"{"key": "value",}"#;  // Invalid JSON
        let result = clean_json(input);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_on_empty_json() {
        let input = r#"{}"#;  // Empty JSON
        let result = clean_json(input);
        assert!(result.is_err());
    }
}
