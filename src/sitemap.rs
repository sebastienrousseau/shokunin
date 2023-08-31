// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::SiteMapData;
use std::collections::HashMap;

/// Function to create SiteMapData
pub fn create_site_map_data(metadata: &HashMap<String, String>) -> SiteMapData {
    SiteMapData {
        loc: metadata.get("permalink").cloned().unwrap_or_default(),
        lastmod: metadata.get("last_build_date").cloned().unwrap_or_default(),
        changefreq: metadata.get("weekly").cloned().unwrap_or_default(),
    }
}

