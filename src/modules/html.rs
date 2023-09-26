// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
use regex::Regex;
use std::{error::Error, collections::HashMap};
use comrak::{ComrakOptions, markdown_to_html};
use crate::utilities::directory::{
    create_comrak_options, extract_front_matter,
    format_header_with_id_class, update_class_attributes,
};


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
/// use ssg::modules::html::generate_html;
///
/// let content = "## Hello, world!\n\nThis is a test.";
/// let title = "My Page";
/// let description = "This is a test page";
/// let html = generate_html(content, title, description, None);
/// let html_str = html.unwrap_or_else(|e| panic!("Error: {:?}", e));
///
/// assert_eq!(html_str, "<h1 id=\"h1-my\" id=\"\" tabindex=\"0\" class=\"my\" itemprop=\"headline\">My Page</h1><p>This is a test page</p><h2 id=\"h2-hello\" tabindex=\"0\" class=\"hello\">Hello, world!</h2>\n<p>This is a test.</p>\n");
///
/// ```
//
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
    json_content: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    let class_regex = Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)").unwrap();

    // 1. Preprocess the content
    let markdown_content = extract_front_matter(content);
    let processed_content = preprocess_content(markdown_content, &class_regex, &img_regex).unwrap();
      // Extract the front matter from the content string.
    //   let markdown_content = extract_front_matter(content);

    // 2. Convert Markdown to HTML
    let options = create_comrak_options();
    let markdown_html = convert_markdown_to_html(&processed_content, &options);

    // 3. Post-process the HTML
    let processed_html = post_process_html(&markdown_html, &class_regex, &img_regex).unwrap();

    // 4. Generate headers and descriptions
    let header = generate_header(title, &id_regex);
    let description = generate_description(description);

    // Regex to remove non-alphanumeric characters from IDs.
    // let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();

    // Create the header and subheader for the HTML page.
    // let header = generate_header(title, &id_regex);

    // Create the subheader for the HTML page.
    // let description = generate_description(description);

    // Create a new HashMap to store the HTML content.
    let mut html_map = HashMap::new();

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
    Ok(format!("{}{}{}{}", header, description, json_html, html_string))
}


/// Generate header HTML string based on title
fn generate_header(title: &str, id_regex: &Regex) -> String {
    if title.is_empty() {
        return String::new();
    }
    let header_str = format!("<h1 id=\"\" tabindex=\"0\" itemprop=\"headline\">{}</h1>", title);
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
pub fn preprocess_content(content: &str, class_regex: &Regex, img_regex: &Regex) -> Result<String, Box<dyn Error>> {
    // Map each line in the content through the update_class_attributes function
    let processed_content: Vec<String> = content
        .lines()
        .map(|line| {
            update_class_attributes(line, class_regex, img_regex)
        })
        .collect();

    // Join the processed lines back into a single string
    Ok(processed_content.join("\n"))
}

/// Converts Markdown content to HTML using the Comrak library.
///
/// # Arguments
///
/// * `markdown_content` - A string containing the Markdown content to be converted.
/// * `options` - A reference to `ComrakOptions` which specifies the parsing options for the Comrak library.
///
/// # Returns
///
/// A `String` containing the converted HTML content.
///
pub fn convert_markdown_to_html(markdown_content: &str, options: &ComrakOptions) -> String {
    let html_content = markdown_to_html(markdown_content, options);

    // Return the HTML content
    html_content.to_string()
}

/// Post-processes HTML content by performing various transformations.
///
/// # Arguments
///
/// * `html` - The original HTML content as a string.
/// * `class_regex` - A `Regex` object for matching class attributes in HTML tags.
/// * `img_regex` - A `Regex` object for matching `<img>` tags in HTML.
///
/// # Returns
///
/// A `Result` containing the transformed HTML content as a string if successful,
/// or a `Box<dyn Error>` if an error occurs.
///
/// # Errors
///
/// Returns an error if:
/// - A class attribute cannot be captured by the regular expression.
/// - A class value cannot be retrieved from the captured class attribute.
///
/// # Example
///
/// ```rust
/// use regex::Regex;
/// use std::error::Error;
/// use ssg::modules::html::post_process_html;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let html = "<img src=\"image.jpg\" class=\"img-fluid\">";
///     let class_regex = Regex::new(r#".class=&quot;([^&]+)&quot;"#)?;
///     let img_regex = Regex::new(r#"(<img[^>]*?)(/?>)"#)?;
///
///     let processed_html = post_process_html(html, &class_regex, &img_regex)?;
///     println!("{}", processed_html);
///
///     Ok(())
/// }
/// ```
pub fn post_process_html(html: &str, class_regex: &Regex, img_regex: &Regex) -> Result<String, Box<dyn Error>> {
    let mut processed_html = String::new();

    for line in html.lines() {
        if line.contains(".class=&quot;") {
            let captures = class_regex.captures(line).ok_or("Failed to capture class attributes")?;
            let class_value = captures.get(1).ok_or("Failed to get class value")?.as_str();
            let updated_line = class_regex.replace(line, "");
            let updated_line_with_class = img_regex.replace(
                &updated_line,
                &format!("$1 class=\"{}\"$2", class_value),
            );
            processed_html.push_str(&updated_line_with_class);
        } else {
            processed_html.push_str(line);
        }
        processed_html.push('\n');
    }

    Ok(processed_html)
}
