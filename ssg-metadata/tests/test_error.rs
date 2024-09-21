//! Unit tests for the `error` module in the ssg-metadata library.
//!
//! This module tests the various custom error types defined in `MetadataError`
//! and their functionality.

#[cfg(test)]
mod tests {
    use serde_json::Error as JsonError;
    use serde_yml::Error as YamlError;
    use ssg_metadata::error::MetadataError;
    use std::io;
    use toml::de::Error as TomlError;

    /// Test `ExtractionError` construction.
    ///
    /// This test ensures that the `ExtractionError` variant is created and its message is correct.
    #[test]
    fn test_extraction_error() {
        let error = MetadataError::ExtractionError(
            "Invalid metadata format".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Failed to extract metadata: Invalid metadata format"
        );
    }

    /// Test `ProcessingError` construction.
    ///
    /// This test ensures that the `ProcessingError` variant is created and its message is correct.
    #[test]
    fn test_processing_error() {
        let error =
            MetadataError::ProcessingError("Unknown field".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to process metadata: Unknown field"
        );
    }

    /// Test `MissingFieldError` construction.
    ///
    /// This test ensures that the `MissingFieldError` variant is created and its message is correct.
    #[test]
    fn test_missing_field_error() {
        let error =
            MetadataError::MissingFieldError("description".to_string());
        assert_eq!(
            error.to_string(),
            "Missing required metadata field: description"
        );
    }

    /// Test `DateParseError` construction.
    ///
    /// This test ensures that the `DateParseError` variant is created and its message is correct.
    #[test]
    fn test_date_parse_error() {
        let error = MetadataError::DateParseError(
            "Invalid date format".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Failed to parse date: Invalid date format"
        );
    }

    /// Test `IoError` conversion.
    ///
    /// This test ensures that a standard `io::Error` is correctly converted into the `IoError` variant.
    #[test]
    fn test_io_error() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = MetadataError::from(io_error);
        assert_eq!(error.to_string(), "I/O error: File not found");
    }

    /// Test `YamlError` conversion.
    ///
    /// This test ensures that a `serde_yml::Error` is correctly converted into the `YamlError` variant.
    #[test]
    fn test_yaml_error() {
        // Malformed YAML content
        let invalid_yaml = "invalid: yaml: data";

        // Try to parse the invalid YAML, which will trigger a `serde_yml::Error`
        let yaml_error: Result<serde_yml::Value, YamlError> =
            serde_yml::from_str(invalid_yaml);

        if let Err(yaml_error) = yaml_error {
            // Convert the `serde_yml::Error` into `MetadataError`
            let error = MetadataError::from(yaml_error);

            // Check that the error message is correctly formatted
            assert!(error.to_string().contains("YAML parsing error"));
        } else {
            panic!("Expected YAML parsing error, but got Ok");
        }
    }

    /// Test `JsonError` conversion.
    ///
    /// This test ensures that a `serde_json::Error` is correctly converted into the `JsonError` variant.
    #[test]
    fn test_json_error() {
        let invalid_json = "{ invalid json }"; // Malformed JSON
        let json_error: Result<serde_json::Value, JsonError> =
            serde_json::from_str(invalid_json);

        if let Err(json_error) = json_error {
            let error = MetadataError::from(json_error);
            // Check if the error message contains the correct phrase
            assert!(
                error.to_string().contains("JSON parsing error"),
                "Error message should contain 'JSON parsing error'"
            );
        } else {
            panic!("Expected JSON parsing error, but got Ok");
        }
    }

    /// Test `TomlError` conversion.
    ///
    /// This test ensures that a `toml::de::Error` is correctly converted into the `TomlError` variant.
    #[test]
    fn test_toml_error() {
        let invalid_toml = "invalid = toml"; // Malformed TOML
        let toml_error: Result<toml::Value, TomlError> =
            toml::from_str(invalid_toml);

        if let Err(toml_error) = toml_error {
            let error = MetadataError::from(toml_error);
            assert!(error.to_string().contains("TOML parsing error"));
        } else {
            panic!("Expected TOML parsing error, but got Ok");
        }
    }
}
