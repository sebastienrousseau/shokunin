// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate regex;
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
///     "<h1>My Page</h1><h2>This is a test page</h2><h2>Hello, world!</h2>\n<p>This is a test.</p>"
/// );
///
/// ```
pub fn generate_html(
    content: &str,
    title: &str,
    description: &str,
    json_content: Option<&str>,
) -> String {
    let options = comrak::ComrakOptions::default();
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
    let header = if !title.is_empty() {
        format!("<h1>{}</h1>", title)
    } else {
        "".to_string()
    };
    let subheader = if !description.is_empty() {
        format!("<h2>{}</h2>", description)
    } else {
        "".to_string()
    };
    // let markdown_html =
    //     comrak::markdown_to_html(markdown_content, &options);

    let mut html_map = HashMap::new();
    let markdown_html =
        comrak::markdown_to_html(markdown_content, &options);
    html_map.insert(title.to_string(), markdown_html);

    for (_, html_content) in html_map.iter_mut() {
        let mut image_class = None;
        *html_content = html_content
            .lines()
            .map(|line| {
                if let Some(idx) = line.find(".class=&quot;") {
                    let before = &line[..idx];
                    let after = line[idx + ".class=&quot;".len()..]
                        .split_once('&')
                        .map(|(s, _)| s)
                        .unwrap_or("");
                    if before.contains("img src") {
                        image_class = Some(after.to_string());
                        println!(
                            "Extracted class attribute: {}",
                            image_class.clone().unwrap()
                        ); // print the extracted class attribute
                           // remove the class attribute from the remaining string
                        format!(
                            "{}{}",
                            before,
                            &line[idx
                                + after.len()
                                + ".class=&quot;&quot;".len()..]
                        )
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        if let Some(class) = image_class {
            // set the class attribute for the image tag
            *html_content = html_content.replace(
                "img src",
                &format!("img class=\"{}\" src", class),
            );
            println!("Extracted class attribute: {}", class); // print the extracted class attribute
        }
    }

    let html_string = html_map
        .values()
        .map(|content| content.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    println!("html_string={:?}", html_string);

    let json_html = if let Some(json_str) = json_content {
        format!("<p>{}</p>", json_str)
    } else {
        "".to_string()
    };
    format!("{}{}{}{}", header, subheader, json_html, html_string)
}
