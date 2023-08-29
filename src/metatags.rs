// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use crate::data::MetatagsData;

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
        .filter_map(|(key, value)| value.as_ref().map(|v| (key.clone(), v.clone())))
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
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
pub fn load_metatags(tag_names: &[&str], metadata: &HashMap<String, String>) -> String {
    tag_names.iter().fold(String::new(), |acc, &name| {
        let value = metadata.get(name).unwrap_or(&String::new()).clone();
        acc + &MetatagsData::new(name.to_string(), value).generate()
    })
}

/// Utility function to format a single meta tag into its HTML representation.
///
/// # Arguments
/// * `key` - The name attribute of the meta tag.
/// * `value` - The content attribute of the meta tag.
///
/// # Returns
/// A `String` containing the HTML representation of the meta tag.
///
fn format_meta_tag(key: &str, value: &str) -> String {
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
pub fn generate_apple_meta_tags(metadata: &HashMap<String, String>) -> String {
    let tag_names = [
        "apple_mobile_web_app_orientations", "apple_touch_icon_sizes",
        "apple-mobile-web-app-capable", "apple-mobile-web-app-status-bar-inset",
        "apple-mobile-web-app-status-bar-style", "apple-mobile-web-app-title",
        "apple-touch-fullscreen",
    ];
    load_metatags(&tag_names, metadata)
}

/// Generates HTML meta tags for primary settings like author, description, etc.
///
/// # Arguments
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_primary_meta_tags(metadata: &HashMap<String, String>) -> String {
    let tag_names = [
        "author", "description", "format-detection", "generator", "keywords",
        "language", "permalink", "rating", "referrer", "revisit-after",
        "robots", "theme_color", "title", "viewport",
    ];
    load_metatags(&tag_names, metadata)
}

/// Generates HTML meta tags for Open Graph settings, primarily for social media.
///
/// # Arguments
/// * `og:description` - The description of the content.
/// * `og:image` - The URL of the image to use.
/// * `og:image:alt` - The alt text for the image.
/// * `og:image:height` - The height of the image.
/// * `og:image:width` - The width of the image.
/// * `og:locale` - The locale of the content.
/// * `og:site_name` - The name of the site.
/// * `og:title` - The title of the content.
/// * `og:type` - The type of content.
/// * `og:url` - The URL of the content.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_og_meta_tags(metadata: &HashMap<String, String>) -> String {
    let og_mapping: Vec<(String, Option<String>)> = vec![
        ("og:description", metadata.get("description").cloned()),
        ("og:image", metadata.get("image").cloned()),
        ("og:image:alt", metadata.get("image_alt").cloned()),
        ("og:image:height", metadata.get("image_height").cloned()),
        ("og:image:width", metadata.get("image_width").cloned()),
        ("og:locale", metadata.get("locale").cloned()),
        ("og:site_name", metadata.get("site_name").cloned()),
        ("og:title", metadata.get("title").cloned()),
        ("og:type", metadata.get("type").cloned()),
        ("og:url", metadata.get("permalink").cloned()),
    ].into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    generate_custom_meta_tags(&og_mapping)
}

/// Generates HTML meta tags for Microsoft-specific settings.
///
/// # Arguments
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_ms_meta_tags(metadata: &HashMap<String, String>) -> String {
    let tag_names = ["msapplication-navbutton-color"];
    load_metatags(&tag_names, metadata)
}

/// Generates HTML meta tags for Twitter-specific settings.
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing metadata key-value pairs.
/// * `twitter_card` - The type of Twitter card to use.
/// * `twitter_creator` - The Twitter handle of the content creator.
/// * `twitter_description` - The description of the content.
/// * `twitter_image` - The URL of the image to use.
/// * `twitter_image_alt` - The alt text for the image.
/// * `twitter_image_height` - The height of the image.
/// * `twitter_image_width` - The width of the image.
/// * `twitter_site` - The Twitter handle of the site.
/// * `twitter_title` - The title of the content.
/// * `twitter_url` - The URL of the content.
///
/// # Returns
/// A `String` containing the HTML code for the meta tags.
///
pub fn generate_twitter_meta_tags(metadata: &HashMap<String, String>) -> String {
    let twitter_mapping: Vec<(String, Option<String>)> = vec![
        ("twitter:card", metadata.get("twitter_card").cloned()),
        ("twitter:creator", metadata.get("twitter_creator").cloned()),
        ("twitter:description", metadata.get("description").cloned()),
        ("twitter:image", metadata.get("image").cloned()),
        ("twitter:image:alt", metadata.get("image_alt").cloned()),
        ("twitter:image:height", metadata.get("image_height").cloned()),
        ("twitter:image:width", metadata.get("image_width").cloned()),
        ("twitter:site", metadata.get("url").cloned()),
        ("twitter:title", metadata.get("title").cloned()),
        ("twitter:url", metadata.get("url").cloned()),
        ].into_iter().map(|(k, v)| (k.to_string(), v)).collect();

    generate_custom_meta_tags(&twitter_mapping)
}
