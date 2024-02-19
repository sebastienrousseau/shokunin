// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    models::data::MetaTagGroups,
    modules::{
        frontmatter::extract, keywords::extract_keywords,
        metatags::generate_all_meta_tags,
    },
};
use std::collections::HashMap;

/// Extracts metadata from the content, generates keywords based on the metadata,
/// and prepares meta tag groups.
///
/// This function performs three key tasks:
/// 1. It extracts metadata from the front matter of the content.
/// 2. It extracts keywords based on this metadata.
/// 3. It generates various meta tags required for the page.
///
/// # Arguments
///
/// * `content` - A string slice representing the content from which to extract metadata.
///
/// # Returns
///
/// Returns a tuple containing:
/// * `HashMap<String, String>`: Extracted metadata
/// * `Vec<String>`: A list of keywords
/// * `MetaTagGroups`: A structure containing various meta tags
///
/// # Examples
///
/// ```rust
/// use ssg::models::data::FileData;
/// use ssg::modules::metadata::extract_and_prepare_metadata;
///
/// let file_content = "---\n\n# Front Matter (YAML)\n\nauthor: \"Jane Doe\"\ncategory: \"Rust\"\ndescription: \"A blog about Rust programming.\"\nlayout: \"post\"\npermalink: \"https://example.com/blog/rust\"\ntags: \"rust,programming\"\ntitle: \"Rust\"\n\n---\n\n# Content\n\nThis is a blog about Rust programming.\n";
///
/// let (metadata, keywords, all_meta_tags) = extract_and_prepare_metadata(&file_content);
/// ```
pub fn extract_and_prepare_metadata(
    content: &str,
) -> (HashMap<String, String>, Vec<String>, MetaTagGroups) {
    // Extract metadata from front matter
    let metadata = extract(content);

    // Extract keywords
    let keywords = extract_keywords(&metadata);

    // Generate all meta tags
    let all_meta_tags = generate_all_meta_tags(&metadata);

    (metadata, keywords, all_meta_tags)
}
