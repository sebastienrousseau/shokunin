<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin Static Site Generator Metadata (ssg-metadata)

A Rust-based library for handling metadata extraction and management in static site generators. The library provides tools for efficiently extracting meta tags, keywords, and managing SEO-related metadata in a static site context.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][8] [![Lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][9] [![License][license-badge]][2]

## Overview

`ssg-metadata` is designed for developers working on static site generators (SSG) who need robust tools to extract and handle metadata such as meta tags, keywords, and SEO information from HTML content. It helps ensure that your static sites are SEO-optimized and provides utilities to streamline the metadata extraction process.

## Features

- **Meta Tag Extraction**: Extract meta tags such as title, description, and keywords from HTML content.
- **SEO Keyword Extraction**: Retrieve and manage SEO keywords for content optimization.
- **HTML Escape/Unescape**: Safely escape and unescape HTML entities within your content.
- **Flexible Integration**: Can be easily integrated into any static site generator workflow.
- **Utility Functions**: Includes performance and SEO utilities to enhance metadata management.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ssg-metadata = "0.0.1"
```

## Usage

Here are some examples of how to use the library:

### Basic Usage

```rust
use ssg_metadata::{extract_meta_tags, extract_keywords};

let html_content = "<html>...</html>";
let meta_tags = extract_meta_tags(html_content);
let keywords = extract_keywords(html_content);

println!("Meta tags: {:?}", meta_tags);
println!("Keywords: {:?}", keywords);
```

### HTML Escape/Unescape Example

```rust
use ssg_metadata::escape::{escape_html, unescape_html};

let escaped = escape_html("Hello <World>");
let unescaped = unescape_html(&escaped);

println!("Escaped: {}", escaped);
println!("Unescaped: {}", unescaped);
```

## Modules

- **lib.rs**: The main library module that ties everything together.
- **metatags.rs**: Handles the extraction and management of meta tags from HTML content.
- **keywords.rs**: Manages the extraction and manipulation of keywords for SEO optimization.
- **escape.rs**: Provides functionality for escaping and unescaping HTML entities.
- **extractor.rs**: Core logic for content processing and metadata extraction.

## Documentation

For full API documentation, please visit [docs.rs/ssg-metadata][9].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `ssg-metadata` library.

[9]: https://docs.rs/ssg-metadata
[2]: https://opensource.org/licenses/MIT
[8]: https://crates.io/crates/ssg-html
[10]: https://lib.rs/crates/ssg-html
[14]: https://www.rust-lang.org

[crates-badge]: https://img.shields.io/crates/v/ssg-html.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/ssg-html.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg-html.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
