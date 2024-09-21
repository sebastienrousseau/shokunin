//! # SSG Markdown Processor
//!
//! The `ssg-markdown` crate provides utilities for parsing, converting, and rendering Markdown
//! into HTML. It integrates with the `comrak` library for Markdown processing and offers a
//! flexible interface for customizing the conversion process.
//!
//! ## Features
//!
//! - Markdown to HTML conversion
//! - Custom block extensions (e.g., notes, warnings, tips)
//! - Syntax highlighting for code blocks
//! - Enhanced table processing
//! - Error handling

/// The `error` module contains error types for Markdown processing.
pub mod error;

/// The `extensions` module contains custom block extensions for Markdown processing.
pub mod extensions;

/// The `markdown` module contains functions for parsing, converting, and rendering Markdown.
pub mod markdown;

pub use error::MarkdownError;
pub use extensions::{
    apply_syntax_highlighting, ColumnAlignment, CustomBlockType,
};
pub use markdown::{process_markdown, MarkdownOptions};

/// Re-export comrak options for convenience
pub use comrak::ComrakOptions;
