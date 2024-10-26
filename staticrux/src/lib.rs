// Copyright © 2024 StatiCrux. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// src/lib.rs

// #![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/staticrux/images/favicon.ico",
    html_logo_url = "https://kura.pro/staticrux/images/logos/staticrux.svg",
    html_root_url = "https://docs.rs/staticrux"
)]

/// Compiler module for processing and generating static site content.
pub mod compiler;

/// Locales module for language-specific translations and templates.
pub mod locales;

/// Macro definitions for common operations.
pub mod macros;

/// Data models and structures used throughout the crate.
pub mod models;

/// Various modules for specific functionalities (e.g., HTML generation, RSS feeds).
pub mod modules;

/// Utility functions and helpers.
pub mod utilities;

// Re-export commonly used items for easier access
pub use compiler::service::compile;
pub use http_handle::Server;
pub use utilities::uuid::generate_unique_string;

/// Version of the staticrux library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error type for the shokunin library
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type for the shokunin library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO operation errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Content processing errors
    #[error("Content processing error: {0}")]
    ContentProcessing(String),

    /// Template rendering errors
    #[error("Template error: {0}")]
    Template(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version() {
        assert!(!VERSION.is_empty(), "Version should not be empty");
    }

    #[test]
    fn test_version_format() {
        assert!(
            VERSION.contains('.'),
            "Version should be in semver format"
        );
        assert!(
            VERSION.split('.').count() >= 2,
            "Version should have at least major.minor"
        );
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        );
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_display() {
        let err =
            Error::ContentProcessing("invalid content".to_string());
        assert_eq!(
            err.to_string(),
            "Content processing error: invalid content"
        );
    }
}
