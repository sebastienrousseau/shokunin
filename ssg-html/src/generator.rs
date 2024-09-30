use crate::extract_front_matter;
use crate::HtmlError;
use crate::Result;
use mdx_gen::{process_markdown, ComrakOptions, MarkdownOptions};

/// Generate HTML from Markdown content using `mdx-gen`.
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

/// Convert Markdown to HTML with specified extensions using `mdx-gen`.
///
/// This function applies a set of extensions to enhance the conversion
/// process, such as syntax highlighting, enhanced table formatting,
/// custom blocks, and more.
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
    let content_without_front_matter =
        extract_front_matter(markdown).unwrap_or(markdown.to_string());

    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.superscript = true;

    let options =
        MarkdownOptions::default().with_comrak_options(comrak_options);

    // Process the Markdown to HTML using `mdx-gen`
    match process_markdown(&content_without_front_matter, &options) {
        Ok(html_output) => Ok(html_output),
        Err(err) => {
            Err(HtmlError::MarkdownConversionError(err.to_string()))
        }
    }
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
    /// This test ensures that the Markdown extensions (e.g., custom blocks, enhanced tables, etc.)
    /// are correctly applied when converting Markdown to HTML.
    #[test]
    fn test_markdown_to_html_with_extensions() {
        let markdown = r#"
| Header 1 | Header 2 |
| -------- | -------- |
| Row 1    | Row 2    |
"#;
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("{}", html);

        // Update the test to look for the div wrapper and table classes
        assert!(html.contains("<div class=\"table-responsive\"><table class=\"table\">"), "Table element not found");
        assert!(
            html.contains("<th>Header 1</th>"),
            "Table header not found"
        );
        assert!(
            html.contains("<td class=\"text-left\">Row 1</td>"),
            "Table row not found"
        );
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

        println!("{}", html);

        assert!(
            html.contains("<h1>Unclosed header</h1>"),
            "Header not found"
        );
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

        println!("{}", html); // Print the HTML for inspection

        // Verify the header and subheader
        assert!(
            html.contains("<h1>Header</h1>"),
            "H1 Header not found"
        );
        assert!(
            html.contains("<h2>Subheader</h2>"),
            "H2 Header not found"
        );

        // Verify the inline code and link
        assert!(
            html.contains("<code>inline code</code>"),
            "Inline code not found"
        );
        assert!(
            html.contains(r#"<a href="https://example.com">link</a>"#),
            "Link not found"
        );

        // Verify that the code block starts correctly
        assert!(
            html.contains(r#"<code class="language-rust">"#),
            "Rust code block not found"
        );

        // Match each part of the highlighted syntax separately
        // Check for `fn` keyword in a span with the correct style
        assert!(
            html.contains(r#"<span style="color:#b48ead;">fn </span>"#),
            "`fn` keyword with syntax highlighting not found"
        );

        // Check for `main` in a span with the correct style
        assert!(
            html.contains(
                r#"<span style="color:#8fa1b3;">main</span>"#
            ),
            "`main` function name with syntax highlighting not found"
        );

        // Check for `First item` and `Second item` in the ordered list
        assert!(html.contains("First item"), "First item not found");
        assert!(html.contains("Second item"), "Second item not found");
    }
}
