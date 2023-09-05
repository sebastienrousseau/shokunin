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

# Shokunin Static Site Generator

A Fast and Flexible open-source static site generator (ssg) written in Rust 🦀

*Part of the [Mini Functions][0] family of libraries.*

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

![Banner of the Shokunin Static Site Generator][banner]

[![Made With Rust][made-with-rust-badge]][14] [![Crates.io][crates-badge]][8] [![Lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][9] [![License][license-badge]][3] [![Codecov][codecov-badge]][15]

• [Website][1] • [Documentation][9] • [Report Bug][4] • [Request Feature][4] • [Contributing Guidelines][5]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

![divider][divider]

## Overview 📖

`Shokunin Static Site Generator` is a highly-optimized, Rust-based static site generator (ssg) that aims to provide an easy-to-use and powerful tool for building professional static websites and blogs.

The library extracts metadata and content to generate static HTML files from Markdown, YAML, JSON, and TOML. It also supports HTML themes and custom templates to help you create high quality websites with ease.

## Features ✨

`Shokunin Static Site Generator` feature highlights include:

- Blazing fast and flexible static site generator written in Rust 🦀
- Built-in support for [GitHub Flavoured Markdown][12] (GFM).
- Built-in support for Google Analytics and Bing Analytics.
- Compatible with various HTML themes and Premium templates.
- Generates Atom and RSS feeds for your blog posts.
- Generates minified versions for optimal performance and SEO.
- Includes a built-in Rust development server for local development and testing.
- Supports multiple content formats, including Markdown, YAML, JSON, TOML, XML, etc.
- Built-in support for sitemap generation, robots.txt generation, canonical name (CNAME) records and custom 404 pages.

## Table of Contents 📚

- [Shokunin Static Site Generator](#shokunin-static-site-generator)
  - [Overview 📖](#overview-)
  - [Features ✨](#features-)
  - [Table of Contents 📚](#table-of-contents-)
  - [Getting Started 🚀](#getting-started-)
    - [Installation](#installation)
    - [Requirements](#requirements)
    - [Platform support](#platform-support)
      - [Tier 1 platforms](#tier-1-platforms)
      - [Tier 2 platforms](#tier-2-platforms)
    - [Documentation](#documentation)
  - [Usage 📖](#usage-)
    - [Command Line Interface (CLI)](#command-line-interface-cli)
      - [Arguments](#arguments)
    - [In your project](#in-your-project)
    - [Examples](#examples)
      - [Args](#args)
      - [Return value](#return-value)
  - [Semantic Versioning Policy 🚥](#semantic-versioning-policy-)
  - [License 📝](#license-)
  - [Contribution 🤝](#contribution-)
  - [Acknowledgements 💙](#acknowledgements-)

## Getting Started 🚀

It takes just a few minutes to get up and running with `Shokunin Static Site Generator`.

### Installation

To install `Shokunin Static Site Generator`, you need to have the Rust toolchain installed on your machine. You can install the Rust toolchain by following the instructions on the [Rust website][14].

Once you have the Rust toolchain installed, you can install `Shokunin Static Site Generator` using the following command:

```shell
cargo install ssg
```

For simplicity, we have given `Shokunin Static Site Generator` a simple alias `ssg` which can stand for `Shokunin Site Generator` or `Static Site Generator`.

You can then run the help command to see the available options and commands:

```shell
ssg --help
```

### Requirements

The minimum supported Rust toolchain version is currently Rust
**1.72.0** or later (stable). It is recommended that you install the
latest stable version of Rust.

### Platform support

`Shokunin Static Site Generator` is supported and tested on the following platforms:

#### Tier 1 platforms

| | Operating System | Target | Description |
| --- | --- | --- | --- |
| ✅ | Linux   | aarch64-unknown-linux-gnu | 64-bit Linux systems on ARM architecture |
| ✅ | Linux   | i686-unknown-linux-gnu | 32-bit Linux (kernel 3.2+, glibc 2.17+) |
| ✅ | Linux   | x86_64-unknown-linux-gnu | 64-bit Linux (kernel 2.6.32+, glibc 2.11+) |
| ✅ | macOS   | x86_64-apple-darwin | 64-bit macOS (10.7 Lion or later) |
| ✅ | Windows | i686-pc-windows-gnu | 32-bit Windows (7 or later) |
| ✅ | Windows | i686-pc-windows-msvc | 32-bit Windows (7 or later) |
| ✅ | Windows | x86_64-pc-windows-gnu | 64-bit Windows (7 or later) |
| ✅ | Windows | x86_64-pc-windows-msvc | 64-bit Windows (7 or later) |

#### Tier 2 platforms

| | Operating System | Target | Description |
| --- | --- | --- | --- |
| ✅ | Linux   | aarch64-unknown-linux-musl | 64-bit Linux systems on ARM architecture |
| ✅ | Linux   | arm-unknown-linux-gnueabi | ARMv6 Linux (kernel 3.2, glibc 2.17) |
| ✅ | Linux   | arm-unknown-linux-gnueabihf | ARMv7 Linux, hardfloat (kernel 3.2, glibc 2.17) |
| ✅ | Linux   | armv7-unknown-linux-gnueabihf | ARMv7 Linux, hardfloat (kernel 3.2, glibc 2.17) |
| ✅ | Linux   | mips-unknown-linux-gnu | MIPS Linux (kernel 2.6.32+, glibc 2.11+) |
| ✅ | Linux   | mips64-unknown-linux-gnuabi64 | MIPS64 Linux (kernel 2.6.32+, glibc 2.11+) |
| ✅ | Linux   | mips64el-unknown-linux-gnuabi64 | MIPS64 Linux (kernel 2.6.32+, glibc 2.11+) |
| ✅ | Linux   | mipsel-unknown-linux-gnu | MIPS Linux (kernel 2.6.32+, glibc 2.11+) |
| ✅ | macOS   | aarch64-apple-darwin | 64-bit macOS (10.7 Lion or later) |
| ✅ | Windows | aarch64-pc-windows-msvc | 64-bit Windows (7 or later) |

The [GitHub Actions][11] shows the platforms in which the `Shokunin Static Site Generator` library tests are run.

### Documentation

> ℹ️ **Info:** Please check out our [website][1] for more information.
You can find our documentation on [docs.rs][9], [lib.rs][10] and [crates.io][8].

## Usage 📖

### Command Line Interface (CLI)

The `Shokunin Static Site Generator` library runs in a Terminal window and can be used to generate a static website.

Here’s the first command you can enter in your Terminal window to run `Shokunin Static Site Generator`:

```shell
ssg  --new=mysite --content=content --template=template --output=output --serve=public
```

or

```shell
ssg  -n=mysite -c=content -t=template -o=output -s=public
```

This command will create a new website with the name `mysite` in the current directory. It will use the `content` directory to gather the website content and the `template` directory to generate the website files. It will serve the website directly from the `mysite` directory.

#### Arguments

- `-n`, `--new`: The name of the new website. (required)
- `-c`, `--content`: The directory containing the website content. (required)
- `-t`, `--template`: The directory containing the website templates. (required)
- `-o`, `--output`: The directory where the generated website files will be saved temporarily. (required)
- `-s`, `--serve`: Run the development server. (optional). The directory from which the website will be served.

### In your project

To use the `Shokunin Static Site Generator` library in your project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
shokunin = "0.0.16"
```

Add the following to your `main.rs` file:

```rust
extern crate ssg;
use ssg::*;
```

then you can use the `Shokunin Static Site Generator` functions in your application code.

### Examples

To get started with `Shokunin Static Site Generator`, you can use the examples provided in the `examples` directory of the project.

To run the examples, clone the repository and run the following command in your terminal from the project root directory.

```shell
cargo run --example example
```

The command will generate a static website based on the configuration details in the `examples` directory.

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

The main() function in this code compiles a website from the `content` directory, using the `template` directory to generate the website files. The compiled website is saved in the `build` directory and served directly from the `example.com` directory.

#### Args

- `build_path:` The path to the directory where the compiled website will be saved.
- `content_path:` The path to the directory containing the website content.
- `site_path:` The path to the directory where the generated website files will be served from.
- `template_path:` The path to the directory containing the website templates.

#### Return value

The main() function returns a Result. If the compilation is successful, the Result will be Ok(()). If there is an error, the Result will be Err(e), where e is a Box<dyn std::error::Error>.

## Semantic Versioning Policy 🚥

For transparency into our release cycle and in striving to maintain backward compatibility, `Shokunin Static Site Generator` follows [semantic versioning][7].

## License 📝

The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][2]
- [MIT license][3]

## Contribution 🤝

We welcome all people who want to contribute. Please see the [contributing instructions][5] for more information.

Contributions in any form (issues, pull requests, etc.) to this project must adhere to the [Rust's Code of Conduct][16].

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgements 💙

A big thank you to all the awesome contributors of [Shokunin][6] for their help and support.

A special thank you goes to the [Rust Reddit][13] community for providing a lot of useful suggestions on how to improve this project.

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
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.16-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg.svg?style=for-the-badge 'License badge'
[made-with-rust-badge]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
