// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import models::data::TxtData
use crate::models::data::TxtData;

// Import std::collections::HashMap
use std::collections::HashMap;

/// Function to create TxtData
///
/// The `metadata` parameter is a map of metadata strings.
///
/// Returns a `TxtData` object.
pub fn create_txt_data(metadata: &HashMap<String, String>) -> TxtData {
    TxtData {
        permalink: metadata
            .get("permalink")
            .cloned()
            .unwrap_or_default(),
    }
}
