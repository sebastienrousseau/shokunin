// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Contains macros related to directory operations.

/// Escapes special HTML characters in a string.
///
/// # Examples
///
/// ```
/// use ssg::utilities::escape::escape_html_entities;
///
/// let input = "Hello, <world>!";
/// let expected = "Hello, &lt;world&gt;!";
///
/// assert_eq!(escape_html_entities(input), expected);
/// ```
pub fn escape_html_entities(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}
