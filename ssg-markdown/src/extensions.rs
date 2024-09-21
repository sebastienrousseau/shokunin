//! Extension functionality for the SSG Markdown library.
//!
//! This module provides utilities for enhancing Markdown processing,
//! including syntax highlighting, table formatting, and custom block handling.

use crate::error::MarkdownError;
use lazy_static::lazy_static;
use regex::Regex;
use syntect::{
    highlighting::ThemeSet, html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

lazy_static! {
    /// Cached `SyntaxSet` to avoid reloading on every function call.
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    /// Cached `ThemeSet` to avoid reloading on every function call.
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

/// Alignment options for table columns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnAlignment {
    /// Align the column to the left.
    Left,
    /// Align the column to the center.
    Center,
    /// Align the column to the right.
    Right,
}

/// Represents different types of custom blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CustomBlockType {
    /// A note block.
    Note,
    /// A warning block.
    Warning,
    /// A tip block.
    Tip,
    /// An info block.
    Info,
    /// An important block.
    Important,
    /// A caution block.
    Caution,
}

impl CustomBlockType {
    /// Returns the appropriate Bootstrap alert class for the custom block type.
    fn get_alert_class(&self) -> &'static str {
        match self {
            CustomBlockType::Note => "alert-info",
            CustomBlockType::Warning => "alert-warning",
            CustomBlockType::Tip => "alert-success",
            CustomBlockType::Info => "alert-primary",
            CustomBlockType::Important => "alert-danger",
            CustomBlockType::Caution => "alert-secondary",
        }
    }

    /// Returns the title for the custom block type.
    fn get_title(&self) -> &'static str {
        match self {
            CustomBlockType::Note => "Note",
            CustomBlockType::Warning => "Warning",
            CustomBlockType::Tip => "Tip",
            CustomBlockType::Info => "Info",
            CustomBlockType::Important => "Important",
            CustomBlockType::Caution => "Caution",
        }
    }
}

lazy_static! {
    static ref CUSTOM_BLOCK_REGEX: Regex = Regex::new(
        r#"(?i)<div\s+class=["']?(note|warning|tip|info|important|caution)["']?>(.*?)</div>"#
    ).unwrap();
}

/// Applies syntax highlighting to code blocks in the Markdown.
///
/// # Arguments
///
/// * `code` - The code block string to be highlighted.
/// * `lang` - The programming language of the code block.
///
/// # Returns
///
/// A `Result` containing the HTML for the highlighted code or a `MarkdownError`.
pub fn apply_syntax_highlighting(
    code: &str,
    lang: &str,
) -> Result<String, MarkdownError> {
    // Use cached SyntaxSet and ThemeSet
    let theme = &THEME_SET.themes["base16-ocean.dark"];

    let syntax = SYNTAX_SET
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme)
        .map_err(|e| MarkdownError::SyntaxHighlightError(e.to_string()))
}

/// Processes tables, enhancing them with responsive design and alignment classes.
///
/// # Arguments
///
/// * `table_html` - The HTML string representing the table.
///
/// # Returns
///
/// The enhanced HTML string.
pub fn process_tables(table_html: &str) -> String {
    let table_regex = Regex::new(r"<table>").unwrap();
    let table_html = table_regex.replace(
        table_html,
        r#"<div class="table-responsive"><table class="table">"#,
    );

    let table_end_regex = Regex::new(r"</table>").unwrap();
    let table_html =
        table_end_regex.replace(&table_html, "</table></div>");

    // Add alignment classes to table cells
    let cell_regex = Regex::new(r"<td([^>]*)>").unwrap();
    let table_html = cell_regex.replace_all(
        &table_html,
        |caps: &regex::Captures| {
            let attrs = &caps[1];
            if attrs.contains("align=\"center\"") {
                format!(r#"<td{} class="text-center">"#, attrs)
            } else if attrs.contains("align=\"right\"") {
                format!(r#"<td{} class="text-right">"#, attrs)
            } else {
                format!(r#"<td{} class="text-left">"#, attrs)
            }
        },
    );

    table_html.to_string()
}

/// Processes custom blocks in the Markdown content, such as note, warning, tip, info, important, and caution blocks.
/// These custom blocks are represented by div elements with specific class names.
/// The function replaces these div elements with corresponding Bootstrap alert elements.
///
/// # Arguments
///
/// * `content` - A string containing the Markdown content.
///
/// # Returns
///
/// A string containing the processed Markdown content with custom blocks replaced by Bootstrap alert elements.
pub fn process_custom_blocks(content: &str) -> String {
    CUSTOM_BLOCK_REGEX.replace_all(content, |caps: &regex::Captures| {
        let block_type = match caps.get(1).unwrap().as_str().to_lowercase().as_str() {
            "note" => CustomBlockType::Note,
            "warning" => CustomBlockType::Warning,
            "tip" => CustomBlockType::Tip,
            "info" => CustomBlockType::Info,
            "important" => CustomBlockType::Important,
            "caution" => CustomBlockType::Caution,
            _ => unreachable!(),
        };
        let block_content = &caps[2];
        format!(
            r#"<div class="alert {}" role="alert"><strong>{}:</strong> {}</div>"#,
            block_type.get_alert_class(),
            block_type.get_title(),
            block_content
        )
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_custom_blocks() {
        let input = r#"
            <div class="note">This is a note.</div>
            <div class="WARNING">This is a warning.</div>
            <div class="Tip">This is a tip.</div>
            <div class="INFO">This is an info block.</div>
            <div class="Important">This is important.</div>
            <div class="caution">This is a caution.</div>
        "#;

        let processed = process_custom_blocks(input);

        assert!(processed.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong> This is a note.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-warning" role="alert"><strong>Warning:</strong> This is a warning.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-success" role="alert"><strong>Tip:</strong> This is a tip.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-primary" role="alert"><strong>Info:</strong> This is an info block.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-danger" role="alert"><strong>Important:</strong> This is important.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-secondary" role="alert"><strong>Caution:</strong> This is a caution.</div>"#));
    }

    #[test]
    fn test_process_tables() {
        let input = r#"<table><tr><td align="center">Center</td><td align="right">Right</td><td>Left</td></tr></table>"#;

        let processed = process_tables(input);

        assert!(processed.contains(
            r#"<div class="table-responsive"><table class="table">"#
        ));
        assert!(processed.contains(
            r#"<td align="center" class="text-center">Center</td>"#
        ));
        assert!(processed.contains(
            r#"<td align="right" class="text-right">Right</td>"#
        ));
        assert!(
            processed.contains(r#"<td class="text-left">Left</td>"#)
        );
        assert!(processed.contains("</table></div>"));
    }
}
