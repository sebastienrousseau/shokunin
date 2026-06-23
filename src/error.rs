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

    /// The local LLM endpoint (Ollama, llama.cpp) could not be
    /// reached. Surfaced from the `ureq`-backed `LlmPlugin` HTTP
    /// path (issue #520) when the TCP connection is refused, the
    /// host is unresolvable, or the transport layer fails before
    /// the request is sent.
    #[error("LLM endpoint unreachable at '{url}': {source}")]
    LlmEndpointUnreachable {
        /// The endpoint URL that failed to connect.
        url: String,
        /// The underlying transport error from `ureq`.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The local LLM call exceeded the configured `llm.timeout_secs`
    /// budget before returning a response. Reported as a typed error
    /// (issue #520) so callers can distinguish a slow model from a
    /// genuine network outage. There is no zombie subprocess to
    /// reap — the previous `curl` shellout has been removed.
    #[error("LLM call timed out after {duration:?}")]
    LlmTimeout {
        /// The timeout budget that was exceeded.
        duration: std::time::Duration,
    },

    /// The LLM responded but the payload was not a well-formed JSON
    /// generation response (missing the `response` field, non-UTF-8
    /// body, malformed JSON, or HTTP non-2xx status code without a
    /// usable error envelope). Surfaced from the `ureq` HTTP path
    /// (issue #520).
    #[error("LLM returned an invalid response: {message}")]
    LlmInvalidResponse {
        /// Human-readable description of what was malformed.
        message: String,
    },
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
    fn test_llm_endpoint_unreachable() {
        let io_err =
            io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let err = SsgError::LlmEndpointUnreachable {
            url: "http://localhost:11434".into(),
            source: Box::new(io_err),
        };
        let msg = format!("{err}");
        assert!(msg.contains("LLM endpoint unreachable"));
        assert!(msg.contains("http://localhost:11434"));
    }

    #[test]
    fn test_llm_timeout() {
        let err = SsgError::LlmTimeout {
            duration: std::time::Duration::from_secs(60),
        };
        let msg = format!("{err}");
        assert!(msg.contains("LLM call timed out"));
        assert!(msg.contains("60"));
    }

    #[test]
    fn test_llm_invalid_response() {
        let err = SsgError::LlmInvalidResponse {
            message: "missing 'response' field".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("LLM returned an invalid response"));
        assert!(msg.contains("missing 'response' field"));
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
