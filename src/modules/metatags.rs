// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use crate::models::data::{MetaTag, MetaTagGroups};
use crate::{macro_generate_tags_from_list, macro_generate_tags_from_fields};

// Type alias for better readability
type MetaDataMap = HashMap<String, String>;

/// Generates HTML meta tags based on custom key-value mappings.
///
/// # Arguments
/// * `mapping` - A slice of tuples, where each tuple contains a `String` key and an `Option<String>` value.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
pub fn generate_custom_meta_tags(mapping: &[(String, Option<String>)]) -> String {
    let filtered_mapping: Vec<(String, String)> = mapping
        .iter()
        .filter_map(|(key, value)| {
            if let Some(val) = value.as_ref() {
                if !val.is_empty() {
                    Some((key.clone(), val.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    generate_metatags(&filtered_mapping)
}

/// Generates HTML meta tags based on the provided key-value pairs.
///
/// # Arguments
/// * `meta` - A slice of key-value pairs represented as tuples of `String` objects.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
pub fn generate_metatags(meta: &[(String, String)]) -> String {
    meta.iter()
        .map(|(key, value)| format_meta_tag(key, value))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generates HTML meta tags based on a list of tag names and a metadata HashMap.
///
/// # Arguments
/// * `tag_names` - A slice of tag names as `&str`.
/// * `metadata` - A reference to a `MetaDataMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
pub fn load_metatags(tag_names: &[&str], metadata: &MetaDataMap) -> String {
    let mut result = String::new();
    for &name in tag_names {
        let value = metadata.get(name).cloned().unwrap_or_default();
        result.push_str(&MetaTag::new(name.to_string(), value).generate());
    }
    result
}

/// Utility function to format a single meta tag into its HTML representation.
///
/// # Arguments
/// * `key` - The name attribute of the meta tag.
/// * `value` - The content attribute of the meta tag.
///
/// # Returns
/// A `String` containing the HTML representation of the meta tag.
pub fn format_meta_tag(key: &str, value: &str) -> String {
    format!("<meta name=\"{}\" content=\"{}\">", key, value)
}

/// Generates HTML meta tags for Apple-specific settings.
///
/// # Arguments
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_apple_meta_tags(metadata: &MetaDataMap) -> String {
    let tag_names = [
        "apple_mobile_web_app_orientations", "apple_touch_icon_sizes",
        "apple-mobile-web-app-capable", "apple-mobile-web-app-status-bar-inset",
        "apple-mobile-web-app-status-bar-style", "apple-mobile-web-app-title",
        "apple-touch-fullscreen",
    ];
    macro_generate_tags_from_list!(&tag_names, metadata)
}

/// Generates HTML meta tags for primary settings like author, description, etc.
///
/// # Arguments
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_primary_meta_tags(metadata: &MetaDataMap) -> String {
    let tag_names = [
        "author", "description", "format-detection", "generator", "keywords",
        "language", "permalink", "rating", "referrer", "revisit-after",
        "robots", "theme-color", "title", "viewport",
    ];
    macro_generate_tags_from_list!(&tag_names, metadata)
}

/// Generates HTML meta tags for Open Graph settings, primarily for social media.
///
/// This function expects the `metadata` HashMap to contain keys such as:
///
/// - "og:description": The description of the content.
/// - "og:image": The URL of the image to use.
/// - "og:image:alt": The alt text for the image.
/// - "og:image:height": The height of the image.
/// - "og:image:width": The width of the image.
/// - "og:locale": The locale of the content.
/// - "og:site_name": The name of the site.
/// - "og:title": The title of the content.
/// - "og:type": The type of content.
/// - "og:url": The URL of the content.
///
/// # Arguments
/// * `metadata` - A reference to a `MetaDataMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_og_meta_tags(metadata: &MetaDataMap) -> String {
    macro_generate_tags_from_fields!(
        generate_og_meta_tags,
        metadata,
        "og:description" => description,
        "og:image" => image,
        "og:image:alt" => image_alt,
        "og:image:height" => image_height,
        "og:image:width" => image_width,
        "og:locale" => locale,
        "og:site_name" => site_name,
        "og:title" => title,
        "og:type" => type,
        "og:url" => permalink
    )
}

/// Generates HTML meta tags for Microsoft-specific settings.
///
/// # Arguments
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_ms_meta_tags(metadata: &MetaDataMap) -> String {
    let tag_names = ["msapplication-navbutton-color"];
    macro_generate_tags_from_list!(&tag_names, metadata)
}

/// Generates HTML meta tags for Twitter-specific settings.
///
/// This function expects the `metadata` HashMap to contain keys such as:
/// - "twitter:card": The type of Twitter card to use.
/// - "twitter:creator": The Twitter handle of the content creator.
/// - "twitter:description": The description of the content.
/// - "twitter:image": The URL of the image to use.
/// - "twitter:image:alt": The alt text for the image.
/// - "twitter:image:height": The height of the image.
/// - "twitter:image:width": The width of the image.
/// - "twitter:site": The Twitter handle of the site.
/// - "twitter:title": The title of the content.
/// - "twitter:url": The URL of the content.
///
/// # Arguments
/// * `metadata` - A reference to a `MetaDataMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_twitter_meta_tags(metadata: &MetaDataMap) -> String {
    macro_generate_tags_from_fields!(
        generate_twitter_meta_tags,
        metadata,
        "twitter:card" => twitter_card,
        "twitter:creator" => twitter_creator,
        "twitter:description" => description,
        "twitter:image" => image,
        "twitter:image:alt" => image_alt,
        "twitter:image:height" => image_height,
        "twitter:image:width" => image_width,
        "twitter:site" => url,
        "twitter:title" => title,
        "twitter:url" => url
    )
}

/// Generates meta tags for the given metadata.
///
/// # Arguments
///
/// * `metadata` - The metadata extracted from the file.
///
/// # Returns
///
/// Returns a tuple containing meta tags for Apple devices, primary information, Open Graph, Microsoft, and Twitter.
///
pub fn generate_all_meta_tags(metadata: &MetaDataMap) -> MetaTagGroups {
    MetaTagGroups {
        apple: generate_apple_meta_tags(metadata),
        primary: generate_primary_meta_tags(metadata),
        og: generate_og_meta_tags(metadata),
        ms: generate_ms_meta_tags(metadata),
        twitter: generate_twitter_meta_tags(metadata),
    }
}
