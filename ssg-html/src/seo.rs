use crate::Result;
use scraper::{Html, Selector};

/// Generate meta tags for SEO
pub fn generate_meta_tags(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let mut meta_tags = String::new();

    if let Some(title) =
        document.select(&Selector::parse("title").unwrap()).next()
    {
        meta_tags.push_str(&format!(
            r#"<meta name="title" content="{}">"#,
            title.inner_html()
        ));
    }

    if let Some(description) =
        document.select(&Selector::parse("p").unwrap()).next()
    {
        meta_tags.push_str(&format!(
            r#"<meta name="description" content="{}">"#,
            description.inner_html()
        ));
    }

    meta_tags
        .push_str(r#"<meta property="og:type" content="website">"#);

    Ok(meta_tags)
}

/// Generate structured data (JSON-LD) for SEO
pub fn generate_structured_data(html: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|t| t.inner_html())
        .unwrap_or_else(|| "Untitled".to_string());

    let description = document
        .select(&Selector::parse("p").unwrap())
        .next()
        .map(|p| p.inner_html())
        .unwrap_or_else(|| "No description available".to_string());

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
}
