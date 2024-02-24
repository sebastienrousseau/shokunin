// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{modules::{markdown::convert_markdown_to_html, postprocessor::post_process_html}, utilities::directory::{
    create_comrak_options, extract_front_matter,
    format_header_with_id_class, update_class_attributes,
}};
use regex::Regex;
use std::error::Error;

/// Generates an HTML page from Markdown content, title, and description.
///
/// # Arguments
///
/// * `content` - A string slice containing the Markdown content.
/// * `title` - A string slice containing the title of the HTML page.
/// * `description` - A string slice containing the description of the HTML page.
/// * `json_content` - An optional string slice containing JSON content to be included in the HTML.
///
/// # Returns
///
/// A `Result` containing a `String` representing the generated HTML page if successful, or a `Box<dyn Error>` if an error occurs.
///
/// # Examples
///
/// ```rust
/// use ssg::modules::html::generate_html;
/// use ssg::modules::postprocessor::post_process_html;
///
/// let content = "## Hello, world!\n\nThis is a test.";
/// let title = "My Page";
/// let description = "This is a test page";
/// let html = generate_html(content, title, description, None);
/// let html_str = html.unwrap_or_else(|e| panic!("Error: {:?}", e));
///
/// assert_eq!(html_str, "<h1 id=\"h1-my\" tabindex=\"0\" aria-label=\"My Heading\" itemprop=\"headline\" class=\"my\">My Page</h1><p>This is a test page</p><h2 id=\"h2-hello\" tabindex=\"0\" aria-label=\"Hello Heading\" itemprop=\"name\" class=\"hello\">Hello, world!</h2>\n<p>This is a test.</p>\n");
/// ```
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
    json_content: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    // Regex patterns for ID, class, and image tags
    let id_regex = Regex::new(r"[^a-zA-Z0-9]+")?;
    let class_regex = Regex::new(r#"\.class=&quot;([^&"]+)&quot;"#)?;
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)")?;

    // Extract front matter from content
    let markdown_content = extract_front_matter(content);
    // Preprocess content to update class attributes and image tags
    let processed_content =
        preprocess_content(markdown_content, &class_regex, &img_regex)?;

    // Convert Markdown to HTML
    let options = create_comrak_options();
    let markdown_html =
        convert_markdown_to_html(&processed_content, &options)?;

    // Post-process HTML content
    let processed_html =
        post_process_html(&markdown_html, &class_regex, &img_regex)?;

    // Generate header and description
    let header = generate_header(title, &id_regex);
    let desc = generate_description(description);

    // Process headers in HTML
    let header_tags = vec!["h1", "h2", "h3", "h4", "h5", "h6"];
    let html_string = process_headers(&processed_html, &header_tags, &id_regex);

    // Construct the final HTML with JSON content if available
    let json_html = json_content.map_or_else(
        || "".to_string(),
        |json_str| format!("<p>{}</p>", json_str),
    );

    Ok(format!("{}{}{}{}", header, desc, json_html, html_string))
}

fn process_headers(
    html: &str,
    header_tags: &[&str],
    _id_regex: &Regex,
) -> String {
    let mut html_string = html.to_string();
    for tag in header_tags {
        let re = Regex::new(&format!("<{}>([^<]+)</{}>", tag, tag)).unwrap();
        let mut replacements: Vec<(String, String)> = Vec::new();

        for cap in re.captures_iter(&html_string) {
            let original = cap[0].to_string();
            let replacement = format_header_with_id_class(&original, &re);
            replacements.push((original, replacement));
        }

        for (original, replacement) in replacements {
            html_string = html_string.replace(&original, &replacement);
        }
    }
    html_string
}

/// Generate header HTML string based on title
///
/// # Arguments
///
/// * `title` - A string slice that holds the title to be included in the header tag.
/// * `id_regex` - A reference to a compiled Regex object used for processing the header string.
///
/// # Returns
///
/// A `String` that represents the HTML <h1> header tag with the title.
///
/// # Examples
///
/// ```rust
/// use regex::Regex;
/// use ssg::modules::html::generate_header;
/// let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
/// let header_html = generate_header("My Page Title", &id_regex);
/// assert_eq!(header_html, "<h1 id=\"h1-my\" tabindex=\"0\" aria-label=\"My Heading\" itemprop=\"headline\" class=\"my\">My Page Title</h1>");
/// ```
pub fn generate_header(title: &str, id_regex: &Regex) -> String {
    if title.is_empty() {
        return String::new();
    }

    let header_str = format!("<h1>{}</h1>", title);
    format_header_with_id_class(&header_str, id_regex)
}

/// Generate description HTML string based on description
fn generate_description(description: &str) -> String {
    if description.is_empty() {
        return String::new();
    }
    format!("<p>{}</p>", description)
}

/// Preprocesses the HTML content to update class attributes and image tags.
///
/// # Arguments
///
/// * `content` - A string containing the HTML content to be processed.
/// * `class_regex` - A reference to a `Regex` object for matching class attributes.
/// * `img_regex` - A reference to a `Regex` object for matching image tags.
///
/// # Returns
///
/// A `Result` containing a `String` with the processed HTML content, or a `Box<dyn Error>` if an error occurs.
///
/// # Example
///
/// ```rust
/// use regex::Regex;
/// use std::error::Error;
/// use ssg::modules::html::preprocess_content;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let content = "<div class=\"some-class\">...</div>";
///     let class_regex = Regex::new(r#".class="([^"]+)""#)?;
///     let img_regex = Regex::new(r#"<img([^>]+)>"#)?;
///
///     let processed_content = preprocess_content(content, &class_regex, &img_regex)?;
///     println!("{}", processed_content);
///
///     Ok(())
/// }
/// ```
pub fn preprocess_content(
    content: &str,
    class_regex: &Regex,
    img_regex: &Regex,
) -> Result<String, Box<dyn Error>> {
    let processed_content: Vec<String> = content
        .lines()
        .map(|line| {
            update_class_attributes(line, class_regex, img_regex)
        })
        .collect();
    // println!("{}", processed_content.join("\n"));
    Ok(processed_content.join("\n"))
}
