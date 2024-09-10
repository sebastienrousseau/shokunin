// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::modules::preprocessor::preprocess_content;
use crate::utilities::directory::extract_front_matter;
use anyhow::{Error, Result}; // Ensure anyhow::Error and Result are imported
use pulldown_cmark::TagEnd;
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

/// Type alias for the result of the `generate_plain_text` function
type PlainTextResult =
    Result<(String, String, String, String, String, String), Error>;

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
