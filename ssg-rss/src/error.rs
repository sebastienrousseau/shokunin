// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use quick_xml;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Errors that can occur when generating RSS feeds.
#[derive(Error, Debug)]
pub enum RssError {
    /// Error occurred while writing XML.
    #[error("XML writing error: {0}")]
    XmlWriteError(#[from] quick_xml::Error),

    /// Error occurred during UTF-8 conversion.
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    /// Error indicating a required field is missing.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Error for any I/O operations.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

/// Result type for RSS operations.
pub type Result<T> = std::result::Result<T, RssError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_error_display() {
        let error = RssError::MissingField("title".to_string());
        assert_eq!(error.to_string(), "Missing required field: title");
    }

    #[test]
    fn test_xml_write_error() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::Other, "XML error"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert!(error.to_string().starts_with("XML writing error:"));
    }

    #[test]
    fn test_utf8_error() {
        let utf8_error =
            String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error = RssError::Utf8Error(utf8_error);
        assert!(error
            .to_string()
            .starts_with("UTF-8 conversion error:"));
    }

    #[test]
    fn test_io_error() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = RssError::IoError(io_error);
        assert!(error.to_string().starts_with("I/O error:"));
    }
}
