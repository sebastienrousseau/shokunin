// src/error.rs

use thiserror::Error;

/// Errors that can occur when working with sitemaps.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SitemapError {
    /// Error occurred during XML parsing or writing.
    #[error("XML error: {0}")]
    XmlError(#[from] xml::writer::Error),

    /// Error occurred during date parsing or formatting.
    #[error("Date error: {0}")]
    DateError(String),

    /// Error occurred during URL parsing.
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),

    /// Error occurred during I/O operations.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error occurred during string encoding.
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Invalid change frequency provided.
    #[error("Invalid change frequency: {0}")]
    InvalidChangeFreq(String),
}
