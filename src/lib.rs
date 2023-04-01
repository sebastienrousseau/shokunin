// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//! # Shokunin 職人 🦀
//!
//! [![Shokunin](https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/logo/logo-shokunin.svg)](https://shokunin.one "Shokunin - A Fast and Flexible Static Site Generator written in Rust")
//!
//! A Fast and Flexible Static Site Generator written in Rust 🦀
//!
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org "Rust")
//! [![Crates.io](https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/ssg "Crates.io")
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.5-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/ssg "Lib.rs")
//! [![License](https://img.shields.io/crates/l/ssg.svg?style=for-the-badge&color=007EC6&labelColor=03589B)](https://opensource.org/license/apache-2-0/ "MIT or Apache License, Version 2.0")
//!
//! ## Overview 📖
//!
//! `Shokunin (職人)` is a highly-optimized, Rust-based static site
//! generator (ssg) that aims to provide an easy-to-use and powerful
//! tool for building professional static websites and blogs.
//!
//! The library extracts metadata and content to generate static HTML
//! files from Markdown, YAML, JSON, and TOML. It also supports HTML
//! themes and custom templates to help you create high quality
//! websites with ease.
//!
//! ## Features ✨
//!
//! - Blazing fast and flexible
//! - Easy to use
//! - Written in Rust 🦀
//! - Supports multiple content formats (Markdown, YAML, JSON, TOML)
//! - Compatible with various HTML themes and Premium templates to
//!   create accessible websites quickly and efficiently
//! - Generates minified HTML and JSON versions for optimal performance
//! - Built-in Rust development server with live reloading
//!
//! ## Getting Started 🚀
//!
//! It takes just a few minutes to get up and running with `Shokunin (職人)`.
//!
//! ### Installation
//!
//! To install `Shokunin (職人)`, you need to have the Rust toolchain installed on
//! your machine. You can install the Rust toolchain by following the
//! instructions on the [Rust website](https://www.rust-lang.org/learn/get-started).
//!
//! Once you have the Rust toolchain installed, you can install `Shokunin (職人)`
//! using the following command:
//!
//! ```shell
//! cargo install ssg
//! ```
//!
//! For simplicity, we have given `Shokunin (職人)` a simple alias `ssg` which can
//! stand for `Shokunin Site Generator` or `Static Site Generator`.
//!
//! You can then run the help command to see the available options:
//!
//! ```shell
//! ssg --help
//! ```
//!
//! ## Examples and Usage 📚
//!
//! Check out the examples folder for helpful snippets of code that
//! demonstrate how to use the `Shokunin (職人)` library. You can also check
//! out the [documentation](https://docs.rs/ssg) for more information
//! on how to use the library.
//!
//! ## License 📜
//!
//! The project is licensed under the terms of both the MIT license and
//! the Apache License (Version 2.0).
//!
//! - [Apache License, Version 2.0](https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0")
//! - [MIT license](http://opensource.org/licenses/MIT "MIT license")
//!

#![forbid(unsafe_code)]
#![forbid(unreachable_pub)]
#![forbid(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/icon/ico-shokunin.svg",
    html_logo_url = "https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/icon/ico-shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

use cli::print_banner;
use compiler::compile;
use std::{error::Error, path::Path};

/// The `cli` module contains functions for the command-line interface.
pub mod cli;
/// The `compiler` module contains functions for the compilation process.
pub mod compiler;
/// The `file` module handles file reading and writing operations.
pub mod file;
/// The `frontmatter` module extracts the front matter from files.
pub mod frontmatter;
/// The `html` module generates the HTML content.
pub mod html;
/// The `json` module generates the JSON content.
pub mod json;
/// The `metatags` module generates the meta tags.
pub mod metatags;
/// The `navigation` module generates the navigation menu.
pub mod navigation;
/// The `parser` module contains functions for parsing command-line
/// arguments and options.
pub mod parser;
/// The `rss` module generates the RSS content.
pub mod rss;
/// The `serve` module contains functions for the development server.
pub mod serve;
/// The `template` module renders the HTML content using the pre-defined
/// template.
pub mod template;
/// The `directory` function ensures that a directory exists.
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
    // Print the CLI banner
    print_banner();

    // Build the CLI
    let result = match cli::build() {
        // If CLI is built successfully, parse the arguments and check if the serve flag is set
        Ok(matches) => {
            parser::args(&matches)?;
            if matches.get_one::<String>("serve").is_some() {
                // If serve flag is set, start the server and return
                let server_address = "127.0.0.1:8000";
                let output_dir = matches.get_one::<String>("serve").unwrap();
                let document_root = format!("public/{}", output_dir);
                serve::start(server_address, &document_root)?;
                println!("\n✅ Server started at http://{}", server_address);
                return Ok(());
            }
            Ok(())
        }
        // If CLI build fails, print the error message
        Err(e) => Err(format!("❌ Error: {}", e)),
    };

    // Check the result of CLI build
    match result {
        // If successful, print success message
        Ok(_) => println!("\n✅ All Done"),
        // If failed, print error message
        Err(e) => println!("{}", e),
    }

    // Print the welcome message if no arguments were passed
    if std::env::args().len() == 1 {
        eprintln!("\n\nWelcome to Shokunin 職人 🦀\n\nLet's get started! Please, run `ssg --help` for more information.\n");
    }

    // Set the source and output directories, site name, and template path
    let src_dir = Path::new("src");
    let out_dir = Path::new("public");
    let site_name = String::from("My Site");
    let binding = String::from("templates");
    let template_path = Some(&binding);

    // Call the compile function with the above parameters
    compile(src_dir, out_dir, template_path, site_name)?;
    Ok(())
}
