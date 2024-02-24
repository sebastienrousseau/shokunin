use crate::utilities::directory::update_class_attributes;
use regex::Regex;
use std::error::Error;

/// Preprocesses the Markdown content to update class attributes and image tags.
///
/// # Arguments
///
/// * `content` - A string containing the Markdown content to be processed.
/// * `class_regex` - A reference to a `Regex` object for matching class attributes.
/// * `img_regex` - A reference to a `Regex` object for matching image tags.
///
/// # Returns
///
/// A `Result` containing a `String` with the processed Markdown content, or a `Box<dyn Error>` if an error occurs.
///
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

    let mut result = processed_content.join("\n");

    // Trim trailing newlines
    while result.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}
