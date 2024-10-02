//! This module provides functionality for parsing and serializing frontmatter in various formats.
//! It supports YAML, TOML, and JSON formats, allowing conversion between these formats and the internal `Frontmatter` representation.

use crate::types::Frontmatter;
use crate::{error::FrontmatterError, Format, Value};
use serde_json::Value as JsonValue;
use serde_yml::Value as YmlValue;
use toml::Value as TomlValue;

/// Parses raw frontmatter string into a `Frontmatter` object based on the specified format.
///
/// # Arguments
///
/// * `raw_frontmatter` - A string slice containing the raw frontmatter content.
/// * `format` - The `Format` enum specifying the format of the frontmatter (YAML, TOML, or JSON).
///
/// # Returns
///
/// A `Result` containing the parsed `Frontmatter` object or a `FrontmatterError` if parsing fails.
///
/// # Examples
///
/// ```
/// use ssg_frontmatter::{Format, Frontmatter, parser::parse};
///
/// let yaml_content = "title: My Post\ndate: 2023-05-20\n";
/// let result = parse(yaml_content, Format::Yaml);
/// assert!(result.is_ok());
/// ```
pub fn parse(
    raw_frontmatter: &str,
    format: Format,
) -> Result<Frontmatter, FrontmatterError> {
    match format {
        Format::Yaml => parse_yaml(raw_frontmatter),
        Format::Toml => parse_toml(raw_frontmatter),
        Format::Json => parse_json(raw_frontmatter),
        Format::Unsupported => Err(FrontmatterError::ConversionError(
            "Unsupported format".to_string(),
        )),
    }
}

/// Converts a `Frontmatter` object to a string representation in the specified format.
///
/// # Arguments
///
/// * `frontmatter` - A reference to the `Frontmatter` object to be converted.
/// * `format` - The `Format` enum specifying the target format (YAML, TOML, or JSON).
///
/// # Returns
///
/// A `Result` containing the serialized string or a `FrontmatterError` if serialization fails.
///
/// # Examples
///
/// ```
/// use ssg_frontmatter::{Format, Frontmatter, Value, parser::to_string};
///
/// let mut frontmatter = Frontmatter::new();
/// frontmatter.insert("title".to_string(), Value::String("My Post".to_string()));
/// let result = to_string(&frontmatter, Format::Yaml);
/// assert!(result.is_ok());
/// ```
pub fn to_string(
    frontmatter: &Frontmatter,
    format: Format,
) -> Result<String, FrontmatterError> {
    match format {
        Format::Yaml => to_yaml(frontmatter),
        Format::Toml => to_toml(frontmatter),
        Format::Json => to_json(frontmatter),
        Format::Unsupported => Err(FrontmatterError::ConversionError(
            "Unsupported format".to_string(),
        )),
    }
}

// YAML-specific functions

fn parse_yaml(raw: &str) -> Result<Frontmatter, FrontmatterError> {
    let yml_value: YmlValue =
        serde_yml::from_str(raw).map_err(|e| {
            FrontmatterError::YamlParseError {
                source: e, // Assign the YamlError to source
            }
        })?;
    Ok(parse_yml_value(&yml_value))
}

fn to_yaml(
    frontmatter: &Frontmatter,
) -> Result<String, FrontmatterError> {
    serde_yml::to_string(&frontmatter.0)
        .map_err(|e| FrontmatterError::ConversionError(e.to_string()))
}

fn yml_to_value(yml: &YmlValue) -> Value {
    match yml {
        YmlValue::Null => Value::Null,
        YmlValue::Bool(b) => Value::Boolean(*b),
        YmlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i as f64)
            } else if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Number(0.0) // Fallback, should not happen
            }
        }
        YmlValue::String(s) => Value::String(s.clone()),
        YmlValue::Sequence(seq) => {
            Value::Array(seq.iter().map(yml_to_value).collect())
        }
        YmlValue::Mapping(map) => {
            let mut result = Frontmatter::new();
            for (k, v) in map {
                if let YmlValue::String(key) = k {
                    let _ = result.insert(key.clone(), yml_to_value(v));
                }
            }
            Value::Object(Box::new(result))
        }
        YmlValue::Tagged(tagged) => Value::Tagged(
            tagged.tag.to_string(),
            Box::new(yml_to_value(&tagged.value)),
        ),
    }
}

fn parse_yml_value(yml_value: &YmlValue) -> Frontmatter {
    let mut result = Frontmatter::new();
    if let YmlValue::Mapping(mapping) = yml_value {
        for (key, value) in mapping {
            if let YmlValue::String(k) = key {
                let _ = result.insert(k.clone(), yml_to_value(value));
            }
        }
    }
    result
}

// TOML-specific functions

fn parse_toml(raw: &str) -> Result<Frontmatter, FrontmatterError> {
    let toml_value: TomlValue =
        raw.parse().map_err(FrontmatterError::TomlParseError)?;
    Ok(parse_toml_value(&toml_value))
}

fn to_toml(
    frontmatter: &Frontmatter,
) -> Result<String, FrontmatterError> {
    toml::to_string(&frontmatter.0)
        .map_err(|e| FrontmatterError::ConversionError(e.to_string()))
}

fn toml_to_value(toml: &TomlValue) -> Value {
    match toml {
        TomlValue::String(s) => Value::String(s.clone()),
        TomlValue::Integer(i) => Value::Number(*i as f64),
        TomlValue::Float(f) => Value::Number(*f),
        TomlValue::Boolean(b) => Value::Boolean(*b),
        TomlValue::Array(arr) => {
            Value::Array(arr.iter().map(toml_to_value).collect())
        }
        TomlValue::Table(table) => {
            let mut result = Frontmatter::new();
            for (k, v) in table {
                let _ = result.insert(k.clone(), toml_to_value(v));
            }
            Value::Object(Box::new(result))
        }
        TomlValue::Datetime(dt) => Value::String(dt.to_string()),
    }
}

fn parse_toml_value(toml_value: &TomlValue) -> Frontmatter {
    let mut result = Frontmatter::new();
    if let TomlValue::Table(table) = toml_value {
        for (key, value) in table {
            let _ = result.insert(key.clone(), toml_to_value(value));
        }
    }
    result
}

// JSON-specific functions

fn parse_json(raw: &str) -> Result<Frontmatter, FrontmatterError> {
    let json_value: JsonValue = serde_json::from_str(raw)
        .map_err(FrontmatterError::JsonParseError)?;
    Ok(parse_json_value(&json_value))
}

fn to_json(
    frontmatter: &Frontmatter,
) -> Result<String, FrontmatterError> {
    serde_json::to_string(&frontmatter.0)
        .map_err(|e| FrontmatterError::ConversionError(e.to_string()))
}

fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i as f64)
            } else if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Number(0.0) // Fallback, should not happen
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            Value::Array(arr.iter().map(json_to_value).collect())
        }
        JsonValue::Object(obj) => {
            let mut result = Frontmatter::new();
            for (k, v) in obj {
                let _ = result.insert(k.clone(), json_to_value(v));
            }
            Value::Object(Box::new(result))
        }
    }
}

fn parse_json_value(json_value: &JsonValue) -> Frontmatter {
    let mut result = Frontmatter::new();
    if let JsonValue::Object(obj) = json_value {
        for (key, value) in obj {
            let _ = result.insert(key.clone(), json_to_value(value));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = "title: My Post\ndate: 2023-05-20\n";
        let result = parse(yaml, Format::Yaml);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("title").unwrap().as_str().unwrap(),
            "My Post"
        );
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid_yaml =
            "title: My Post\ndate: 2023-05-20\ninvalid_entry";
        let result = parse(invalid_yaml, Format::Yaml);
        assert!(result.is_err()); // Expecting an error
    }

    #[test]
    fn test_parse_toml() {
        let toml = "title = \"My Post\"\ndate = 2023-05-20\n";
        let result = parse(toml, Format::Toml);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("title").unwrap().as_str().unwrap(),
            "My Post"
        );
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = "title = \"My Post\"\ndate = invalid-date\n";
        let result = parse(toml, Format::Toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{"title": "My Post", "date": "2023-05-20"}"#;
        let result = parse(json, Format::Json);
        assert!(result.is_ok());

        // Work directly with the Frontmatter type
        let frontmatter = result.unwrap();

        // Assuming Frontmatter is a map-like structure, work with it directly
        assert_eq!(
            frontmatter.get("title").unwrap(),
            &Value::String("My Post".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"title": "My Post", "date": invalid-date}"#;
        let result = parse(json, Format::Json);
        assert!(result.is_err()); // Expecting a JSON parsing error
    }

    #[test]
    fn test_to_yaml() {
        let mut frontmatter = Frontmatter::new();
        let _ = frontmatter.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let result = to_string(&frontmatter, Format::Yaml);
        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(yaml.contains("title: My Post"));
    }

    #[test]
    fn test_to_toml() {
        let mut frontmatter = Frontmatter::new();
        let _ = frontmatter.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let result = to_string(&frontmatter, Format::Toml);
        assert!(result.is_ok());
        let toml = result.unwrap();
        assert!(toml.contains("title = \"My Post\""));
    }

    #[test]
    fn test_to_json() {
        let mut frontmatter = Frontmatter::new();
        let _ = frontmatter.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );
        let result = to_string(&frontmatter, Format::Json);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"title\":\"My Post\""));
    }

    #[test]
    fn test_to_invalid_format() {
        let mut frontmatter = Frontmatter::new();
        let _ = frontmatter.insert(
            "title".to_string(),
            Value::String("My Post".to_string()),
        );

        // Using the unsupported format variant
        let result = to_string(&frontmatter, Format::Unsupported);

        // We expect this to fail with an error
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nested_yaml() {
        let yaml = r#"
        parent:
          child1: value1
          child2:
            subchild: value2
          array:
            - item1
            - item2
        "#;
        let result = parse(yaml, Format::Yaml);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();

        let parent =
            frontmatter.get("parent").unwrap().as_object().unwrap();
        let child1 = parent.get("child1").unwrap().as_str().unwrap();
        let subchild = parent
            .get("child2")
            .unwrap()
            .as_object()
            .unwrap()
            .get("subchild")
            .unwrap()
            .as_str()
            .unwrap();
        let array = parent.get("array").unwrap().as_array().unwrap();

        assert_eq!(child1, "value1");
        assert_eq!(subchild, "value2");
        assert_eq!(array[0].as_str().unwrap(), "item1");
        assert_eq!(array[1].as_str().unwrap(), "item2");
    }

    #[test]
    fn test_parse_nested_toml() {
        let toml = r#"
        [parent]
        child1 = "value1"
        child2 = { subchild = "value2" }
        array = ["item1", "item2"]
        "#;
        let result = parse(toml, Format::Toml);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();

        let parent =
            frontmatter.get("parent").unwrap().as_object().unwrap();
        let child1 = parent.get("child1").unwrap().as_str().unwrap();
        let subchild = parent
            .get("child2")
            .unwrap()
            .as_object()
            .unwrap()
            .get("subchild")
            .unwrap()
            .as_str()
            .unwrap();
        let array = parent.get("array").unwrap().as_array().unwrap();

        assert_eq!(child1, "value1");
        assert_eq!(subchild, "value2");
        assert_eq!(array[0].as_str().unwrap(), "item1");
        assert_eq!(array[1].as_str().unwrap(), "item2");
    }

    #[test]
    fn test_parse_nested_json() {
        let json = r#"
        {
            "parent": {
                "child1": "value1",
                "child2": {
                    "subchild": "value2"
                },
                "array": ["item1", "item2"]
            }
        }
        "#;
        let result = parse(json, Format::Json);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();

        let parent =
            frontmatter.get("parent").unwrap().as_object().unwrap();
        let child1 = parent.get("child1").unwrap().as_str().unwrap();
        let subchild = parent
            .get("child2")
            .unwrap()
            .as_object()
            .unwrap()
            .get("subchild")
            .unwrap()
            .as_str()
            .unwrap();
        let array = parent.get("array").unwrap().as_array().unwrap();

        assert_eq!(child1, "value1");
        assert_eq!(subchild, "value2");
        assert_eq!(array[0].as_str().unwrap(), "item1");
        assert_eq!(array[1].as_str().unwrap(), "item2");
    }
}
