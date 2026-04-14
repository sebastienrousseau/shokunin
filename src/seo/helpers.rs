// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Internal helper functions for SEO plugins.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Extract the page title from the `<title>` tag.
pub(super) fn extract_title(html: &str) -> String {
    if let Some(start) = html.find("<title>") {
        let after = &html[start + 7..];
        if let Some(end) = after.find("</title>") {
            let title = strip_tags(&after[..end]);
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

/// Extract plain text from the page content, strip tags, and truncate to
/// `max_len` characters.
///
/// Prefers `<main>` content if present. Falls back to `<body>` with nav,
/// header, footer, script, and style blocks removed.
pub(super) fn extract_description(html: &str, max_len: usize) -> String {
    let content = extract_main_content(html);

    let clean = strip_inline_tags(&content, &["script", "style"]);

    let text = strip_tags(&clean);
    let trimmed = text.trim();
    truncate_at_word_boundary(trimmed, max_len)
}

/// Extracts the inner content of `<main>`, or falls back to `<body>` with
/// non-content elements removed.
fn extract_main_content(html: &str) -> String {
    if let Some(inner) = extract_tag_inner(html, "main") {
        return inner;
    }

    let body =
        extract_tag_inner(html, "body").unwrap_or_else(|| html.to_string());
    strip_inline_tags(&body, &["script", "style", "nav", "header", "footer"])
}

/// Extracts the inner HTML of the first occurrence of `<tag_name>...</tag_name>`.
fn extract_tag_inner(html: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");
    let start = html.find(&open)?;
    let after = &html[start..];
    let gt = after.find('>')?;
    let inner = &after[gt + 1..];
    if let Some(end) = inner.find(&close) {
        Some(inner[..end].to_string())
    } else {
        Some(inner.to_string())
    }
}

/// Removes matched `<tag>...</tag>` blocks for each tag name in `tags`.
fn strip_inline_tags(html: &str, tags: &[&str]) -> String {
    let mut clean = html.to_string();
    for tag in tags {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(start) = clean.find(&open) {
            if let Some(end) = clean[start..].find(&close) {
                clean.replace_range(start..start + end + close.len(), " ");
            } else {
                break;
            }
        }
    }
    clean
}

/// Truncates text to `max_len` at a word boundary.
fn truncate_at_word_boundary(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }
    let mut end = max_len;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = &text[..end];
    if let Some(last_space) = truncated.rfind(' ') {
        truncated[..last_space].to_string()
    } else {
        truncated.to_string()
    }
}

/// Remove all HTML tags and collapse whitespace.
pub(super) fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_space = false;
        }
    }
    collapsed.trim().to_string()
}

/// Collect all `.html` files under `dir` (delegates to `crate::walk`).
pub(super) fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
}

/// Escape a string for safe inclusion in an HTML attribute value.
pub(super) fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Check for an actual `<meta` tag (not just an HTML comment marker).
///
/// Staticdatagen may emit empty comment blocks like:
/// ```html
/// <!-- # Start Open Graph / Facebook Meta Tags -->
/// <!-- # End Open Graph / Facebook Meta Tags -->
/// ```
/// These should NOT count as "tag present" — only real `<meta` tags do.
pub(super) fn has_meta_tag(html: &str, attr: &str) -> bool {
    html.contains(&format!("<meta property=\"{attr}\""))
        || html.contains(&format!("<meta property='{attr}'"))
        || html.contains(&format!("<meta name=\"{attr}\""))
        || html.contains(&format!("<meta name='{attr}'"))
}

/// Extract the canonical URL from a `<link rel="canonical">` tag.
pub(super) fn extract_canonical(html: &str) -> String {
    if let Some(pos) = html.find("rel=\"canonical\"") {
        let region_start = pos.saturating_sub(200);
        let region = &html[region_start..html.len().min(pos + 200)];
        if let Some(href_start) = region.find("href=\"") {
            let after = &region[href_start + 6..];
            if let Some(end) = after.find('"') {
                return after[..end].to_string();
            }
        }
    }
    String::new()
}

/// Extract the content of a specific meta tag by name or property.
pub(super) fn extract_existing_meta(html: &str, attr: &str) -> String {
    for prefix in &[
        format!("<meta name=\"{attr}\" content=\""),
        format!("<meta property=\"{attr}\" content=\""),
        format!("<meta name='{attr}' content='"),
        format!("<meta property='{attr}' content='"),
    ] {
        if let Some(pos) = html.find(prefix.as_str()) {
            let after = &html[pos + prefix.len()..];
            let delim = if prefix.ends_with('\'') { '\'' } else { '"' };
            if let Some(end) = after.find(delim) {
                let value = after[..end].trim();
                if !value.is_empty() {
                    return value.to_string();
                }
            }
        }
    }
    String::new()
}

/// Extract the `lang` attribute from the `<html>` tag.
pub(super) fn extract_html_lang(html: &str) -> String {
    if let Some(start) = html.find("<html") {
        let tag_end = html[start..].find('>').unwrap_or(200);
        let tag = &html[start..start + tag_end];
        if let Some(lang_pos) = tag.find("lang=\"") {
            let after = &tag[lang_pos + 6..];
            if let Some(end) = after.find('"') {
                return after[..end].to_string();
            }
        }
        if let Some(lang_pos) = tag.find("lang='") {
            let after = &tag[lang_pos + 6..];
            if let Some(end) = after.find('\'') {
                return after[..end].to_string();
            }
        }
    }
    String::new()
}

/// Extract the first image URL from `<main>` or `<article>` content.
pub(super) fn extract_first_content_image(html: &str) -> String {
    // Look in <main> or <article> first
    let search_region = if let Some(start) = html.find("<main") {
        &html[start..]
    } else if let Some(start) = html.find("<article") {
        &html[start..]
    } else {
        return String::new();
    };

    if let Some(img_pos) = search_region.find("<img") {
        let after_img = &search_region[img_pos..];
        let tag_end = after_img.find('>').unwrap_or(500).min(500);
        let img_tag = &after_img[..tag_end];
        if let Some(src_pos) = img_tag.find("src=\"") {
            let after_src = &img_tag[src_pos + 5..];
            if let Some(end) = after_src.find('"') {
                return after_src[..end].to_string();
            }
        }
    }
    String::new()
}

/// Extract the author name from `<meta name="author">` or byline markup.
pub(super) fn extract_meta_author(html: &str) -> String {
    // Try meta tag first
    let from_meta = extract_existing_meta(html, "author");
    if !from_meta.is_empty() {
        return from_meta;
    }
    // Try <span class="author"> or similar byline patterns
    for pattern in &["class=\"author\">", "class='author'>", "rel=\"author\">"]
    {
        if let Some(pos) = html.find(pattern) {
            let after = &html[pos + pattern.len()..];
            if let Some(end) = after.find('<') {
                let name = after[..end].trim();
                // Strip "by " prefix
                let name = name.strip_prefix("by ").unwrap_or(name).trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    String::new()
}

/// Extract a date from an existing JSON-LD block in the HTML.
pub(super) fn extract_date_from_html(
    html: &str,
    field: &str,
) -> Option<String> {
    let pattern = format!("\"{field}\":\"");
    if let Some(pos) = html.find(&pattern) {
        let after = &html[pos + pattern.len()..];
        if let Some(end) = after.find('"') {
            let date = &after[..end];
            if !date.is_empty() {
                return Some(date.to_string());
            }
        }
    }
    None
}

/// Extract a date from `<time datetime="...">` or `<meta property="article:published_time">`.
pub(super) fn extract_meta_date(html: &str) -> Option<String> {
    // Try article:published_time meta
    let meta = extract_existing_meta(html, "article:published_time");
    if !meta.is_empty() {
        return Some(meta);
    }
    // Try first <time datetime="..."> in the page
    if let Some(pos) = html.find("datetime=\"") {
        let after = &html[pos + 10..];
        if let Some(end) = after.find('"') {
            let date = &after[..end];
            if !date.is_empty() {
                return Some(date.to_string());
            }
        }
    }
    None
}

/// Recursively collects HTML files (delegates to `crate::walk`).
pub(super) fn collect_html_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
}
