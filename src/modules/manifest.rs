// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{models::data::{IconData, ManifestData}, macro_metadata_option};
use std::collections::HashMap;

/// Function to create ManifestData
///
/// The `metadata` parameter is a map of metadata strings.
///
/// Returns a `ManifestData` object.
pub fn create_manifest_data(metadata: &HashMap<String, String>) -> ManifestData {
    ManifestData {
        name: metadata.get("name").cloned().unwrap_or_default(),
        short_name: macro_metadata_option!(metadata, "short_name"),
        start_url: ".".to_string(),
        display: "standalone".to_string(),
        background_color: "#ffffff".to_string(),
        description: macro_metadata_option!(metadata, "description"),
        icons: metadata.get("icon")
            .map(|icon| vec![
                IconData {
                    src: icon.to_string(),
                    sizes: "512x512".to_string(),
                    icon_type: Some("image/svg+xml".to_string()),
                    purpose: Some("any maskable".to_string()),
                }
            ])
            .unwrap_or_default(),
        orientation: "portrait-primary".to_string(),
        scope: "/".to_string(),
        theme_color: macro_metadata_option!(metadata, "theme-color"),
    }
}

