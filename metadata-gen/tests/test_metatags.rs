//! Unit tests for the `metatags` module.
//!
//! This module tests the handling and validation of meta tags in HTML documents.

#[cfg(test)]
mod tests {
    use metadata_gen::MetaTagGroups;
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

    /// Test adding a custom primary meta tag.
    #[test]
    fn test_add_custom_primary_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags.add_custom_tag("custom-tag", "custom value");

        assert!(meta_tags.primary.contains("custom-tag"));
        assert!(meta_tags.primary.contains("custom value"));
    }

    /// Test adding a custom OpenGraph (og) meta tag.
    #[test]
    fn test_add_custom_og_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags.add_custom_tag("og:custom", "custom og value");

        assert!(meta_tags.og.contains("og:custom"));
        assert!(meta_tags.og.contains("custom og value"));
    }

    /// Test adding a custom Twitter meta tag.
    #[test]
    fn test_add_custom_twitter_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags
            .add_custom_tag("twitter:custom", "custom twitter value");

        assert!(meta_tags.twitter.contains("twitter:custom"));
        assert!(meta_tags.twitter.contains("custom twitter value"));
    }

    /// Test adding a custom Apple meta tag.
    #[test]
    fn test_add_custom_apple_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags.add_custom_tag(
            "apple-mobile-web-app-custom",
            "custom apple value",
        );

        assert!(meta_tags
            .apple
            .contains("apple-mobile-web-app-custom"));
        assert!(meta_tags.apple.contains("custom apple value"));
    }

    /// Test adding a custom Microsoft meta tag.
    #[test]
    fn test_add_custom_ms_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags
            .add_custom_tag("msapplication-custom", "custom ms value");

        assert!(meta_tags.ms.contains("msapplication-custom"));
        assert!(meta_tags.ms.contains("custom ms value"));
    }

    /// Test adding multiple custom tags of different types.
    #[test]
    fn test_add_multiple_custom_tags() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags.add_custom_tag("custom-primary", "primary value");
        meta_tags.add_custom_tag("og:custom", "og value");
        meta_tags.add_custom_tag("twitter:custom", "twitter value");
        meta_tags.add_custom_tag(
            "apple-mobile-web-app-custom",
            "apple value",
        );
        meta_tags.add_custom_tag("msapplication-custom", "ms value");

        // Check primary meta tags
        assert!(meta_tags.primary.contains("<meta name=\"custom-primary\" content=\"primary value\">"),
        "Primary meta tag should contain 'custom-primary'");

        // Check Open Graph (og) meta tags
        assert!(
            meta_tags.og.contains(
                "<meta name=\"og:custom\" content=\"og value\">"
            ),
            "OG meta tag should contain 'og:custom'"
        );

        // Check Twitter meta tags
        assert!(meta_tags.twitter.contains("<meta name=\"twitter:custom\" content=\"twitter value\">"),
        "Twitter meta tag should contain 'twitter:custom'");

        // Check Apple meta tags
        assert!(meta_tags.apple.contains("<meta name=\"apple-mobile-web-app-custom\" content=\"apple value\">"),
        "Apple meta tag should contain 'apple-mobile-web-app-custom'");

        // Check Microsoft meta tags
        assert!(meta_tags.ms.contains("<meta name=\"msapplication-custom\" content=\"ms value\">"),
        "MS meta tag should contain 'msapplication-custom'");
    }
}
