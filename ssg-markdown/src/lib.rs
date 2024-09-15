//! # SSG Markdown Processor
//!
//! The `ssg-markdown` crate provides utilities for parsing, converting, and rendering Markdown
//! into HTML. It integrates with the `comrak` library for Markdown processing and offers a
//! flexible interface for customizing the conversion process.
//!
//! ## Modules
//!
//! - [`converter`]: Handles the conversion of parsed Markdown into HTML.
//! - [`error`]: Defines errors that can occur during the Markdown processing workflow.
//! - [`parser`]: Responsible for parsing Markdown into an Abstract Syntax Tree (AST).
//! - [`renderer`]: Provides the logic for rendering HTML from the converted content.
//!
//! ## Example Usage
//!
//! ```rust
//! use comrak::ComrakOptions;
//! use ssg_markdown::process_markdown;
//!
//! let markdown = "# Hello, world!";
//! let options = ComrakOptions::default();
//! let result = process_markdown(markdown, &options);
//! assert!(result.is_ok());
//! ```
//!
//! This crate can be used to process Markdown files or content and convert them to HTML in a customizable way.

use crate::converter::convert_markdown_to_html;
use crate::error::MarkdownError;
use crate::parser::parse_markdown;
use crate::renderer::render_html;
use comrak::{Arena, ComrakOptions};

/// Handles conversion of parsed Markdown into HTML.
pub mod converter;

/// Defines errors that can occur during the Markdown processing workflow.
pub mod error;

/// Parses Markdown content into an Abstract Syntax Tree (AST).
pub mod parser;

/// Renders HTML content from converted Markdown.
pub mod renderer;

/// Processes Markdown content and returns HTML.
///
/// This function combines parsing, conversion, and rendering steps.
///
/// # Arguments
///
/// * `content` - A string slice containing the Markdown content.
/// * `options` - A reference to `ComrakOptions` for customizing the conversion process.
///
/// # Returns
///
/// A `Result` containing the processed HTML as a `String`, or a `MarkdownError` if processing fails.
///
/// # Examples
///
/// ```
/// use comrak::ComrakOptions;
/// use ssg_markdown::process_markdown;
///
/// let markdown = "# Hello, world!";
/// let options = ComrakOptions::default();
/// let result = process_markdown(markdown, &options);
/// assert!(result.is_ok());
/// ```
pub fn process_markdown(
    content: &str,
    options: &ComrakOptions,
) -> Result<String, MarkdownError> {
    let arena = Arena::new(); // Create the Arena instance
    let _ast = parse_markdown(&arena, content, options)?; // Keep the AST for any further processing if needed
    let html = convert_markdown_to_html(content, options)?; // Pass raw content instead of AST
    render_html(&html)
}
