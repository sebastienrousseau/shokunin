use crate::error::MarkdownError;
use comrak::{markdown_to_html, ComrakOptions};

/// Converts Markdown content to HTML using the Comrak library.
///
/// # Arguments
///
/// * `markdown_content` - A string slice containing the Markdown content to be converted.
/// * `options` - A reference to `ComrakOptions` which specifies the parsing options for the Comrak library.
///
/// # Returns
///
/// A `Result` containing the converted HTML as a `String`, or a `MarkdownError` if conversion fails.
///
/// # Examples
///
/// ```
/// use comrak::ComrakOptions;
/// use ssg_markdown::converter::convert_markdown_to_html;
///
/// let markdown_content = "# Hello, world!";
/// let options = ComrakOptions::default();
/// let result = convert_markdown_to_html(markdown_content, &options);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), "<h1>Hello, world!</h1>\n");
/// ```
pub fn convert_markdown_to_html(
    markdown_content: &str,
    options: &ComrakOptions,
) -> Result<String, MarkdownError> {
    Ok(markdown_to_html(markdown_content, options))
}
