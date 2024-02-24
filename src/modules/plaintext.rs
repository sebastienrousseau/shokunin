// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::utilities::directory::extract_front_matter;
use crate::modules::preprocessor::preprocess_content;
use pulldown_cmark::{Parser, Event, Tag};
use pulldown_cmark::TagEnd;
use regex::Regex;
use std::error::Error;

/// Generate a plain text representation of the Markdown content.
///
/// This function takes Markdown content as input and produces a plain text representation
/// by processing the Markdown syntax and removing any formatting.
///
/// # Arguments
///
/// * `content` - A string slice containing the Markdown content.
///
/// # Returns
///
/// A `Result` containing a `String` representing the generated plain text if successful,
/// or a `Box<dyn Error>` if an error occurs during processing.
///
/// # Errors
///
/// This function may return an error if there is an issue with parsing or processing the Markdown content.
///
/// # Examples
///
/// ```rust
/// use ssg::modules::plaintext::generate_plain_text;
///
/// let content = "## Hello, *world*!";
/// let plain_text = generate_plain_text(content).unwrap();
///
/// assert_eq!(plain_text, "Hello, world !");
/// ```
pub fn generate_plain_text(content: &str) -> Result<String, Box<dyn Error>> {
    // Regex patterns for class, and image tags
    let class_regex = Regex::new(r#"\.class\s*=\s*"\s*[^"]*"\s*"#)?;
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)")?;
    let link_ref_regex = Regex::new(r"\[([^\]]+)\]\[\d+\]")?;

    // Extract front matter from content
    let markdown_content = extract_front_matter(content);
    // Preprocess content to update class attributes and image tags
    let processed_content = preprocess_content(markdown_content, &class_regex, &img_regex)?;

    // Further preprocess to remove Markdown link references.
    let no_markdown_links = link_ref_regex.replace_all(&processed_content, "$1");

    let mut plain_text = String::new();
    let parser = Parser::new(&no_markdown_links);

    let mut last_was_text = false;
    let mut need_extra_line_break = false;

    for event in parser {
        match event {
            Event::Text(text) => {
                if need_extra_line_break && !text.trim().is_empty() {
                    plain_text.push('\n');
                    need_extra_line_break = false;
                }
                if last_was_text && !text.trim().is_empty() {
                    plain_text.push('\n');
                }
                plain_text.push_str(text.trim_end());
                last_was_text = true;
            }
            Event::Start(tag) => {
                if tag == Tag::Paragraph && last_was_text {
                    need_extra_line_break = true;
                    plain_text.push('\n');
                }
                if tag == Tag::Emphasis {
                    plain_text.push(' ');
                }
                if tag == Tag::Strong {
                    plain_text.push_str("");
                }
                match tag {
                    Tag::Heading { .. } => {
                        plain_text.push(' ');
                    }
                    Tag::Link { .. } => {
                        plain_text.push(' ');
                    }
                    _ => {}
                }
                last_was_text = false;
            }
            Event::End(tag) => {
                if tag == Tag::Paragraph.into() {
                    plain_text.push('\n');
                }
                if tag == Tag::Emphasis.into() {
                    plain_text.push(' ');
                }
                if tag == Tag::Strong.into() {
                    plain_text.push_str("");
                }
                match tag {
                    TagEnd::Heading { .. } => {
                        plain_text.push('\n');
                    }
                    TagEnd::Link { .. } => {
                        plain_text.push(' ');
                    }
                    _ => {}
                }
                last_was_text = false;
            }
            _ => {}
        }
    }
    let plain_text = plain_text.trim();
    Ok(plain_text.to_string())
}
