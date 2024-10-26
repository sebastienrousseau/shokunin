// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Robots.txt Generation Module
//!
//! This module handles the creation and generation of robots.txt files, which provide
//! instructions to web robots about accessing the website. The robots.txt file follows
//! the Robots Exclusion Protocol standard.
//!
//! # Features
//! - Creation of robots.txt data structures from metadata
//! - Validation of URLs and directives
//! - Generation of standard-compliant robots.txt content
//! - Secure handling of robot control directives
//!
//! # Example
//! ```
//! use std::collections::HashMap;
//! use staticrux::modules::robots::{create_txt_data, generate_txt_content};
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("permalink".to_string(), "https://example.com".to_string());
//!
//! let txt_data = create_txt_data(&metadata);
//! let content = generate_txt_content(&txt_data);
//! ```

use crate::models::data::TxtData;
use std::collections::HashMap;

/// Creates a TxtData object from metadata.
///
/// This function processes metadata to create a robots.txt configuration.
/// It validates the permalink URL and ensures it's properly formatted.
///
/// # Arguments
/// * `metadata` - A reference to a HashMap containing metadata key-value pairs
///
/// # Returns
/// * `TxtData` - A struct containing the robots.txt configuration
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use staticrux::modules::robots::create_txt_data;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("permalink".to_string(), "https://example.com".to_string());
///
/// let txt_data = create_txt_data(&metadata);
/// assert_eq!(txt_data.permalink, "https://example.com");
/// ```
pub fn create_txt_data(metadata: &HashMap<String, String>) -> TxtData {
    TxtData {
        permalink: sanitize_url(
            metadata.get("permalink").unwrap_or(&String::new()),
        ),
    }
}

/// Generates robots.txt content.
///
/// This function takes a TxtData object and generates properly formatted
/// robots.txt content following the standard protocol.
///
/// # Arguments
/// * `data` - A reference to a TxtData object containing the configuration
///
/// # Returns
/// * `String` - The generated robots.txt content
///
/// # Example
/// ```
/// use staticrux::models::data::TxtData;
/// use staticrux::modules::robots::generate_txt_content;
///
/// let data = TxtData {
///     permalink: "https://example.com".to_string(),
/// };
///
/// let content = generate_txt_content(&data);
/// assert!(content.contains("User-agent: *"));
/// assert!(content.contains("Sitemap:"));
/// ```
pub fn generate_txt_content(data: &TxtData) -> String {
    if data.permalink.is_empty() {
        return String::new();
    }

    format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml",
        data.permalink.trim_end_matches('/')
    )
}

/// Sanitizes and validates a URL.
///
/// Ensures the URL:
/// - Starts with http:// or https://
/// - Contains no dangerous characters
/// - Is properly formatted
///
/// # Arguments
/// * `url` - The URL to sanitize
///
/// # Returns
/// * `String` - The sanitized URL or empty string if invalid
fn sanitize_url(url: &str) -> String {
    // Check for empty URL
    if url.is_empty() {
        return String::new();
    }

    // Validate URL scheme
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return String::new();
    }

    // Remove any trailing slashes
    let clean_url = url.trim_end_matches('/');

    // Check for dangerous characters
    if clean_url.contains('<')
        || clean_url.contains('>')
        || clean_url.contains('"')
        || clean_url.contains('\'')
        || clean_url.contains('\\')
    {
        return String::new();
    }

    // Basic URL structure validation
    let parts: Vec<&str> = clean_url.split('/').collect();
    if parts.len() < 3 || !parts[2].contains('.') {
        return String::new();
    }

    clean_url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_txt_data() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "permalink".to_string(),
            "https://example.com".to_string(),
        );

        let data = create_txt_data(&metadata);
        assert_eq!(data.permalink, "https://example.com");
    }

    #[test]
    fn test_generate_txt_content() {
        let data = TxtData {
            permalink: "https://example.com".to_string(),
        };

        let content = generate_txt_content(&data);
        assert_eq!(
            content,
            "User-agent: *\nAllow: /\nSitemap: https://example.com/sitemap.xml"
        );
    }

    #[test]
    fn test_sanitize_url_valid() {
        assert_eq!(
            sanitize_url("https://example.com"),
            "https://example.com"
        );
        assert_eq!(
            sanitize_url("http://example.com"),
            "http://example.com"
        );
        assert_eq!(
            sanitize_url("https://example.com/"),
            "https://example.com"
        );
    }

    #[test]
    fn test_sanitize_url_invalid() {
        assert!(sanitize_url("").is_empty());
        assert!(sanitize_url("not-a-url").is_empty());
        assert!(sanitize_url("ftp://example.com").is_empty());
        assert!(sanitize_url("http://example").is_empty());
        assert!(sanitize_url("https://example.com<script>").is_empty());
    }

    #[test]
    fn test_generate_txt_content_empty() {
        let data = TxtData {
            permalink: String::new(),
        };

        let content = generate_txt_content(&data);
        assert!(content.is_empty());
    }

    #[test]
    fn test_generate_txt_content_trailing_slash() {
        let data = TxtData {
            permalink: "https://example.com/".to_string(),
        };

        let content = generate_txt_content(&data);
        assert_eq!(
            content,
            "User-agent: *\nAllow: /\nSitemap: https://example.com/sitemap.xml"
        );
    }

    #[test]
    fn test_create_txt_data_missing_permalink() {
        let metadata = HashMap::new();
        let data = create_txt_data(&metadata);
        assert!(data.permalink.is_empty());
    }

    #[test]
    fn test_sanitize_url_with_path() {
        assert_eq!(
            sanitize_url("https://example.com/path/to/resource"),
            "https://example.com/path/to/resource"
        );
    }

    #[test]
    fn test_sanitize_url_with_query_params() {
        assert_eq!(
            sanitize_url("https://example.com?param=value"),
            "https://example.com?param=value"
        );
    }
}
