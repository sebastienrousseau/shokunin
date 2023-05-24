<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/shokunin/images/logos/shokunin.svg" alt="Logo of the Shokunin (職人) Static Site Generator" width="240" height="240" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Shokunin (職人) Static Site Generator

A Fast and Flexible Static Site Generator written in Rust 🦀

![Banner of the Shokunin (職人) Static Site Generator][banner]

[![Made With Rust][made-with-rust-badge]][5] [![Crates.io][crates-badge]][7] [![Lib.rs][libs-badge]][9] [![Docs.rs][docs-badge]][8] [![License][license-badge]][2] [![Codecov][codecov-badge]][14]

<!-- markdownlint-disable MD033 -->
<center>

**[Website][0]
• [Documentation][8]
• [Report Bug][3]
• [Request Feature][3]
• [Contributing Guidelines][4]**

</center>

<!-- markdownlint-enable MD033 -->

## Welcome to Shokunin (職人) 👋

## Overview 📖

`Shokunin (職人) Static Site Generator` is a highly-optimized, Rust-based static site generator
(ssg) that aims to provide an easy-to-use and powerful tool for building
professional static websites and blogs.

The library extracts metadata and content to generate static HTML files
from Markdown, YAML, JSON, and TOML. It also supports HTML themes and
custom templates to help you create high quality websites with ease.

## Features ✨

- Blazing fast and flexible
- Easy to use
- Written in Rust 🦀
- Supports multiple content formats (Markdown, YAML, JSON, TOML)
- Compatible with various HTML themes and Premium templates to create
  accessible websites quickly and efficiently
- Generates minified HTML and JSON versions for optimal performance
- Built-in Rust development server with live reloading

## Getting Started 🚀

It takes just a few minutes to get up and running with `Shokunin (職人) Static Site Generator`.

### Installation

To install `Shokunin (職人) Static Site Generator`, you need to have the Rust toolchain
installed on your machine. You can install the Rust toolchain by
following the instructions on the [Rust website][13].

Once you have the Rust toolchain installed, you can install
`Shokunin (職人) Static Site Generator` using the following command:

```shell
cargo install ssg
```

For simplicity, we have given `Shokunin (職人) Static Site Generator` a simple alias `ssg`
which can stand for `Shokunin (職人) Site Generator` or
`Static Site Generator`.

You can then run the help command to see the available options and
commands:

```shell
ssg --help
```

### Requirements

The minimum supported Rust toolchain version is currently Rust
**1.69.0** or later (stable). It is recommended that you install the
latest stable version of Rust.

### Platform support

`Shokunin (職人) Static Site Generator` is supported and tested on the following platforms:

### Tier 1 platforms 🏆

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

### Tier 2 platforms 🥈

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

The [GitHub Actions][10] shows the platforms in which the
`Shokunin (職人) Static Site Generator` library tests are run.

### Documentation

> ℹ️ **Info:** Please check out our [website][0] for more information.
You can find our documentation on [docs.rs][8], [lib.rs][9] and
[crates.io][7].

## Usage 📖

### Command Line Interface (CLI)

The `Shokunin (職人) Static Site Generator` library runs in a Terminal window and can be used
to generate a static website.

Here’s the first command you can enter in your Terminal window to run
`Shokunin (職人) Static Site Generator`:

```shell
ssg  --new=mysite --content=content --output=output --template=template
```

This command will create a new `mysite` project in a directory called
`public/mysite` and generate a static website in the `mysite`
directory.

To run with the built-in Rust development server, you can use the
following command:

```shell
ssg  --new=mysite --content=content --output=output --template=template --serve=mysite
```

### In your project

To use the `Shokunin (職人) Static Site Generator` library in your project, add the following
to your `Cargo.toml` file:

```toml
[dependencies]
shokunin = "0.0.12"
```

Add the following to your `main.rs` file:

```rust
extern crate ssg;
use ssg::*;
```

then you can use the `Shokunin (職人) Static Site Generator` functions in your application
code.

### Examples

To get started with `Shokunin (職人) Static Site Generator`, you can use the examples
provided in the `examples` directory of the project.

To run the examples, clone the repository and run the following
command in your terminal from the project root directory.

```shell
cargo run --example example
```

## Semantic Versioning Policy 🚥

For transparency into our release cycle and in striving to maintain
backward compatibility, `Shokunin (職人) Static Site Generator` follows
[semantic versioning][6].

## License 📝

The project is licensed under the terms of both the MIT license and the
Apache License (Version 2.0).

- [Apache License, Version 2.0][1]
- [MIT license][2]

## Contribution 🤝

We welcome all people who want to contribute. Please see the
[contributing instructions][4] for more information.

Contributions in any form (issues, pull requests, etc.) to this project
must adhere to the [Rust's Code of Conduct][11].

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Acknowledgements 💙

A big thank you to all the awesome contributors of [Shokunin (職人)][5]
for their help and support.

A special thank you goes to the [Rust Reddit][12] community for
providing a lot of useful suggestions on how to improve this project.

[0]: https://shokunin.one
[1]: https://opensource.org/license/apache-2-0/
[2]: http://opensource.org/licenses/MIT
[3]: https://github.com/sebastienrousseau/shokunin/issues
[4]: https://github.com/sebastienrousseau/shokunin/blob/main/CONTRIBUTING.md
[5]: https://github.com/sebastienrousseau/shokunin/graphs/contributors
[6]: http://semver.org/
[7]: https://crates.io/crates/ssg
[8]: https://docs.rs/crate/ssg/
[9]: https://lib.rs/crates/ssg
[10]: https://github.com/sebastienrousseau/shokunin/actions
[11]: https://www.rust-lang.org/policies/code-of-conduct
[12]: https://www.reddit.com/r/rust/
[13]: https://www.rust-lang.org/learn/get-started
[14]: https://codecov.io/github/sebastienrousseau/shokunin?branch=main

[banner]: https://kura.pro/shokunin/images/titles/title-shokunin.svg "Banner of the Shokunin (職人) Static Site Generator"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/shokunin?style=for-the-badge&token=wAcpid8YEt 'Codecov'

[crates-badge]: https://img.shields.io/crates/v/ssg.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/ssg.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.12-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/ssg.svg?style=for-the-badge 'License badge'
[made-with-rust-badge]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
