use crate::error::MarkdownError;
use comrak::{parse_document, Arena, ComrakOptions};

/// Parses Markdown content into an Abstract Syntax Tree (AST).
///
/// # Arguments
///
/// * `arena` - A reference to an `Arena` instance that manages the memory for the AST nodes.
/// * `content` - A string slice containing the Markdown content to be parsed.
/// * `options` - A reference to `ComrakOptions` which specifies the parsing options.
///
/// # Returns
///
/// A `Result` containing the root node of the AST, or a `MarkdownError` if parsing fails.
///
/// # Examples
///
/// ```
/// use comrak::{Arena, ComrakOptions};
/// use ssg_markdown::parser::parse_markdown;
///
/// let arena = Arena::new();
/// let content = "# Hello, world!";
/// let options = ComrakOptions::default();
/// let result = parse_markdown(&arena, content, &options);
/// assert!(result.is_ok());
/// ```
pub fn parse_markdown<'a>(
    arena: &'a Arena<comrak::nodes::AstNode<'a>>,
    content: &str,
    options: &ComrakOptions,
) -> Result<&'a comrak::nodes::AstNode<'a>, MarkdownError> {
    Ok(parse_document(arena, content, options))
}
