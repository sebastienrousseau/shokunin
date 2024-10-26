// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Web App Manifest Generation Module
//!
//! This module handles the creation and generation of web app manifest files
//! (manifest.json). The manifest provides information about the web application
//! required for installation and presentation to the user.

use crate::{
    macro_metadata_option,
    models::data::{IconData, ManifestData},
};
use std::collections::HashMap;

/// Default values for manifest fields
const DEFAULT_START_URL: &str = ".";
const DEFAULT_DISPLAY: &str = "standalone";
const DEFAULT_BACKGROUND: &str = "#ffffff";
const DEFAULT_ORIENTATION: &str = "portrait-primary";
const DEFAULT_SCOPE: &str = "/";
const DEFAULT_ICON_SIZE: &str = "512x512";
const DEFAULT_ICON_TYPE: &str = "image/svg+xml";
const DEFAULT_ICON_PURPOSE: &str = "any maskable";

/// Creates a ManifestData object from metadata.
///
/// This function processes metadata to create a web app manifest configuration.
/// It handles all required and optional fields with appropriate defaults.
///
/// # Arguments
/// * `metadata` - A reference to a HashMap containing metadata key-value pairs
///
/// # Returns
/// * `ManifestData` - A struct containing the manifest configuration
pub fn create_manifest_data(
    metadata: &HashMap<String, String>,
) -> ManifestData {
    ManifestData {
        name: metadata
            .get("name")
            .map_or_else(String::new, |v| sanitize_name(v)),
        short_name: sanitize_name(&macro_metadata_option!(
            metadata,
            "short_name"
        )),
        start_url: DEFAULT_START_URL.to_string(),
        display: DEFAULT_DISPLAY.to_string(),
        background_color: metadata.get("background-color").map_or_else(
            || DEFAULT_BACKGROUND.to_string(),
            |color| sanitize_color(color),
        ),
        description: sanitize_description(&macro_metadata_option!(
            metadata,
            "description"
        )),
        icons: generate_icons(metadata),
        orientation: DEFAULT_ORIENTATION.to_string(),
        scope: DEFAULT_SCOPE.to_string(),
        theme_color: metadata
            .get("theme-color")
            .map_or_else(String::new, |color| {
                sanitize_color(&format!("rgb({})", color))
            }),
    }
}

/// Sanitizes a manifest name or short name.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .take(45) // PWA name length recommendation
        .collect()
}

/// Sanitizes a manifest description.
fn sanitize_description(desc: &str) -> String {
    desc.chars()
        .filter(|c| !c.is_control())
        .take(120) // Reasonable description length
        .collect()
}

/// Sanitizes a color value.
fn sanitize_color(color: &str) -> String {
    if (color.starts_with('#')
        && (color.len() == 4 || color.len() == 7)
        && color[1..].chars().all(|c| c.is_ascii_hexdigit()))
        || (color.starts_with("rgb(") && color.ends_with(')'))
    {
        color.to_string()
    } else {
        DEFAULT_BACKGROUND.to_string()
    }
}

/// Generates icon configurations from metadata.
fn generate_icons(metadata: &HashMap<String, String>) -> Vec<IconData> {
    metadata
        .get("icon")
        .map(|icon| {
            vec![IconData {
                src: sanitize_icon_url(icon),
                sizes: DEFAULT_ICON_SIZE.to_string(),
                icon_type: Some(DEFAULT_ICON_TYPE.to_string()),
                purpose: Some(DEFAULT_ICON_PURPOSE.to_string()),
            }]
        })
        .unwrap_or_default()
}

/// Sanitizes an icon URL.
fn sanitize_icon_url(url: &str) -> String {
    if url.contains('<')
        || url.contains('>')
        || url.contains('"')
        || url.contains('\'')
    {
        String::new()
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manifest_data() {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "Test App".to_string());
        metadata.insert(
            "description".to_string(),
            "Test Description".to_string(),
        );

        let manifest = create_manifest_data(&metadata);

        assert_eq!(manifest.name, "Test App");
        assert_eq!(manifest.description, "Test Description");
        assert_eq!(manifest.start_url, ".");
        assert_eq!(manifest.display, "standalone");
    }

    #[test]
    fn test_manifest_with_icon() {
        let mut metadata = HashMap::new();
        metadata.insert("icon".to_string(), "/icon.svg".to_string());

        let manifest = create_manifest_data(&metadata);

        assert!(!manifest.icons.is_empty());
        assert_eq!(manifest.icons[0].src, "/icon.svg");
        assert_eq!(manifest.icons[0].sizes, "512x512");
    }

    #[test]
    fn test_sanitize_name() {
        let long_name = "a".repeat(100);
        assert_eq!(sanitize_name(&long_name).len(), 45);

        let name_with_control = "Test\nApp\tName";
        assert_eq!(sanitize_name(name_with_control), "TestAppName");
    }

    #[test]
    fn test_sanitize_color() {
        assert_eq!(sanitize_color("#fff"), "#fff");
        assert_eq!(sanitize_color("#ffffff"), "#ffffff");
        assert_eq!(
            sanitize_color("rgb(255,255,255)"),
            "rgb(255,255,255)"
        );
        assert_eq!(sanitize_color("invalid"), DEFAULT_BACKGROUND);
    }

    #[test]
    fn test_sanitize_icon_url() {
        assert_eq!(
            sanitize_icon_url("/valid/path.svg"),
            "/valid/path.svg"
        );
        assert_eq!(
            sanitize_icon_url("https://example.com/icon.svg"),
            "https://example.com/icon.svg"
        );
        assert!(sanitize_icon_url("<script>alert('xss')</script>")
            .is_empty());
    }

    #[test]
    fn test_empty_manifest() {
        let manifest = create_manifest_data(&HashMap::new());

        assert!(manifest.name.is_empty());
        assert!(manifest.description.is_empty());
        assert_eq!(manifest.display, "standalone");
        assert_eq!(manifest.background_color, "#ffffff");
        assert!(manifest.icons.is_empty());
    }

    #[test]
    fn test_theme_color() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "theme-color".to_string(),
            "255,255,255".to_string(),
        );

        let manifest = create_manifest_data(&metadata);
        assert_eq!(manifest.theme_color, "rgb(255,255,255)");
    }
}
