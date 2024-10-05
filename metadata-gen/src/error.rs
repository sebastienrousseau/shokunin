use serde_yml::Error as SerdeYmlError;
use thiserror::Error;

/// Custom error types for the metadata-gen library
#[derive(Error, Debug)]
pub enum MetadataError {
    /// Error occurred while extracting metadata
    #[error("Failed to extract metadata: {0}")]
    ExtractionError(String),

    /// Error occurred while processing metadata
    #[error("Failed to process metadata: {0}")]
    ProcessingError(String),

    /// Error occurred due to missing required field
    #[error("Missing required metadata field: {0}")]
    MissingFieldError(String),

    /// Error occurred while parsing date
    #[error("Failed to parse date: {0}")]
    DateParseError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] SerdeYmlError),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
}
