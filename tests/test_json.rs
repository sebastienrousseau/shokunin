#[cfg(test)]
mod tests {
    // Import necessary dependencies
    use serde_json::{json, Value};
    use ssg::{
        models::data::{CnameData, ManifestData, TxtData},
        modules::json::{cname, manifest, txt},
    };

    #[test]
    fn test_manifest_with_empty_options() {
        // Create a default instance of ManifestData
        let options = ManifestData::default();

        // Define the expected result as a raw string with consistent indentation
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

        // Generate the result using the manifest function
        let result = manifest(&options);

        // Parse both JSON strings into `serde_json::Value` and compare those
        let expected_result_value: Value =
            serde_json::from_str(expected_result).unwrap();
        let result_value: Value =
            serde_json::from_str(&result).unwrap();
        assert_eq!(result_value, expected_result_value);
    }

    #[test]
    fn test_manifest_with_non_empty_options() {
        // Create an instance of ManifestData with custom values
        let options = ManifestData {
            name: "My App".to_string(),
            short_name: "My App".to_string(),
            start_url: "/".to_string(),
            theme_color: "#ffffff".to_string(),
            ..Default::default()
        };

        // Define the expected result using the json! macro
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

        // Generate the result using the manifest function
        let result = manifest(&options);

        // Assert that the deserialized result matches the expected result
        assert_eq!(
            serde_json::from_str::<Value>(&result).unwrap(),
            expected_result
        );
    }

    #[test]
    fn test_cname_full_domain() {
        let options = CnameData {
            cname: "example.com".to_string(),
        };

        let output = cname(&options);
        assert_eq!(output, "example.com\nwww.example.com");
    }

    #[test]
    fn test_cname_empty() {
        let options = CnameData {
            cname: "".to_string(),
        };

        let output = cname(&options);
        assert_eq!(output, "\nwww.");
    }

    #[test]
    fn test_txt() {
        let expected =
            "User-agent: *\nSitemap: https://example.com/sitemap.xml"
                .to_string();
        let txt_options: TxtData = TxtData {
            permalink: "https://example.com".to_string(),
        };
        let result = txt(&txt_options);
        assert_eq!(result, expected);
    }
}
