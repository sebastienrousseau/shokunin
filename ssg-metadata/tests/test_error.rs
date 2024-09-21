//! Unit tests for the `error` module in the ssg-metadata library.
//!
//! This module tests the various custom error types defined in `MetadataError`
//! and their functionality.

#[cfg(test)]
mod tests {
    use ssg_metadata::error::MetadataError;
    use std::io;

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
}
