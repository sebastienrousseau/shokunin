use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;
use toml::Value as TomlValue;
use yaml_rust::YamlLoader;

/// ## Function: `extract` - Extracts front matter from a string of content
///
/// This function extracts front matter from a string of content. It
/// supports YAML, TOML, and JSON front matter. It returns a `HashMap`
/// of the front matter key-value pairs.
///
/// ### Arguments
///
/// * `content` - The string of content to extract front matter from
/// (e.g. a Markdown file)
///
/// ### Returns
///
/// A `HashMap` of the front matter key-value pairs. If no front matter
/// is found, an empty `HashMap` is returned.
///
///
pub fn extract(content: &str) -> HashMap<String, String> {
    let mut front_matter = HashMap::new();

    if let Some(front_matter_str) =
        extract_front_matter_str(content, "---\n", "\n---\n")
    {
        if let Ok(doc) = parse_yaml_document(front_matter_str) {
            front_matter
                .extend(parse_yaml_hash(doc.as_hash().unwrap()));
        }
    } else if let Some(front_matter_str) =
        extract_front_matter_str(content, "+++\n", "\n+++\n")
    {
        if let Ok(toml_value) = front_matter_str.parse::<TomlValue>() {
            front_matter.extend(parse_toml_table(
                toml_value.as_table().unwrap(),
            ));
        }
    } else if let Some(front_matter_str) =
        extract_json_object_str(content)
    {
        if let Ok(json_value) =
            serde_json::from_str::<serde_json::Value>(front_matter_str)
        {
            if let Some(obj) = json_value
                .get("frontmatter")
                .and_then(|v| v.as_object())
            {
                front_matter.extend(parse_json_object(obj));
            } else {
                eprintln!("Error: Could not find frontmatter in JSON");
            }
            if let Some(content) =
                json_value.get("content").and_then(|v| v.as_str())
            {
                front_matter
                    .insert("content".to_string(), content.to_string());
            }
        } else {
            eprintln!("Error parsing JSON");
        }
    }

    front_matter
}

/// ## Function: `extract_front_matter_str` - Extracts front matter from a string of content
///
/// This function extracts front matter from a string of content. It
/// supports YAML, TOML, and JSON front matter. It returns a `HashMap`
/// of the front matter key-value pairs.
///
/// ### Arguments
///
/// * `content` - The string of content to extract front matter from
/// (e.g. a Markdown file)
/// * `start_delim` - The start delimiter of the front matter
/// * `end_delim` - The end delimiter of the front matter
///
/// ### Returns
///
/// A `HashMap` of the front matter key-value pairs. If no front matter
/// is found, an empty `HashMap` is returned.
///
pub fn extract_front_matter_str<'a>(
    content: &'a str,
    start_delim: &str,
    end_delim: &str,
) -> Option<&'a str> {
    if content.starts_with(start_delim) {
        if let Some(end_pos) = content.find(end_delim) {
            return Some(&content[start_delim.len()..end_pos]);
        }
    }
    None
}
/// ## Function: `parse_yaml_document` - Parses a YAML document into a `Yaml` object
///
/// This function parses a YAML document into a `Yaml` object.
///
/// ### Arguments
///
/// * `front_matter_str` - The string of front matter to parse into a
/// `Yaml` object
///
/// ### Returns
///
/// A `Yaml` object representing the front matter string. If the front
/// matter string is not valid YAML, an error is returned.
///
pub fn parse_yaml_document(
    front_matter_str: &str,
) -> Result<yaml_rust::Yaml, yaml_rust::ScanError> {
    YamlLoader::load_from_str(front_matter_str)
        .map(|docs| docs.into_iter().next().unwrap())
}

/// ## Function: `parse_yaml_hash` - Parses a YAML hash into a `HashMap` of key-value pairs
///
/// This function parses a YAML hash into a `HashMap` of key-value pairs.
///
/// ### Arguments
///
/// * `yaml_hash` - The YAML hash to parse into a `HashMap` of key-value
/// pairs
///
/// ### Returns
///
/// A `HashMap` of key-value pairs representing the YAML hash.
/// If the YAML hash is not valid, an error is returned.
///
pub fn parse_yaml_hash(
    yaml_hash: &yaml_rust::yaml::Hash,
) -> HashMap<String, String> {
    let mut entries: Vec<_> = yaml_hash
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|value| {
                (k.as_str().unwrap().to_string(), value.to_string())
            })
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.into_iter().collect()
}

/// ## Function: `parse_toml_table` - Parses a TOML table into a `HashMap` of key-value pairs
///
/// This function parses a TOML table into a `HashMap` of key-value pairs.
///
/// ### Arguments
///
/// * `toml_table` - The TOML table to parse into a `HashMap` of key-
/// value pairs.
///
/// ### Returns
///
/// A `HashMap` of key-value pairs representing the TOML table.
/// If the TOML table is not valid, an error is returned.
///
pub fn parse_toml_table(
    toml_table: &toml::value::Table,
) -> HashMap<String, String> {
    toml_table
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| (k.to_string(), s.to_string()))
        })
        .collect()
}
/// ## Function: `extract_json_object_str` - Extracts a JSON object from a string of content
///
/// This function extracts a JSON object from a string of content.
///
/// ### Arguments
///
/// * `content` - The string of content to extract a JSON object from
///
/// ### Returns
///
/// A `&str` representing the JSON object. If no JSON object is found,
/// `None` is returned.
///
pub fn extract_json_object_str(content: &str) -> Option<&str> {
    if content.starts_with('{') {
        let end_pos = content.rfind('}')?;
        Some(&content[..=end_pos])
    } else {
        None
    }
}
/// ## Function: `parse_json_object` - Parses a JSON object into a `HashMap` of key-value pairs
///
/// This function parses a JSON object into a `HashMap` of key-value pairs.
///
/// ### Arguments
///
/// * `json_object` - The JSON object to parse into a `HashMap` of key-
/// value pairs.
///
/// ### Returns
///
/// A `HashMap` of key-value pairs representing the JSON object.
/// If the JSON object is not valid, an error is returned. If the JSON
/// object is not a string, an empty string is returned.
///
pub fn parse_json_object(
    json_object: &Map<String, JsonValue>,
) -> HashMap<String, String> {
    let mut result = json_object
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                match v {
                    JsonValue::String(s) => s.to_string(),
                    JsonValue::Number(n) => match n.as_f64() {
                        Some(f) => f.to_string(),
                        None => match n.as_i64() {
                            Some(i) => i.to_string(),
                            None => "".to_string(),
                        },
                    },
                    JsonValue::Bool(b) => b.to_string(),
                    JsonValue::Object(o) => serde_json::to_string(o)
                        .unwrap_or_else(|_| "".to_string()),
                    JsonValue::Array(a) => serde_json::to_string(a)
                        .unwrap_or_else(|_| "".to_string()),
                    _ => "".to_string(),
                },
            )
        })
        .collect::<Vec<_>>();
    result.sort_by_key(|(k, _)| k.to_string());
    result.into_iter().collect()
}
