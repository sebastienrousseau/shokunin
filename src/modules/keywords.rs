// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

/// Extracts keywords from the metadata and returns them as a vector of strings.
///
/// This function takes a reference to a HashMap containing metadata key-value pairs.
/// It looks for the "keywords" key in the metadata and extracts the keywords from its value.
/// Keywords are expected to be comma-separated. The extracted keywords are trimmed of any
/// leading or trailing whitespace, and returned as a vector of strings.
///
/// # Arguments
///
/// * `metadata` - A reference to a HashMap containing metadata.
///
/// # Returns
///
/// A vector of strings representing the extracted keywords.
///
pub fn extract_keywords(metadata: &HashMap<String, String>) -> Vec<String> {
    // Check if the "keywords" key exists in the metadata.
    // If it exists, split the keywords using a comma and process each keyword.
    // If it doesn't exist, return an empty vector as the default value.
    metadata
        .get("keywords")
        .map(|keywords| {
            // Split the keywords using commas and process each keyword.
            keywords
                .split(',')
                .map(|kw| kw.trim().to_string())  // Trim whitespace from each keyword.
                .collect::<Vec<_>>()  // Collect the processed keywords into a vector.
        })
        .unwrap_or_default()  // Return an empty vector if "keywords" is not found.
}
