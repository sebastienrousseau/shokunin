// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import the HumansData struct from the data module.
use crate::models::data::HumansData;

// Import the HashMap struct from the collections module.
use std::collections::HashMap;

/// Function to create a HumansData object from a HashMap of metadata.
///
/// The `metadata` HashMap must contain the following keys:
///
/// * `author_location`: The location of the author.
/// * `author_twitter`: The Twitter handle of the author.
/// * `author_website`: The website of the author.
/// * `author`: The name of the author of the website or blog.
/// * `site_components`: The components that the website or blog uses.
/// * `site_last_updated`: The date on which the website or blog was last updated.
/// * `site_software`: The software that the website or blog uses.
/// * `site_standards`: The standards that the website or blog follows.
/// * `thanks`: A list of people or organizations to thank for their contributions to the website or blog.
///
/// Returns a HumansData object with the metadata.
pub fn create_human_data(metadata: &HashMap<String, String>) -> HumansData {
    HumansData {
        author_location: metadata.get("author_location").cloned().unwrap_or_default(),
        author_twitter: metadata.get("author_twitter").cloned().unwrap_or_default(),
        author_website: metadata.get("author_website").cloned().unwrap_or_default(),
        author: metadata.get("author").cloned().unwrap_or_default(),
        site_components: metadata.get("site_components").cloned().unwrap_or_default(),
        site_last_updated: metadata.get("site_last_updated").cloned().unwrap_or_default(),
        site_software: metadata.get("site_software").cloned().unwrap_or_default(),
        site_standards: metadata.get("site_standards").cloned().unwrap_or_default(),
        thanks: metadata.get("thanks").cloned().unwrap_or_default(),
    }
}

