#[cfg(test)]
mod tests {
    use serde_json::json;
    use serde_json::Map;
    use serde_json::Value;
    use ssg::json::manifest;
    use ssg::options::ManifestOptions;

    #[test]
    fn test_manifest_with_empty_options() {
        let options = ManifestOptions::default();
        let expected_result = r#"{
  "background_color": "",
  "description": "",
  "display": "",
  "icons": [],
  "name": "",
  "orientation": "",
  "scope": "",
  "short_name": "",
  "start_url": "",
  "theme_color": ""
}"#;
        let result = manifest(&options);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_manifest_with_non_empty_options() {
        let options = ManifestOptions {
            name: "My App".to_string(),
            short_name: "My App".to_string(),
            start_url: "/".to_string(),
            theme_color: "#ffffff".to_string(),
            ..Default::default()
        };

        let expected_result = json!({
            "background_color": "",
            "description": "",
            "display": "",
            "icons": [],
            "name": "My App",
            "orientation": "",
            "scope": "",
            "short_name": "My App",
            "start_url": "/",
            "theme_color": "#ffffff"
        });

        let result = manifest(&options);
        assert_eq!(
            serde_json::from_str::<Value>(&result).unwrap(),
            expected_result
        );
    }
}
