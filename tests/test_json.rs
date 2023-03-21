#[cfg(test)]
mod tests {
    use serde_json::Value;
    use ssg::json::generate_json;
    use std::error::Error;

    #[test]
    fn test_generate_json() -> Result<(), Box<dyn Error>> {
        let expected_json = r#"{
            "background_color": "ffffff",
            "description": "test",
            "display": "standalone",
            "lang": "en-US",
            "name": "Test Web App",
            "scope": "/",
            "short_name": "Test",
            "start_url": "/index.html",
            "theme_color": "007bff"
        }"#;

        let actual_json = generate_json(
            "ffffff",
            "test",
            "standalone",
            "en-US",
            "Test Web App",
            "/",
            "/index.html",
            "Test",
            "007bff",
        );

        let expected_value: Value =
            serde_json::from_str(expected_json)?;
        let actual_value: Value = serde_json::from_str(&actual_json)?;

        assert_eq!(
            actual_value, expected_value,
            "Expected: {}, Actual: {}",
            expected_json, actual_json
        );

        Ok(())
    }

    #[test]
    fn test_generate_json_with_empty_values(
    ) -> Result<(), Box<dyn Error>> {
        let expected_json = r#"{
            "background_color": "",
            "description": "",
            "display": "",
            "lang": "",
            "name": "",
            "scope": "",
            "short_name": "",
            "start_url": "",
            "theme_color": ""
        }"#;

        let actual_json =
            generate_json("", "", "", "", "", "", "", "", "");

        let expected_value: Value =
            serde_json::from_str(expected_json)?;
        let actual_value: Value = serde_json::from_str(&actual_json)?;

        assert_eq!(
            actual_value, expected_value,
            "Expected: {}, Actual: {}",
            expected_json, actual_json
        );

        Ok(())
    }
}
