//! Unit tests for the `metatags` module.
//!
//! This module tests the handling and validation of meta tags in HTML documents.

#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::collections::HashMap;

    /// Parses meta tags from an HTML string and returns a HashMap.
    fn parse_metatags(html: &str) -> HashMap<String, String> {
        let mut metatags = HashMap::new();
        let re = Regex::new(r#"<meta\s+name=["']([^"']+)["']\s+content=["']([^"']*)["']\s*/?>"#).unwrap();

        for cap in re.captures_iter(html) {
            // Only add the meta tag if the content is not empty
            if !cap[2].trim().is_empty() {
                metatags.insert(cap[1].to_string(), cap[2].to_string());
            }
        }

        metatags
    }

    /// Test if valid meta tags are correctly identified.
    ///
    /// This test checks that the system correctly parses and validates meta tags.
    #[test]
    fn test_valid_metatags() {
        let html = "<meta name='keywords' content='rust, testing'>";
        let metatags = parse_metatags(html);
        assert!(metatags.contains_key("keywords"));
        assert_eq!(
            metatags.get("keywords"),
            Some(&"rust, testing".to_string())
        );
    }

    /// Test for invalid meta tags.
    ///
    /// This test ensures that invalid or malformed meta tags are handled gracefully.
    #[test]
    fn test_invalid_metatags() {
        let html = "<meta name='keywords' content=''>";
        let metatags = parse_metatags(html);
        assert!(
            metatags.is_empty(),
            "Empty meta tags should not be parsed"
        );
    }
}
