// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::HumansData;
use std::collections::HashMap;

/// Function to create HumansData
pub fn create_human_data(metadata: &HashMap<String, String>) -> HumansData {
    HumansData {
        author: metadata.get("author").cloned().unwrap_or_default(),
        author_website: metadata.get("author_website").cloned().unwrap_or_default(),
        author_twitter: metadata.get("author_twitter").cloned().unwrap_or_default(),
        author_location: metadata.get("author_location").cloned().unwrap_or_default(),
        thanks: metadata.get("thanks").cloned().unwrap_or_default(),
        site_last_updated: metadata.get("site_last_updated").cloned().unwrap_or_default(),
        site_standards: metadata.get("site_standards").cloned().unwrap_or_default(),
        site_components: metadata.get("site_components").cloned().unwrap_or_default(),
        site_software: metadata.get("site_software").cloned().unwrap_or_default(),
    }
}

