<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin Static Site Generator Markdown (ssg-markdown)

A Rust-based library for processing and enhancing Markdown content in static site generators. The library provides tools for converting Markdown to HTML with support for custom blocks, syntax highlighting, and enhanced table formatting.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][8] [![Lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][9] [![License][license-badge]][2]

## Overview

`ssg-markdown` is designed for developers working on static site generators (SSG) who need robust tools to process Markdown content and convert it to HTML with additional features. It helps ensure that your static sites have rich, well-formatted content with support for custom elements and syntax highlighting.

## Features

- **Markdown to HTML Conversion**: Convert Markdown content to HTML using the `comrak` library.
- **Custom Block Extensions**: Support for custom blocks such as notes, warnings, and tips.
- **Syntax Highlighting**: Apply syntax highlighting to code blocks in various programming languages.
- **Enhanced Table Formatting**: Improve the formatting and responsiveness of HTML tables.
- **Flexible Configuration**: Easily customize the Markdown processing behavior through `MarkdownOptions`.
- **Error Handling**: Robust error handling with detailed error types and context.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ssg-markdown = "0.0.1"
```

## Usage

Here are some examples of how to use the library:

### Basic Usage

```rust
use ssg_markdown::{process_markdown, MarkdownOptions};

let markdown_content = "# Hello, world!\n\nThis is a paragraph.";
let options = MarkdownOptions::new();
let html = process_markdown(markdown_content, &options).unwrap();
println!("HTML output: {}", html);
```

### Custom Blocks and Syntax Highlighting

```rust
use ssg_markdown::{process_markdown, MarkdownOptions};

let markdown_content = r#"
# Example

<div class="note">This is a note.</div>

```rust
fn main() {
    println!("Hello, world!");
}

"#;

let options = MarkdownOptions::new()
    .with_custom_blocks(true)
    .with_syntax_highlighting(true);

let html = process_markdown(markdown_content, &options).unwrap();
println!("HTML output: {}", html);

```

## Modules

- **lib.rs**: The main library module that ties everything together.
- **markdown.rs**: Core functionality for Markdown processing and conversion.
- **extensions.rs**: Handles custom block extensions, syntax highlighting, and table processing.
- **error.rs**: Defines error types and implements error handling for the library.

## Documentation

For full API documentation, please visit [docs.rs/ssg-markdown][9].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `ssg-markdown` library.

[9]: https://docs.rs/ssg-markdown
[2]: https://opensource.org/licenses/MIT
[8]: https://crates.io/crates/ssg-markdown
[10]: https://lib.rs/crates/ssg-markdown
[14]: https://www.rust-lang.org

[crates-badge]: https://img.shields.io/crates/v/ssg-html.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/ssg-html.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg-html.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
