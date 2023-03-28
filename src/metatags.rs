// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// ## Function: `generate_metatags` - Generates HTML meta tags
///
/// Generates HTML meta tags for the given metadata key-value pairs.
///
/// Returns a string containing the HTML code for the meta tags.
///
/// Each meta tag is created using the `name` and `content` attributes
/// of the input metadata, with the `name` attribute corresponding to
/// the key and the `content` attribute corresponding to the value.
///
/// The resulting tags are concatenated into a single string, separated
/// by newline characters.
///
/// # Arguments
///
/// * `meta` - A slice of key-value pairs representing the metadata to
///            be used in the generated meta tags. Each key-value pair
///            is represented as a tuple of `String` objects, with the
///            first element representing the `name` attribute and the
///            second element representing the `content` attribute of
///            the meta tag.
///
/// # Example
///
/// ```
/// use ssg::metatags::generate_metatags;
///
/// let meta = vec![
///     ("description".to_owned(), "My awesome website".to_owned()),
///     ("keywords".to_owned(), "rust, programming, web development".to_owned()),
/// ];
///
/// let result = generate_metatags(&meta);
/// assert_eq!(result, "<meta name=\"description\" content=\"My awesome website\">\n<meta name=\"keywords\" content=\"rust, programming, web development\">");
///
/// ```
pub fn generate_metatags(meta: &[(String, String)]) -> String {
    meta.iter()
        .map(|(key, value)| {
            format!("<meta name=\"{}\" content=\"{}\">", key, value)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
