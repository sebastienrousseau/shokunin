// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CLI error types and the type-safe `LanguageCode` wrapper.

use serde::{Deserialize, Serialize};

/// Type-safe representation of a language code.
///
/// # Examples
/// ```
/// use ssg::cmd::LanguageCode;
/// assert!(LanguageCode::new("en-GB").is_ok());
/// assert!(LanguageCode::new("invalid").is_err());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageCode(String);

impl LanguageCode {
    /// Creates a new `LanguageCode` instance from a string.
    pub fn new(code: &str) -> Result<Self, CliError> {
        if code.len() != 5 || code.chars().nth(2) != Some('-') {
            return Err(CliError::ValidationError(
                "Invalid language code format".into(),
            ));
        }

        let (lang, region) = code.split_at(2);
        let region = &region[1..]; // Skip hyphen

        if !lang.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(CliError::ValidationError(
                "Language code must be lowercase".into(),
            ));
        }

        if !region.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CliError::ValidationError(
                "Region code must be uppercase".into(),
            ));
        }

        Ok(Self(code.to_string()))
    }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Possible errors that can occur during CLI operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum CliError {
    /// Error indicating an invalid path with additional details.
    InvalidPath {
        /// Field name where the path is used.
        field: String,
        /// Additional details about the invalid path.
        details: String,
    },

    /// Error indicating a missing required argument.
    MissingArgument(String),

    /// Error indicating an invalid URL.
    InvalidUrl(String),

    /// Error indicating an I/O error.
    IoError(std::io::Error),

    /// Error indicating a TOML parsing error.
    TomlError(toml::de::Error),

    /// Error indicating a validation error.
    ValidationError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath { field, details } => {
                write!(f, "Invalid path '{field}': {details}")
            }
            Self::MissingArgument(arg) => {
                write!(f, "Required argument missing: {arg}")
            }
            Self::InvalidUrl(url) => write!(f, "Invalid URL: {url}"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::TomlError(e) => write!(f, "TOML parsing error: {e}"),
            Self::ValidationError(msg) => {
                write!(f, "Validation error: {msg}")
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::TomlError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlError(e)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert!(LanguageCode::new("en-GB").is_ok());
        assert!(LanguageCode::new("en-gb").is_err());
        assert!(LanguageCode::new("EN-GB").is_err());
        assert!(LanguageCode::new("e-GB").is_err());
    }

    #[test]
    fn test_language_code_display() {
        let code = LanguageCode::new("en-GB").unwrap();
        assert_eq!(code.to_string(), "en-GB");
    }

    #[test]
    fn test_language_code_edge_cases() {
        assert!(LanguageCode::new("enGB").is_err());
        assert!(LanguageCode::new("e-G").is_err());
        assert!(LanguageCode::new("").is_err());
    }

    // -----------------------------------------------------------------
    // CliError Display impl -- each variant
    // -----------------------------------------------------------------

    #[test]
    fn cli_error_display_invalid_path() {
        let err = CliError::InvalidPath {
            field: "content_dir".into(),
            details: "contains backslashes".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("content_dir"));
        assert!(msg.contains("contains backslashes"));
    }

    #[test]
    fn cli_error_display_missing_argument() {
        let err = CliError::MissingArgument("site_name".into());
        let msg = format!("{err}");
        assert!(msg.contains("site_name"));
        assert!(msg.contains("missing"));
    }

    #[test]
    fn cli_error_display_invalid_url() {
        let err = CliError::InvalidUrl("bad://url".into());
        let msg = format!("{err}");
        assert!(msg.contains("bad://url"));
    }

    #[test]
    fn cli_error_display_io_error() {
        let io_err =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CliError::IoError(io_err);
        let msg = format!("{err}");
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn cli_error_display_toml_error() {
        let toml_err: toml::de::Error =
            toml::from_str::<crate::cmd::SsgConfig>("invalid {{{").unwrap_err();
        let err = CliError::TomlError(toml_err);
        let msg = format!("{err}");
        assert!(msg.contains("TOML"));
    }

    #[test]
    fn cli_error_display_validation_error() {
        let err = CliError::ValidationError("name too long".into());
        let msg = format!("{err}");
        assert!(msg.contains("name too long"));
        assert!(msg.contains("Validation"));
    }
}
