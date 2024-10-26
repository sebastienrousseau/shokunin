// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! News Sitemap Generation Module
//!
//! This module handles the creation and generation of Google News sitemaps,
//! which help search engines discover and index news content. It follows the
//! Google News Sitemap protocol specification.
//!
//! # Features
//! - Creation of news sitemap data structures from metadata
//! - Validation of news publication dates
//! - Proper XML formatting for news sitemaps
//! - Support for news-specific metadata (genres, keywords, etc.)
//!
//! # Example
//! ```
//! use std::collections::HashMap;
//! use staticrux::modules::news_sitemap::{create_news_site_map_data, convert_date_format};
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("news_title".to_string(), "Breaking News".to_string());
//! metadata.insert(
//!     "news_publication_date".to_string(),
//!     "Tue, 20 Feb 2024 15:15:15 GMT".to_string()
//! );
//!
//! let news_data = create_news_site_map_data(&metadata);
//! ```

use crate::models::data::NewsData;
use std::collections::HashMap;

/// Creates a NewsData object from metadata.
///
/// This function processes metadata to create a news sitemap entry.
/// All dates are converted to the required format and content is validated.
///
/// # Arguments
/// * `metadata` - A reference to a HashMap containing news metadata
///
/// # Returns
/// * `NewsData` - A struct containing the news sitemap information
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use staticrux::modules::news_sitemap::create_news_site_map_data;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("news_title".to_string(), "Breaking News".to_string());
///
/// let news_data = create_news_site_map_data(&metadata);
/// ```
pub fn create_news_site_map_data(
    metadata: &HashMap<String, String>,
) -> NewsData {
    NewsData {
        news_genres: sanitize_genres(
            metadata.get("news_genres").unwrap_or(&String::new()),
        ),
        news_image_loc: sanitize_url(
            metadata.get("news_image_loc").unwrap_or(&String::new()),
        ),
        news_keywords: sanitize_keywords(
            metadata.get("news_keywords").unwrap_or(&String::new()),
        ),
        news_language: sanitize_language(
            metadata.get("news_language").unwrap_or(&String::new()),
        ),
        news_loc: sanitize_url(
            metadata.get("news_loc").unwrap_or(&String::new()),
        ),
        news_publication_date: convert_date_format(
            metadata
                .get("news_publication_date")
                .unwrap_or(&String::new()),
        ),
        news_publication_name: sanitize_text(
            metadata
                .get("news_publication_name")
                .unwrap_or(&String::new()),
        ),
        news_title: sanitize_text(
            metadata.get("news_title").unwrap_or(&String::new()),
        ),
    }
}

/// Converts date strings from "Tue, 20 Feb 2024 15:15:15 GMT" format to ISO 8601.
///
/// # Arguments
/// * `input` - A string slice representing the input date
///
/// # Returns
/// * `String` - The date in ISO 8601 format (YYYY-MM-DDTHH:MM:SS+00:00)
///
/// # Example
/// ```
/// use staticrux::modules::news_sitemap::convert_date_format;
///
/// let date = convert_date_format("Tue, 20 Feb 2024 15:15:15 GMT");
/// assert_eq!(date, "2024-02-20T15:15:15+00:00");
/// ```
pub fn convert_date_format(input: &str) -> String {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 6 {
        return String::new();
    }

    let day = parts[1];
    let month = match parts[2] {
        "Jan" => "01",
        "Feb" => "02",
        "Mar" => "03",
        "Apr" => "04",
        "May" => "05",
        "Jun" => "06",
        "Jul" => "07",
        "Aug" => "08",
        "Sep" => "09",
        "Oct" => "10",
        "Nov" => "11",
        "Dec" => "12",
        _ => return String::new(),
    };
    let year = parts[3];
    let time = parts[4];

    format!("{}-{}-{}T{}+00:00", year, month, day, time)
}

/// Sanitizes news genres.
///
/// Ensures genres are valid according to Google News sitemap specifications.
fn sanitize_genres(genres: &str) -> String {
    let valid_genres = [
        "PressRelease",
        "Satire",
        "Blog",
        "OpEd",
        "Opinion",
        "UserGenerated",
    ];

    genres
        .split(',')
        .filter_map(|g| {
            let cleaned = g.trim();
            if valid_genres.contains(&cleaned) {
                Some(cleaned.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(", ")
}

/// Sanitizes news keywords.
///
/// Ensures keywords are properly formatted and within length limits.
fn sanitize_keywords(keywords: &str) -> String {
    keywords
        .split(',')
        .take(10) // Google News limit
        .map(|k| k.trim())
        .filter(|k| !k.is_empty())
        .collect::<Vec<&str>>()
        .join(", ")
}

/// Sanitizes language codes.
///
/// Ensures language codes follow ISO 639-1 format.
fn sanitize_language(lang: &str) -> String {
    if lang.len() == 2 && lang.chars().all(|c| c.is_ascii_lowercase()) {
        lang.to_string()
    } else {
        "en".to_string() // Default to English
    }
}

/// Sanitizes URLs.
///
/// Ensures URLs are valid and safe.
fn sanitize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        if url.contains('<') || url.contains('>') || url.contains('"') {
            String::new()
        } else {
            url.to_string()
        }
    } else {
        String::new()
    }
}

/// Sanitizes text content.
///
/// Removes any unsafe characters and limits length.
fn sanitize_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(1000) // Reasonable limit for titles/names
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_news_site_map_data() {
        let mut metadata = HashMap::new();
        metadata
            .insert("news_title".to_string(), "Test News".to_string());
        metadata.insert(
            "news_publication_date".to_string(),
            "Tue, 20 Feb 2024 15:15:15 GMT".to_string(),
        );

        let news_data = create_news_site_map_data(&metadata);
        assert_eq!(news_data.news_title, "Test News");
        assert_eq!(
            news_data.news_publication_date,
            "2024-02-20T15:15:15+00:00"
        );
    }

    #[test]
    fn test_convert_date_format() {
        let input = "Tue, 20 Feb 2024 15:15:15 GMT";
        assert_eq!(
            convert_date_format(input),
            "2024-02-20T15:15:15+00:00"
        );

        // Invalid formats
        assert!(convert_date_format("Invalid Date").is_empty());
        assert!(convert_date_format("").is_empty());
    }

    #[test]
    fn test_sanitize_genres() {
        assert_eq!(
            sanitize_genres("Blog, OpEd, Invalid"),
            "Blog, OpEd"
        );
        assert_eq!(
            sanitize_genres("PressRelease,Satire"),
            "PressRelease, Satire"
        );
        assert!(sanitize_genres("Invalid").is_empty());
    }

    #[test]
    fn test_sanitize_keywords() {
        assert_eq!(
            sanitize_keywords("news, breaking, update"),
            "news, breaking, update"
        );
        // Test limit
        let many_keywords = (0..20)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            sanitize_keywords(&many_keywords).split(", ").count(),
            10
        );
    }

    #[test]
    fn test_sanitize_language() {
        assert_eq!(sanitize_language("en"), "en");
        assert_eq!(sanitize_language("fr"), "fr");
        assert_eq!(sanitize_language("invalid"), "en");
        assert_eq!(sanitize_language(""), "en");
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(
            sanitize_url("https://example.com"),
            "https://example.com"
        );
        assert!(sanitize_url("invalid-url").is_empty());
        assert!(sanitize_url("https://example.com<script>").is_empty());
    }

    #[test]
    fn test_sanitize_text() {
        assert_eq!(sanitize_text("Normal text"), "Normal text");
        assert_eq!(
            sanitize_text("Text\nwith\tcontrol\rchars"),
            "Textwithcontrolchars"
        );

        // Test length limit
        let long_text = "a".repeat(2000);
        assert_eq!(sanitize_text(&long_text).len(), 1000);
    }

    #[test]
    fn test_empty_metadata() {
        let news_data = create_news_site_map_data(&HashMap::new());
        assert!(news_data.news_title.is_empty());
        assert!(news_data.news_publication_date.is_empty());
        assert_eq!(news_data.news_language, "en");
    }
}
