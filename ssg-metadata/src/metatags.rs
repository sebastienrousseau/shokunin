use std::{collections::HashMap, fmt};

/// Holds collections of meta tags for different platforms and categories.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct MetaTagGroups {
    /// The `apple` meta tags.
    pub apple: String,
    /// The primary meta tags.
    pub primary: String,
    /// The `og` meta tags.
    pub og: String,
    /// The `ms` meta tags.
    pub ms: String,
    /// The `twitter` meta tags.
    pub twitter: String,
}

impl MetaTagGroups {
    /// Adds a custom meta tag to the appropriate group.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the meta tag.
    /// * `content` - The content of the meta tag.
    pub fn add_custom_tag(&mut self, name: &str, content: &str) {
        if name.starts_with("apple") {
            self.apple.push_str(&format_meta_tag(name, content));
        } else if name.starts_with("og") {
            self.og.push_str(&format_meta_tag(name, content));
        } else if name.starts_with("ms") {
            self.ms.push_str(&format_meta_tag(name, content));
        } else if name.starts_with("twitter") {
            self.twitter.push_str(&format_meta_tag(name, content));
        } else {
            self.primary.push_str(&format_meta_tag(name, content));
        }
    }
}

/// Implement `Display` for `MetaTagGroups`.
impl fmt::Display for MetaTagGroups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}",
            self.apple, self.primary, self.og, self.ms, self.twitter
        )
    }
}

/// Generates HTML meta tags based on the provided metadata.
///
/// This function takes metadata from a `HashMap` and generates meta tags for various platforms (e.g., Apple, Open Graph, Twitter).
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing the metadata.
///
/// # Returns
///
/// A `MetaTagGroups` structure with meta tags grouped by platform.
pub fn generate_metatags(
    metadata: &HashMap<String, String>,
) -> MetaTagGroups {
    MetaTagGroups {
        apple: generate_apple_meta_tags(metadata),
        primary: generate_primary_meta_tags(metadata),
        og: generate_og_meta_tags(metadata),
        ms: generate_ms_meta_tags(metadata),
        twitter: generate_twitter_meta_tags(metadata),
    }
}

/// Generates meta tags for Apple devices.
fn generate_apple_meta_tags(
    metadata: &HashMap<String, String>,
) -> String {
    const APPLE_TAGS: [&str; 3] = [
        "apple-mobile-web-app-capable",
        "apple-mobile-web-app-status-bar-style",
        "apple-mobile-web-app-title",
    ];
    generate_tags(metadata, &APPLE_TAGS)
}

/// Generates primary meta tags like `author`, `description`, and `keywords`.
fn generate_primary_meta_tags(
    metadata: &HashMap<String, String>,
) -> String {
    const PRIMARY_TAGS: [&str; 4] =
        ["author", "description", "keywords", "viewport"];
    generate_tags(metadata, &PRIMARY_TAGS)
}

/// Generates Open Graph (`og`) meta tags for social media.
fn generate_og_meta_tags(metadata: &HashMap<String, String>) -> String {
    const OG_TAGS: [&str; 5] = [
        "og:title",
        "og:description",
        "og:image",
        "og:url",
        "og:type",
    ];
    generate_tags(metadata, &OG_TAGS)
}

/// Generates Microsoft-specific meta tags.
fn generate_ms_meta_tags(metadata: &HashMap<String, String>) -> String {
    const MS_TAGS: [&str; 2] =
        ["msapplication-TileColor", "msapplication-TileImage"];
    generate_tags(metadata, &MS_TAGS)
}

/// Generates Twitter meta tags for embedding rich media in tweets.
fn generate_twitter_meta_tags(
    metadata: &HashMap<String, String>,
) -> String {
    const TWITTER_TAGS: [&str; 5] = [
        "twitter:card",
        "twitter:site",
        "twitter:title",
        "twitter:description",
        "twitter:image",
    ];
    generate_tags(metadata, &TWITTER_TAGS)
}

/// Generates meta tags based on the provided list of tag names.
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing the metadata.
/// * `tags` - A reference to an array of tag names.
///
/// # Returns
///
/// A string containing the generated meta tags.
fn generate_tags(
    metadata: &HashMap<String, String>,
    tags: &[&str],
) -> String {
    tags.iter()
        .filter_map(|&tag| {
            metadata.get(tag).map(|value| format_meta_tag(tag, value))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formats a single meta tag.
///
/// # Arguments
///
/// * `name` - The name of the meta tag (e.g., `author`, `description`).
/// * `content` - The content of the meta tag.
///
/// # Returns
///
/// A formatted meta tag string.
fn format_meta_tag(name: &str, content: &str) -> String {
    format!(
        "<meta name=\"{}\" content=\"{}\">",
        name,
        content.replace('"', "&quot;")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_metatags() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "John Doe".to_string());
        metadata.insert(
            "description".to_string(),
            "A test page".to_string(),
        );
        metadata
            .insert("og:title".to_string(), "Test Title".to_string());

        let meta_tags = generate_metatags(&metadata);

        assert!(meta_tags.primary.contains("author"));
        assert!(meta_tags.primary.contains("description"));
        assert!(meta_tags.og.contains("og:title"));
        assert!(meta_tags.apple.is_empty());
        assert!(meta_tags.ms.is_empty());
        assert!(meta_tags.twitter.is_empty());
    }

    #[test]
    fn test_add_custom_tag() {
        let mut meta_tags = MetaTagGroups::default();
        meta_tags.add_custom_tag("custom-tag", "custom value");
        meta_tags.add_custom_tag("og:custom", "custom og value");

        assert!(meta_tags.primary.contains("custom-tag"));
        assert!(meta_tags.og.contains("og:custom"));
    }
}
