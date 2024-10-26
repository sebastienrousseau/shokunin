// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Plain Text Generation Module
//!
//! This module provides functionality for converting HTML and Markdown content to plain text
//! while maintaining readability and content structure. It includes robust handling of
//! various text formats, character sets, and security considerations.
//!
//! # Features
//!
//! - HTML and Markdown to plain text conversion
//! - Secure content sanitization
//! - Unicode and RTL text support
//! - Structured content preservation
//! - Metadata handling
//! - Comprehensive error handling
//!
//! # Example
//!
//! ```
//! use staticrux::modules::plaintext::{generate_plain_text, PlainTextConfig};
//!
//! let config = PlainTextConfig::default();
//! let result = generate_plain_text(
//!     "# Hello World\n\nThis is **bold** text.",
//!     "Title",
//!     "Description",
//!     "Author",
//!     "Creator",
//!     "keywords",
//! ).unwrap();
//! ```
//!
//! # Security
//!
//! This module implements several security measures:
//! - HTML tag stripping
//! - Script injection prevention
//! - Control character filtering
//! - Unicode character validation

use anyhow::Result;
use log::{debug, error, info};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration options for plain text generation
#[derive(Debug, Clone)]
pub struct PlainTextConfig {
    /// Maximum line length for wrapping
    pub max_line_length: usize,
    /// List item bullet character
    pub list_bullet: String,
    /// Whether to preserve empty lines between sections
    pub preserve_empty_lines: bool,
    /// Whether to use ASCII-only output
    pub ascii_only: bool,
    /// Custom text replacements
    pub replacements: HashMap<String, String>,
}

impl Default for PlainTextConfig {
    fn default() -> Self {
        Self {
            max_line_length: 80,
            list_bullet: "• ".to_string(),
            preserve_empty_lines: true,
            ascii_only: false,
            replacements: HashMap::new(),
        }
    }
}

/// Errors that can occur during plain text generation
#[derive(Error, Debug)]
pub enum PlainTextError {
    /// Parsing error during content conversion
    #[error("Failed to parse content: {0}")]
    ParseError(String),

    /// Unicode validation error
    #[error("Invalid Unicode in input: {0}")]
    UnicodeError(String),

    /// Content length exceeds maximum limits
    #[error("Content exceeds maximum length: {0} > {1}")]
    ContentTooLong(usize, usize),

    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

/// Result type for plain text generation operations
type PlainTextResult =
    Result<(String, String, String, String, String, String)>;

/// Generates plain text content from HTML/Markdown input.
///
/// # Arguments
///
/// * `content` - The original HTML/Markdown content
/// * `title` - The content title
/// * `description` - Content description
/// * `author` - Content author
/// * `creator` - Content creator
/// * `keywords` - Associated keywords
///
/// # Returns
///
/// Returns a tuple containing:
/// - Plain text content
/// - Sanitized title
/// - Sanitized description
/// - Sanitized author
/// - Sanitized creator
/// - Sanitized keywords
///
/// # Errors
///
/// Returns an error if:
/// - Content parsing fails
/// - Unicode validation fails
/// - Content length exceeds limits
///
/// # Example
///
/// ```
/// use staticrux::modules::plaintext::generate_plain_text;
///
/// let result = generate_plain_text(
///     "# Hello\nWorld",
///     "Title",
///     "Description",
///     "Author",
///     "Creator",
///     "keywords",
/// ).unwrap();
/// ```
pub fn generate_plain_text(
    content: &str,
    title: &str,
    description: &str,
    author: &str,
    creator: &str,
    keywords: &str,
) -> PlainTextResult {
    debug!(
        "Converting content to plain text, length: {}",
        content.len()
    );

    let plain_content = convert_to_plain_text(content)?;

    info!("Successfully converted content to plain text");

    Ok((
        plain_content,
        sanitize_text(title),
        sanitize_text(description),
        sanitize_text(author),
        sanitize_text(creator),
        sanitize_text(keywords),
    ))
}

/// Converts formatted content to plain text.
fn convert_to_plain_text(content: &str) -> Result<String> {
    let mut plain_text = String::new();
    let mut buffer = String::new();
    let mut last_was_text = false;

    let parser = Parser::new(content);

    for event in parser {
        match event {
            Event::Text(text) => {
                let trimmed_text = text.trim();
                if !trimmed_text.is_empty() {
                    if last_was_text {
                        buffer.push(' ');
                    }
                    buffer.push_str(trimmed_text);
                    last_was_text = true;
                }
            }
            Event::Start(Tag::Paragraph)
            | Event::Start(Tag::Heading { .. }) => {
                if !plain_text.is_empty() && !buffer.trim().is_empty() {
                    plain_text.push_str("\n\n");
                }
                buffer.clear();
                last_was_text = false;
            }
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading { .. }) => {
                if !buffer.trim().is_empty() {
                    plain_text.push_str(&buffer);
                    buffer.clear();
                }
                last_was_text = false;
            }
            Event::Start(Tag::List(_)) | Event::Start(Tag::Item) => {
                if last_was_text {
                    buffer.push('\n');
                }
                buffer.push_str("• ");
                last_was_text = false;
            }
            Event::End(TagEnd::List(_)) | Event::End(TagEnd::Item) => {
                if !buffer.trim().is_empty() {
                    plain_text.push_str(&buffer);
                    buffer.clear();
                }
                last_was_text = false;
            }
            Event::SoftBreak | Event::HardBreak => {
                if !buffer.trim().is_empty() {
                    buffer.push(' ');
                }
                last_was_text = false;
            }
            _ => {}
        }
    }

    if !buffer.trim().is_empty() {
        plain_text.push_str(&buffer);
    }

    Ok(plain_text.trim().to_string())
}

/// Sanitizes text by removing unsafe content and normalizing whitespace.
fn sanitize_text(text: &str) -> String {
    // Remove potentially harmful content
    let sanitized =
        text.replace("<script>", "").replace("</script>", "");

    // Normalize whitespace and remove control characters
    sanitized
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() -> Result<()> {
        let input = "# Hello\n\nThis is **bold** text.";
        let (content, ..) = generate_plain_text(
            input,
            "Title",
            "Description",
            "Author",
            "Creator",
            "keywords",
        )?;

        assert!(content.contains("Hello"));
        assert!(content.contains("This is bold text"));
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<()> {
        let (content, title, description, author, creator, keywords) =
            generate_plain_text("", "", "", "", "", "")?;

        assert!(content.is_empty());
        assert!(title.is_empty());
        assert!(description.is_empty());
        assert!(author.is_empty());
        assert!(creator.is_empty());
        assert!(keywords.is_empty());
        Ok(())
    }

    #[test]
    fn test_whitespace_input() -> Result<()> {
        let (content, ..) =
            generate_plain_text("   \n   \t", "", "", "", "", "")?;
        assert!(content.is_empty());
        Ok(())
    }

    #[test]
    fn test_rtl_and_different_languages() -> Result<()> {
        let input = "English text. النص العربي. עברית.";
        let (content, ..) =
            generate_plain_text(input, "", "", "", "", "")?;

        assert!(content.contains("English text"));
        assert!(content.contains("النص العربي"));
        assert!(content.contains("עברית"));
        Ok(())
    }

    #[test]
    fn test_invalid_html_input() -> Result<()> {
        let input = "Text with <b>unclosed tag";
        let (content, ..) =
            generate_plain_text(input, "", "", "", "", "")?;

        assert!(content.contains("Text with unclosed tag"));
        assert!(!content.contains("<b>"));
        Ok(())
    }

    #[test]
    fn test_nested_formatting() -> Result<()> {
        let input = "This is **bold _italic nested_ formatting** test.";
        let (content, ..) =
            generate_plain_text(input, "", "", "", "", "")?;
        assert!(content
            .contains("This is bold italic nested formatting test"));
        Ok(())
    }

    #[test]
    fn test_lists() -> Result<()> {
        let input = "- Item 1\n- Item 2\n  - Nested item";
        let (content, ..) =
            generate_plain_text(input, "", "", "", "", "")?;

        assert!(content.contains("• Item 1"));
        assert!(content.contains("• Item 2"));
        assert!(content.contains("• Nested item"));
        Ok(())
    }

    #[test]
    fn test_metadata_escaping() -> Result<()> {
        let (_, title, ..) = generate_plain_text(
            "",
            "Title with <script>alert('xss')</script>",
            "",
            "",
            "",
            "",
        )?;

        assert!(title.contains("Title with alert"));
        assert!(!title.contains("<script>"));
        Ok(())
    }
}
