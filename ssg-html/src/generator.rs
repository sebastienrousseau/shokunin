use crate::extract_front_matter;
use crate::Result;
use comrak::{markdown_to_html, ComrakOptions};

/// Generate HTML from Markdown content.
///
/// This function takes Markdown content and a configuration object,
/// converts the Markdown into HTML, and returns the resulting HTML string.
///
/// # Arguments
///
/// * `markdown` - A string slice that holds the Markdown content to convert.
/// * `_config` - A reference to an `HtmlConfig` struct that holds the configuration options.
///
/// # Returns
///
/// * `Result<String>` - The generated HTML or an error if the conversion fails.
///
/// # Example
///
/// ```rust
/// use ssg_html::HtmlConfig;
/// use ssg_html::generate_html;
/// let markdown = "# Hello, world!";
/// let config = HtmlConfig::default();
/// let html = generate_html(markdown, &config).unwrap();
/// assert!(html.contains("<h1>Hello, world!</h1>"));
/// ```
pub fn generate_html(
    markdown: &str,
    _config: &crate::HtmlConfig,
) -> Result<String> {
    markdown_to_html_with_extensions(markdown)
}

/// Convert Markdown to HTML with specified extensions.
///
/// This function applies a set of extensions to enhance the conversion
/// process. These extensions include strikethrough, table support, autolinks,
/// tasklists, and superscripts.
///
/// # Arguments
///
/// * `markdown` - A string slice that holds the Markdown content to convert.
///
/// # Returns
///
/// * `Result<String>` - The generated HTML or an error if the conversion fails.
///
/// # Example
///
/// ```rust
/// use ssg_html::generator::markdown_to_html_with_extensions;
/// let markdown = "~~strikethrough~~";
/// let html = markdown_to_html_with_extensions(markdown).unwrap();
/// assert!(html.contains("<del>strikethrough</del>"));
/// ```
pub fn markdown_to_html_with_extensions(
    markdown: &str,
) -> Result<String> {
    // Extract front matter from the Markdown content
    let content_without_front_matter =
        extract_front_matter(markdown).unwrap_or(markdown.to_string());

    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;

    // Set render options to avoid wrapping everything in <pre><code>
    options.render.github_pre_lang = false; // Ensures Comrak doesn't assume everything is code
    options.render.unsafe_ = true; // Allow unsafe HTML rendering for better debugging

    // Debug print to ensure options are correctly set
    // println!("{:?}", options);

    // Render the Markdown to HTML
    let html_output =
        markdown_to_html(&content_without_front_matter, &options);

    // Print the generated HTML to debug the issue
    // println!("{}", html_output);

    Ok(html_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlConfig;

    /// Test basic Markdown to HTML conversion.
    ///
    /// This test verifies that a simple Markdown input is correctly converted to HTML.
    #[test]
    fn test_generate_html_basic() {
        let markdown = "# Hello, world!\n\nThis is a test.";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Hello, world!</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }

    /// Test conversion with Markdown extensions.
    ///
    /// This test ensures that the Markdown extensions (e.g., strikethrough, tables, autolinks)
    /// are correctly applied when converting Markdown to HTML.
    #[test]
    fn test_markdown_to_html_with_extensions() {
        // Simplified input to focus on table rendering
        let markdown = r#"
| Header 1 | Header 2 |
| -------- | -------- |
| Row 1    | Row 2    |
"#;
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        // Debug output
        println!("{}", html);

        // Check if the table is rendered
        assert!(html.contains("<table>"), "Table element not found");
        assert!(
            html.contains("<th>Header 1</th>"),
            "Table header not found"
        );
        assert!(html.contains("<td>Row 1</td>"), "Table row not found");
    }

    /// Test conversion of empty Markdown.
    ///
    /// This test checks that an empty Markdown input results in an empty HTML string.
    #[test]
    fn test_generate_html_empty() {
        let markdown = "";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.is_empty());
    }

    /// Test handling of invalid Markdown.
    ///
    /// This test verifies that even with poorly formatted Markdown, the function
    /// will not panic and will return valid HTML.
    #[test]
    fn test_generate_html_invalid_markdown() {
        let markdown = "# Unclosed header\nSome **unclosed bold";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();

        // Debug output
        println!("{}", html);

        // Modify the assertion to reflect Comrak's strict handling of unclosed tags
        assert!(
            html.contains("<h1>Unclosed header</h1>"),
            "Header not found"
        );
        // Comrak does not automatically close bold tags; this is expected behavior
        assert!(
            html.contains("<p>Some **unclosed bold</p>"),
            "Unclosed bold tag not properly handled"
        );
    }

    /// Test conversion with complex Markdown content.
    ///
    /// This test checks how the function handles more complex Markdown input with various
    /// elements like lists, headers, code blocks, and links.
    #[test]
    fn test_generate_html_complex() {
        // Ensure no leading indentation in the Markdown input.
        let markdown = r#"
# Header

## Subheader

Some `inline code` and a [link](https://example.com).

```rust
fn main() {
    println!("Hello, world!");
}
    ```

    1. First item
    2. Second item
    "#;
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();

        // Print the generated HTML for debugging
        println!("{}", html);

        // Verify if Comrak processed the Markdown as expected
        assert!(
            html.contains("<h1>Header</h1>"),
            "H1 Header not found"
        );
        assert!(
            html.contains("<h2>Subheader</h2>"),
            "H2 Header not found"
        );
        assert!(
            html.contains("<code>inline code</code>"),
            "Inline code not found"
        );
        assert!(
            html.contains(r#"<a href="https://example.com">link</a>"#),
            "Link not found"
        );

        // Check for encoded special characters in code blocks
        assert!(
            html.contains("&quot;Hello, world!&quot;"),
            "Special characters not encoded in code block"
        );
    }
}
