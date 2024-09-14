use crate::Result;
use comrak::{markdown_to_html, ComrakOptions};

/// Generate HTML from Markdown content
pub fn generate_html(
    markdown: &str,
    _config: &crate::HtmlConfig,
) -> Result<String> {
    markdown_to_html_with_extensions(markdown)
}

/// Convert Markdown to HTML with specified extensions
fn markdown_to_html_with_extensions(markdown: &str) -> Result<String> {
    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;

    Ok(markdown_to_html(markdown, &options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlConfig;

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
}
