// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//! # Shokunin Static Site Generator
//!
//! [![Shokunin Static Site Generator Logo](https://kura.pro/shokunin/images/banners/banner-shokunin.svg)](https://shokunin.one "Shokunin - A Fast and Flexible Static Site Generator written in Rust")
//!
//! A Fast and Flexible open-source static site generator (ssg) written in Rust 🦀
//!
//! *Part of the [Mini Functions][0] family of libraries.*
//!
//! [![Crates.io](https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/ssg "Crates.io")
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.17-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/ssg "Lib.rs")
//! [![License](https://img.shields.io/crates/l/ssg.svg?style=for-the-badge&color=007EC6&labelColor=03589B)](https://opensource.org/license/apache-2-0/ "MIT or Apache License, Version 2.0")
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org "Rust")
//!
//! ## Overview 📖
//!
//! `Shokunin Static Site Generator` is a highly-optimized, Rust-based static site generator (ssg) that aims to provide an easy-to-use and powerful tool for building professional static websites and blogs.
//!
//! The library extracts metadata and content to generate static HTML files from Markdown, YAML, JSON, and TOML. It also supports HTML themes and custom templates to help you create high quality websites with ease.
//!
//! ## Features ✨
//!
//! `Shokunin Static Site Generator` feature highlights include:
//!
//! - Blazing fast and flexible static site generator written in Rust 🦀
//! - Built-in support for [GitHub Flavoured Markdown][1] (GFM).
//! - Built-in support for Google Analytics and Bing Analytics.
//! - Compatible with various HTML themes and Premium templates.
//! - Generates Atom and RSS feeds for your blog posts.
//! - Generates minified versions for optimal performance and SEO.
//! - Includes a built-in Rust development server for local development and testing.
//! - Supports multiple content formats, including Markdown, YAML, JSON, TOML, XML, etc.
//! - Built-in support for sitemap generation, robots.txt generation, canonical name (CNAME) records and custom 404 pages.
//!
//! ## Usage 📖
//!
//! ### Command Line Interface (CLI)
//!
//! The `Shokunin Static Site Generator` library runs in a Terminal window and can be used to generate a static website.
//!
//! Here’s the first command you can enter in your Terminal window to run `Shokunin Static Site Generator`:
//!
//! ```shell
//! ssg  --new=mysite --content=content --template=template --output=output --serve=public
//! ```
//!
//! or
//!
//! ```shell
//! ssg  -n=mysite -c=content -t=template -o=output -s=public
//! ```
//!
//! This command will create a new website with the name `mysite` in the current directory. It will use the `content` directory to gather the website content and the `template` directory to generate the website files. It will serve the website directly from the `mysite` directory.
//!
//! #### Arguments
//!
//! - `-n`, `--new`: The name of the new website. (required)
//! - `-c`, `--content`: The directory containing the website content. (required)
//! - `-t`, `--template`: The directory containing the website templates. (required)
//! - `-o`, `--output`: The directory where the generated website files will be saved temporarily. (required)
//! - `-s`, `--serve`: Run the development server. (optional). The directory from which the website will be served.
//!
//! ### In your project
//!
//! To use the `Shokunin Static Site Generator` library in your project, add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! shokunin = "0.0.17"
//! ```
//!
//! Add the following to your `main.rs` file:
//!
//! ```rust
//! use ssg::compiler::compile;
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Define the paths to the build, site, source and template directories.
//!     // let build_path = Path::new("examples/example.com/build");
//!     // let site_path = Path::new("examples/example.com/public");
//!     // let content_path = Path::new("examples/example.com/contents");
//!     // let template_path = Path::new("examples/example.com/templates");
//!
//!     // compile(build_path, content_path, site_path, template_path)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! then you can use the `Shokunin Static Site Generator` functions in your application code.
//!
//! ### Examples
//!
//! To get started with `Shokunin Static Site Generator`, you can use the examples provided in the `examples` directory of the project.
//!
//! To run the examples, clone the repository and run the following command in your terminal from the project root directory.
//!
//! ```shell
//! cargo run --example example
//! ```
//!
//! The command will generate a static website based on the configuration details in the `examples` directory.
//!
//! ```shell
//! use ssg::compiler::compile;
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Define the paths to the build, site, source and template directories.
//!     let build_path = Path::new("examples/example.com/build");
//!     let site_path = Path::new("examples/example.com/public");
//!     let content_path = Path::new("examples/example.com/contents");
//!     let template_path = Path::new("examples/example.com/templates");
//!
//!     compile(build_path, content_path, site_path, template_path)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! The main() function in this code compiles a website from the `content` directory, using the `template` directory to generate the website files. The compiled website is saved in the `build` directory and served directly from the `example.com` directory.
//!
//! #### Args
//!
//! - `build_path:` The path to the directory where the compiled website will be saved.
//! - `content_path:` The path to the directory containing the website content.
//! - `site_path:` The path to the directory where the generated website files will be served from.
//! - `template_path:` The path to the directory containing the website templates.
//!
//! #### Return value
//!
//! The main() function returns a Result. If the compilation is successful, the Result will be Ok(()). If there is an error, the Result will be Err(e), where e is a `Box<dyn std::error::Error>`.
//!
//! ## License 📜
//!
//! The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).
//!
//! - [Apache License, Version 2.0](https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0")
//! - [MIT license](http://opensource.org/licenses/MIT "MIT license")
//!
//! [0]: https://minifunctions.com/ "MiniFunctions"
//! [1]: https://github.github.com/gfm/ "GitHub Flavoured Markdown"
//!

#![forbid(unsafe_code)]
#![forbid(unreachable_pub)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc(
    html_favicon_url = "https://kura.pro/shokunin/images/favicon.ico",
    html_logo_url = "https://kura.pro/shokunin/images/logos/shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

use crate::utilities::serve::start;
use cli::print_banner;
use compiler::compile;
use std::{error::Error, path::Path};


/// The `loggers` module contains functions for logging.
pub mod loggers;

/// The `cli` module contains functions for the command-line interface.
pub mod cli;
/// The `compiler` module contains functions for the compilation process.
pub mod compiler;
/// The `data` module contains the structs.
pub mod data;

/// The `macros` module contains functions for generating macros.
pub mod macros;

/// The `modules` module contains the application modules.
pub mod modules;
/// The `navigation` module generates the navigation menu.
pub mod navigation;
/// The `parser` module contains functions for parsing command-line
/// arguments and options.
pub mod process;
/// The `utilities` module contains utility functions.
pub mod utilities;


#[allow(non_camel_case_types)]

/// ## Function: `run` - Runs the static site generator command-line tool.
///
/// This function prints a banner containing the title and description of the tool,
/// and then processes any command-line arguments passed to it. If no
/// arguments are passed, it prints a welcome message and instructions
/// on how to use the tool.
///
/// The function uses the `build` function from the `cli` module to
/// create the command-line interface for the tool. It then processes
/// any arguments passed to it using the `parser` function from the
/// `args` module.
///
/// If any errors occur during the process (e.g. an invalid argument is
/// passed), an error message is printed and returned. Otherwise,
/// `Ok(())` is returned.
pub fn run() -> Result<(), Box<dyn Error>> {
    // Print the CLI banner and welcome message
    print_banner();

    // Build the CLI and parse the arguments
    let matches = cli::build()?;
    process::args(&matches)?;

    if let Some(site_name) = matches.get_one::<String>("new") {
        // Start the server using the specified server address and site name.
        // If an error occurs, propagate it up the call stack.
        macro_serve!("127.0.0.1:8000", site_name);
    }

    // Set the build, content, site and template paths for the compile function.
    let build_path = Path::new("public");
    let content_path = Path::new("content");
    let site_path = Path::new("site");
    let template_path = Path::new("templates");

    // Call the compile function with the above parameters to compile the site.
    compile(build_path, content_path, site_path, template_path)?;

    Ok(())
}
