//! Enhanced table processing for Markdown.
//!
//! This module provides functionality to process and enhance Markdown tables,
//! adding features like alignment, custom classes, and responsive design.

use comrak::nodes::{AstNode, NodeValue};
use crate::error::MarkdownError;

/// Alignment options for table columns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
}

/// Processes and enhances tables in the Markdown AST.
///
/// This function walks through the AST, identifies tables, and applies
/// enhancements such as alignment and responsive design.
///
/// # Arguments
///
/// * `root` - A reference to the root node of the Markdown AST.
///
/// # Returns
///
/// A `Result` indicating success or a `MarkdownError` if processing fails.
///
/// # Examples
///
/// ```
/// use comrak::{parse_document, Arena, ComrakOptions};
/// use ssg_markdown::extensions::tables::process_tables;
///
/// let arena = Arena::new();
/// let root = parse_document(&arena, "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |", &ComrakOptions::default());
/// let result = process_tables(root);
/// assert!(result.is_ok());
/// ```
pub fn process_tables(root: &AstNode) -> Result<(), MarkdownError> {
    walk_ast(root, &mut |node| {
        if let NodeValue::Table(ref mut table) = node.data.borrow_mut().value {
            enhance_table(node, table)?;
        }
        Ok(())
    })?;
    Ok(())
}

/// Enhances a table node with additional features.
fn enhance_table(node: &AstNode, table: &mut comrak::nodes::NodeTable) -> Result<(), MarkdownError> {
    // Add a wrapper div for responsive design
    let wrapper_start = create_html_node("<div class=\"table-responsive\">")?;
    let wrapper_end = create_html_node("</div>")?;

    node.insert_before(wrapper_start);
    node.insert_after(wrapper_end);

    // Add alignment classes to cells
    for (col, alignment) in table.alignments.iter().enumerate() {
        let alignment_class = match alignment {
            comrak::nodes::TableAlignment::Left => "text-left",
            comrak::nodes::TableAlignment::Center => "text-center",
            comrak::nodes::TableAlignment::Right => "text-right",
            comrak::nodes::TableAlignment::None => continue,
        };

        add_class_to_column(node, col, alignment_class)?;
    }

    Ok(())
}

/// Adds a CSS class to all cells in a specific column of the table.
fn add_class_to_column(table_node: &AstNode, col: usize, class: &str) -> Result<(), MarkdownError> {
    let mut row = table_node.first_child();
    while let Some(row_node) = row {
        let mut cell = row_node.first_child();
        for _ in 0..col {
            cell = cell.and_then(|n| n.next_sibling());
        }
        if let Some(cell_node) = cell {
            add_class_to_node(cell_node, class)?;
        }
        row = row_node.next_sibling();
    }
    Ok(())
}

/// Adds a CSS class to a node by wrapping it in a `<div>` with the specified class.
fn add_class_to_node(node: &AstNode, class: &str) -> Result<(), MarkdownError> {
    let div_start = create_html_node(&format!("<div class=\"{}\">", class))?;
    let div_end = create_html_node("</div>")?;

    node.insert_before(div_start);
    node.insert_after(div_end);

    Ok(())
}

/// Creates a new AST node with HTML content.
fn create_html_node(html: &str) -> Result<&AstNode, MarkdownError> {
    let arena = comrak::Arena::new();
    let node = arena.alloc(AstNode::new(NodeValue::HtmlInline(html.as_bytes().to_vec())));
    Ok(node)
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
    fn test_process_tables() {
        let arena = Arena::new();
        let root = parse_document(&arena, "| Header 1 | Header 2 |\n|:---------|----------:|\n| Cell 1   | Cell 2   |", &ComrakOptions::default());
        let result = process_tables(root);
        assert!(result.is_ok());

        // You would typically check the resulting HTML here
        // For a complete test, you'd need to implement HTML rendering and check the output
    }
}
