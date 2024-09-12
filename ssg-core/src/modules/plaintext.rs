// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::modules::preprocessor::preprocess_content;
use crate::utilities::directory::extract_front_matter;
use anyhow::{Error, Result};
use pulldown_cmark::TagEnd;
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

/// Type alias for the result of the `generate_plain_text` function
type PlainTextResult =
    Result<(String, String, String, String, String, String), Error>;

/// Generates plain text content from Markdown input and associated metadata.
///
/// This function takes Markdown content and associated metadata (title, description, etc.),
/// processes the Markdown to remove formatting, and returns plain text versions of the
/// content and metadata.
///
/// # Arguments
///
/// * `content` - A string slice containing the Markdown content to be processed.
/// * `title` - A string slice containing the title of the content.
/// * `description` - A string slice containing a description of the content.
/// * `author` - A string slice containing the author's name.
/// * `creator` - A string slice containing the creator's name (if different from author).
/// * `keywords` - A string slice containing keywords associated with the content.
///
/// # Returns
///
/// Returns a `Result` containing a tuple of six `String`s:
/// 1. The plain text content derived from the Markdown input.
/// 2. The plain text title.
/// 3. The plain text description.
/// 4. The plain text author name.
/// 5. The plain text creator name.
/// 6. The plain text keywords.
///
/// If an error occurs during processing, an `Error` is returned.
///
/// # Errors
///
/// This function will return an error if:
/// - Regular expression compilation fails.
/// - Content preprocessing fails.
///
/// # Example
///
/// ```
/// use ssg_core::modules::plaintext::generate_plain_text;
///
/// let markdown = "# Hello, world!\n\nThis is **bold** text.";
/// let result = generate_plain_text(
///     markdown,
///     "My Page",
///     "A simple page",
///     "John Doe",
///     "Jane Doe",
///     "example, markdown"
/// );
///
/// assert!(result.is_ok());
/// if let Ok((content, title, description, author, creator, keywords)) = result {
///     assert_eq!(content, "Hello, world! This is bold text.");
///     assert_eq!(title, "My Page");
///     // ... other assertions ...
/// }
/// ```
pub fn generate_plain_text(
    content: &str,
    title: &str,
    description: &str,
    author: &str,
    creator: &str,
    keywords: &str,
) -> PlainTextResult {
    // Regex patterns for class, and image tags
    let class_regex = Regex::new(r#"\.class\s*=\s*"\s*[^"]*"\s*"#)?;
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)")?;
    let link_ref_regex = Regex::new(r"\[([^\]]+)\]\[\d+\]")?;

    // Extract front matter from content
    let markdown_content = extract_front_matter(content);

    // Preprocess content to update class attributes and image tags
    let processed_content =
        preprocess_content(markdown_content, &class_regex, &img_regex)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?; // Convert error to a string for compatibility

    // Further preprocess to remove Markdown link references.
    let no_markdown_links =
        link_ref_regex.replace_all(&processed_content, "$1");

    let mut plain_text = String::new();
    let parser = Parser::new(&no_markdown_links);

    let mut last_was_text = false;
    let mut need_extra_line_break = false;

    for event in parser {
        match event {
            Event::Text(text) => {
                if need_extra_line_break && !text.trim().is_empty() {
                    plain_text.push(' ');
                    need_extra_line_break = false;
                }
                if last_was_text && !text.trim().is_empty() {
                    plain_text.push(' ');
                }
                plain_text.push_str(text.trim());
                last_was_text = true;
            }
            Event::Start(tag) => {
                if tag == Tag::Paragraph && last_was_text {
                    need_extra_line_break = true;
                }
                match tag {
                    Tag::Heading { .. } | Tag::Paragraph => {
                        if !plain_text.is_empty()
                            && !plain_text.ends_with(' ')
                        {
                            plain_text.push(' ');
                        }
                    }
                    Tag::Emphasis | Tag::Strong | Tag::Link { .. } => {
                        if !plain_text.ends_with(' ') {
                            plain_text.push(' ');
                        }
                    }
                    _ => {}
                }
                last_was_text = false;
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading { .. } | TagEnd::Paragraph => {
                        if !plain_text.ends_with(' ') {
                            plain_text.push(' ');
                        }
                    }
                    TagEnd::Emphasis
                    | TagEnd::Strong
                    | TagEnd::Link => {
                        if !plain_text.ends_with(' ') {
                            plain_text.push(' ');
                        }
                    }
                    _ => {}
                }
                last_was_text = false;
            }
            _ => {}
        }
    }
    let plain_text = plain_text.trim();
    let plain_title = title.trim();
    let plain_description = description.trim();
    let plain_author = author.trim();
    let plain_creator = creator.trim();
    let plain_keywords = keywords.trim();
    Ok((
        plain_text.to_string(),
        plain_title.to_string(),
        plain_description.to_string(),
        plain_author.to_string(),
        plain_creator.to_string(),
        plain_keywords.to_string(),
    ))
}
