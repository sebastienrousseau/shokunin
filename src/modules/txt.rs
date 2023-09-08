// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::TxtData;
use std::collections::HashMap;

/// Function to create TxtData
pub fn create_txt_data(metadata: &HashMap<String, String>) -> TxtData {
    TxtData {
        permalink: metadata.get("permalink").cloned().unwrap_or_default(),
    }
}

