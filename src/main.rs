use std::io::Read;
use serde_json::{Value, Map};

fn clean_value(val: &Value) -> Option<Value> {
    match val {
        Value::Null => None,
        Value::String(s) => {
            let trimmed = s.trim().to_owned();
            if trimmed.is_empty() { None } else { Some(Value::String(trimmed)) }
        },
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr.iter()
                .filter_map(clean_value)
                .collect();
            if cleaned.is_empty() { None } else { Some(Value::Array(cleaned)) }
        },
        Value::Object(map) => {
            let cleaned: Map<String, Value> = map.iter()
                .filter_map(|(k, v)| clean_value(v).map(|v| (k.trim().to_owned(), v)))
                .collect();
            if cleaned.is_empty() { None } else { Some(Value::Object(cleaned)) }
        },
        _ => Some(val.clone()),
    }
}

fn clean_json(json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(json)?;
    
    // Check if the parsed JSON is an empty object
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
            "  empty string  ": "",
            "  null  ": null,
            "  nested  ": {
                "  key  ": "  false  ",
                "  empty array  ": [],
                "  empty object  ": {},
                "  empty string  ": "",
                "  null  ": null
            }
        }
        "#;
        let expected = r#"{"key":"true","nested":{"key":"false"}}"#;
        let cleaned = clean_json(input).unwrap();
        assert_eq!(cleaned, expected);
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
