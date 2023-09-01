// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::data::{FileData, TagsData};
use regex::Regex;
use std::collections::HashMap;

/// Generates a tag list from the given `FileData` structs and prints it.
///
/// # Arguments
///
/// * `files` - A list of `FileData` structs, each of which contains the data
/// for a Markdown file.
///
/// # Returns
///
/// None.
///
/// # Example
///
/// ```
/// use ssg::data::FileData;
/// use ssg::tags::generate_tags;
/// let files = vec![FileData { content: "This is a test".to_string(), ..Default::default() }];
///
/// generate_tags(&files, &["test"]);
/// ```
///
pub fn generate_tags(files: &[FileData], target_tags: &[&str]) -> HashMap<String, Vec<HashMap<String, String>>> {
    let title_regex = Regex::new(r"<title>([^<]+)</title>").unwrap();
    let meta_regex = Regex::new(r#"<meta\s+content="([^"]+)"\s+name="([^"]+)">"#).unwrap();

    let mut keywords_data_map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

    for file in files {
        let file_content = &file.content;

        for tag in target_tags {
            if file_content.contains(tag) {
                let mut tags_data = HashMap::new();

                // Extract the title from the file content.
                let title = title_regex.captures(file_content)
                    .map(|caps| caps.get(1).unwrap().as_str())
                    .unwrap_or("");
                tags_data.insert("title".to_string(), title.to_string());

                // Extract the meta tags from the file content.
                for capture in meta_regex.captures_iter(file_content) {
                    let content = capture.get(1).unwrap().as_str();
                    let name = capture.get(2).unwrap().as_str();

                    // Match the name of the meta tag and add the content to the tags_data HashMap.
                    match name {
                        "description" => tags_data.insert("description".to_string(), content.to_string()),
                        "permalink" => tags_data.insert("permalink".to_string(), content.to_string()),
                        "keywords" => tags_data.insert("keywords".to_string(), content.to_string()),
                        _ => None,
                    };
                }

                // Insert or update the entry in keywords_data_map.
                keywords_data_map.entry(tag.to_string()).or_insert_with(Vec::new).push(tags_data);
            }
        }
    }
    keywords_data_map
}

/// Function to create TagsData
pub fn create_tags_data(metadata: &HashMap<String, String>) -> TagsData {
    TagsData {
        titles: metadata.get("title").cloned().unwrap_or_default(),
        descriptions: metadata.get("description").cloned().unwrap_or_default(),
        permalinks: metadata.get("permalink").cloned().unwrap_or_default(),
        keywords: metadata.get("keywords").cloned().unwrap_or_default(),
    }
}


