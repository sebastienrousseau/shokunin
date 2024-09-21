//! # ssg-metadata
//!
//! `ssg-metadata` is a library for extracting and processing metadata
//! for static site generators. It supports various formats including
//! YAML, TOML, and JSON front matter in Markdown files.
//!
//! This crate provides functionality to:
//! - Extract metadata from content files
//! - Process and validate metadata
//! - Generate keywords based on metadata
//! - Create meta tags for HTML documents
//! - Asynchronously extract metadata from files

/// The `error` module contains error types for metadata processing.
pub mod error;
/// The `metadata` module contains functions for extracting and processing metadata.
pub mod metadata;
/// The `metatags` module contains functions for generating meta tags.
pub mod metatags;
/// The `utils` module contains utility functions for metadata processing.
pub mod utils;

pub use error::MetadataError;
pub use metadata::{extract_metadata, process_metadata, Metadata};
pub use metatags::{generate_metatags, MetaTagGroups};
pub use utils::{
    async_extract_metadata_from_file, escape_html_entities,
};

use std::collections::HashMap;

/// Type aliases for improving readability and reducing complexity
type MetadataMap = HashMap<String, String>;
type Keywords = Vec<String>;
type MetadataResult =
    Result<(MetadataMap, Keywords, MetaTagGroups), MetadataError>;

/// Extracts metadata from the content, generates keywords based on the metadata,
/// and prepares meta tag groups.
///
/// This function performs three key tasks:
/// 1. It extracts metadata from the front matter of the content.
/// 2. It extracts keywords based on this metadata.
/// 3. It generates various meta tags required for the page.
///
/// # Arguments
///
/// * `content` - A string slice representing the content from which to extract metadata.
///
/// # Returns
///
/// Returns a Result containing a tuple with:
/// * `HashMap<String, String>`: Extracted metadata
/// * `Vec<String>`: A list of keywords
/// * `MetaTagGroups`: A structure containing various meta tags
///
/// # Errors
///
/// This function will return a `MetadataError` if metadata extraction or processing fails.
///
/// # Example
///
/// ```
/// use ssg_metadata::extract_and_prepare_metadata;
///
/// let content = r#"---
/// title: My Page
/// description: A sample page
/// ---
/// # Content goes here
/// "#;
///
/// let result = extract_and_prepare_metadata(content);
/// assert!(result.is_ok());
/// ```
pub fn extract_and_prepare_metadata(content: &str) -> MetadataResult {
    let metadata = extract_metadata(content)?;
    let metadata_map = metadata.into_inner();
    let keywords = extract_keywords(&metadata_map);
    let all_meta_tags = generate_metatags(&metadata_map);

    Ok((metadata_map, keywords, all_meta_tags))
}

/// Extracts keywords from the metadata.
///
/// This function looks for a "keywords" key in the metadata and splits its value into a vector of strings.
///
/// # Arguments
///
/// * `metadata` - A reference to a HashMap containing the metadata.
///
/// # Returns
///
/// A vector of strings representing the keywords.
pub fn extract_keywords(
    metadata: &HashMap<String, String>,
) -> Vec<String> {
    metadata
        .get("keywords")
        .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}
