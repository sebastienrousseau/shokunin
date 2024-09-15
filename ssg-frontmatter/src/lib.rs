//! # SSG Frontmatter
//!
//! This module provides functionality to extract and parse frontmatter from various file formats
//! commonly used in static site generators. It supports YAML, TOML, and JSON frontmatter.

use serde_json::{Map, Value as JsonValue};
use serde_yml::Value as YamlValue;
use std::collections::HashMap;
use thiserror::Error;
use toml::Value as TomlValue;

/// Errors that can occur during frontmatter parsing.
#[derive(Error, Debug)]
pub enum FrontmatterError {
    /// Error occurred while parsing YAML.
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yml::Error),

    /// Error occurred while parsing TOML.
    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    /// Error occurred while parsing JSON.
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// The frontmatter format is invalid or unsupported.
    #[error("Invalid frontmatter format")]
    InvalidFormat,
}

/// Extracts frontmatter from a string of content.
///
/// This function attempts to extract frontmatter from the given content string.
/// It supports YAML, TOML, and JSON formats.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to parse.
///
/// # Returns
///
/// * `Ok(HashMap<String, String>)` - A hashmap of key-value pairs from the frontmatter.
/// * `Err(FrontmatterError)` - An error if parsing fails or the format is invalid.
///
/// # Examples
///
/// ```
/// use ssg_frontmatter::extract;
///
/// let yaml_content = r#"---
/// title: My Post
/// date: 2023-05-20
/// ---
/// Content here"#;
///
/// let frontmatter = extract(yaml_content).unwrap();
/// assert_eq!(frontmatter.get("title"), Some(&"My Post".to_string()));
/// ```
pub fn extract(content: &str) -> Result<HashMap<String, String>, FrontmatterError> {
    if let Some(yaml) = extract_delimited_frontmatter(content, "---\n", "\n---\n") {
        parse_yaml_frontmatter(yaml)
    } else if let Some(toml) = extract_delimited_frontmatter(content, "+++\n", "\n+++\n") {
        parse_toml_frontmatter(toml)
    } else if let Some(json) = extract_json_frontmatter(content) {
        parse_json_frontmatter(json)
    } else {
        Err(FrontmatterError::InvalidFormat)
    }
}

/// Extracts frontmatter enclosed by delimiters.
fn extract_delimited_frontmatter<'a>(content: &'a str, start_delim: &str, end_delim: &str) -> Option<&'a str> {
    content.strip_prefix(start_delim)?.split(end_delim).next()
}

/// Extracts JSON frontmatter.
fn extract_json_frontmatter(content: &str) -> Option<&str> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }

    let mut brace_count = 0;
    for (idx, ch) in trimmed.char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(&trimmed[..=idx]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parses YAML frontmatter.
fn parse_yaml_frontmatter(yaml: &str) -> Result<HashMap<String, String>, FrontmatterError> {
    let yaml_value: YamlValue = serde_yml::from_str(yaml)?;
    Ok(parse_yaml_value(&yaml_value))
}

/// Parses TOML frontmatter.
fn parse_toml_frontmatter(toml: &str) -> Result<HashMap<String, String>, FrontmatterError> {
    let toml_value: TomlValue = toml.parse()?;
    Ok(parse_toml_table(toml_value.as_table().ok_or(FrontmatterError::InvalidFormat)?))
}

/// Parses JSON frontmatter.
fn parse_json_frontmatter(json: &str) -> Result<HashMap<String, String>, FrontmatterError> {
    let json_value: JsonValue = serde_json::from_str(json)?;
    parse_json_value(&json_value)
}

/// Converts a YAML value to a HashMap.
fn parse_yaml_value(yaml_value: &YamlValue) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let YamlValue::Mapping(mapping) = yaml_value {
        for (key, value) in mapping {
            if let (YamlValue::String(k), YamlValue::String(v)) = (key, value) {
                result.insert(k.clone(), v.clone());
            }
        }
    }
    result
}

/// Converts a TOML table to a HashMap.
fn parse_toml_table(toml_table: &toml::Table) -> HashMap<String, String> {
    toml_table
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.to_string(), s.to_string()))
        })
        .collect()
}

/// Converts a JSON value to a HashMap.
fn parse_json_value(json_value: &JsonValue) -> Result<HashMap<String, String>, FrontmatterError> {
    match json_value {
        JsonValue::Object(obj) => Ok(parse_json_object(obj)),
        _ => Err(FrontmatterError::InvalidFormat),
    }
}

/// Converts a JSON object to a HashMap.
fn parse_json_object(json_object: &Map<String, JsonValue>) -> HashMap<String, String> {
    json_object
        .iter()
        .filter_map(|(k, v)| {
            Some((k.to_string(), match v {
                JsonValue::String(s) => s.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::Bool(b) => b.to_string(),
                _ => return None,
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_frontmatter() {
        let content = r#"---
title: Test Post
date: 2023-05-20
---
# Actual content here"#;

        let result = extract(content).unwrap();
        assert_eq!(result.get("title"), Some(&"Test Post".to_string()));
        assert_eq!(result.get("date"), Some(&"2023-05-20".to_string()));
    }

    #[test]
    fn test_extract_toml_frontmatter() {
        let content = r#"+++
title = "Test Post"
date = "2023-05-20"
+++
# Actual content here"#;

        let result = extract(content).unwrap();
        assert_eq!(result.get("title"), Some(&"Test Post".to_string()));
        assert_eq!(result.get("date"), Some(&"2023-05-20".to_string()));
    }

    #[test]
    fn test_extract_json_frontmatter() {
        let content = r#"
{
    "title": "Test Post",
    "date": "2023-05-20",
    "content": "Actual content here"
}
# Actual content here"#;

        let result = extract(content).unwrap();
        assert_eq!(result.get("title"), Some(&"Test Post".to_string()));
        assert_eq!(result.get("date"), Some(&"2023-05-20".to_string()));
        assert_eq!(result.get("content"), Some(&"Actual content here".to_string()));
    }

    #[test]
    fn test_invalid_frontmatter() {
        let content = "No frontmatter here";
        let result = extract(content);
        assert!(matches!(result, Err(FrontmatterError::InvalidFormat)));
    }
}
