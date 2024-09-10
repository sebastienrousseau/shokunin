// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use comrak::{markdown_to_html, ComrakOptions};
use std::error::Error;

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
/// # Errors
///
/// Returns a `Box<dyn Error>` if conversion fails.
///
/// # Examples
///
/// ```rust
/// use ssg::modules::markdown::convert_markdown_to_html;
/// use comrak::ComrakOptions;
///
/// let markdown_content = "# Hello, world!";
/// let options = ComrakOptions::default();
/// let html_content = convert_markdown_to_html(markdown_content, &options).unwrap();
///
/// assert_eq!(html_content, "<h1>Hello, world!</h1>\n");
/// ```
pub fn convert_markdown_to_html(
    markdown_content: &str,
    options: &ComrakOptions,
) -> Result<String, Box<dyn Error>> {
    let html_content = markdown_to_html(markdown_content, options);
    Ok(html_content.to_string())
}
