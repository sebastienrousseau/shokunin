// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Error handling types and context extension traits for the SSG library.

use std::path::PathBuf;
use thiserror::Error;

/// Error variants for the main `ssg` library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SsgError {
    /// Errors originating from the pure-logic compilation core.
    #[error("Core compilation error: {0}")]
    Core(#[from] ssg_core::Error),

    /// File I/O failure with context.
    #[error("I/O error at '{path}': {source}")]
    Io {
        /// The path where the I/O error occurred.
        path: PathBuf,
        /// The underlying I/O error source.
        #[source]
        source: std::io::Error,
    },

    /// Path traversal detected in configuration paths.
    #[error(
        "Security violation: path contains directory traversal ('..'): {path}"
    )]
    PathTraversal {
        /// The path violating safety requirements.
        path: PathBuf,
    },

    /// Symlinks are not allowed for security reasons.
    #[error("Security violation: path resolves to a symlink: {path}")]
    SymlinkForbidden {
        /// The symlink path.
        path: PathBuf,
    },

    /// Configuration field validation failure.
    #[error("Validation failed for field '{field}': {message}")]
    Validation {
        /// The configuration field that failed validation.
        field: String,
        /// The validation failure message.
        message: String,
    },

    /// Template engine rendering errors. Gated by template feature.
    #[cfg(feature = "templates")]
    #[error("Template engine error: {0}")]
    Template(#[from] minijinja::Error),
}

impl SsgError {
    /// Converts a generic error and path context into an `SsgError::Io` variant.
    pub fn io(err: impl Into<anyhow::Error>, path: impl Into<PathBuf>) -> Self {
        let anyhow_err = err.into();
        let io_err = anyhow_err
            .downcast::<std::io::Error>()
            .unwrap_or_else(|e| std::io::Error::other(e.to_string()));
        Self::Io {
            path: path.into(),
            source: io_err,
        }
    }
}

/// Context extension trait for mapping `std::io::Error` contexts with path info.
pub trait PathErrorExt<T> {
    /// Converts a `std::io::Result` into an `SsgError` mapping the path context.
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T, SsgError>;
}

impl<T> PathErrorExt<T> for std::io::Result<T> {
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T, SsgError> {
        self.map_err(|source| SsgError::Io {
            path: path.into(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_core_error() {
        let core_err = ssg_core::Error::InvalidSlug {
            input: "foo bar".into(),
        };
        let err = SsgError::Core(core_err);
        let msg = format!("{err}");
        assert!(msg.contains("Core compilation error"));
    }

    #[test]
    fn test_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = SsgError::Io {
            path: PathBuf::from("foo/bar"),
            source: io_err,
        };
        let msg = format!("{err}");
        assert!(msg.contains("I/O error at 'foo/bar'"));
    }

    #[test]
    fn test_path_traversal() {
        let err = SsgError::PathTraversal {
            path: PathBuf::from("../escaped"),
        };
        let msg = format!("{err}");
        assert!(msg
            .contains("Security violation: path contains directory traversal"));
    }

    #[test]
    fn test_symlink_forbidden() {
        let err = SsgError::SymlinkForbidden {
            path: PathBuf::from("symlink/path"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Security violation: path resolves to a symlink"));
    }

    #[test]
    fn test_validation() {
        let err = SsgError::Validation {
            field: "output".into(),
            message: "cannot be empty".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Validation failed for field 'output'"));
    }

    #[test]
    #[cfg(feature = "templates")]
    fn test_template_error() {
        let source = minijinja::Error::new(
            minijinja::ErrorKind::TemplateNotFound,
            "missing template",
        );
        let err = SsgError::from(source);
        let msg = format!("{err}");
        assert!(msg.contains("Template engine error"));
    }

    #[test]
    fn test_path_error_ext() {
        let res: io::Result<()> =
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"));
        let ssg_res = res.with_path("restricted/file");
        assert!(ssg_res.is_err());
        let err = ssg_res.unwrap_err();
        if let SsgError::Io { path, .. } = err {
            assert_eq!(path, PathBuf::from("restricted/file"));
        } else {
            panic!("Expected SsgError::Io");
        }
    }
}
