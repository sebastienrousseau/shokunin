//! Unit tests for the `metadata` module.
//!
//! This module contains tests for metadata extraction and manipulation.

#[cfg(test)]
mod tests {
    use metadata_gen::metadata::extract_metadata;
    use metadata_gen::MetadataError;

    /// Test metadata extraction from a valid YAML source.
    ///
    /// This test ensures that metadata is correctly extracted from valid YAML input.
    #[test]
    fn test_yaml_metadata_extraction() {
        let yaml = r#"---
title: "Test Title"
description: "Test description"
---
Content here
"#;

        let metadata = extract_metadata(yaml).unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Title".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(&"Test description".to_string())
        );
    }

    /// Test metadata extraction from a valid TOML source.
    ///
    /// This test ensures that metadata is correctly extracted from valid TOML input.
    /// Test metadata extraction from a valid TOML source.
    #[test]
    fn test_toml_metadata_extraction() {
        let toml = r#"
+++
title = "Test Title"
description = "Test description"
+++
Content here
"#;

        let metadata = extract_metadata(toml).unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Title".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(&"Test description".to_string())
        );
    }

    /// Test metadata extraction from a valid JSON source.
    #[test]
    fn test_json_metadata_extraction() {
        let json = r#"
{
    "title": "Test Title",
    "description": "Test description"
}
Content here
"#;

        let metadata = extract_metadata(json).unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Title".to_string())
        );
        assert_eq!(
            metadata.get("description"),
            Some(&"Test description".to_string())
        );
    }

    /// Test metadata extraction with missing fields.
    ///
    /// This test checks how the module handles cases where certain metadata fields are absent.
    #[test]
    fn test_missing_metadata() {
        let yaml = r#"---
title: "Test Title"
---
Content here
"#;

        let metadata = extract_metadata(yaml).unwrap();
        assert_eq!(
            metadata.get("title"),
            Some(&"Test Title".to_string())
        );
        assert!(metadata.get("description").is_none());
    }

    /// Test metadata extraction with invalid front matter format.
    ///
    /// This test checks if the function returns an appropriate error when the front matter is malformed.
    #[test]
    fn test_invalid_metadata_format() {
        let invalid_yaml = r#"---
title: Test Title
description: Test description
Content here
"#;

        let result = extract_metadata(invalid_yaml);
        assert!(
            result.is_err(),
            "Invalid YAML front matter should result in an error"
        );

        if let Err(MetadataError::ExtractionError(_)) = result {
            // Expected error
        } else {
            panic!("Expected MetadataError::ExtractionError");
        }
    }
}
