// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import models::data::SiteMapData
use crate::models::data::SiteMapData;

// Import std::collections::HashMap
use std::collections::HashMap;

/// Function to create SiteMapData
///
/// The `metadata` parameter is a map of metadata strings.
///
/// Returns a `SiteMapData` object.
pub fn create_site_map_data(metadata: &HashMap<String, String>) -> SiteMapData {
    SiteMapData {
        // The change frequency of the website.
        changefreq: metadata.get("changefreq").cloned().unwrap_or_default(),

        // The last modified date of the website.
        lastmod: metadata.get("last_build_date").cloned().unwrap_or_default(),

        // The base URL of the website.
        loc: metadata.get("permalink").cloned().unwrap_or_default(),
    }
}
