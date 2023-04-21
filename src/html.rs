// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
use std::collections::HashMap;

use regex::Regex;
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
///     "<h1 id=\"h1-id-my-page\" id=\"\" class=\"h1-id-my-page\">My Page</h1><p>This is a test page</p><h2><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello, world!</h2>\n<p>This is a test.</p>\n"
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
    let markdown_content = if content.starts_with("---\n") {
        if let Some(end_pos) = content.find("\n---\n") {
            &content[end_pos + 5..] // Skip the "---\n\n" that follows the front matter
        } else {
            ""
        }
    } else if content.starts_with("+++\n") {
        if let Some(end_pos) = content.find("\n+++\n") {
            &content[end_pos + 5..] // Skip the "+++\n\n" that follows the front matter
        } else {
            ""
        }
    } else if content.starts_with("{\n") {
        if let Some(end_pos) = content.find("\n}\n") {
            &content[end_pos + 2..]
        } else {
            ""
        }
    } else {
        content
    };

    // Regex to remove non-alphanumeric characters from IDs:
    let id_regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();

    // Create the header and subheader for the HTML page:
    let header = if !title.is_empty() {
        let header_str = format!("<h1 id=\"\">{}</h1>", title);
        format_header_with_id_class(&header_str, &id_regex)
    } else {
        String::new()
    };

    // Create the subheader for the HTML page:
    let description = if !description.is_empty() {
        let description_str = format!("<p>{}</p>", description);
        description_str
    } else {
        String::new()
    };

    // Create a new HashMap to store the HTML content.
    let mut html_map = HashMap::new();

    // Configure Comrak's Markdown rendering options:
    let mut options = comrak::ComrakOptions::default();

    // Enable non-standard Markdown features:
    options.extension.autolink = true; // Detects URLs and email addresses and makes them clickable.
    options.extension.description_lists = true; // Allows you to create description lists.
    options.extension.footnotes = true; // Allows you to create footnotes.
    options.extension.front_matter_delimiter = Some("---".to_owned()); // Ignore front-mater starting with '---'
    options.extension.header_ids = Some("".to_string()); // Adds an ID to each header.
    options.extension.strikethrough = true; // Allows you to create strikethrough text.
    options.extension.superscript = true; // Allows you to create superscript text.
    options.extension.table = true; // Allows you to create tables.
    options.extension.tagfilter = true; // Allows you to filter HTML tags.
    options.extension.tasklist = true; // Allows you to create task lists.
    options.parse.smart = true; // Enables smart punctuation.
    options.render.github_pre_lang = true; // Renders GitHub-style fenced code blocks.
    options.render.hardbreaks = false; // Renders hard line breaks as <br> tags.
    options.render.unsafe_ = true; // Allows raw HTML to be rendered.

    // Preprocess the content to extract and move the class attributes
    let mut processed_content = String::new();
    let class_regex = Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*)(>)").unwrap();

    for line in content.lines() {
        if line.contains(".class=&quot;") {
            let captures = class_regex.captures(line).unwrap();
            let class_value = captures.get(1).unwrap().as_str();
            let updated_line = class_regex.replace(line, "");
            let updated_line_with_class =
                img_regex.replace(&updated_line, &format!("$1 class=\"{}\"$2", class_value));
            processed_content.push_str(&updated_line_with_class);
            processed_content.push('\n');
        } else {
            processed_content.push_str(line);
            processed_content.push('\n');
        }
    }

    // Render the Markdown content to HTML:
    let markdown_html = comrak::markdown_to_html(markdown_content, &options);

    let class_regex = Regex::new(r"\.class=&quot;([^&]+)&quot;").unwrap();
    let img_regex = Regex::new(r"(<img[^>]*?)(/?>)").unwrap();

    let mut processed_html = String::new();
    for line in markdown_html.lines() {
        if line.contains(".class=&quot;") {
            let captures = class_regex.captures(line).unwrap();
            let class_value = captures.get(1).unwrap().as_str();
            let updated_line = class_regex.replace(line, "");
            let updated_line_with_class =
                img_regex.replace(&updated_line, &format!("$1 class=\"{}\"$2", class_value));
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
        .map(|content| content.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    // println!("html_string={:?}", html_string);

    let json_html = if let Some(json_str) = json_content {
        format!("<p>{}</p>", json_str)
    } else {
        "".to_string()
    };
    format!("{}{}{}{}", header, description, json_html, html_string)
}

fn format_header_with_id_class(header_str: &str, id_regex: &Regex) -> String {
    let mut formatted_header_str = String::new();
    let mut in_header_tag = false;
    let mut id_attribute_added = false;
    let mut class_attribute_added = false;

    for c in header_str.chars() {
        if !in_header_tag {
            formatted_header_str.push(c);
            if c == '<' {
                in_header_tag = true;
            }
        } else {
            if !id_attribute_added && c == ' ' {
                formatted_header_str.push_str(&format!(
                    " id=\"{}\"",
                    id_regex
                        .replace_all(&header_str.to_lowercase(), "-")
                        .trim_matches('-')
                        .trim_end_matches("-h1")
                        .trim_end_matches("-h2")
                        .trim_end_matches("-h3")
                        .trim_end_matches("-h4")
                        .trim_end_matches("-h5")
                        .trim_end_matches("-h6")
                ));
                id_attribute_added = true;
            }
            if !class_attribute_added && c == '>' {
                formatted_header_str.push_str(&format!(
                    " class=\"{}\"",
                    id_regex
                        .replace_all(&header_str.to_lowercase(), "-")
                        .trim_matches('-')
                        .trim_end_matches("-h1")
                        .trim_end_matches("-h2")
                        .trim_end_matches("-h3")
                        .trim_end_matches("-h4")
                        .trim_end_matches("-h5")
                        .trim_end_matches("-h6")
                ));
                class_attribute_added = true;
            }
            formatted_header_str.push(c);
            if c == '>' {
                in_header_tag = false;
            }
        }
    }
    formatted_header_str
}
