# Shokunin Static Site Generator (SSG)

The fastest Rust-based Static Site Generator (SSG) for building professional
websites and blogs.

<!-- markdownlint-disable MD033 MD041 -->

<img
  align="right"
  alt="Logo of the Shokunin Static Site Generator"
  height="261"
  src="https://kura.pro/shokunin/images/logos/shokunin.svg"
  title="Logo of the Shokunin Static Site Generator"
  width="261"
  />

<!-- markdownlint-enable MD033 MD041 -->

*Part of the [Mini Functions][0] family of Rust libraries.*

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

![Banner of the Shokunin Static Site Generator][banner]

[![Made With Rust][made-with-rust-badge]][14]
[![Crates.io][crates-badge]][8]
[![Lib.rs][libs-badge]][10]
[![Docs.rs][docs-badge]][9]
[![License][license-badge]][3]
[![Codecov][codecov-badge]][15]

• [Website][1]
• [Documentation][9]
• [Report Bug][4]
• [Request Feature][4]
• [Contributing Guidelines][5]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

![divider][divider]

## Overview

Shokunin is a lightning-fast static site generator (SSG) that is optimised for
Search Engine Optimisation (SEO) and fully aligned with Accessibility Standards.

The library extracts metadata and content to generate static HTML files from
Markdown, YAML, JSON, and TOML. It also supports HTML themes and custom
templates to help you create high quality websites with ease.

## Features

Shokunin Static Site Generator (SSG) feature highlights include:

- Blazing fast and flexible static site generator written in Rust
- Built-in support for [GitHub Flavoured Markdown][12] (GFM)
- Built-in support for Google Analytics and Bing Analytics
- Experimental support for PDF generation
- Compatible with various HTML themes and premium templates
- Generates Atom and RSS feeds for your blog posts automatically
- Generates minified HTML for optimal performance and SEO
- Includes a built-in Rust development server for local testing
- Supports multiple content formats:
  - Markdown
  - YAML
  - JSON
  - TOML
  - XML
- Built-in generation for:
  - Sitemaps
  - robots.txt
  - Canonical name (CNAME) records
  - Custom 404 pages
- Comprehensive documentation

## Table of Contents

- [Shokunin Static Site Generator (SSG)](#shokunin-static-site-generator-ssg)
  - [Overview](#overview)
  - [Features](#features)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
    - [Installation](#installation)
    - [Requirements](#requirements)
    - [Platform support](#platform-support)
    - [Documentation](#documentation)
  - [Usage](#usage)
    - [Command Line Interface (CLI)](#command-line-interface-cli)
      - [Arguments](#arguments)
    - [In your project](#in-your-project)
    - [Examples](#examples)
      - [Args](#args)
  - [Semantic Versioning Policy](#semantic-versioning-policy)
  - [License](#license)
  - [Contribution](#contribution)
  - [Acknowledgements](#acknowledgements)

## Getting Started

It takes just a few minutes to get up and running with Shokunin Static Site
Generator (SSG).

### Installation

To install Shokunin Static Site Generator (SSG), you need to have the Rust
toolchain installed on your machine. You can install the Rust toolchain by
following the instructions on the [Rust website][14].

Once you have the Rust toolchain installed, you can install Shokunin Static Site
Generator (SSG) using the following command:

```shell
cargo install ssg
```

For simplicity, we have given Shokunin Static Site Generator (SSG) a simple
alias `ssg` which can stand for `Shokunin Site Generator` or
`Static Site Generator`.

You can then run the help command to see the available options and commands:

```shell
ssg --help
```

### Requirements

The minimum supported Rust toolchain version is currently Rust **1.71.1** or
later (stable). It is recommended that you install the latest stable version of
Rust.

### Platform support

Shokunin Static Site Generator (SSG) is supported and tested on the following
platforms and architectures as part of our [CI/CD pipeline][11].

The [GitHub Actions][11] shows the platforms in which the Shokunin Static Site
Generator (SSG) library tests are run.

### Documentation

> ℹ️ **Info:** Please check out our [website][1] for more information.
You can find our documentation on [docs.rs][9], [lib.rs][10] and [crates.io][8].

## Usage

### Command Line Interface (CLI)

The Shokunin Static Site Generator (SSG) library runs in a Terminal window and
can be used to easily generate a static website. To get started, run:

```shell
ssg  --new=docs --content=content --template=template --output=output --serve=public
```

or

```shell
ssg  -n=docs -c=content -t=template -o=output -s=public
```

This creates a new website in a directory called `docs` using the markdown content from the `content` directory and the HTML templates from the `template` directory. The static and compiled HTML files and artefacts are then generated in a `docs` folder.

Shokunin is ideal for hosting your site on GitHub Pages. Simply commit and push the `docs` folder to your main branch, and set the GitHub Pages publishing source to point to that folder.

During development, you can use the `--serve` or `--s` option to start a local development server to preview content changes.

With Shokunin's GFM and theme support, you can focus on writing markdown content while the SSG handles delivering a fast, SEO-friendly site.

#### Arguments

- `-n`, `--new`: The name of the folder for your new website. (required)
- `-c`, `--content`: The directory containing the website markdown content. (required)
- `-t`, `--template`: The directory containing the HTML website templates.
  (required)
- `-o`, `--output`: The directory where the generated website files will be
  saved temporarily. (required)
- `-s`, `--serve`: Run the development server. (optional). The directory from
  which the website will be served. (optional)

### In your project

To use the Shokunin Static Site Generator (SSG) library in your project, add the
following to your `Cargo.toml` file:

```toml
[dependencies]
shokunin = "0.0.29"
```

Add the following to your `main.rs` file:

```rust
extern crate ssg;
use ssg::*;
```

then you can use the Shokunin Static Site Generator (SSG) functions in your
application code.

### Examples

To get started with Shokunin Static Site Generator (SSG), you can use the
examples provided in the `examples` directory of the project.

To run the examples, clone the repository and run the following command in your
terminal from the project root directory.

```shell
cargo run --example example
```

The command will generate a static website based on the configuration details
in the `examples` directory.

```shell
use ssg::compiler::compile;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the paths to the build, site, content and template directories.
    let build_path = Path::new("examples/example.com/build");
    let content_path = Path::new("examples/example.com/content");
    let site_path = Path::new("examples/example.com/public");
    let template_path = Path::new("examples/example.com/template");

    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}
```

The main() function in this code compiles a website from the `content`
directory, using the `template` directory to generate the website files. The
compiled website is saved in the `build` directory and served directly from
the `example.com` directory.

#### Args

- `build_path:` The path to the directory where the compiled website will be
saved.
- `content_path:` The path to the directory containing the website content.
- `site_path:` The path to the directory where the generated website files will
be served from.
- `template_path:` The path to the directory containing the website templates.

## Semantic Versioning Policy

For transparency into our release cycle and in striving to maintain backward
compatibility, Shokunin Static Site Generator (SSG) follows
[semantic versioning][7].

## License

The project is licensed under the terms of both the MIT license and the Apache
License (Version 2.0).

- [Apache License, Version 2.0][2]
- [MIT license][3]

## Contribution

We welcome all people who want to contribute. Please see the
[contributing instructions][5] for more information.

Contributions in any form (issues, pull requests, etc.) to this project must
adhere to the [Rust's Code of Conduct][16].

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Acknowledgements

A big thank you to all the awesome contributors of [Shokunin][6] for their help
and support.

A special thank you goes to the [Rust Reddit][13] community for providing a lot
of useful suggestions on how to improve this project.

[0]: https://minifunctions.com/ "The Rust Mini Functions"
[1]: https://shokunin.one "Shokunin Static Site Generator"
[2]: https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0"
[3]: http://opensource.org/licenses/MIT "MIT license"
[4]: https://github.com/sebastienrousseau/shokunin/issues "Issues"
[5]: https://github.com/sebastienrousseau/shokunin/blob/main/CONTRIBUTING.md "Contributing"
[6]: https://github.com/sebastienrousseau/shokunin/graphs/contributors "Contributors"
[7]: http://semver.org/ "Semantic Versioning"
[8]: https://crates.io/crates/ssg "Crate.io"
[9]: https://docs.rs/crate/ssg/ "Docs.rs"
[10]: https://lib.rs/crates/ssg "Lib.rs"
[11]: https://github.com/sebastienrousseau/shokunin/actions "Actions"
[12]: https://github.github.com/gfm/ "GitHub Flavoured Markdown"
[13]: https://www.reddit.com/r/rust/ "Rust Reddit"
[14]: https://www.rust-lang.org/learn/get-started "Rust"
[15]: https://codecov.io/github/sebastienrousseau/shokunin?branch=main "Codecov"
[16]: https://www.rust-lang.org/policies/code-of-conduct "Rust's Code of Conduct"

[banner]: https://kura.pro/shokunin/images/titles/title-shokunin.svg "Banner of the Shokunin Static Site Generator"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/shokunin?style=for-the-badge&token=wAcpid8YEt 'Codecov'

[crates-badge]: https://img.shields.io/crates/v/ssg.svg?style=for-the-badge 'Crates.io badge'
[divider]: https://kura.pro/common/images/elements/divider.svg "divider"
[docs-badge]: https://img.shields.io/docsrs/ssg.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.29-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg.svg?style=for-the-badge 'License badge'
[made-with-rust-badge]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
