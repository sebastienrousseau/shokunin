<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# `Shokunin Static Site Generator (SSG)`

A modern, high-performance static site generator crafted in Rust, optimised for content-first development.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

Shokunin is a lightning-fast static site generator (SSG) built with Rust, delivering exceptional performance while maintaining strict accessibility standards. It prioritises content management, search engine optimisation (SEO), and WCAG 2.1 Level AA compliance.

## Key Features

- **⚡ Exceptional Performance**: Leverages Rust's zero-cost abstractions for optimal speed
- **📱 Advanced SEO**: Built-in optimisations for maximum search engine visibility
- **🛠️ Versatile Content Support**: Seamlessly handles Markdown, YAML, JSON, and TOML
- **📊 Analytics Integration**: Native support for Google Analytics and Bing Analytics
- **🔄 Automated Feed Generation**: Auto-generates Atom and RSS feeds
- **🎨 Customisable Themes**: Supports bespoke HTML themes and templates
- **📱 Development Tools**: Integrated Rust server for local development

### Accessibility Features

Shokunin automatically implements WCAG 2.1 Level AA standards through:

- Semantic HTML structure
- ARIA landmark roles
- Keyboard navigation support
- Screen reader optimisation
- Colour contrast compliance
- Responsive text scaling
- Alt text management
- Clear document hierarchy
- Focus state handling

## Getting Started

### Installation

Add Shokunin to your `Cargo.toml`:

```toml
[dependencies]
shokunin = "0.0.31"
```

### Basic Usage

```rust
use staticdatagen::compiler::service::compile;
use std::{path::Path, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // Define paths to existing directories
    let build_dir = Path::new("./examples/build");        // For temporary build files
    let content_dir = Path::new("./examples/content");    // Your markdown content
    let public_dir = Path::new("./examples/public");      // Generated site output
    let template_dir = Path::new("./examples/templates"); // HTML templates

    // Generate the static site
    compile(build_dir, content_dir, public_dir, template_dir)?;

    println!("✨ Site generated successfully!");
    Ok(())
}
```

### Command-Line Interface

Create a new site with the following command:

```bash
ssg --content=content \
    --template=templates \
    --output=output \
    --serve=public
```

Or use the shorter form:

```bash
ssg -c=content -t=templates -o=output -s=public
```

```bash
cargo run --bin ssg -- -c="./examples/content" -t="./examples/templates" -o="./examples/output" -s="./examples/public"
```

### CLI Options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--content` | `-c` | Content directory path | Yes |
| `--template` | `-t` | Template directory path | Yes |
| `--output` | `-o` | Output directory path | Yes |
| `--serve` | `-s` | Development server path | No |

## Documentation

For comprehensive API documentation, visit [docs.rs/crate/ssg/](https://docs.rs/crate/ssg/).

## Examples

Explore example implementations:

- Basic Example

```shell
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin
cargo run --example basic
```

- Quick Start Example

```shell
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin
cargo run --example quickstart
```

- Multilingual Example

```shell
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin
cargo run --example multilingual
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

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/shokunin/release.yml?branch=main&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/shokunin?style=for-the-badge&token=wAcpid8YEt&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-ssg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/ssg-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
