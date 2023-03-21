use serde_json::{json, Map};

/// Generates a JSON string from the given parameters. The parameters
/// are used to populate the fields of a Web App Manifest.
/// The resulting JSON string is returned.
///
pub fn generate_json(
    background_color: &str,
    description: &str,
    display: &str,
    lang: &str,
    name: &str,
    scope: &str,
    start_url: &str,
    short_name: &str,
    theme_color: &str,
) -> String {
    let mut json_map = Map::new();
    json_map.insert(
        "background_color".to_string(),
        json!(background_color),
    );
    json_map.insert("description".to_string(), json!(description));
    json_map.insert("display".to_string(), json!(display));
    json_map.insert("lang".to_string(), json!(lang));
    json_map.insert("name".to_string(), json!(name));
    json_map.insert("scope".to_string(), json!(scope));
    json_map.insert("start_url".to_string(), json!(start_url));
    json_map.insert("short_name".to_string(), json!(short_name));
    json_map.insert("theme_color".to_string(), json!(theme_color));
    serde_json::to_string_pretty(&json_map).unwrap()
}
