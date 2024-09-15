//! # Basic Markdown to HTML Conversion Example
//!
//! This example demonstrates how to use the `ssg-markdown` crate to convert Markdown content
//! into HTML using the `comrak` library. It shows how to configure various Markdown extensions
//! (e.g., strikethrough, tables, and autolinks) and then process the Markdown content to generate HTML.
//!
//! ## Usage
//!
//! Simply run the example, and it will print the converted HTML to the console. You can customize
//! the Markdown content and options to see how different configurations affect the output.

use comrak::ComrakOptions;
use ssg_markdown::process_markdown;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"
# Welcome to SSG Markdown

This is a **bold** statement and this is *italic*.

## Features

- Easy to use
- Extensible
- Fast

Check out [our website](https://example.com) for more information.
    "#;

    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;

    let html = process_markdown(markdown, &options)?;
    println!("Converted HTML:\n\n{}", html);

    Ok(())
}
