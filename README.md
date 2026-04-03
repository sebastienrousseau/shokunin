<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin Static Site Generator (SSG)

A content-first static site generator crafted in Rust, optimized for performance, accessibility, and search engine visibility.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

[Website][00] | [Documentation][04] | [Report Bug][02] | [Request Feature][02] | [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

Shokunin is a high-performance static site generator (SSG) engineered in Rust that prioritises:

- Content-first development approach
- Lightning-fast site generation
- WCAG 2.1 Level AA accessibility compliance
- Advanced SEO optimization
- Type-safe operations with comprehensive error handling

## Key features

- **Exceptional performance**: Zero-cost abstractions through Rust
- **SEO optimization**: Built-in enhancements for search visibility
- **Accessibility**: Automatic WCAG 2.1 Level AA compliance
- **Multi-format support**: Handles Markdown, YAML, JSON, and TOML
- **Feed generation**: Automatic Atom and RSS feed creation
- **Analytics**: Native Google and Bing Analytics integration
- **Custom theming**: HTML themes and template support
- **CLI tools**: Comprehensive command-line interface
- **Dev server**: Built-in Rust server for local development
- **Async support**: Full asynchronous operation capabilities

## Prerequisites

- **Rust** 1.74.0 or later ([install](https://rustup.rs/))

Verify your installation:

```bash
rustc --version   # must be >= 1.74.0
cargo --version
```

## Getting started

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ssg = "0.0.33"
```

Or install the CLI via Cargo:

```bash
cargo install ssg
```

### Basic usage (library)

```rust
use staticdatagen::compiler::service::compile;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let build_dir = Path::new("build");
    let content_dir = Path::new("content");
    let site_dir = Path::new("public");
    let template_dir = Path::new("templates");

    compile(build_dir, content_dir, site_dir, template_dir)?;
    println!("Site generated successfully!");
    Ok(())
}
```

### CLI usage

```bash
# Create a new site with all directories specified
ssg --new mysite \
    --content=content \
    --output=build \
    --template=templates

# Short form
ssg -n mysite -c content -o build -t templates

# Using cargo run
cargo run --bin ssg -- -n mysite -c ./examples/content -t ./examples/templates -o ./examples/build

# With optional dev server directory
ssg -n mysite -c content -o build -t templates --serve public

# Load from a config file
ssg --config config.toml
```

### CLI options

| Option | Short | Description | Required |
|--------|-------|-------------|----------|
| `--new` | `-n` | Project name | No |
| `--content` | `-c` | Content directory | No |
| `--output` | `-o` | Output/build directory | No |
| `--template` | `-t` | Template directory | No |
| `--serve` | `-s` | Dev server directory | No |
| `--config` | `-f` | Config file path (TOML) | No |
| `--watch` | `-w` | Watch for changes | No |

When no flags are provided, sensible defaults are used (`content/`, `public/`, `templates/`).

## Examples

```bash
git clone https://github.com/sebastienrousseau/shokunin.git
cd shokunin

# Basic example — convert Markdown to a static site
cargo run --example basic

# Quick start — create, compile, and host a static site
cargo run --example quickstart

# Multilingual — build language-specific sites from a single source
cargo run --example multilingual
```

## Development

```bash
# Build
make build        # or: cargo build

# Run tests
make test         # or: cargo test

# Lint
make lint         # or: cargo clippy --all-targets

# Format
make format       # or: cargo fmt --all

# Security audit
make deny         # or: cargo deny check
```

## Documentation

- [API Reference (docs.rs)][04]
- [Website][00]
- [Contributing Guidelines][05]

## Contributing

We welcome contributions. Please see our [Contributing Guidelines][05] for details.

All commits must be signed. See `CONTRIBUTING.md` for setup instructions.

## License

Dual-licensed under your choice of:

- [Apache License, Version 2.0][10]
- [MIT License][11]

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
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.33-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
