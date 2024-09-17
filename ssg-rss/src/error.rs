// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use thiserror::Error;

/// Errors that can occur when generating RSS feeds.
#[derive(Error, Debug)]
pub enum RssError {
    /// Error occurred while writing XML.
    #[error("XML writing error: {0}")]
    XmlWriteError(#[from] quick_xml::Error),

    /// Error occurred during UTF-8 conversion.
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    /// Error indicating a required field is missing.
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Result type for RSS operations.
pub type Result<T> = std::result::Result<T, RssError>;
