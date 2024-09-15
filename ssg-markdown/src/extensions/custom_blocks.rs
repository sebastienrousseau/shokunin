//! Custom block extensions for Markdown processing.
//!
//! This module provides functionality to define and process custom Markdown blocks,
//! allowing for extended syntax and specialized rendering of content.

use comrak::nodes::{AstNode, NodeValue};
use comrak::Arena;
use crate::error::MarkdownError;

/// Represents a custom block type.
#[derive(Debug, Clone, PartialEq)]
pub enum CustomBlockType {
    Note,
    Warning,
    Tip,
    // Add more custom block types as needed
}

/// Processes custom blocks in the Markdown AST.
///
/// This function walks through the AST and identifies custom blocks,
/// applying special processing or rendering as needed.
///
/// # Arguments
///
/// * `root` - A reference to the root node of the Markdown AST.
/// * `arena` - A reference to the memory arena used for AST allocation.
///
/// # Returns
///
/// A `Result` indicating success or a `MarkdownError` if processing fails.
///
/// # Examples
///
/// ```
/// use comrak::{parse_document, Arena, ComrakOptions};
/// use ssg_markdown::extensions::custom_blocks::process_custom_blocks;
///
/// let arena = Arena::new();
/// let root = parse_document(&arena, ":::note\nThis is a note\n:::", &ComrakOptions::default());
/// let result = process_custom_blocks(root, &arena);
/// assert!(result.is_ok());
/// ```
pub fn process_custom_blocks<'a>(root: &'a AstNode<'a>, arena: &'a Arena<AstNode<'a>>) -> Result<(), MarkdownError> {
    walk_ast(root, &mut |node| {
        if let NodeValue::CustomBlock(ref mut block_type) = node.data.borrow_mut().value {
            match identify_custom_block(block_type) {
                Some(custom_type) => transform_custom_block(node, custom_type, arena)?,
                None => {} // Not a recognized custom block, skip
            }
        }
        Ok(())
    })?;
    Ok(())
}

/// Identifies the type of custom block based on its content.
fn identify_custom_block(block_type: &str) -> Option<CustomBlockType> {
    match block_type.trim().to_lowercase().as_str() {
        "note" => Some(CustomBlockType::Note),
        "warning" => Some(CustomBlockType::Warning),
        "tip" => Some(CustomBlockType::Tip),
        _ => None,
    }
}

/// Transforms a custom block node in the AST.
fn transform_custom_block<'a>(
    node: &'a AstNode<'a>,
    block_type: CustomBlockType,
    arena: &'a Arena<AstNode<'a>>,
) -> Result<(), MarkdownError> {
    let mut new_node = arena.alloc(AstNode::new(NodeValue::HtmlBlock(Vec::new())));
    let html_content = match block_type {
        CustomBlockType::Note => "<div class=\"note\">",
        CustomBlockType::Warning => "<div class=\"warning\">",
        CustomBlockType::Tip => "<div class=\"tip\">",
    };
    if let NodeValue::HtmlBlock(ref mut html) = new_node.data.borrow_mut().value {
        html.extend_from_slice(html_content.as_bytes());
    }
    node.insert_before(new_node);

    // Move the content of the custom block
    while let Some(child) = node.first_child() {
        child.detach();
        new_node.append(child);
    }

    // Add closing tag
    let closing_node = arena.alloc(AstNode::new(NodeValue::HtmlBlock(Vec::new())));
    if let NodeValue::HtmlBlock(ref mut html) = closing_node.data.borrow_mut().value {
        html.extend_from_slice(b"</div>");
    }
    new_node.insert_after(closing_node);

    // Remove the original custom block node
    node.detach();

    Ok(())
}

/// Recursively walks the AST, applying a function to each node.
fn walk_ast<'a, F>(node: &'a AstNode<'a>, f: &mut F) -> Result<(), MarkdownError>
where
    F: FnMut(&'a AstNode<'a>) -> Result<(), MarkdownError>,
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
    fn test_process_custom_blocks() {
        let arena = Arena::new();
        let root = parse_document(&arena, ":::note\nThis is a note\n:::", &ComrakOptions::default());
        let result = process_custom_blocks(root, &arena);
        assert!(result.is_ok());

        // You would typically check the resulting HTML here
        // For a complete test, you'd need to implement HTML rendering and check the output
    }
}
