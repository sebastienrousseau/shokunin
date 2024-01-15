// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
use regex::Regex;
use std::error::Error;
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
/// The function handles errors more gracefully, returning a `Result`
/// instead of panicking in cases where processing fails.
///
/// An optional JSON content string can be provided and will be included
/// in the generated HTML if present.
///
/// The resulting HTML page is returned as a string.
///
/// # Examples
///
/// ```
/// use ssg::modules::html::generate_html;
///
/// let content = "## Hello, world!\n\nThis is a test.";
/// let title = "My Page";
/// let description = "This is a test page";
/// let html = generate_html(content, title, description, None);
/// let html_str = html.unwrap_or_else(|e| panic!("Error: {:?}", e));
///
/// assert_eq!(html_str, "<h1 id=\"h1-my\" tabindex=\"0\" itemprop=\"headline\" id=\"\" class=\"my\">My Page</h1><p>This is a test page</p><h2 id=\"h2-hello\" tabindex=\"0\" itemprop=\"name\" class=\"hello\">Hello, world!</h2>\n<p>This is a test.</p>\n");
/// ```
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
    json_content: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let id_regex = Regex::new(r"[^a-zA-Z0-9]+")?;
    let class_regex = Regex::new(r"\.class=&quot;([^&]+)&quot;")?;
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)")?;

    // 1. Preprocess the content
    let markdown_content = extract_front_matter(content);
    let processed_content = preprocess_content(markdown_content, &class_regex, &img_regex)?;

    // 2. Convert Markdown to HTML
    let options = create_comrak_options();
    let markdown_html = convert_markdown_to_html(&processed_content, &options);

    // 3. Post-process the HTML
    let processed_html = post_process_html(&markdown_html, &class_regex, &img_regex)?;

    // 4. Generate headers and descriptions
    let header = generate_header(title, &id_regex);
    let desc = generate_description(description);

    // Process headers in HTML
    let header_tags = vec!["h1", "h2", "h3", "h4", "h5", "h6"];
    let mut html_string = processed_html;
    for tag in header_tags {
        let re = Regex::new(&format!("<{}>([^<]+)</{}>", tag, tag))?;
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

    // Construct the final HTML with JSON content if available
    let json_html = json_content.map_or_else(|| "".to_string(), |json_str| format!("<p>{}</p>", json_str));
    Ok(format!("{}{}{}{}", header, desc, json_html, html_string))
}

/// Generate header HTML string based on title
///
/// This function takes a title string and a compiled regular expression (id_regex) and 
/// generates an HTML header tag (<h1>) with the given title.
///
/// Arguments:
/// * `title`: A string slice that holds the title to be included in the header tag.
/// * `id_regex`: A reference to a compiled Regex object used for processing the header string.
///
/// Returns:
/// A `String` that represents the HTML <h1> header tag with the title.
///
/// # Examples
///
/// ```
/// use regex::Regex;
/// use ssg::modules::html::generate_header;
/// let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
/// let header_html = generate_header("My Page Title", &id_regex);
/// assert_eq!(header_html, "<h1 id=\"h1-my\" tabindex=\"0\" itemprop=\"headline\" id=\"\" class=\"my\">My Page Title</h1>");
/// ```
pub fn generate_header(title: &str, id_regex: &Regex) -> String {
    // Check if the title is empty. If so, return an empty string as no header is needed.
    if title.is_empty() {
        return String::new();
    }

    // Format the title into an HTML <h1> tag. Initially, the id attribute is left empty.
    let header_str = format!("<h1 id=\"\">{}</h1>", title);

    // Call format_header_with_id_class function to add appropriate id and class attributes
    // based on the id_regex. This function is expected to process the header string and
    // return the final HTML header tag string with id and possibly class attributes set.
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
/// This function processes each line of the HTML content to:
/// - Replace class attributes in HTML tags using `class_regex`.
/// - Ensure that each `<img>` tag has both `alt` and `title` attributes.
///   If `title` is missing, it is set to the value of `alt`. If both are missing,
///   they remain unchanged.
///
/// # Arguments
///
/// * `html` - The original HTML content as a string.
/// * `class_regex` - A `Regex` object for matching and replacing class attributes in HTML tags.
/// * `img_regex` - A `Regex` object for matching `<img>` tags in HTML.
/// * `title` - A placeholder title string, not used in current logic but kept for potential future use.
///
/// # Returns
///
/// A `Result` containing the transformed HTML content as a string if successful,
/// or a `Box<dyn Error>` if an error occurs.
///
/// # Errors
///
/// Returns an error if:
/// - A class attribute cannot be captured or a class value cannot be retrieved from the captured class attribute.
/// - The regular expression processing fails for any reason.
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
///     let title = "Unused Placeholder Title";
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
        let mut processed_line = line.to_string();

        // Temporarily store class value
        let class_value = if line.contains(".class=&quot;") {
            class_regex.captures(&processed_line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        } else {
            None
        };

        // Process class attributes
        if let Some(class_value) = class_value {
            processed_line = class_regex.replace(&processed_line, "").to_string();
            processed_line = img_regex.replace(&processed_line, &format!("$1 class=\"{}\"$2", class_value)).to_string();
        }

        // Add alt and title attributes to img tags
        processed_line = img_regex.replace_all(&processed_line, |caps: &regex::Captures| {
            let img_tag_start = &caps[1]; // <img... up to the closure
            let img_tag_end = &caps[2];   // /> or >

            let mut new_img_tag = img_tag_start.to_string();

            // Regex to find the alt attribute
            let alt_regex = regex::Regex::new(r#"alt="([^"]*)""#).unwrap();

            // Extract the value of the alt attribute and convert it to lowercase
            let alt_value = alt_regex.captures(img_tag_start)
                            .and_then(|c| c.get(1))
                            .map_or(String::new(), |m| m.as_str().to_lowercase());

            // Check if 'title' is present; if not, add it. If it is, replace it with the alt value
            if new_img_tag.contains("title=") {
                let title_prefix = if !alt_value.is_empty() { "Image of " } else { "" };
                let max_alt_length = 66 - title_prefix.len();

                // Ensure that we slice at a valid character boundary
                let alt_substr = if !alt_value.is_empty() && alt_value.chars().count() > max_alt_length {
                    alt_value.char_indices().enumerate().take_while(|&(char_count, _)| {
                        char_count < max_alt_length
                    }).last().map_or(&alt_value[..], |(_, (idx, _))| {
                        &alt_value[..idx]
                    })
                } else {
                    &alt_value
                };

                let title_regex = regex::Regex::new(r#"title="([^"]*)""#).unwrap();
                new_img_tag = title_regex.replace(&new_img_tag, format!(r#"title="{}{}""#, title_prefix, alt_substr)).to_string();
            } else {
                let title_prefix = if !alt_value.is_empty() { "Image of " } else { "" };
                let max_alt_length = 66 - title_prefix.len();

                // Ensure that we slice at a valid character boundary
                let alt_substr = if !alt_value.is_empty() && alt_value.chars().count() > max_alt_length {
                    alt_value.char_indices().enumerate().take_while(|&(char_count,_)| {
                        char_count < max_alt_length
                    }).last().map_or(&alt_value[..], |(_, (idx, _))| {
                        &alt_value[..idx]
                    })
                } else {
                    &alt_value
                };
                new_img_tag.push_str(&format!(" title=\"{}{}\"", title_prefix, alt_substr));
            }

            // Append the closure of the tag (either /> or >)
            new_img_tag.push_str(img_tag_end);
            new_img_tag
        }).to_string();

        processed_html.push_str(&processed_line);
        processed_html.push('\n');
    }

    Ok(processed_html)
}
