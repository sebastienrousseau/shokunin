// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
use regex::Regex;

use crate::utilities::{
    create_comrak_options, extract_front_matter,
    format_header_with_id_class, update_class_attributes,
};
use std::collections::HashMap;

/// ## Function: `generate_html` - Generates an HTML page from Markdown
///
/// Generates an HTML page from the given Markdown content, title, and
/// description.
///
/// This function takes in a Markdown string `content`, as well as a
/// `title` and `description` string to use for the HTML page. The
/// function converts the Markdown content to HTML using the Comrak
/// library, and adds a header and subheader to the HTML page using the
/// title and description strings, respectively.
///
/// If `content` begins with front matter (denoted by "---\n"), the
/// front matter is skipped and only the Markdown content below it is
/// used to generate the HTML. If `title` or `description` are empty
/// strings, they are not included in the generated HTML page.
///
/// The resulting HTML page is returned as a string.
///
/// # Examples
///
/// ```
///
/// use ssg::html::generate_html;
///
/// let content = "## Hello, world!\n\nThis is a test.";
/// let title = "My Page";
/// let description = "This is a test page";
/// let html = generate_html(content, title, description, None);
/// assert_eq!(
///     html,
///     "<h1 id=\"h1-my\" id=\"\" id=\"h1-my\" class=\"my\">My Page</h1 id=\"h1-my\" class=\"my\"><p>This is a test page</p><h2 id=\"h2-hello\" class=\"hello\">Hello, world!</h2 id=\"h2-hello\" class=\"hello\">\n<p>This is a test.</p>\n"
/// );
///
/// ```
//
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
    json_content: Option<&str>,
) -> String {
    // Extract the front matter from the content string.
    let markdown_content = extract_front_matter(content);

    // Regex to remove non-alphanumeric characters from IDs.
    let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();

    // Create the header and subheader for the HTML page.
    let header = if !title.is_empty() {
        let header_str = format!("<h1 id=\"\">{}</h1>", title);
        format_header_with_id_class(&header_str, &id_regex)
    } else {
        String::new()
    };

    // Create the subheader for the HTML page.
    let description = if !description.is_empty() {
        let description_str = format!("<p>{}</p>", description);
        description_str
    } else {
        String::new()
    };

    // Create a new HashMap to store the HTML content.
    let mut html_map = HashMap::new();

    // Create a new Comrak options object.
    let options = create_comrak_options();

    // Preprocess the content to extract and move the class attributes
    let class_regex =
        Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*)(>)").unwrap();
    let processed_content: Vec<String> = content
        .lines()
        .map(|line| {
            update_class_attributes(line, &class_regex, &img_regex)
        })
        .collect();

    // Render the Markdown content to HTML:
    let markdown_html = comrak::markdown_to_html(
        &processed_content.join("\n"),
        &options,
    );

    let class_regex =
        Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)").unwrap();
    let processed_html: Vec<String> = markdown_html
        .lines()
        .map(|line| {
            update_class_attributes(line, &class_regex, &img_regex)
        })
        .collect();

    // Add the processed HTML content to the HashMap:
    html_map.insert(title.to_string(), processed_html.join("\n"));

    // Render the Markdown content to HTML:
    let markdown_html =
        comrak::markdown_to_html(markdown_content, &options);

    let class_regex =
        Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)").unwrap();

    let mut processed_html = String::new();
    for line in markdown_html.lines() {
        if line.contains(".class=&quot;") {
            let captures = class_regex.captures(line).unwrap();
            let class_value = captures.get(1).unwrap().as_str();
            let updated_line = class_regex.replace(line, "");
            let updated_line_with_class = img_regex.replace(
                &updated_line,
                &format!("$1 class=\"{}\"$2", class_value),
            );
            processed_html.push_str(&updated_line_with_class);
            processed_html.push('\n');
        } else {
            processed_html.push_str(line);
            processed_html.push('\n');
        }
    }

    // Add the processed HTML content to the HashMap:
    html_map.insert(title.to_string(), processed_html);

    let html_string = html_map
    .values()
    .map(|content| {
        let header_tags = vec!["h1", "h2", "h3", "h4", "h5", "h6"];
        let mut result = content.to_string();

        for tag in header_tags {
            let re = Regex::new(&format!("<{}>([^<]+)</{}>", tag, tag)).unwrap();
            let mut replacements: Vec<(String, String)> = Vec::new();

            for cap in re.captures_iter(&result) {
                let original = cap[0].to_string();
                let replacement = format_header_with_id_class(&original, &re);
                replacements.push((original, replacement));
            }

            for (original, replacement) in replacements {
                result = result.replace(&original, &replacement);
            }
        }
        result
    }).collect::<Vec<String>>().join("\n");

    // println!("html_string={:?}", html_string);

    let json_html = if let Some(json_str) = json_content {
        format!("<p>{}</p>", json_str)
    } else {
        "".to_string()
    };
    format!("{}{}{}{}", header, description, json_html, html_string)
}
