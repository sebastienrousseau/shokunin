<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="NucleusFlow logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# `Shokunin Static Site Generator (SSG)`

A Content-First Open Source Static Site Generator (SSG) crafted in Rust.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

Shokunin is a lightning-fast static site generator (SSG) optimised for search engine visibility (SEO) and compliant with WCAG 2.1 Level AA accessibility standards.

## Features

- **⚡ Blazing Fast Performance**: Built in Rust for optimal speed and efficiency
- **📱 SEO Optimised**: Built-in features for maximum search engine visibility
- **🛠️ Multiple Content Formats**: Support for Markdown, YAML, JSON, and TOML
- **📊 Analytics Ready**: Built-in support for Google Analytics and Bing Analytics
- **🔄 Automated Feeds**: Automatic generation of Atom and RSS feeds
- **🎨 Flexible Theming**: Compatible with custom HTML themes and templates
- **📱 Development Server**: Built-in Rust server for local testing

### Accessibility Compliance

Shokunin generates sites that meet Web Content Accessibility Guidelines (WCAG) standards:

- **WCAG 2.1 Level AA** compliance
- Accessible Rich Internet Applications (ARIA) support
- Semantic HTML structure
- Keyboard navigation support
- Screen reader compatibility
- Sufficient color contrast
- Responsive text scaling
- Alternative text for images
- Clear document structure
- Focus management

These accessibility features are automatically implemented in generated sites through:

- Semantic HTML templates
- ARIA landmark roles
- Proper heading hierarchy
- Skip navigation links
- Form input labels
- Keyboard focus indicators
- Color contrast validation

## Installation

Add Shokunin to your Rust project:

```toml
# Cargo.toml
[dependencies]
shokunin = "0.0.30"
```

Basic implementation:

```rust
use staticdatagen::compiler::service::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the paths to the build, site, content and template directories.
    let build_path = Path::new("build");
    let content_path = Path::new("content");
    let site_path = Path::new("public");
    let template_path = Path::new("templates");

    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}
```

### Usage

Create a new static site:

```bash
ssg --new=docs \
    --content=content \
    --template=templates \
    --output=output \
    --serve=public
```

Or use the short form:

```bash
ssg -n=docs -c=content -t=templates -o=output -s=public
```

### Command-Line Options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--new` | `-n` | New site directory name | Yes |
| `--content` | `-c` | Content directory path | Yes |
| `--template` | `-t` | Template directory path | Yes |
| `--output` | `-o` | Output directory path | Yes |
| `--serve` | `-s` | Development server directory | No |

## Documentation

For full API documentation, please visit [https://docs.rs/crate/ssg/](https://docs.rs/crate/ssg/).

## Examples

To explore more examples, clone the repository and run the following command:

```shell
cargo run --example example_name
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0][10]
- [MIT license][11]

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `ssg` library.

[00]: https://shokunin.one
[01]: https://lib.rs/crates/ssg
[02]: https://github.com/sebastienrousseau/shokunin/issues
[03]: https://crates.io/crates/ssg
[04]: https://docs.rs/ssg
[05]: https://github.com/sebastienrousseau/shokunin/blob/main/CONTRIBUTING.md
[06]: https://codecov.io/gh/sebastienrousseau/shokunin
[07]: https://github.com/sebastienrousseau/shokunin/actions?query=branch%3Amain
[08]: https://www.rust-lang.org/
[09]: https://github.com/sebastienrousseau/shokunin
[10]: https://www.apache.org/licenses/LICENSE-2.0
[11]: https://opensource.org/licenses/MIT

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/ssg/release.yml?branch=main&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/shokunin?style=for-the-badge&token=wAcpid8YEt&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-ssg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/ssg-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
