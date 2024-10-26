// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CNAME Record Generation Module
//!
//! This module handles the creation and generation of CNAME (Canonical Name) records
//! for domain name configuration. CNAME records are DNS entries that map one domain
//! name (alias) to another (canonical name).
//!
//! # Features
//! - Creation of CNAME data structures from metadata
//! - Generation of CNAME record content
//! - Validation and sanitization of domain names
//! - Secure handling of domain data
//!
//! # Example
//! ```
//! use std::collections::HashMap;
//! use staticrux::modules::cname::{create_cname_data, generate_cname_content};
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("cname".to_string(), "example.com".to_string());
//!
//! let cname_data = create_cname_data(&metadata);
//! let content = generate_cname_content(&cname_data);
//! ```

use crate::models::data::CnameData;
use std::collections::HashMap;

/// Creates a CnameData object from metadata.
///
/// This function extracts CNAME information from the provided metadata and creates
/// a CnameData object. The domain name is sanitized and validated before being stored.
///
/// # Arguments
/// * `metadata` - A reference to a HashMap containing metadata key-value pairs
///
/// # Returns
/// * `CnameData` - A struct containing the CNAME record information
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use staticrux::modules::cname::create_cname_data;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("cname".to_string(), "example.com".to_string());
///
/// let cname_data = create_cname_data(&metadata);
/// assert_eq!(cname_data.cname, "example.com");
/// ```
pub fn create_cname_data(
    metadata: &HashMap<String, String>,
) -> CnameData {
    CnameData {
        cname: metadata
            .get("cname")
            .map(|s| sanitize_domain(s))
            .unwrap_or_default(),
    }
}

/// Generates CNAME record content.
///
/// This function takes a CnameData object and generates the appropriate CNAME
/// record content. It creates both the base domain and www subdomain entries.
///
/// # Arguments
/// * `data` - A reference to a CnameData object containing the CNAME information
///
/// # Returns
/// * `String` - The generated CNAME record content
///
/// # Example
/// ```
/// use staticrux::models::data::CnameData;
/// use staticrux::modules::cname::generate_cname_content;
///
/// let data = CnameData {
///     cname: "example.com".to_string(),
/// };
///
/// let content = generate_cname_content(&data);
/// assert_eq!(content, "example.com\nwww.example.com");
/// ```
pub fn generate_cname_content(data: &CnameData) -> String {
    if data.cname.is_empty() {
        return String::new();
    }

    format!("{}\nwww.{}", data.cname, data.cname)
}

/// Sanitizes and validates a domain name.
///
/// This function ensures that domain names:
/// - Contain only valid characters (alphanumeric, hyphens, periods)
/// - Follow DNS naming rules:
///   * Contain at least one period
///   * Don't exceed 253 characters
///   * Don't start or end with a hyphen
///   * Don't have consecutive periods
///   * Have valid label lengths (<=63 characters)
///
/// # Arguments
/// * `domain` - The domain name to sanitize and validate
///
/// # Returns
/// * `String` - The sanitized domain name, or empty string if invalid
fn sanitize_domain(domain: &str) -> String {
    // Remove invalid characters and keep only alphanumeric, hyphen, and period
    let sanitized: String = domain
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
        .collect();

    // Ensure the domain is not empty after sanitization
    if sanitized.is_empty() || !sanitized.contains('.') {
        return String::new();
    }

    // Validate domain length (must be <= 253 chars)
    if sanitized.len() > 253 {
        return String::new();
    }

    // Ensure no consecutive periods
    if sanitized.contains("..") {
        return String::new();
    }

    // Split domain into labels
    let labels: Vec<&str> = sanitized.split('.').collect();

    // Validate each label (part between periods)
    for label in labels {
        // Label must be non-empty and <= 63 characters
        if label.is_empty() || label.len() > 63 {
            return String::new();
        }

        // Check for valid characters
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return String::new();
        }

        // No label can start or end with hyphen
        if label.starts_with('-') || label.ends_with('-') {
            return String::new();
        }
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cname_data() {
        let mut metadata = HashMap::new();
        metadata.insert("cname".to_string(), "example.com".to_string());

        let data = create_cname_data(&metadata);
        assert_eq!(data.cname, "example.com");
    }

    #[test]
    fn test_create_cname_data_empty() {
        let metadata = HashMap::new();
        let data = create_cname_data(&metadata);
        assert!(data.cname.is_empty());
    }

    #[test]
    fn test_generate_cname_content() {
        let data = CnameData {
            cname: "example.com".to_string(),
        };
        assert_eq!(
            generate_cname_content(&data),
            "example.com\nwww.example.com"
        );
    }

    #[test]
    fn test_generate_cname_content_empty() {
        let data = CnameData {
            cname: String::new(),
        };
        assert!(generate_cname_content(&data).is_empty());
    }

    #[test]
    fn test_sanitize_domain_valid() {
        assert_eq!(sanitize_domain("example.com"), "example.com");
        assert_eq!(
            sanitize_domain("sub-domain.example.com"),
            "sub-domain.example.com"
        );
    }

    #[test]
    fn test_sanitize_domain_edge_cases() {
        // Test near-maximum valid length (253 chars total, including .com)
        let near_max_domain = format!("{}.com", "a".repeat(249)); // 249 'a' + 4 = 253 chars total
        assert!(sanitize_domain(&near_max_domain).is_empty()); // This should now be invalid as it exceeds max length

        // Test maximum label length (63 chars is valid)
        let max_label = format!("{}.com", "a".repeat(63));
        assert_eq!(sanitize_domain(&max_label), max_label);

        // Test invalid label length (64 chars is invalid)
        let invalid_label = format!("{}.com", "a".repeat(64));
        assert!(sanitize_domain(&invalid_label).is_empty());
    }

    #[test]
    fn test_sanitize_domain_invalid() {
        // Invalid characters
        assert_eq!(sanitize_domain("example!@#$.com"), "example.com");

        // Invalid format
        assert!(sanitize_domain("example").is_empty());
        assert!(sanitize_domain("-example.com").is_empty());
        assert!(sanitize_domain("example-.com").is_empty());
        assert!(sanitize_domain("example..com").is_empty());
        assert!(sanitize_domain(&"a".repeat(254)).is_empty());

        // Additional edge cases
        assert!(sanitize_domain("example.-com").is_empty());
        assert!(sanitize_domain(".example.com").is_empty());
        assert!(sanitize_domain("example.com-").is_empty());
    }

    #[test]
    fn test_cname_data_with_invalid_domain() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "cname".to_string(),
            "invalid!@#domain.com".to_string(),
        );

        let data = create_cname_data(&metadata);
        assert_eq!(data.cname, "invaliddomain.com");
    }
}
