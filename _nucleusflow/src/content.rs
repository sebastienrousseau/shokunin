//! # Content Processing Module
//!
//! This module provides functionality for processing content in various formats.

use crate::{ContentProcessor, Result};
use pulldown_cmark::{html, Options, Parser};

/// Markdown content processor
#[derive(Debug, Copy, Clone)]
pub struct MarkdownProcessor;

impl MarkdownProcessor {
    /// Create a new MarkdownProcessor instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentProcessor for MarkdownProcessor {
    /// Process Markdown content and convert it to HTML
    ///
    /// This function takes Markdown content as input and returns the corresponding HTML.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the Markdown content to be processed
    ///
    /// # Returns
    ///
    /// * `Result<String>` - A Result containing the processed HTML string if successful,
    ///   or an error if the processing fails
    fn process(&self, content: &str) -> Result<String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);

        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(html_output)
    }
}
