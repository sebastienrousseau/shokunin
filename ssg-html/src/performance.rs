use crate::Result;
use comrak::{markdown_to_html, ComrakOptions};

/// Minify HTML content
pub fn minify_html(html: &str) -> Result<String> {
    // Implement HTML minification logic here
    // For now, we'll just return the original HTML
    Ok(html.to_string())
}

/// Asynchronously generate HTML
pub async fn async_generate_html(markdown: &str) -> Result<String> {
    let options = ComrakOptions::default();
    Ok(markdown_to_html(markdown, &options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_html() {
        let html = "<html><body><p>Test</p></body></html>";
        let result = minify_html(html);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), html);
    }

    #[tokio::test]
    async fn test_async_generate_html() {
        let markdown = "# Test\n\nThis is a test.";
        let result = async_generate_html(markdown).await;
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }
}
