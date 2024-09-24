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

impl RssError {
    /// Creates a new `RssError::MissingField` error.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the missing field.
    ///
    /// # Returns
    ///
    /// Returns a new `RssError::MissingField` instance.
    pub fn missing_field<S: Into<String>>(field_name: S) -> Self {
        RssError::MissingField(field_name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_rss_error_display() {
        let error = RssError::missing_field("title");
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
        let utf8_error = String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error = RssError::Utf8Error(utf8_error);
        assert!(error.to_string().starts_with("UTF-8 conversion error:"));
    }

    #[test]
    fn test_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = RssError::IoError(io_error);
        assert!(error.to_string().starts_with("I/O error:"));
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RssError>();
    }

    #[test]
    fn test_error_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = RssError::IoError(io_error);
        assert!(error.source().is_some());
    }

    // New tests start here

    #[test]
    fn test_missing_field_with_string() {
        let error = RssError::missing_field(String::from("author"));
        assert_eq!(error.to_string(), "Missing required field: author");
    }

    #[test]
    fn test_missing_field_with_str() {
        let error = RssError::missing_field("description");
        assert_eq!(error.to_string(), "Missing required field: description");
    }

    #[test]
    fn test_xml_write_error_details() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert!(error.to_string().contains("Permission denied"));
    }

    #[test]
    fn test_utf8_error_details() {
        let utf8_error = String::from_utf8(vec![0xFF, 0xFF]).unwrap_err();
        let error = RssError::Utf8Error(utf8_error);
        assert!(error.to_string().contains("invalid utf-8 sequence"));
    }

    #[test]
    fn test_io_error_details() {
        let io_error = io::Error::new(io::ErrorKind::AddrInUse, "Address already in use");
        let error = RssError::IoError(io_error);
        assert!(error.to_string().contains("Address already in use"));
    }

    #[test]
    fn test_error_downcast() {
        let error: Box<dyn Error> = Box::new(RssError::missing_field("category"));
        let downcast_result = error.downcast::<RssError>();
        assert!(downcast_result.is_ok());
    }

    #[test]
    fn test_error_chain() {
        let io_error = io::Error::new(io::ErrorKind::Other, "Underlying IO error");
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(io_error));
        let error = RssError::XmlWriteError(xml_error);

        let mut error_chain = error.source();
        assert!(error_chain.is_some());
        error_chain = error_chain.unwrap().source();
        assert!(error_chain.is_some());
        assert_eq!(error_chain.unwrap().to_string(), "Underlying IO error");
    }

    #[test]
    fn test_from_quick_xml_error() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected EOF"),
        ));
        let error: RssError = xml_error.into();
        assert!(matches!(error, RssError::XmlWriteError(_)));
    }

    #[test]
    fn test_from_utf8_error() {
        let utf8_error = String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error: RssError = utf8_error.into();
        assert!(matches!(error, RssError::Utf8Error(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error: RssError = io_error.into();
        assert!(matches!(error, RssError::IoError(_)));
    }
}
