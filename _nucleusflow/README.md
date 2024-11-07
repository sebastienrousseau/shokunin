<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/shokunin/images/logos/shokunin.svg"
alt="Shokunin logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# Shokunin Static Site Generator CLI (nucleusflow)

A Rust-based Command Line Interface for the Shokunin Static Site Generator.

[![Made With Love][made-with-rust]][14] [![Crates.io][crates-badge]][8] [![Lib.rs][libs-badge]][10] [![Docs.rs][docs-badge]][9] [![License][license-badge]][2]

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[Website][1] | [Documentation][9] | [Report Bug][4] | [Request Feature][4] | [Contributing Guidelines][5]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

The `nucleusflow` is a powerful and flexible command-line interface for the Shokunin Static Site Generator. It provides a user-friendly way to create, manage, and deploy static websites using the Shokunin framework.

## Features

- **Project Creation**: Easily create new Shokunin projects with predefined structures.
- **Content Management**: Manage your content, templates, and static assets efficiently.
- **Build Process**: Compile your static site with a single command.
- **Development Server**: Run a local development server for real-time preview.
- **Flexible Configuration**: Customize your build process through command-line options.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nucleusflow = "0.0.1"
```

## Usage

Here are some examples of how to use the `nucleusflow`:

### Basic Usage

```bash
# Create a new Shokunin project
ssg --new my-site

# Build the site
ssg --content content --output public --template templates

# Serve the site locally
ssg --serve public
```

### CLI Options

- `--new <NAME>`: Create a new project
- `--content <DIR>`: Specify the content directory
- `--output <DIR>`: Specify the output directory
- `--template <DIR>`: Specify the template directory
- `--serve <DIR>`: Serve the specified directory

## Documentation

For full API documentation, please visit [docs.rs/nucleusflow][9].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

Special thanks to all the contributors who have helped shape the Shokunin Static Site Generator.

[1]: https://shokunin.one
[2]: https://opensource.org/licenses/MIT
[4]: https://github.com/sebastienrousseau/shokunin/issues
[5]: https://github.com/sebastienrousseau/shokunin/blob/main/CONTRIBUTING.md
[8]: https://crates.io/crates/nucleusflow
[9]: https://docs.rs/nucleusflow
[10]: https://lib.rs/crates/nucleusflow
[14]: https://www.rust-lang.org

[crates-badge]: https://img.shields.io/crates/v/nucleusflow.svg?style=for-the-badge 'Crates.io badge'
[docs-badge]: https://img.shields.io/docsrs/nucleusflow.svg?style=for-the-badge 'Docs.rs badge'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge 'Lib.rs badge'
[license-badge]: https://img.shields.io/crates/l/nucleusflow.svg?style=for-the-badge 'License badge'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust badge'
