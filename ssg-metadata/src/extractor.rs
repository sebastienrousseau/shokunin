use anyhow::Result;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use toml::Value as TomlValue;
use yaml_rust2::YamlLoader;

use crate::models::Metadata;

/// Extracts metadata from the content string.
pub fn extract_metadata(content: &str) -> Result<Metadata> {
    if let Some(yaml_metadata) = extract_yaml_metadata(content) {
        Ok(yaml_metadata)
    } else if let Some(toml_metadata) = extract_toml_metadata(content) {
        Ok(toml_metadata)
    } else if let Some(json_metadata) = extract_json_metadata(content) {
        Ok(json_metadata)
    } else {
        Ok(Metadata::default())
    }
}

fn extract_yaml_metadata(content: &str) -> Option<Metadata> {
    let re = Regex::new(r"(?s)^---\s*\n(.*?)\n---").ok()?;
    let captures = re.captures(content)?;
    let yaml_str = captures.get(1)?.as_str();

    let docs = YamlLoader::load_from_str(yaml_str).ok()?;
    let yaml = docs.into_iter().next()?;

    let metadata: HashMap<String, String> = yaml
        .as_hash()?
        .iter()
        .filter_map(|(k, v)| {
            Some((k.as_str()?.to_string(), v.as_str()?.to_string()))
        })
        .collect();

    Some(Metadata::new(metadata))
}

fn extract_toml_metadata(content: &str) -> Option<Metadata> {
    let re = Regex::new(r"(?s)^\+\+\+\s*\n(.*?)\n\+\+\+").ok()?;
    let captures = re.captures(content)?;
    let toml_str = captures.get(1)?.as_str();

    let toml_value: TomlValue = toml::from_str(toml_str).ok()?;
    let toml_table = toml_value.as_table()?;

    let metadata: HashMap<String, String> = toml_table
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.clone(), s.to_string()))
        })
        .collect();

    Some(Metadata::new(metadata))
}

fn extract_json_metadata(content: &str) -> Option<Metadata> {
    let re = Regex::new(r"(?s)^\{(.*?)\}").ok()?;
    let captures = re.captures(content)?;
    let json_str = format!("{{{}}}", captures.get(1)?.as_str());

    let json_value: JsonValue = serde_json::from_str(&json_str).ok()?;
    let json_object = json_value.as_object()?;

    let metadata: HashMap<String, String> = json_object
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.clone(), s.to_string()))
        })
        .collect();

    Some(Metadata::new(metadata))
}
