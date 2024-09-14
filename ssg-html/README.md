<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin HTML Generator (ssg-html)

A Rust-based HTML generation and optimization library for static site generators.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][8] [![Lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][9] [![License][license-badge]][2]

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[Website][1] | [Documentation][9] | [Report Bug][4] | [Request Feature][4] | [Contributing Guidelines][5]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

The `ssg-html` crate is a powerful and flexible HTML generation and optimization library designed specifically for static site generators. It provides a robust set of tools for converting Markdown to HTML, enhancing SEO, improving accessibility, and optimizing performance.

## Features

- **Markdown to HTML Conversion**: Convert Markdown content to HTML with support for custom extensions.
- **Advanced Header Processing**: Automatically generate id and class attributes for headers.
- **SEO Optimization**: Generate meta tags and structured data (JSON-LD) for improved search engine visibility.
- **Accessibility Enhancements**: Add ARIA attributes and validate against WCAG guidelines.
- **Performance Optimization**: Minify HTML output and support asynchronous generation for large sites.
- **Flexible Configuration**: Customize the HTML generation process through a comprehensive set of options.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ssg-html = "0.1.0"
```

## Usage

Here's a basic example of how to use `ssg-html`:

```rust
use ssg_html::{generate_html, HtmlConfig};

fn main() {
    let markdown = "# Hello, world!\n\nThis is a test.";
    let config = HtmlConfig::default();
    
    match generate_html(markdown, &config) {
        Ok(html) => println!("{}", html),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

For more advanced usage and examples, please refer to the [documentation][9].

## Configuration Options

The `HtmlConfig` struct allows you to customize the HTML generation process:

- `enable_syntax_highlighting`: Enable syntax highlighting for code blocks
- `minify_output`: Minify the generated HTML output
- `add_aria_attributes`: Automatically add ARIA attributes for accessibility
- `generate_structured_data`: Generate structured data (JSON-LD) based on content

## Documentation

For full API documentation, please visit [docs.rs/ssg-html][9].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

Special thanks to all the contributors who have helped shape the Shokunin Static Site Generator and its components.

[1]: https://shokunin.one
[2]: https://opensource.org/licenses/MIT
[4]: https://github.com/sebastienrousseau/shokunin/issues
[5]: https://github.com/sebastienrousseau/shokunin/blob/main/CONTRIBUTING.md
[8]: https://crates.io/crates/ssg-html
[9]: https://docs.rs/ssg-html
[10]: https://lib.rs/crates/ssg-html
[14]: https://www.rust-lang.org

[crates-badge]: https://img.shields.io/crates/v/ssg-html.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/ssg-html.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.1.0-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg-html.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
