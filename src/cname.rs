// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::CnameData;
use std::collections::HashMap;

/// Function to create CnameData
pub fn create_cname_data(metadata: &HashMap<String, String>) -> CnameData {
    CnameData {
        cname: metadata.get("cname").cloned().unwrap_or_default(),
    }
}

