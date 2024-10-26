// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Humans.txt Generation Module
//!
//! This module handles the creation and generation of humans.txt files, which provide
//! information about the people behind a website. The humans.txt file is a text file
//! that contains information about the different people who have contributed to
//! building the website.
//!
//! # Features
//! - Creation of humans.txt data structures from metadata
//! - Generation of humans.txt content
//! - Validation and sanitization of content
//! - Structured output following humans.txt convention
//!
//! # Example
//! ```
//! use std::collections::HashMap;
//! use staticrux::modules::human::{create_human_data, generate_humans_content};
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("author".to_string(), "John Doe".to_string());
//! metadata.insert("author_website".to_string(), "https://example.com".to_string());
//!
//! let humans_data = create_human_data(&metadata);
//! let content = generate_humans_content(&humans_data);
//! ```

use crate::models::data::HumansData;
use std::collections::HashMap;

/// Creates a HumansData object from metadata.
///
/// This function extracts humans.txt information from the provided metadata and creates
/// a HumansData object. All fields are sanitized before being stored.
///
/// # Arguments
/// * `metadata` - A reference to a HashMap containing metadata key-value pairs
///
/// # Returns
/// * `HumansData` - A struct containing the humans.txt information
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use staticrux::modules::human::create_human_data;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("author".to_string(), "John Doe".to_string());
///
/// let humans_data = create_human_data(&metadata);
/// assert_eq!(humans_data.author, "John Doe");
/// ```
pub fn create_human_data(
    metadata: &HashMap<String, String>,
) -> HumansData {
    HumansData {
        author: sanitize_text(
            metadata.get("author").unwrap_or(&String::new()),
        ),
        author_website: sanitize_url(
            metadata.get("author_website").unwrap_or(&String::new()),
        ),
        author_twitter: sanitize_twitter_handle(
            metadata.get("author_twitter").unwrap_or(&String::new()),
        ),
        author_location: sanitize_text(
            metadata.get("author_location").unwrap_or(&String::new()),
        ),
        site_components: sanitize_text(
            metadata.get("site_components").unwrap_or(&String::new()),
        ),
        site_last_updated: sanitize_date(
            metadata.get("site_last_updated").unwrap_or(&String::new()),
        ),
        site_standards: sanitize_text(
            metadata.get("site_standards").unwrap_or(&String::new()),
        ),
        site_software: sanitize_text(
            metadata.get("site_software").unwrap_or(&String::new()),
        ),
        thanks: sanitize_text(
            metadata.get("thanks").unwrap_or(&String::new()),
        ),
    }
}

/// Generates humans.txt content.
///
/// This function takes a HumansData object and generates properly formatted
/// humans.txt content following the established convention.
///
/// # Arguments
/// * `data` - A reference to a HumansData object containing the information
///
/// # Returns
/// * `String` - The generated humans.txt content
///
/// # Example
/// ```
/// use staticrux::models::data::HumansData;
/// use staticrux::modules::human::generate_humans_content;
///
/// let data = HumansData {
///     author: "John Doe".to_string(),
///     author_website: "https://example.com".to_string(),
///     ..Default::default()
/// };
///
/// let content = generate_humans_content(&data);
/// assert!(content.contains("/* TEAM */"));
/// assert!(content.contains("Name: John Doe"));
/// ```
pub fn generate_humans_content(data: &HumansData) -> String {
    let mut content = String::with_capacity(500); // Pre-allocate reasonable capacity

    // TEAM section
    content.push_str("/* TEAM */\n");
    if !data.author.is_empty() {
        content.push_str(&format!("    Name: {}\n", data.author));
    }
    if !data.author_website.is_empty() {
        content.push_str(&format!(
            "    Website: {}\n",
            data.author_website
        ));
    }
    if !data.author_twitter.is_empty() {
        content.push_str(&format!(
            "    Twitter: {}\n",
            data.author_twitter
        ));
    }
    if !data.author_location.is_empty() {
        content.push_str(&format!(
            "    Location: {}\n",
            data.author_location
        ));
    }

    // THANKS section
    content.push_str("\n/* THANKS */\n");
    if !data.thanks.is_empty() {
        content.push_str(&format!("    Thanks: {}\n", data.thanks));
    }

    // SITE section
    content.push_str("\n/* SITE */\n");
    if !data.site_last_updated.is_empty() {
        content.push_str(&format!(
            "    Last update: {}\n",
            data.site_last_updated
        ));
    }
    if !data.site_standards.is_empty() {
        content.push_str(&format!(
            "    Standards: {}\n",
            data.site_standards
        ));
    }
    if !data.site_components.is_empty() {
        content.push_str(&format!(
            "    Components: {}\n",
            data.site_components
        ));
    }
    if !data.site_software.is_empty() {
        content.push_str(&format!(
            "    Software: {}\n",
            data.site_software
        ));
    }

    content
}

/// Sanitizes general text content.
///
/// Removes any control characters and limits length to 100 characters.
fn sanitize_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(100).collect()
}

/// Sanitizes and validates a URL.
///
/// Ensures URL starts with http:// or https:// and contains valid characters.
fn sanitize_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return String::new();
    }

    // Basic URL character validation
    if !url.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == ':'
            || c == '/'
            || c == '.'
            || c == '-'
            || c == '_'
    }) {
        return String::new();
    }

    url.to_string()
}

/// Sanitizes and validates a Twitter handle.
///
/// Ensures handle starts with @ and contains only valid Twitter username characters.
fn sanitize_twitter_handle(handle: &str) -> String {
    if handle.is_empty() {
        return String::new();
    }

    if !handle.starts_with('@') {
        return String::new();
    }

    // Twitter handle validation (1-15 characters after @, alphanumeric and underscore only)
    let username = &handle[1..];
    if username.len() > 15
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return String::new();
    }

    handle.to_string()
}

/// Sanitizes and validates a date string.
///
/// Accepts dates in YYYY-MM-DD format.
fn sanitize_date(date: &str) -> String {
    if date.len() != 10 {
        return String::new();
    }

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return String::new();
    }

    // Basic validation of year, month, day
    if parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
    {
        return String::new();
    }

    date.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_human_data() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "John Doe".to_string());
        metadata.insert(
            "author_website".to_string(),
            "https://example.com".to_string(),
        );
        metadata.insert(
            "author_twitter".to_string(),
            "@johndoe".to_string(),
        );

        let data = create_human_data(&metadata);
        assert_eq!(data.author, "John Doe");
        assert_eq!(data.author_website, "https://example.com");
        assert_eq!(data.author_twitter, "@johndoe");
    }

    #[test]
    fn test_generate_humans_content() {
        let data = HumansData {
            author: "John Doe".to_string(),
            author_website: "https://example.com".to_string(),
            author_twitter: "@johndoe".to_string(),
            author_location: "New York".to_string(),
            thanks: "Contributors".to_string(),
            site_last_updated: "2024-01-01".to_string(),
            site_standards: "HTML5, CSS3".to_string(),
            site_components: "Rust, SSG".to_string(),
            site_software: "Shokunin".to_string(),
        };

        let content = generate_humans_content(&data);
        assert!(content.contains("/* TEAM */"));
        assert!(content.contains("Name: John Doe"));
        assert!(content.contains("Website: https://example.com"));
        assert!(content.contains("Twitter: @johndoe"));
    }

    #[test]
    fn test_sanitize_text() {
        assert_eq!(sanitize_text("Normal text"), "Normal text");
        assert_eq!(
            sanitize_text("Text\nwith\tcontrol\rchars"),
            "Textwithcontrolchars"
        );
        assert_eq!(sanitize_text(&"a".repeat(150)), "a".repeat(100));
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(
            sanitize_url("https://example.com"),
            "https://example.com"
        );
        assert_eq!(
            sanitize_url("http://example.com"),
            "http://example.com"
        );
        assert!(sanitize_url("not-a-url").is_empty());
        assert!(sanitize_url("ftp://example.com").is_empty());
    }

    #[test]
    fn test_sanitize_twitter_handle() {
        assert_eq!(sanitize_twitter_handle("@username"), "@username");
        assert!(sanitize_twitter_handle("username").is_empty());
        assert!(sanitize_twitter_handle("@invalid!handle").is_empty());
        assert!(
            sanitize_twitter_handle("@toolong_username_123").is_empty()
        );
    }

    #[test]
    fn test_sanitize_date() {
        assert_eq!(sanitize_date("2024-01-01"), "2024-01-01");
        assert!(sanitize_date("2024/01/01").is_empty());
        assert!(sanitize_date("24-01-01").is_empty());
        assert!(sanitize_date("2024-1-1").is_empty());
        assert!(sanitize_date("not-a-date").is_empty());
    }

    #[test]
    fn test_empty_humans_data() {
        let data = create_human_data(&HashMap::new());
        let content = generate_humans_content(&data);
        assert!(content.contains("/* TEAM */"));
        assert!(content.contains("/* THANKS */"));
        assert!(content.contains("/* SITE */"));
        assert!(!content.contains("Name:"));
    }
}
