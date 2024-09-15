//! Utility functions for HTML and Markdown processing.
//!
//! This module provides various utility functions for tasks such as
//! extracting front matter from Markdown content and formatting HTML headers.

use crate::error::{HtmlError, Result};
use once_cell::sync::Lazy;
use regex::Regex;

static FRONT_MATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ms)^---\s*\n(.*?)\n---\s*\n")
        .expect("Failed to compile FRONT_MATTER_REGEX")
});

static HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<(h[1-6])>(.+?)</h[1-6]>")
        .expect("Failed to compile HEADER_REGEX")
});

static CONSECUTIVE_HYPHENS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-{2,}")
        .expect("Failed to compile CONSECUTIVE_HYPHENS_REGEX")
});

/// Extracts front matter from Markdown content.
///
/// This function removes the front matter (if present) from the given content
/// and returns the rest of the content.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to process.
///
/// # Returns
///
/// * `Result<String>` - The content with front matter removed, or an error.
///
/// # Errors
///
/// This function will return an error if the front matter is invalidly formatted.
///
/// # Examples
///
/// ```
/// use ssg_html::utils::extract_front_matter;
///
/// let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
/// let result = extract_front_matter(content).unwrap();
/// assert_eq!(result, "# Hello, world!\n\nThis is a test.");
/// ```
pub fn extract_front_matter(content: &str) -> Result<String> {
    // Check if content starts with front matter indicators
    if content.starts_with("---") {
        // Capture the front matter block using the regex
        if let Some(captures) = FRONT_MATTER_REGEX.captures(content) {
            // Remove the front matter and return the rest of the content
            let remaining_content =
                &content[captures.get(0).unwrap().end()..];
            Ok(remaining_content.trim().to_string())
        } else {
            // Invalid front matter format
            Err(HtmlError::InvalidFrontMatterFormat(
                "Invalid front matter format".to_string(),
            ))
        }
    } else {
        // If there's no front matter at all, return an error
        Err(HtmlError::InvalidFrontMatterFormat(
            "No front matter found".to_string(),
        ))
    }
}

/// Formats a header with an ID and class.
///
/// This function takes an HTML header and adds an id and class attribute
/// based on the header's content.
///
/// # Arguments
///
/// * `header` - A string slice that holds the HTML header to process.
///
/// # Returns
///
/// * `Result<String>` - The formatted HTML header, or an error.
///
/// # Errors
///
/// This function will return an error if the header is invalidly formatted.
///
/// # Examples
///
/// ```
/// use ssg_html::utils::format_header_with_id_class;
///
/// let header = "<h2>Hello, World!</h2>";
/// let result = format_header_with_id_class(header).unwrap();
/// assert_eq!(result, "<h2 id=\"hello-world\" class=\"hello-world\">Hello, World!</h2>\n");
/// ```
pub fn format_header_with_id_class(header: &str) -> Result<String> {
    let captures = HEADER_REGEX.captures(header).ok_or_else(|| {
        HtmlError::InvalidHeaderFormat(
            "Invalid header format".to_string(),
        )
    })?;

    let tag = &captures[1];
    let content = &captures[2];
    let id = CONSECUTIVE_HYPHENS_REGEX
        .replace_all(
            &content
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "-"),
            "-",
        )
        .trim_matches('-')
        .to_string();

    Ok(format!(
        r#"<{} id="{}" class="{}">{}</{}>
"#,
        tag, id, id, content, tag
    ))
}

/// Generates a table of contents from HTML content.
///
/// This function extracts all headers (h1-h6) from the provided HTML content
/// and generates a table of contents as an HTML unordered list.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - The generated table of contents as an HTML string, or an error.
///
/// # Examples
///
/// ```
/// use ssg_html::utils::generate_table_of_contents;
///
/// let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p>";
/// let toc = generate_table_of_contents(html).unwrap();
/// assert_eq!(toc, "<ul><li class=\"toc-h1\"><a href=\"#title\">Title</a></li><li class=\"toc-h2\"><a href=\"#subtitle\">Subtitle</a></li></ul>");
/// ```
pub fn generate_table_of_contents(html: &str) -> Result<String> {
    let mut toc = String::from("<ul>");
    let headers = HEADER_REGEX.captures_iter(html);

    for captures in headers {
        let tag = &captures[1];
        let content = &captures[2];
        let id = CONSECUTIVE_HYPHENS_REGEX
            .replace_all(
                &content
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric(), "-"),
                "-",
            )
            .trim_matches('-')
            .to_string();

        toc.push_str(&format!(
            "<li class=\"toc-{}\"><a href=\"#{}\">{}</a></li>",
            tag, id, content
        ));
    }

    toc.push_str("</ul>");
    Ok(toc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter() {
        let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
        let result = extract_front_matter(content).unwrap();
        assert_eq!(result, "# Hello, world!\n\nThis is a test.");
    }

    #[test]
    fn test_extract_front_matter_no_front_matter() {
        let content =
            "# Hello, world!\n\nThis is a test without front matter.";
        let result = extract_front_matter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::InvalidFrontMatterFormat(_)
        ));
    }

    #[test]
    fn test_format_header_with_id_class() {
        let header = "<h2>Hello, World!</h2>";
        let result = format_header_with_id_class(header).unwrap();
        assert_eq!(result, "<h2 id=\"hello-world\" class=\"hello-world\">Hello, World!</h2>\n");
    }

    #[test]
    fn test_format_header_with_special_characters() {
        let header = "<h3>Test: Special & Characters</h3>";
        let result = format_header_with_id_class(header).unwrap();
        assert_eq!(result, "<h3 id=\"test-special-characters\" class=\"test-special-characters\">Test: Special & Characters</h3>\n");
    }

    #[test]
    fn test_format_header_with_consecutive_hyphens() {
        let header = "<h4>Multiple---Hyphens</h4>";
        let result = format_header_with_id_class(header).unwrap();
        assert_eq!(result, "<h4 id=\"multiple-hyphens\" class=\"multiple-hyphens\">Multiple---Hyphens</h4>\n");
    }

    #[test]
    fn test_format_header_with_invalid_format() {
        let header = "<p>Not a header</p>";
        let result = format_header_with_id_class(header);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::InvalidHeaderFormat(_)
        ));
    }

    #[test]
    fn test_generate_table_of_contents() {
        let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p><h3>Sub-subtitle</h3>";
        let result = generate_table_of_contents(html).unwrap();
        assert_eq!(result, "<ul><li class=\"toc-h1\"><a href=\"#title\">Title</a></li><li class=\"toc-h2\"><a href=\"#subtitle\">Subtitle</a></li><li class=\"toc-h3\"><a href=\"#sub-subtitle\">Sub-subtitle</a></li></ul>");
    }
}
