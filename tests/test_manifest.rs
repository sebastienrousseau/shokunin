// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ssg::{
    macro_metadata_option,
    models::data::{IconData, ManifestData},
};
use std::collections::HashMap;

/// Function to create ManifestData
///
/// The `metadata` parameter is a map of metadata strings.
///
/// Returns a `ManifestData` object.
pub fn create_manifest_data(
    metadata: &HashMap<String, String>,
) -> ManifestData {
    ManifestData {
        name: metadata.get("name").cloned().unwrap_or_default(),
        short_name: macro_metadata_option!(metadata, "short_name"),
        start_url: ".".to_string(),
        display: "standalone".to_string(),
        background_color: "#ffffff".to_string(),
        description: macro_metadata_option!(metadata, "description"),
        icons: metadata
            .get("icon")
            .map(|icon| {
                vec![IconData {
                    src: icon.to_string(),
                    sizes: "512x512".to_string(),
                    icon_type: Some("image/svg+xml".to_string()),
                    purpose: Some("any maskable".to_string()),
                }]
            })
            .unwrap_or_default(),
        orientation: "portrait-primary".to_string(),
        scope: "/".to_string(),
        theme_color: metadata
            .get("theme-color")
            .map(|color| format!("rgb({})", color))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use ssg::modules::manifest::create_manifest_data;
    use std::collections::HashMap;

    /// Test case for creating `ManifestData` with valid metadata.
    #[test]
    fn test_create_manifest_data_with_valid_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "My Web App".to_string());
        metadata.insert("short_name".to_string(), "App".to_string());
        metadata.insert(
            "description".to_string(),
            "A cool web app".to_string(),
        );
        metadata.insert("icon".to_string(), "app-icon.svg".to_string());
        metadata
            .insert("theme-color".to_string(), "#00aabb".to_string());

        let manifest_data = create_manifest_data(&metadata);

        assert_eq!(manifest_data.name, "My Web App");
        assert_eq!(manifest_data.short_name, "App");
        assert_eq!(manifest_data.start_url, ".");
        assert_eq!(manifest_data.display, "standalone");
        assert_eq!(manifest_data.background_color, "#ffffff");
        assert_eq!(manifest_data.description, "A cool web app");
        assert_eq!(manifest_data.icons.len(), 1);
        assert_eq!(manifest_data.icons[0].src, "app-icon.svg");
        assert_eq!(manifest_data.icons[0].sizes, "512x512");
        assert_eq!(
            manifest_data.icons[0].icon_type,
            Some("image/svg+xml".to_string())
        );
        assert_eq!(
            manifest_data.icons[0].purpose,
            Some("any maskable".to_string())
        );
        assert_eq!(manifest_data.orientation, "portrait-primary");
        assert_eq!(manifest_data.scope, "/");
        assert_eq!(manifest_data.theme_color, "rgb(#00aabb)");
    }

    /// Test case for creating `ManifestData` with missing metadata.
    #[test]
    fn test_create_manifest_data_with_missing_metadata() {
        let metadata = HashMap::new(); // Empty metadata

        let manifest_data = create_manifest_data(&metadata);

        assert_eq!(manifest_data.name, "");
        assert_eq!(manifest_data.short_name, "");
        assert_eq!(manifest_data.start_url, ".");
        assert_eq!(manifest_data.display, "standalone");
        assert_eq!(manifest_data.background_color, "#ffffff");
        assert_eq!(manifest_data.description, "");
        assert!(manifest_data.icons.is_empty());
        assert_eq!(manifest_data.orientation, "portrait-primary");
        assert_eq!(manifest_data.scope, "/");
        assert_eq!(manifest_data.theme_color, "");
    }

    /// Test case for creating `ManifestData` with invalid metadata types.
    #[test]
    fn test_create_manifest_data_with_invalid_metadata_types() {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "".to_string()); // Invalid type for name

        let manifest_data = create_manifest_data(&metadata);

        // Assert that default values are used for invalid metadata types
        assert_eq!(manifest_data.name, "");
        assert_eq!(manifest_data.short_name, "");
        assert_eq!(manifest_data.start_url, ".");
        assert_eq!(manifest_data.display, "standalone");
        assert_eq!(manifest_data.background_color, "#ffffff");
        assert_eq!(manifest_data.description, "");
        assert!(manifest_data.icons.is_empty());
        assert_eq!(manifest_data.orientation, "portrait-primary");
        assert_eq!(manifest_data.scope, "/");
        assert_eq!(manifest_data.theme_color, "");
    }
}
