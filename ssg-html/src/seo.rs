//! SEO-related functionality for HTML processing.
//!
//! This module provides functions for generating meta tags and structured data
//! to improve the Search Engine Optimization (SEO) of web pages.

use crate::error::{HtmlError, Result};
use html_escape::encode_text;
use scraper::{Html, Selector};

/// Generates meta tags for SEO purposes.
///
/// This function parses the provided HTML, extracts relevant information,
/// and generates meta tags for title and description.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - A string containing the generated meta tags, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The HTML selectors fail to parse.
/// * Required HTML elements (title, description) are missing.
///
/// # Examples
///
/// ```
/// use ssg_html::seo::generate_meta_tags;
///
/// let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
/// let meta_tags = generate_meta_tags(html).unwrap();
/// assert!(meta_tags.contains(r#"<meta name="title" content="Test Page">"#));
/// assert!(meta_tags.contains(r#"<meta name="description" content="This is a test page.">"#));
/// ```
pub fn generate_meta_tags(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let mut meta_tags = String::new();

    let title_selector = Selector::parse("title").map_err(|e| {
        HtmlError::SelectorParseError(
            "title".to_string(),
            e.to_string(),
        )
    })?;
    let p_selector = Selector::parse("p").map_err(|e| {
        HtmlError::SelectorParseError("p".to_string(), e.to_string())
    })?;

    if let Some(title) = document.select(&title_selector).next() {
        let title_html = title.inner_html();
        let escaped_title = encode_text(&title_html);
        meta_tags.push_str(&format!(
            r#"<meta name="title" content="{}">"#,
            escaped_title
        ));
    } else {
        return Err(HtmlError::MissingHtmlElement("title".to_string()));
    }

    if let Some(description) = document.select(&p_selector).next() {
        let description_html = description.inner_html();
        let escaped_description = encode_text(&description_html);
        meta_tags.push_str(&format!(
            r#"<meta name="description" content="{}">"#,
            escaped_description
        ));
    } else {
        return Err(HtmlError::MissingHtmlElement(
            "description".to_string(),
        ));
    }

    meta_tags
        .push_str(r#"<meta property="og:type" content="website">"#);
    Ok(meta_tags)
}

/// Generates structured data (JSON-LD) for SEO purposes.
///
/// This function creates a JSON-LD script tag with basic webpage information
/// extracted from the provided HTML content.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - A string containing the generated JSON-LD script, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The HTML selectors fail to parse.
/// * Required HTML elements (title, description) are missing.
///
/// # Examples
///
/// ```
/// use ssg_html::seo::generate_structured_data;
///
/// let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
/// let structured_data = generate_structured_data(html).unwrap();
/// assert!(structured_data.contains(r#""@type": "WebPage""#));
/// assert!(structured_data.contains(r#""name": "Test Page""#));
/// assert!(structured_data.contains(r#""description": "This is a test page.""#));
/// ```
pub fn generate_structured_data(html: &str) -> Result<String> {
    let document = Html::parse_document(html);

    let title_selector = Selector::parse("title").map_err(|e| {
        HtmlError::SelectorParseError(
            "title".to_string(),
            e.to_string(),
        )
    })?;
    let p_selector = Selector::parse("p").map_err(|e| {
        HtmlError::SelectorParseError("p".to_string(), e.to_string())
    })?;

    let title = document
        .select(&title_selector)
        .next()
        .map(|t| encode_text(&t.inner_html()).into_owned())
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("title".to_string())
        })?;

    let description = document
        .select(&p_selector)
        .next()
        .map(|p| encode_text(&p.inner_html()).into_owned())
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("description".to_string())
        })?;

    let structured_data = format!(
        r#"<script type="application/ld+json">
        {{
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "{}",
            "description": "{}"
        }}
        </script>"#,
        title, description
    );

    Ok(structured_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_meta_tags() {
        let html = "<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>";
        let result = generate_meta_tags(html);
        assert!(result.is_ok());
        let meta_tags = result.unwrap();
        assert!(meta_tags
            .contains(r#"<meta name="title" content="Test Page">"#));
        assert!(meta_tags.contains(r#"<meta name="description" content="This is a test page.">"#));
    }

    #[test]
    fn test_generate_structured_data() {
        let html = "<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>";
        let result = generate_structured_data(html);
        assert!(result.is_ok());
        let structured_data = result.unwrap();
        assert!(structured_data.contains(r#""@type": "WebPage""#));
        assert!(structured_data.contains(r#""name": "Test Page""#));
        assert!(structured_data
            .contains(r#""description": "This is a test page.""#));
    }

    #[test]
    fn test_generate_meta_tags_missing_title() {
        let html =
            "<html><body><p>This is a test page.</p></body></html>";
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_structured_data_missing_description() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        let result = generate_structured_data(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }
}
