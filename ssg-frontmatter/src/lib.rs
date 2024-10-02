//! # SSG Frontmatter
//!
//! `ssg-frontmatter` is a Rust library for parsing and serializing frontmatter in various formats, including YAML, TOML, and JSON.
//! Frontmatter is commonly used in static site generators (SSG) to store metadata at the beginning of content files.
//!
//! This library provides functions to extract, parse, and convert frontmatter between different formats, making it easy to work with frontmatter data in Rust applications.
//!
//! ## Features
//! - Extract frontmatter from content files.
//! - Parse frontmatter into a structured format.
//! - Convert frontmatter between YAML, TOML, and JSON formats.
//!
//! ## Example
//! ```rust
//! use ssg_frontmatter::{Format, Frontmatter, to_format};
//!
//! let mut frontmatter = Frontmatter::new();
//! frontmatter.insert("title".to_string(), "My Post".into());
//! frontmatter.insert("date".to_string(), "2023-05-20".into());
//!
//! let yaml = to_format(&frontmatter, Format::Yaml).unwrap();
//! assert!(yaml.contains("title: My Post"));
//! assert!(yaml.contains("date: '2023-05-20'"));
//! ```
//!
//! ## Modules
//! - `error`: Contains error types used throughout the library.
//! - `extractor`: Provides functions for extracting raw frontmatter.
//! - `parser`: Handles the parsing of frontmatter from raw strings.
//! - `types`: Defines the core types such as `Frontmatter`, `Value`, and `Format`.

/// The `error` module contains error types related to the frontmatter parsing process.
pub mod error;
/// The `extractor` module contains functions for extracting raw frontmatter from content.
pub mod extractor;
/// The `parser` module contains functions for parsing frontmatter into a structured format.
pub mod parser;
/// The `types` module contains types related to the frontmatter parsing process.
pub mod types;

use error::FrontmatterError;
use extractor::{detect_format, extract_raw_frontmatter};
use parser::{parse, to_string};
// Re-export types for external access
pub use types::{Format, Frontmatter, Value}; // Add `Frontmatter` and `Format` to the public interface

/// Extracts frontmatter from a string of content.
///
/// This function attempts to extract frontmatter from the given content string.
/// It supports YAML, TOML, and JSON formats.
///
/// # Arguments
///
/// * `content` - A string slice containing the content to parse.
///
/// # Returns
///
/// * `Ok((Frontmatter, &str))` - A tuple containing the parsed frontmatter and the remaining content.
/// * `Err(FrontmatterError)` - An error if extraction or parsing fails.
///
/// # Examples
///
/// ```
/// use ssg_frontmatter::{extract, Frontmatter};
///
/// let yaml_content = r#"---
/// title: My Post
/// date: 2023-05-20
/// ---
/// Content here"#;
///
/// let (frontmatter, remaining_content) = extract(yaml_content).unwrap();
/// assert_eq!(frontmatter.get("title").unwrap().as_str().unwrap(), "My Post");
/// assert_eq!(remaining_content, "Content here");
/// ```
pub fn extract(
    content: &str,
) -> Result<(Frontmatter, &str), FrontmatterError> {
    let (raw_frontmatter, remaining_content) =
        extract_raw_frontmatter(content)?;
    let format = detect_format(raw_frontmatter)?;
    let frontmatter = parse(raw_frontmatter, format)?;
    Ok((frontmatter, remaining_content))
}

/// Converts frontmatter to a specific format.
///
/// # Arguments
///
/// * `frontmatter` - The Frontmatter to convert.
/// * `format` - The target Format to convert to.
///
/// # Returns
///
/// * `Ok(String)` - The frontmatter converted to the specified format.
/// * `Err(FrontmatterError)` - An error if conversion fails.
///
/// # Examples
///
/// ```
/// use ssg_frontmatter::{Frontmatter, Format, to_format};
///
/// let mut frontmatter = Frontmatter::new();
/// frontmatter.insert("title".to_string(), "My Post".into());
/// frontmatter.insert("date".to_string(), "2023-05-20".into());
///
/// let yaml = to_format(&frontmatter, Format::Yaml).unwrap();
/// assert!(yaml.contains("title: My Post"));
/// assert!(yaml.contains("date: '2023-05-20'"));
/// ```
pub fn to_format(
    frontmatter: &Frontmatter,
    format: Format,
) -> Result<String, FrontmatterError> {
    to_string(frontmatter, format)
}
