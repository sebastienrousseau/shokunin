<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin Static Site Generator (SSG)

A content-first static site generator built in Rust, optimized for performance, accessibility, and search engine visibility.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

Shokunin is a high-performance static site generator (SSG) built in Rust that focuses on:

- Content-first development
- Fast site generation
- WCAG 2.1 Level AA accessibility compliance
- Advanced SEO optimization
- Type-safe operations with thorough error handling

## Key features

### Core capabilities

- **High performance**: Zero-cost abstractions through Rust
- **SEO optimization**: Built-in enhancements for search visibility
- **Accessibility**: Automatic WCAG 2.1 Level AA compliance
- **Multi-format support**: Handles Markdown, YAML, JSON, and TOML
- **Feed generation**: Automatic Atom and RSS feed creation
- **Analytics**: Native Google and Bing Analytics integration
- **Theming**: Custom HTML themes and template support

### Development features

- **CLI tools**: Full-featured command-line interface
- **Dev server**: Built-in Rust server for local development
- **Hot reload**: Automatic content updates during development
- **Type safety**: Guaranteed memory and thread safety
- **Async support**: Full asynchronous operation capabilities

## Getting started

### Installation

Add Shokunin to your `Cargo.toml`:

```toml
[dependencies]
ssg = "0.0.32"
```

Or install with Cargo:

```bash
cargo install ssg
```

### Basic usage

```rust
use staticdatagen::compiler::service::compile;
use std::{path::Path, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // Define directory paths
    let build_dir = Path::new("./examples/build");          // Build directory
    let content_dir = Path::new("./examples/content");      // Content directory
    let public_dir = Path::new("./examples/public");        // Public directory
    let template_dir = Path::new("./examples/templates");   // Templates

    // Generate site
    compile(build_dir, content_dir, public_dir, template_dir)?;
    println!("Site generated successfully!");
    Ok(())
}
```

### CLI usage

Create a new site:

```bash
# Full command syntax
ssg --content=content --template=templates --serve=public

# Short form
ssg -c=content -t=templates -o=output -s=public

# Using cargo run
cargo run --bin ssg -- -c="./examples/content" -t="./examples/templates" -s="./examples/public"
```

### CLI options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--content` | `-c` | Content path | Yes |
| `--template` | `-t` | Template path | Yes |
| `--serve` | `-s` | Server public path | Yes |

## Examples

Try these example implementations:

```bash
# Basic example
## Convert Markdown to static sites effortlessly, with templates, organized builds, and instant local hosting.
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin
cargo run --example basic

# Quick start example
## Create, compile, and host a static site effortlessly with Shokunin: simple setup, error handling, and instant local server for previews.
cargo run --example quickstart

# Multilingual example
## Build multilingual static sites effortlessly: generate language-specific sites, create a language selector, and serve all from a single directory.
cargo run --example multilingual
```

## Documentation

- [API documentation][04]
- [Guide][00]
- [Contributing guidelines][05]

## Contributing

We welcome contributions. See the [Contributing guidelines][05] for details on:

- Code of conduct
- Development process
- Pull request guidelines
- Issue reporting

## License

This project is dual-licensed under:

- [Apache License, Version 2.0][10]
- [MIT License][11]

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build Shokunin.

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
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.32-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
