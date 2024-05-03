// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    modules::{
        markdown::convert_markdown_to_html,
        postprocessor::post_process_html,
    },
    utilities::directory::{
        extract_front_matter, format_header_with_id_class,
        update_class_attributes,
    },
};
use regex::Regex;

/// Error enum for HTML generation.
#[derive(Debug)]
pub enum HtmlGenerationError {
    /// Title cannot be empty
    EmptyTitle,
    /// Description cannot be empty
    EmptyDescription,
    /// Regex compilation error
    RegexCompilationError(regex::Error),
}

impl std::fmt::Display for HtmlGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::EmptyTitle => write!(f, "Title cannot be empty."),
            Self::EmptyDescription => {
                write!(f, "Description cannot be empty.")
            }
            Self::RegexCompilationError(ref err) => {
                write!(f, "Regex compilation error: {}", err)
            }
        }
    }
}

impl std::error::Error for HtmlGenerationError {}

impl From<regex::Error> for HtmlGenerationError {
    fn from(err: regex::Error) -> Self {
        Self::RegexCompilationError(err)
    }
}

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
/// A `Result` containing a `String` representing the generated HTML page if successful, or a `HtmlGenerationError` if an error occurs.
///
/// # Examples
///
/// ```rust
/// use ssg::modules::html::generate_html;
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
) -> Result<String, HtmlGenerationError> {
    // Validate arguments
    if title.is_empty() {
        return Err(HtmlGenerationError::EmptyTitle);
    }

    if description.is_empty() {
        return Err(HtmlGenerationError::EmptyDescription);
    }

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
    let markdown_html = convert_markdown_to_html(
        &processed_content,
        &Default::default(),
    );

    // Unwrap the Result to get the String
    let markdown_html = markdown_html.unwrap();

    // Post-process HTML content
    let processed_html =
        post_process_html(&markdown_html, &class_regex, &img_regex);

    // Unwrap the Result to get the String
    let processed_html = processed_html.unwrap();

    // Generate page header and description
    let header = generate_page_header(title, &id_regex);
    let desc = generate_description(description);

    // Process headers in HTML
    let header_tags = vec!["h1", "h2", "h3", "h4", "h5", "h6"];
    let html_string =
        process_headers(&processed_html, &header_tags, &id_regex);

    // Construct the final HTML with JSON content if available
    let json_html = json_content.map_or_else(
        || "".to_string(),
        |json_str| format!("<p>{}</p>", json_str),
    );

    Ok(format!("{}{}{}{}", header, desc, json_html, html_string))
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
/// A `Result` containing a `String` with the processed HTML content, or a `HtmlGenerationError` if an error occurs.
///
/// # Example
///
/// ```rust
/// use regex::Regex;
/// use ssg::modules::html::preprocess_content;
///
/// let content = "<div class=\"some-class\">...</div>";
/// let class_regex = Regex::new(r#".class="([^"]+)""#).unwrap();
/// let img_regex = Regex::new(r#"<img([^>]+)>"#).unwrap();
///
/// let processed_content = preprocess_content(content, &class_regex, &img_regex).unwrap();
/// println!("{}", processed_content);
/// ```
pub fn preprocess_content(
    content: &str,
    class_regex: &Regex,
    img_regex: &Regex,
) -> Result<String, HtmlGenerationError> {
    let processed_content: Vec<String> = content
        .lines()
        .map(|line| {
            update_class_attributes(line, class_regex, img_regex)
        })
        .collect();
    Ok(processed_content.join("\n"))
}

fn process_headers(
    html: &str,
    header_tags: &[&str],
    _id_regex: &Regex,
) -> String {
    let mut html_string = html.to_string();
    for tag in header_tags {
        let re =
            Regex::new(&format!("<{}>([^<]+)</{}>", tag, tag)).unwrap();
        let mut replacements: Vec<(String, String)> = Vec::new();

        for cap in re.captures_iter(&html_string) {
            let original = cap[0].to_string();
            let replacement =
                format_header_with_id_class(&original, &re);
            replacements.push((original, replacement));
        }

        for (original, replacement) in replacements {
            html_string = html_string.replace(&original, &replacement);
        }
    }
    html_string
}

/// Generate page header HTML string based on title
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
/// use ssg::modules::html::generate_page_header;
/// let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
/// let header_html = generate_page_header("My Page Title", &id_regex);
/// assert_eq!(header_html, "<h1 id=\"h1-my\" tabindex=\"0\" aria-label=\"My Heading\" itemprop=\"headline\" class=\"my\">My Page Title</h1>");
/// ```
pub fn generate_page_header(title: &str, id_regex: &Regex) -> String {
    let header_str = format!("<h1>{}</h1>", title);
    format_header_with_id_class(&header_str, id_regex)
}

/// Generate description HTML string based on description
fn generate_description(description: &str) -> String {
    format!("<p>{}</p>", description)
}
