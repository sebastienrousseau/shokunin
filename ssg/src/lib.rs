// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator (SSG)
//!
//! [![Shokunin Static Site Generator Logo](https://kura.pro/shokunin/images/banners/banner-shokunin.svg)](https://shokunin.one "Shokunin - A Fast and Flexible Static Site Generator written in Rust")
//!
//! ## A Content-First Open Source Static Site Generator (SSG) written in [Rust][2].
//!
//! *Part of the [Mini Functions][0] family of Rust libraries.*
//!
//! [![Crates.io](https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/ssg "Crates.io")
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.30-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/ssg "Lib.rs")
//! [![License](https://img.shields.io/crates/l/ssg.svg?style=for-the-badge&color=007EC6&labelColor=03589B)](https://opensource.org/license/apache-2-0/ "MIT or Apache License, Version 2.0")
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org "Rust")
//!
//! ## Overview
//!
//! Discover Shokunin: The high-performance, Rust-backed Static Site Generator (SSG) that puts content at the forefront of your web experience.
//!
//! ## Features
//!
//! Shokunin Static Site Generator (SSG) has several notable features, including but not limited to:
//!
//! - **Speed and Flexibility:** Built in Rust, offering optimal performance.
//! - **Built-in Supports:**
//!     - GitHub Flavoured Markdown (GFM) for intuitive content creation.
//!     - Integrated support for Google Analytics and Bing Analytics.
//!     - Automated sitemap generation, robots.txt, canonical name (CNAME) records, and custom 404 pages.
//! - **Compatibility:** Extensive support for various HTML themes and Premium templates.
//! - **Advanced Features:**
//!     - Atom and RSS feeds for blog posts, offering greater discoverability.
//!     - Minified HTML, CSS, and JavaScript files for better performance and SEO.
//! - **Development Server:** Comes with a Rust-based local development server for easier debugging and testing.
//! - **Format Support:** Comprehensive format support including Markdown, YAML, JSON, TOML, XML, etc.
//!
//! ## Usage
//!
//! ### Command Line Interface (CLI)
//!
//! The CLI is straightforward. Below are examples to guide you:
//!
//! ```shell
//! # Create a new site named docs
//! ssg  --new=docs --content=content --template=template --output=output --serve=public
//! ```
//!
//! or
//!
//! ```shell
//! # Alternative shorter command
//! ssg  -n=docs -c=content -t=template -o=output -s=public
//! ```
//!
//! **Arguments Explained:**
//!
//! - `-n`, `--new`: Name of the new site to be created. (e.g., `--new=docs`). Defaults to `docs` which allows you to publish your site to GitHub Pages.
//! - `-c`, `--content`: Directory containing the website content. (e.g., `--content=content`)
//! - `-t`, `--template`: Directory containing website templates. (e.g., `--template=templates`)
//! - `-o`, `--output`: Directory where generated website files will be saved temporarily. (e.g., `--output=build`)
//! - `-s`, `--serve`: (Optional) Directory from which the website will be served. (e.g., `--serve=public`)
//!
//! ### In your project
//!
//! To incorporate Shokunin Static Site Generator (SSG) in your Rust project, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! shokunin = "0.0.30"
//! ```
//!
//! And in your `main.rs`:
//!
//! ```rust
//! use ssg_core::compiler::service::compile;
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Uncomment and replace these paths with your directory paths
//!     // let build_path = Path::new("your_build_directory");
//!     // let site_path = Path::new("your_site_directory");
//!     // let content_path = Path::new("your_content_directory");
//!     // let template_path = Path::new("your_template_directory");
//!
//!     // compile(build_path, content_path, site_path, template_path)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Contributing
//! We welcome contributions! Please see [CONTRIBUTING.md][3] for details on how to contribute.
//!
//! ## License
//!
//! This project is dual-licensed under the terms of both the MIT license and the Apache License (Version 2.0).
//!
//! - [Apache License, Version 2.0](https://opensource.org/license/apache-2-0/ "Apache License, Version 2.0")
//! - [MIT license](http://opensource.org/licenses/MIT "MIT license")
//!
//! [0]: https://minifunctions.com/ "MiniFunctions"
//! [1]: https://github.github.com/gfm/ "GitHub Flavoured Markdown"
//! [2]: https://www.rust-lang.org/ "Rust"
//! [3]: https://shokunin.one/contribute/index.html "Contribute to Shokunin"
#![doc(
    html_favicon_url = "https://kura.pro/shokunin/images/favicon.ico",
    html_logo_url = "https://kura.pro/shokunin/images/logos/shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

// Re-export ssg-core
pub use ssg_core;

// Re-export ssg-cli
pub use ssg_cli;

use anyhow::Result;
use dtt::datetime::DateTime;
use http_handle::Server;
use rlg::{log_format::LogFormat, log_level::LogLevel, macro_log};
use ssg_cli::cli::print_banner;
use ssg_core::macro_serve;
use ssg_core::{
    compiler::service::compile, loggers::init_logger,
    utilities::uuid::generate_unique_string,
};
use ssg_i18n::languages::en::translate;
use std::{fs::File, io::Write, path::Path};

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
/// Run the static site generator command-line tool.
pub fn run() -> Result<()> {
    // Initialize the logger using the `env_logger` crate
    init_logger(None)?;

    // Get the current date and time
    let date = DateTime::new();
    // let iso = DateTime::new();

    // Open or create the log file
    let mut log_file = create_log_file("./ssg.log")?;

    // Print the CLI banner and welcome message
    print_banner();

    // Generate a log entry for the banner
    let banner_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_banner_log_msg").unwrap(),
        &LogFormat::CLF
    );

    // Write the log to both the console and the file
    writeln!(log_file, "{}", banner_log)?;

    // Build the CLI and parse the arguments
    let matches = ssg_cli::cli::build().get_matches();
    ssg_cli::process::args(&matches)?;

    // Generate a log entry for the arguments
    let args_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_args_log_msg").unwrap(),
        &LogFormat::CLF
    );

    // Write the log to both the console and the file
    writeln!(log_file, "{}", args_log)?;

    if let Some(site_name) = matches.get_one::<String>("new") {
        // Generate a log entry for the server
        let server_log = macro_log!(
            &generate_unique_string(),
            &date.to_string(),
            &LogLevel::INFO,
            "process",
            &translate("lib_server_log_msg").unwrap(),
            &LogFormat::CLF
        );

        // Write the log to both the console and the file
        writeln!(log_file, "{}", server_log)?;

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

/// Create a log file at the specified path.
fn create_log_file(file_path: &str) -> Result<File> {
    Ok(File::create(file_path)?)
}
