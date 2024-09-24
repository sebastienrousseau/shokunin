// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use quick_xml;
use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;

/// Errors that can occur when generating RSS feeds.
///
/// This enum represents various error types that may occur during RSS feed generation,
/// including XML writing errors, UTF-8 conversion errors, missing required fields,
/// and general I/O errors.
#[derive(Debug)]
pub enum RssError {
    /// Error occurred while writing XML.
    XmlWriteError(quick_xml::Error),

    /// Error occurred during UTF-8 conversion.
    Utf8Error(FromUtf8Error),

    /// Error indicating a required field is missing.
    MissingField(String),

    /// Error for any I/O operations.
    IoError(String),

    /// Error for invalid input data.
    InvalidInput,
}

/// Custom implementation to avoid leaking sensitive information in error messages
impl fmt::Display for RssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssError::XmlWriteError(_) => {
                write!(f, "XML writing error occurred")
            }
            RssError::Utf8Error(_) => {
                write!(f, "UTF-8 conversion error occurred")
            }
            RssError::MissingField(_) => {
                write!(f, "A required field is missing")
            }
            RssError::IoError(_) => write!(f, "An I/O error occurred"),
            RssError::InvalidInput => {
                write!(f, "Invalid input data provided")
            }
        }
    }
}

impl Error for RssError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RssError::XmlWriteError(e) => Some(e),
            RssError::Utf8Error(e) => Some(e),
            RssError::IoError(_) => None,
            RssError::MissingField(_) | RssError::InvalidInput => None,
        }
    }
}

impl Clone for RssError {
    fn clone(&self) -> Self {
        match self {
            RssError::XmlWriteError(e) => {
                RssError::XmlWriteError(e.clone())
            }
            RssError::Utf8Error(e) => RssError::Utf8Error(e.clone()),
            RssError::MissingField(s) => {
                RssError::MissingField(s.clone())
            }
            RssError::IoError(s) => RssError::IoError(s.clone()),
            RssError::InvalidInput => RssError::InvalidInput,
        }
    }
}

/// Result type for RSS operations.
///
/// This type alias provides a convenient way to return results from RSS operations,
/// where the error type is always `RssError`.
pub type Result<T> = std::result::Result<T, RssError>;

impl RssError {
    /// Creates a new `RssError::MissingField` error.
    ///
    /// This method provides a convenient way to create a `MissingField` error
    /// with a given field name.
    ///
    /// # Arguments
    ///
    /// * `field_name` - The name of the missing field.
    ///
    /// # Returns
    ///
    /// Returns a new `RssError::MissingField` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_rss::error::RssError;
    ///
    /// let error = RssError::missing_field("title");
    /// assert_eq!(error.to_string(), "A required field is missing");
    /// ```
    pub fn missing_field<S: Into<String>>(field_name: S) -> Self {
        RssError::MissingField(field_name.into())
    }

    /// Securely logs an error without exposing sensitive details.
    ///
    /// This method should be used to log errors in a way that doesn't reveal
    /// sensitive information to log files or monitoring systems.
    pub fn log(&self) {
        // Implement secure logging here. For example:
        // log::error!("RSS Error occurred: {}", self);
    }
}

// Implement From for RssError to allow ? operator usage
impl From<std::string::FromUtf8Error> for RssError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        RssError::Utf8Error(error)
    }
}

impl From<quick_xml::Error> for RssError {
    fn from(error: quick_xml::Error) -> Self {
        RssError::XmlWriteError(error)
    }
}

impl From<std::io::Error> for RssError {
    fn from(error: std::io::Error) -> Self {
        RssError::IoError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_rss_error_display() {
        let error = RssError::missing_field("title");
        assert_eq!(error.to_string(), "A required field is missing");
    }

    #[test]
    fn test_xml_write_error() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::Other, "XML error"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert_eq!(error.to_string(), "XML writing error occurred");
    }

    #[test]
    fn test_utf8_error() {
        let utf8_error =
            String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error = RssError::Utf8Error(utf8_error);
        assert_eq!(
            error.to_string(),
            "UTF-8 conversion error occurred"
        );
    }

    #[test]
    fn test_io_error() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = RssError::IoError(io_error.to_string());
        assert_eq!(error.to_string(), "An I/O error occurred");
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RssError>();
    }

    #[test]
    fn test_error_source() {
        let xml_error = quick_xml::Error::Io(std::sync::Arc::new(
            io::Error::new(io::ErrorKind::NotFound, "File not found"),
        ));
        let error = RssError::XmlWriteError(xml_error);
        assert!(error.source().is_some());

        let error = RssError::IoError("File not found".to_string());
        assert!(error.source().is_none());
    }

    #[test]
    fn test_missing_field_with_string() {
        let error = RssError::missing_field(String::from("author"));
        assert_eq!(error.to_string(), "A required field is missing");
    }

    #[test]
    fn test_missing_field_with_str() {
        let error = RssError::missing_field("description");
        assert_eq!(error.to_string(), "A required field is missing");
    }

    #[test]
    fn test_error_downcast() {
        let error: Box<dyn Error> =
            Box::new(RssError::missing_field("category"));
        let downcast_result = error.downcast::<RssError>();
        assert!(downcast_result.is_ok());
    }

    #[test]
    fn test_from_quick_xml_error() {
        let xml_error =
            quick_xml::Error::Io(std::sync::Arc::new(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected EOF",
            )));
        let error: RssError = xml_error.into();
        assert!(matches!(error, RssError::XmlWriteError(_)));
    }

    #[test]
    fn test_from_utf8_error() {
        let utf8_error =
            String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let error: RssError = utf8_error.into();
        assert!(matches!(error, RssError::Utf8Error(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_error =
            io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error: RssError = io_error.into();
        assert!(matches!(error, RssError::IoError(_)));
    }

    #[test]
    fn test_invalid_input_error() {
        let error = RssError::InvalidInput;
        assert_eq!(error.to_string(), "Invalid input data provided");
    }

    #[test]
    fn test_error_clone() {
        let error = RssError::missing_field("title");
        let cloned_error = error.clone();
        assert_eq!(error.to_string(), cloned_error.to_string());
    }
}
