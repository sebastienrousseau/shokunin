#![forbid(unsafe_code)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ssg-core — Platform-independent SSG compilation pipeline
//!
//! This crate contains the pure-logic core of SSG, with no system
//! dependencies (`openssl`, `rayon`, `http-handle`). It compiles to
//! `wasm32-wasi` and `wasm32-unknown-unknown` (via `wasm-bindgen`).
//!
//! ## Features
//!
//! - Markdown → HTML compilation (pulldown-cmark with GFM extensions)
//! - Frontmatter parsing (TOML/JSON/YAML)
//! - Template rendering (when `minijinja` is enabled)
//! - Shortcode expansion
//! - SEO metadata generation
//! - Search index generation

use anyhow::Result;
use std::collections::HashMap;

/// Compile a Markdown string to HTML.
///
/// Supports GitHub Flavored Markdown: tables, strikethrough, task lists.
///
/// # Example
///
/// ```
/// let html = ssg_core::compile_markdown("# Hello\n\nWorld");
/// assert!(html.contains("<h1>Hello</h1>"));
/// assert!(html.contains("<p>World</p>"));
/// ```
#[must_use]
pub fn compile_markdown(input: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(input, options);
    let mut html_output = String::with_capacity(input.len() * 2);
    html::push_html(&mut html_output, parser);
    html_output
}

/// Parse frontmatter from a Markdown file.
///
/// Supports TOML (`+++`), YAML (`---`), and JSON (`{`) delimiters.
/// Returns `(frontmatter_map, body_without_frontmatter)`.
///
/// # Example
///
/// ```
/// let input = "---\ntitle: Hello\n---\n# Body";
/// let (fm, body) = ssg_core::parse_frontmatter(input);
/// assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Hello"));
/// assert!(body.contains("# Body"));
/// ```
pub fn parse_frontmatter(
    input: &str,
) -> (HashMap<String, serde_json::Value>, String) {
    let trimmed = input.trim_start();

    // TOML frontmatter: +++...+++
    if trimmed.starts_with("+++") {
        if let Some(end) = trimmed[3..].find("+++") {
            let fm_str = &trimmed[3..3 + end];
            let body = &trimmed[3 + end + 3..];
            if let Ok(value) = toml::from_str::<serde_json::Value>(fm_str) {
                if let Some(map) = value.as_object() {
                    return (
                        map.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                        body.to_string(),
                    );
                }
            }
            return (HashMap::new(), body.to_string());
        }
    }

    // YAML frontmatter: ---...---
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("---") {
            let fm_str = &trimmed[3..3 + end].trim();
            let body = &trimmed[3 + end + 3..];
            // Simple key: value parser for common YAML frontmatter
            let mut map = HashMap::new();
            for line in fm_str.lines() {
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let val = val.trim().to_string();
                    let _ = map.insert(key, serde_json::Value::String(val));
                }
            }
            return (map, body.to_string());
        }
    }

    // JSON frontmatter: {...}
    if trimmed.starts_with('{') {
        // Find matching closing brace
        let mut depth = 0;
        let mut end = None;
        for (i, c) in trimmed.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end_pos) = end {
            let fm_str = &trimmed[..end_pos];
            let body = &trimmed[end_pos..];
            if let Ok(map) = serde_json::from_str::<
                HashMap<String, serde_json::Value>,
            >(fm_str)
            {
                return (map, body.to_string());
            }
        }
    }

    (HashMap::new(), input.to_string())
}

/// Compile a complete page: parse frontmatter, render Markdown to HTML.
///
/// Returns `(frontmatter, html_body)`.
pub fn compile_page(
    input: &str,
) -> Result<(HashMap<String, serde_json::Value>, String)> {
    let (frontmatter, body) = parse_frontmatter(input);
    let html = compile_markdown(&body);
    Ok((frontmatter, html))
}

/// Generate a search index entry from HTML content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchEntry {
    /// Page title.
    pub title: String,
    /// Page URL.
    pub url: String,
    /// Plain text content for search matching.
    pub content: String,
}

/// Strip HTML tags from a string (simple implementation).
#[must_use]
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

/// Build a search index entry from HTML content and metadata.
#[must_use]
pub fn build_search_entry(title: &str, url: &str, html: &str) -> SearchEntry {
    let content = strip_html_tags(html);
    // Collapse whitespace for compact index
    let content: String =
        content.split_whitespace().collect::<Vec<_>>().join(" ");
    SearchEntry {
        title: title.to_string(),
        url: url.to_string(),
        content,
    }
}

/// Estimates reading time in minutes from text content.
///
/// Uses 200 words-per-minute average, with a minimum of 1 minute.
#[must_use]
pub fn reading_time(text: &str) -> usize {
    (text.split_whitespace().count() / 200).max(1)
}

/// Converts a string to a URL-safe slug.
#[must_use]
pub fn slugify(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_markdown_basic() {
        let html = compile_markdown("# Hello\n\nParagraph.");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>Paragraph.</p>"));
    }

    #[test]
    fn compile_markdown_gfm_tables() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = compile_markdown(input);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn compile_markdown_strikethrough() {
        let html = compile_markdown("~~deleted~~");
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn parse_frontmatter_yaml() {
        let (fm, body) = parse_frontmatter(
            "---\ntitle: Hello\ndate: 2026-01-01\n---\n# Body",
        );
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Hello"));
        assert!(body.contains("# Body"));
    }

    #[test]
    fn parse_frontmatter_toml() {
        let (fm, body) =
            parse_frontmatter("+++\ntitle = \"Hello\"\n+++\n# Body");
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Hello"));
        assert!(body.contains("# Body"));
    }

    #[test]
    fn parse_frontmatter_json() {
        let (fm, body) = parse_frontmatter("{\"title\": \"Hello\"}\n# Body");
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Hello"));
        assert!(body.contains("# Body"));
    }

    #[test]
    fn parse_frontmatter_none() {
        let (fm, body) = parse_frontmatter("Just content");
        assert!(fm.is_empty());
        assert_eq!(body, "Just content");
    }

    #[test]
    fn compile_page_full() {
        let input = "---\ntitle: Test\n---\n# Hello\n\nWorld";
        let (fm, html) = compile_page(input).unwrap();
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("Test"));
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn strip_html_tags_empty() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn build_search_entry_strips_tags() {
        let entry =
            build_search_entry("Title", "/page", "<p>Hello <b>world</b></p>");
        assert_eq!(entry.title, "Title");
        assert_eq!(entry.content, "Hello world");
    }

    #[test]
    fn reading_time_short() {
        assert_eq!(reading_time("one two three"), 1);
    }

    #[test]
    fn reading_time_long() {
        let text = "word ".repeat(600);
        assert_eq!(reading_time(&text), 3);
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Rust & Web"), "rust-web");
    }
}
