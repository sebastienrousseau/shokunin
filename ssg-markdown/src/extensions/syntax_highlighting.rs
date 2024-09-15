//! Syntax highlighting for code blocks in Markdown.
//!
//! This module provides functionality to apply syntax highlighting to code blocks
//! within Markdown content using the Syntect library.

use comrak::nodes::{AstNode, NodeValue};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use crate::error::MarkdownError;

/// Applies syntax highlighting to code blocks in the Markdown AST.
///
/// This function walks through the AST, identifies code blocks, and applies
/// syntax highlighting based on the specified language.
///
/// # Arguments
///
/// * `root` - A reference to the root node of the Markdown AST.
///
/// # Returns
///
/// A `Result` indicating success or a `MarkdownError` if highlighting fails.
///
/// # Examples
///
/// ```
/// use comrak::{parse_document, Arena, ComrakOptions};
/// use ssg_markdown::extensions::syntax_highlighting::apply_syntax_highlighting;
///
/// let arena = Arena::new();
/// let root = parse_document(&arena, "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```", &ComrakOptions::default());
/// let result = apply_syntax_highlighting(root);
/// assert!(result.is_ok());
/// ```
pub fn apply_syntax_highlighting(root: &AstNode) -> Result<(), MarkdownError> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    walk_ast(root, &mut |node| {
        if let NodeValue::CodeBlock(ref mut block) = node.data.borrow_mut().value {
            let lang = block.info.split_whitespace().next().unwrap_or("text");
            let syntax = ss.find_syntax_by_token(lang).unwrap_or_else(|| ss.find_syntax_plain_text());

            let highlighted = highlighted_html_for_string(
                &block.literal,
                &ss,
                syntax,
                theme
            ).map_err(|e| MarkdownError::SyntaxHighlightingError(e.to_string()))?;

            block.literal = highlighted.into_bytes();
            block.info = String::from("html");
        }
        Ok(())
    })?;
    Ok(())
}

/// Recursively walks the AST, applying a function to each node.
fn walk_ast<F>(node: &AstNode, f: &mut F) -> Result<(), MarkdownError>
where
    F: FnMut(&AstNode) -> Result<(), MarkdownError>,
{
    f(node)?;
    let mut child = node.first_child();
    while let Some(next) = child {
        walk_ast(next, f)?;
        child = next.next_sibling();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{parse_document, Arena, ComrakOptions};

    #[test]
    fn test_apply_syntax_highlighting() {
        let arena = Arena::new();
        let root = parse_document(&arena, "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```", &ComrakOptions::default());
        let result = apply_syntax_highlighting(root);
        assert!(result.is_ok());

        // You would typically check the resulting HTML here
        // For a complete test, you'd need to implement HTML rendering and check the output
    }
}
