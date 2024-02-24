// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::utilities::directory::{
    create_comrak_options, extract_front_matter,
    format_header_with_id_class, update_class_attributes,
};
use crate::modules::postprocessor::post_process_html;
use comrak::{markdown_to_html, ComrakOptions};



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
pub fn convert_markdown_to_html(
    markdown_content: &str,
    options: &ComrakOptions,
) -> Result<String, Box<dyn Error>> {
    let html_content = markdown_to_html(markdown_content, options);
    Ok(html_content.to_string())
}

// /// Post-processes HTML content by performing various transformations.
// ///
// /// This function processes each line of the HTML content to:
// /// - Replace class attributes in HTML tags using `class_regex`.
// /// - Ensure that each `<img>` tag has both `alt` and `title` attributes.
// ///   If `title` is missing, it is set to the value of `alt`. If both are missing,
// ///   they remain unchanged.
// ///
// /// Efficiency is enhanced by pre-compiling regex objects for `alt` and `title`
// /// attributes outside the main processing loop. This approach minimizes redundant
// /// computations, especially for large HTML contents.
// ///
// /// Robust error handling is incorporated for regex compilation, ensuring that
// /// the function responds gracefully to invalid regex patterns.
// ///
// /// # Arguments
// ///
// /// * `html` - The original HTML content as a string.
// /// * `class_regex` - A `Regex` object for matching and replacing class attributes in HTML tags.
// /// * `img_regex` - A `Regex` object for matching `<img>` tags in HTML.
// ///
// /// # Returns
// ///
// /// A `Result` containing the transformed HTML content as a string if successful,
// /// or a `Box<dyn Error>` if an error occurs during regex compilation or processing.
// ///
// /// # Errors
// ///
// /// Returns an error if regex compilation or processing fails for any reason.
// pub fn post_process_html(
//     html: &str,
//     class_regex: &Regex,
//     img_regex: &Regex,
// ) -> Result<String, Box<dyn Error>> {
//     let alt_regex = Regex::new(r#"alt="([^"]*)""#)
//         .map_err(|e| format!("Failed to compile alt regex: {}", e))?;
//     let _title_regex = Regex::new(r#"title="([^"]*)""#)
//         .map_err(|e| format!("Failed to compile title regex: {}", e))?;

//     let mut processed_html = String::new();

//     for line in html.lines() {
//         let mut processed_line = line.to_string();
//         let mut modified_line = processed_line.clone();

//         for class_captures in class_regex.captures_iter(&processed_line)
//         {
//             let class_attribute =
//                 class_captures.get(1).unwrap().as_str();
//             modified_line = class_regex
//                 .replace(
//                     &modified_line,
//                     format!("<p class=\"{}\">", class_attribute)
//                         .as_str(),
//                 )
//                 .to_string();
//         }

//         if let Some(class_value) = img_regex
//             .captures(&processed_line)
//             .and_then(|caps| caps.get(1))
//             .map(|m| m.as_str().to_string())
//         {
//             modified_line = img_regex
//                 .replace(&modified_line, &class_value.to_string())
//                 .to_string();
//         }

//         processed_line = modified_line;

//         processed_line =
//             img_regex
//                 .replace_all(&processed_line, |caps: &Captures| {
//                     let img_tag_start = &caps[1];
//                     let img_tag_end = &caps[2];

//                     let mut new_img_tag = img_tag_start.to_string();

//                     let alt_value = alt_regex
//                         .captures(img_tag_start)
//                         .map_or(String::new(), |c| {
//                             c.get(1).map_or(String::new(), |m| {
//                                 m.as_str().to_lowercase()
//                             })
//                         });

//                     if !new_img_tag.contains("title=")
//                         && !alt_value.is_empty()
//                     {
//                         let title_prefix = "Image of ";
//                         let max_alt_length = 66 - title_prefix.len();

//                         let alt_substr = alt_value
//                             .chars()
//                             .take(max_alt_length)
//                             .collect::<String>();
//                         new_img_tag.push_str(
//                             &format!(" title=\"{}\"", alt_substr)
//                         );
//                     }

//                     new_img_tag.push_str(img_tag_end);
//                     new_img_tag
//                 })
//                 .to_string();

//         processed_html.push_str(&processed_line);
//         // processed_html.push('\n');
//     }

//     Ok(processed_html)
// }