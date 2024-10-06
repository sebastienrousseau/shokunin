use crate::HtmlError;
use crate::Result;
use comrak::{markdown_to_html, ComrakOptions};
use minify_html::{minify, Cfg};
use std::{fs, path::Path};

/// Returns a default `Cfg` for HTML minification.
///
/// This helper function creates a default configuration for minifying HTML
/// with pre-set options for CSS, JS, and attributes.
///
/// # Returns
/// A `Cfg` object containing the default minification settings.
fn default_minify_cfg() -> Cfg {
    let mut cfg = Cfg::new();
    cfg.do_not_minify_doctype = true;
    cfg.ensure_spec_compliant_unquoted_attribute_values = true;
    cfg.keep_closing_tags = true;
    cfg.keep_html_and_head_opening_tags = true;
    cfg.keep_spaces_between_attributes = true;
    cfg.keep_comments = false;
    cfg.minify_css = true;
    cfg.minify_js = true;
    cfg.remove_bangs = true;
    cfg.remove_processing_instructions = true;
    cfg
}

/// Minifies a single HTML file.
///
/// This function takes a reference to a `Path` object for an HTML file and
/// returns a string containing the minified HTML.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` object for the HTML file.
///
/// # Returns
///
/// * `Result<String, HtmlError>` - A result containing a string
///    containing the minified HTML.
///     - `Ok(String)` if the HTML file was minified successfully.
///     - `Err(HtmlError)` if the HTML file could not be minified.
///
pub fn minify_html(file_path: &Path) -> Result<String> {
    // Read the file content
    let file_content = fs::read(file_path)
        .map_err(|e| HtmlError::MinificationError(e.to_string()))?;

    // Ensure that the content is valid UTF-8 before proceeding with minification
    let content_str = String::from_utf8(file_content).map_err(|e| {
        HtmlError::MinificationError(format!(
            "Invalid UTF-8 sequence: {}",
            e
        ))
    })?;

    // Minify the valid UTF-8 content
    let minified_content =
        minify(content_str.as_bytes(), &default_minify_cfg());

    // Convert the minified content back to a UTF-8 string
    String::from_utf8(minified_content)
        .map_err(|e| HtmlError::MinificationError(e.to_string()))
}

/// Asynchronously generate HTML from Markdown.
///
/// This function converts a Markdown string into an HTML string using
/// Comrak, a CommonMark-compliant Markdown parser and renderer.
///
/// # Arguments
///
/// * `markdown` - A reference to a Markdown string.
///
/// # Returns
///
/// * `Result<String, HtmlError>` - A result containing a string with the
///   generated HTML.
pub async fn async_generate_html(markdown: &str) -> Result<String> {
    let options = ComrakOptions::default();
    Ok(markdown_to_html(markdown, &options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Helper function to create an HTML file for testing.
    fn create_html_file(file_path: &Path, content: &str) {
        let mut file = File::create(file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_minify_html_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.html");
        let html = "<html>  <body>    <p>Test</p>  </body>  </html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "<html><body><p>Test</p></body></html>"
        );
    }

    #[test]
    fn test_minify_html_with_comments() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_comments.html");
        let html = "<html>  <body>    <!-- This is a comment -->    <p>Test</p>  </body>  </html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "<html><body><p>Test</p></body></html>"
        );
    }

    #[test]
    fn test_minify_html_with_css() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_css.html");
        let html = "<html><head><style>  body  {  color:  red;  }  </style></head><body><p>Test</p></body></html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<html><head><style>body{color:red}</style></head><body><p>Test</p></body></html>");
    }

    #[test]
    fn test_minify_html_with_js() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_js.html");
        let html = "<html><head><script>  function  test()  {  console.log('Hello');  }  </script></head><body><p>Test</p></body></html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        let minified = result.unwrap();
        assert!(minified.contains("<script>"));
        assert!(minified.contains("console.log"));
        assert!(minified.contains("Hello"));
        assert!(minified.contains("<p>Test</p>"));
    }

    #[test]
    fn test_minify_html_non_existent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.html");

        let result = minify_html(&file_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MinificationError(_)
        ));
    }

    #[test]
    fn test_minify_html_invalid_utf8() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid_utf8.html");
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence

        let mut file = File::create(&file_path).unwrap();
        file.write_all(&invalid_utf8).unwrap();

        let result = minify_html(&file_path);

        // Ensure the result is an error, as expected due to invalid UTF-8
        assert!(
            result.is_err(),
            "Expected an error due to invalid UTF-8 sequence"
        );
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MinificationError(_)
        ));
    }

    #[tokio::test]
    async fn test_async_generate_html_basic() {
        let markdown = "# Test\n\nThis is a test.";
        let result = async_generate_html(markdown).await;
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }

    #[tokio::test]
    async fn test_async_generate_html_complex() {
        let markdown = "# Header\n\n## Subheader\n\n- List item 1\n- List item 2\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let result = async_generate_html(markdown).await;
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Header</h1>"));
        assert!(html.contains("<h2>Subheader</h2>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>List item 1</li>"));
        assert!(html.contains("<li>List item 2</li>"));
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main()"));
        assert!(html.contains("println!"));
        assert!(html.contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_async_generate_html_empty_input() {
        let markdown = "";
        let result = async_generate_html(markdown).await;
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.trim().is_empty());
    }
}
