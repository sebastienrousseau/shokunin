// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::models::data::{FileData, TagsData, PageData};
use crate::utilities::directory::to_title_case;
use std::{io::{Read, Write}, collections::HashMap, path::Path, fs};

/// Generates a tag list from the given `FileData` and metadata, and returns it as a `HashMap`.
///
/// # Arguments
///
/// * `file` - A reference to a `FileData` struct, which contains the content of a single file.
/// * `metadata` - A reference to a `HashMap` containing metadata like tags, title, etc.
///
/// # Returns
///
/// A `HashMap` mapping each tag to a vector of `HashMap`s containing associated data like title, description, etc.
///
/// # Example
///
/// ```rust
/// use ssg::models::data::FileData;
/// use ssg::modules::tags::generate_tags;
/// use std::collections::HashMap;
///
/// let file = FileData { content: "This is a test".to_string(), ..Default::default() };
/// let mut metadata = HashMap::new();
/// metadata.insert("tags".to_string(), "tag1,tag2".to_string());
///
/// let result = generate_tags(&file, &metadata);
/// ```
///
pub fn generate_tags(file: &FileData, metadata: &HashMap<String, String>) -> HashMap<String, Vec<HashMap<String, String>>> {
    let mut keywords_data_map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
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

            // Extract title from metadata.
            let title = metadata.get("title").cloned().unwrap_or_else(|| {
                println!("Failed to extract title for tag: {}", tag);
                String::new()
            });
            tags_data.insert("title".to_string(), title);

            let dates = metadata.get("date").cloned().unwrap_or_else(|| {
                println!("Failed to extract date for tag: {}", tag);
                String::new()
            });
            tags_data.insert("date".to_string(), dates);

            // Extract description from metadata.
            let description = metadata.get("description").cloned().unwrap_or_else(|| {
                println!("Failed to extract description for tag: {}", tag);
                String::new()
            });
            tags_data.insert("description".to_string(), description);

            // Extract permalink from metadata.
            let permalink = metadata.get("permalink").cloned().unwrap_or_else(|| {
                println!("Failed to extract permalink for tag: {}", tag);
                String::new()
            });
            tags_data.insert("permalink".to_string(), permalink);

            // Extract keywords from metadata.
            let keywords = metadata.get("keywords").cloned().unwrap_or_else(|| {
                println!("Failed to extract keywords for tag: {}", tag);
                String::new()
            });
            tags_data.insert("keywords".to_string(), keywords);


            // Insert or update the entry in keywords_data_map.
            keywords_data_map.entry(tag.to_string()).or_default().push(tags_data);
        }
    }
    keywords_data_map
}

/// Creates a `TagsData` struct populated with metadata.
///
/// This function takes a reference to a `HashMap` containing metadata, such as
/// dates, descriptions, keywords, permalinks, and titles. It then constructs and returns a `TagsData`
/// struct populated with this metadata.
///
/// # Arguments
///
/// * `metadata` - A reference to a `HashMap` containing the metadata for the tags.
///
/// # Returns
///
/// Returns a `TagsData` struct populated with the metadata.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use ssg::modules::tags::create_tags_data;
///
/// let mut metadata = HashMap::new();
/// metadata.insert("date".to_string(), "2021-09-04".to_string());
/// metadata.insert("description".to_string(), "A sample description".to_string());
/// let tags_data = create_tags_data(&metadata);
/// ```
///
pub fn create_tags_data(
    metadata: &HashMap<String, String>,
) -> TagsData {
    let dates = metadata.get("date").cloned().unwrap_or_default();
    let descriptions = metadata.get("description").cloned().unwrap_or_default();
    let keywords = metadata.get("keywords").cloned().unwrap_or_default();
    let permalinks = metadata.get("permalink").cloned().unwrap_or_default();
    let titles = metadata.get("title").cloned().unwrap_or_default();

    TagsData {
        dates,
        titles,
        descriptions,
        permalinks,
        keywords,
    }
}


/// Generates the HTML content for displaying tags and their associated pages.
///
/// This function takes a `HashMap` that maps tags to a list of `PageData` objects,
/// where each `PageData` object contains metadata about a page that uses the tag.
/// It then generates HTML content that displays these tags along with the pages that use them.
///
/// # Arguments
///
/// * `global_tags_data` - A reference to a `HashMap` where each key is a tag
///   and the corresponding value is a `Vec` of `PageData` objects that use the tag.
///
/// # Returns
///
/// A `String` containing the generated HTML content.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use ssg::models::data::PageData;
/// use ssg::modules::tags::generate_tags_html;
///
/// let mut global_tags_data = HashMap::new();
/// global_tags_data.insert(
///     "tag1".to_string(),
///     vec![
///         PageData {
///             date: "2022-09-01".to_string(),
///             description: "Description 1".to_string(),
///             permalink: "/page1".to_string(),
///             title: "Page 1".to_string(),
///         },
///     ],
/// );
///
/// global_tags_data.insert(
///     "tag2".to_string(),
///     vec![
///         PageData {
///             date: "2022-09-02".to_string(),
///             description: "Description 2".to_string(),
///             permalink: "/page2".to_string(),
///             title: "Page 2".to_string(),
///         },
///     ],
/// );
///
/// let html_content = generate_tags_html(&global_tags_data);
/// ```
///
pub fn generate_tags_html(global_tags_data: &HashMap<String, Vec<PageData>>) -> String {

    let mut html_content = String::new();

    // Create a sorted Vec of keys
    let mut keys: Vec<&String> = global_tags_data.keys().collect();
    keys.sort();

    // First, calculate the total number of posts
    let total_posts: usize = global_tags_data.values().map(|pages| pages.len()).sum();

    // Add an h2 element for the total number of posts
    html_content.push_str(&format!("<h2 class=\"featured-tags\" id=\"h2-featured-tags\" tabindex=\"0\">Featured Tags ({})</h2>", total_posts));

    // Existing loop code for each tag
    for key in keys {
        let tag = key;
        let pages = &global_tags_data[key];
        let count = pages.len();
        html_content.push_str(&format!("<h3 class=\"{}\" id=\"h3-{}\" tabindex=\"0\">{} ({} Posts)</h3>\n<ul>", tag.replace(' ', "-"), tag.replace(' ', "-"), to_title_case(tag), count));
        for page in pages.iter() {
            html_content.push_str(&format!("<li>{}: <a href=\"{}\">{}</a> - <strong>{}</strong></li>\n", page.date, page.permalink, page.title, page.description));
        }
        html_content.push_str("</ul>\n");
    }

    html_content

}


/// Writes the given HTML content into an existing `index.html` file, replacing a placeholder.
///
/// This function takes in the generated HTML content and a path to the output directory.
/// It then reads an existing `index.html` file located in `tags/` sub-directory of the given output directory,
/// replaces a `[[content]]` placeholder with the given HTML content, and writes it back to the file.
///
/// # Arguments
///
/// * `html_content` - The generated HTML content to be written.
/// * `output_path` - The path to the output directory where the `index.html` file is located.
///
/// # Returns
///
/// Returns an `std::io::Result<()>` which is `Ok` if the operation was successful.
/// Any IO error that occurs will be propagated in the `Err` variant of the result.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use ssg::modules::tags::write_tags_html_to_file;
///
/// let html_content = "<h1>Hello World</h1>";
/// let output_path = Path::new("/path/to/output");
/// write_tags_html_to_file(html_content, &output_path);
/// ```
///
pub fn write_tags_html_to_file(html_content: &str, output_path: &Path) -> std::io::Result<()> {

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