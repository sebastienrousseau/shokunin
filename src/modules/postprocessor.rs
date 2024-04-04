// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::error::Error;
use regex::{Captures, Regex};

/// Post-processes HTML content by performing various transformations.
///
/// This function processes each line of the HTML content to:
/// - Replace class attributes in HTML tags using `class_regex`.
/// - Ensure that each `<img>` tag has both `alt` and `title` attributes.
///   If `title` is missing, it is set to the value of `alt`. If both are missing,
///   they remain unchanged.
///
/// Efficiency is enhanced by pre-compiling regex objects for `alt` and `title`
/// attributes outside the main processing loop. This approach minimizes redundant
/// computations, especially for large HTML contents.
///
/// Robust error handling is incorporated for regex compilation, ensuring that
/// the function responds gracefully to invalid regex patterns.
///
/// # Arguments
///
/// * `html` - The original HTML content as a string.
/// * `class_regex` - A `Regex` object for matching and replacing class attributes in HTML tags.
/// * `img_regex` - A `Regex` object for matching `<img>` tags in HTML.
///
/// # Returns
///
/// A `Result` containing the transformed HTML content as a string if successful,
/// or a `Box<dyn Error>` if an error occurs during regex compilation or processing.
///
/// # Errors
///
/// Returns an error if regex compilation or processing fails for any reason.
pub fn post_process_html(
    html: &str,
    class_regex: &Regex,
    img_regex: &Regex,
) -> Result<String, Box<dyn Error>> {
    let alt_regex = Regex::new(r#"alt="([^"]*)""#)
        .map_err(|e| format!("Failed to compile alt regex: {}", e))?;
    let _title_regex = Regex::new(r#"title="([^"]*)""#)
        .map_err(|e| format!("Failed to compile title regex: {}", e))?;

    let mut processed_html = String::new();

    for line in html.lines() {
        let mut processed_line = line.to_string();
        let mut modified_line = processed_line.clone();

        for class_captures in class_regex.captures_iter(&processed_line)
        {
            let class_attribute =
                class_captures.get(1).unwrap().as_str();
            modified_line = class_regex
                .replace(
                    &modified_line,
                    format!("<p class=\"{}\">", class_attribute)
                        .as_str(),
                )
                .to_string();
        }

        if let Some(class_value) = img_regex
            .captures(&processed_line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
        {
            modified_line = img_regex
                .replace(&modified_line, &class_value.to_string())
                .to_string();
        }

        processed_line = modified_line;

        processed_line =
            img_regex
                .replace_all(&processed_line, |caps: &Captures<'_>| {
                    let img_tag_start = &caps[1];
                    let img_tag_end = &caps[2];

                    let mut new_img_tag = img_tag_start.to_string();

                    let alt_value = alt_regex
                        .captures(img_tag_start)
                        .map_or(String::new(), |c| {
                            c.get(1).map_or(String::new(), |m| {
                                m.as_str().to_lowercase()
                            })
                        });

                    if !new_img_tag.contains("title=")
                        && !alt_value.is_empty()
                    {
                        let title_prefix = "Image of ";
                        let max_alt_length = 66 - title_prefix.len();

                        let alt_substr = alt_value
                            .chars()
                            .take(max_alt_length)
                            .collect::<String>();
                        new_img_tag.push_str(
                            &format!(" title=\"{}\"", alt_substr)
                        );
                    }

                    new_img_tag.push_str(img_tag_end);
                    new_img_tag
                })
                .to_string();

        processed_html.push_str(&processed_line);
        processed_html.push('\n');
    }

    Ok(processed_html)
}
