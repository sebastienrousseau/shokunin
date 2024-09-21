//! Error handling for the SSG Markdown library.

use anyhow::{Context, Result};
use thiserror::Error;

/// Errors that can occur during Markdown processing.
#[derive(Error, Debug)]
pub enum MarkdownError {
    /// An error occurred while parsing the Markdown content.
    #[error("Failed to parse Markdown: {0}")]
    ParseError(String),

    /// An error occurred while converting Markdown to HTML.
    #[error("Failed to convert Markdown to HTML: {0}")]
    ConversionError(String),

    /// An error occurred while processing a custom block.
    #[error("Failed to process custom block: {0}")]
    CustomBlockError(String),

    /// An error occurred while applying syntax highlighting.
    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlightError(String),

    /// An error occurred due to invalid options.
    #[error("Invalid Markdown options: {0}")]
    InvalidOptionsError(String),

    /// An error occurred while loading a syntax set.
    #[error("Failed to load syntax set: {0}")]
    SyntaxHighlightingError(String),
}

/// A helper function that adds context to errors occurring during Markdown processing.
pub fn parse_markdown_with_context(input: &str) -> Result<String> {
    // Example of adding context using anyhow
    let parsed_content = some_markdown_parsing_function(input)
        .context("Failed while parsing markdown content")?;

    Ok(parsed_content)
}

// Placeholder for the actual markdown parsing function
fn some_markdown_parsing_function(input: &str) -> Result<String> {
    // Simulate success or failure
    if input.is_empty() {
        Err(MarkdownError::ParseError("Input is empty".to_string()))?;
    }
    Ok("Parsed markdown content".to_string())
}
