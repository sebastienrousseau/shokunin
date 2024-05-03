// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    macro_metadata_option,
    models::data::{IconData, ManifestData},
};
use std::collections::HashMap;

/// Creates a `ManifestData` object based on the provided metadata.
///
/// # Arguments
///
/// * `metadata` - A map of metadata strings.
///
/// # Returns
///
/// A `ManifestData` object.
pub fn create_manifest_data(
    metadata: &HashMap<String, String>,
) -> ManifestData {
    ManifestData {
        name: metadata
            .get("name")
            .map_or_else(|| "".to_string(), |v| v.to_string()),
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
