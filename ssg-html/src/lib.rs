//! # ssg-html
//!
//! `ssg-html` is a specialized HTML generation library designed for static site generators.
//! It provides optimized Markdown to HTML conversion, SEO tools, accessibility enhancements,
//! and performance optimizations.
//!
//! ## Features
//!
//! - Markdown to HTML conversion with custom extensions
//! - Advanced header processing with automatic ID and class generation
//! - SEO optimization including meta tag and structured data generation
//! - Accessibility enhancements with ARIA attribute injection and WCAG validation
//! - Performance optimizations including HTML minification and async generation
//!
//! ## Usage
//!
//! ```rust
//! use ssg_html::{generate_html, HtmlConfig};
//!
//! let markdown = "# Hello, world!\n\nThis is a test.";
//! let config = HtmlConfig::default();
//!
//! match generate_html(markdown, &config) {
//!     Ok(html) => println!("{}", html),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

//! HTML generation functionality optimized for static site generators

mod accessibility;
mod generator;
mod performance;
mod seo;
mod utils;

pub use accessibility::{add_aria_attributes, validate_wcag};
pub use generator::generate_html;
pub use performance::{async_generate_html, minify_html};
pub use seo::{generate_meta_tags, generate_structured_data};
pub use utils::{extract_front_matter, format_header_with_id_class};

use thiserror::Error;

/// Configuration options for HTML generation
#[derive(Debug, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,
    /// Minify the generated HTML output
    pub minify_output: bool,
    /// Automatically add ARIA attributes for accessibility
    pub add_aria_attributes: bool,
    /// Generate structured data (JSON-LD) based on content
    pub generate_structured_data: bool,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        HtmlConfig {
            enable_syntax_highlighting: true,
            minify_output: false,
            add_aria_attributes: true,
            generate_structured_data: false,
        }
    }
}

/// Error type for HTML generation
#[derive(Debug, Error)]
pub enum HtmlError {
    /// Error occurred during Markdown conversion
    #[error("Markdown conversion error: {0}")]
    MarkdownConversionError(String),
    /// Error occurred during template rendering
    #[error("Template rendering error: {0}")]
    TemplateRenderingError(String),
    /// Error occurred during HTML minification
    #[error("Minification error: {0}")]
    MinificationError(String),
    /// Error occurred during SEO optimization
    #[error("SEO optimization error: {0}")]
    SeoError(String),
    /// Error occurred during accessibility enhancements
    #[error("Accessibility error: {0}")]
    AccessibilityError(String),
}

/// Result type for HTML generation
pub type Result<T> = std::result::Result<T, HtmlError>;
