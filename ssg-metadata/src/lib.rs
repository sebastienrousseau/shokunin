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
//!
//! ## Main Components
//!
//! - `extract_metadata`: Extracts metadata from content
//! - `process_metadata`: Processes and validates extracted metadata
//! - `extract_and_prepare_metadata`: Combines extraction, processing, and meta tag generation

use anyhow::{Context, Result};
use std::collections::HashMap;

/// Functions for escaping special characters in metadata values
pub mod escape;

/// Functions for extracting front matter from content files
pub mod extractor;

/// Functions for processing and validating metadata
pub mod processor;

/// Data structures for representing metadata and related information
pub mod models;

/// Functions for generating keywords from metadata
pub mod keywords;

/// Functions for generating HTML meta tags
pub mod metatags;

/// Macros for common metadata operations
pub mod macros;

pub use extractor::extract_metadata;
use models::MetaTagGroups;
pub use processor::process_metadata;

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
/// This function will return an error if metadata extraction fails.
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
pub fn extract_and_prepare_metadata(
    content: &str,
) -> Result<(HashMap<String, String>, Vec<String>, MetaTagGroups)> {
    let metadata = extract_metadata(content)
        .context("Failed to extract metadata")?;
    let metadata_map = metadata.into_inner();
    let keywords = keywords::extract_keywords(&metadata_map);
    let all_meta_tags = metatags::generate_all_meta_tags(&metadata_map);

    Ok((metadata_map, keywords, all_meta_tags))
}
