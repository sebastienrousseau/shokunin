// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    path::Path,
};

use ssg::models::data::{FileData, PageData, TagsData};

/// Generates a tag list from the given `FileData` and metadata, and returns it as a `HashMap`.
pub fn generate_tags(
    file: &FileData,
    metadata: &HashMap<String, String>,
) -> HashMap<String, Vec<HashMap<String, String>>> {
    let mut keywords_data_map: HashMap<
        String,
        Vec<HashMap<String, String>>,
    > = HashMap::new();
    let file_content = &file.content;

    // Extract target tags from metadata if available.
    let default_tags = String::from("");
    let target_tags: Vec<&str> = metadata
        .get("tags")
        .unwrap_or(&default_tags)
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if target_tags.is_empty() {
        println!("No tags found in metadata.");
        return keywords_data_map;
    }

    for tag in &target_tags {
        if file_content.contains(tag) {
            let mut tags_data = HashMap::new();

            // Extract metadata for the tag
            let metadata_keys = [
                "title",
                "date",
                "description",
                "permalink",
                "keywords",
            ];
            for key in &metadata_keys {
                if let Some(value) = metadata.get(*key) {
                    tags_data.insert((*key).to_string(), value.clone());
                }
            }

            // Insert or update the entry in keywords_data_map
            keywords_data_map
                .entry((*tag).to_string())
                .or_default()
                .push(tags_data);
        }
    }
    keywords_data_map
}

/// Creates a `TagsData` struct populated with metadata.
pub fn create_tags_data(
    metadata: &HashMap<String, String>,
) -> TagsData {
    TagsData {
        dates: metadata.get("date").cloned().unwrap_or_default(),
        descriptions: metadata
            .get("description")
            .cloned()
            .unwrap_or_default(),
        keywords: metadata.get("keywords").cloned().unwrap_or_default(),
        permalinks: metadata
            .get("permalink")
            .cloned()
            .unwrap_or_default(),
        titles: metadata.get("title").cloned().unwrap_or_default(),
    }
}

/// Generates the HTML content for displaying tags and their associated pages.
pub fn generate_tags_html(
    global_tags_data: &HashMap<String, Vec<PageData>>,
) -> String {
    let mut html_content = String::new();

    // Create a sorted Vec of keys
    let mut keys: Vec<&String> = global_tags_data.keys().collect();
    keys.sort();

    // First, calculate the total number of posts
    let total_posts: usize =
        global_tags_data.values().map(|pages| pages.len()).sum();

    // Add an h2 element for the total number of posts
    html_content.push_str(&format!(
        "<h2 class=\"featured-tags\" id=\"h2-featured-tags\" tabindex=\"0\">Featured Tags ({})</h2>",
        total_posts
    ));

    // Loop through each tag and its associated pages
    for key in keys {
        let tag = key;
        let pages = &global_tags_data[key];
        let count = pages.len();
        html_content.push_str(&format!(
            "<h3 class=\"{}\" id=\"h3-{}\" tabindex=\"0\">{} ({} Posts)</h3>\n<ul>",
            tag.replace(' ', "-"),
            tag.replace(' ', "-"),
            to_title_case(tag),
            count
        ));
        for page in pages.iter() {
            html_content.push_str(&format!(
                "<li>{}: <a href=\"{}\">{}</a> - <strong>{}</strong></li>\n",
                page.date, page.permalink, page.title, page.description
            ));
        }
        html_content.push_str("</ul>\n");
    }

    html_content
}

/// Writes the given HTML content into an existing `index.html` file, replacing a placeholder.
pub fn write_tags_html_to_file(
    html_content: &str,
    output_path: &Path,
) -> io::Result<()> {
    // Define the file path for the output
    let file_path = output_path.join("tags/index.html");

    // Read the existing HTML content from the file
    let mut file = fs::File::open(&file_path)?;
    let mut base_html = String::new();
    file.read_to_string(&mut base_html)?;

    // Replace [[content]] with the generated HTML content
    base_html = base_html.replace("[[content]]", html_content);

    // Write the modified HTML content back to the file
    let mut file = fs::File::create(&file_path)?;
    file.write_all(base_html.as_bytes())?;

    Ok(())
}

/// Helper function to convert a string to title case
fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == ' ' {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tags_with_tags_and_metadata() {
        let file_data = FileData {
            content: "This is a test with tags: tag1, tag2".to_string(),
            ..Default::default()
        };
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), "tag1, tag2".to_string());
        metadata.insert("title".to_string(), "Test Page".to_string());

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 2);

        let tag1_data = tags_data_map.get("tag1").unwrap();
        assert_eq!(tag1_data.len(), 1);
        let tag1_entry = &tag1_data[0];
        assert_eq!(tag1_entry["title"], "Test Page");

        let tag2_data = tags_data_map.get("tag2").unwrap();
        assert_eq!(tag2_data.len(), 1);
        let tag2_entry = &tag2_data[0];
        assert_eq!(tag2_entry["title"], "Test Page");
    }

    #[test]
    fn test_generate_tags_with_no_tags_in_metadata() {
        let file_data = FileData {
            content: "This is a test".to_string(),
            ..Default::default()
        };
        let metadata = HashMap::new(); // No tags in metadata

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 0);
    }

    #[test]
    fn test_create_tags_data_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("date".to_string(), "2021-09-04".to_string());
        metadata.insert(
            "description".to_string(),
            "A sample description".to_string(),
        );

        let tags_data = create_tags_data(&metadata);

        assert_eq!(tags_data.dates, "2021-09-04");
        assert_eq!(tags_data.descriptions, "A sample description");
        assert_eq!(tags_data.titles, "");
        assert_eq!(tags_data.permalinks, "");
        assert_eq!(tags_data.keywords, "");
    }

    #[test]
    fn test_generate_tags_html_with_data() {
        let mut global_tags_data = HashMap::new();
        global_tags_data.insert(
            "tag1".to_string(),
            vec![PageData {
                date: "2022-09-01".to_string(),
                description: "Description 1".to_string(),
                permalink: "/page1".to_string(),
                title: "Page 1".to_string(),
            }],
        );

        let global_tags_data = generate_tags_html(&global_tags_data);

        assert!(global_tags_data.contains("<h2 class=\"featured-tags\" id=\"h2-featured-tags\" tabindex=\"0\">Featured Tags (1)</h2>"));
        assert!(global_tags_data.contains("<h3 class=\"tag1\" id=\"h3-tag1\" tabindex=\"0\">Tag1 (1 Posts)</h3>"));
        assert!(global_tags_data.contains("<li>2022-09-01: <a href=\"/page1\">Page 1</a> - <strong>Description 1</strong></li>"));
    }

    #[test]
    fn test_generate_tags_metadata_missing_tags_key() {
        // Test case for when metadata is missing the "tags" key
        let file_data = FileData {
            content: "This is a test with tags: tag1, tag2".to_string(),
            ..Default::default()
        };
        let metadata = HashMap::new(); // No tags in metadata

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 0);
    }

    #[test]
    fn test_generate_tags_tags_value_empty_string() {
        // Test case for when the "tags" value is an empty string
        let file_data = FileData {
            content: "This is a test".to_string(),
            ..Default::default()
        };
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), "".to_string());

        let tags_data_map = generate_tags(&file_data, &metadata);

        assert_eq!(tags_data_map.len(), 0);
    }

    #[test]
    fn test_generate_tags_long_tag_values_close_to_capacity_limits() {
        // Create a sample file and metadata
        let file = FileData {
            content: "This is a test".to_string(),
            ..Default::default()
        };
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), "tag1,tag2".to_string());
        // Insert very long values for metadata
        metadata.insert(
            "title".to_string(),
            "a".repeat(1000), /* very long value */
        );
        metadata.insert(
            "date".to_string(),
            "a".repeat(1000), /* very long value */
        );
        metadata.insert(
            "description".to_string(),
            "a".repeat(1000), /* very long value */
        );
        metadata.insert(
            "permalink".to_string(),
            "a".repeat(1000), /* very long value */
        );
        metadata.insert(
            "keywords".to_string(),
            "a".repeat(1000), /* very long value */
        );

        // Call the function under test
        let result = generate_tags(&file, &metadata);

        // Assert that the result is as expected
        assert_eq!(result.len(), 0); // No tags should be generated
    }

    #[test]
    fn test_generate_tags_tag_contains_special_characters() {
        // Test case for tag containing special characters like "!" or emojis
        let special_tag = "special!tag🙂";
        let file_data = FileData {
            content: format!(
                "This is a test with a special tag: {}",
                special_tag
            ),
            ..Default::default()
        };
        let mut metadata = HashMap::new();
        metadata.insert("tags".to_string(), special_tag.to_string());

        let tags_data_map = generate_tags(&file_data, &metadata);

        // Expecting the tags_data_map to contain the special tag
        assert_eq!(tags_data_map.len(), 1);
        assert!(tags_data_map.contains_key(special_tag));
    }

    #[test]
    fn test_generate_tags_extremely_large_number_of_tags() {
        // Test case for an extremely large number of tags
        let mut metadata = HashMap::new();
        let tags: Vec<String> =
            (1..=1000).map(|i| format!("tag{}", i)).collect();
        let file_data = FileData {
            content: "This is a test".to_string(),
            ..Default::default()
        };
        // Join tags with commas to form a metadata string
        metadata.insert("tags".to_string(), tags.join(", "));

        let tags_data_map = generate_tags(&file_data, &metadata);

        // Count the number of unique tags in the tags_data_map
        let unique_tags_count = tags_data_map.len();

        // Expecting the number of unique tags to be less than or equal to 1000
        assert!(unique_tags_count <= 1000);
    }
}
