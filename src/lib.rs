// Copyright © 2023 shokunin. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//!<!-- markdownlint-disable MD033 MD041 -->
//!
//!<img src="https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/icon/ico-shokunin.svg" alt="shokunin logo" width="240" align="right" />
//!
//!<!-- markdownlint-enable MD033 MD041 -->
//!
//! # A Fast and Flexible Static Site Generator written in Rust 🦀
//! [![shokunin](https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/title/title-shokunin.svg)](https://shokunin.one)
//!
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org)
//! [![Crates.io](https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/ssg)
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.3-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/ssg)
//! [![License](https://img.shields.io/crates/l/ssg.svg?style=for-the-badge&color=007EC6&labelColor=03589B)](https://opensource.org/license/apache-2-0/)
//!
//! ## Overview 📖
//!
//! `Shokunin (職人)` is a fast and flexible static site generator (ssg) written in Rust. It aims to provide an easy-to-use and powerful tool for building static websites.
//!
//! ## Features ✨
//!
//! - Fast and flexible
//! - Easy to use
//! - Written in Rust
//! - Supports templates and themes
//! - Generates optimized HTML, CSS, and JavaScript
//! - Built-in development server
//! - Live reloading
//! - Markdown support
//!
//! ## Getting Started 🚀
//!
//! It takes just a few minutes to get up and running with `shokunin`.
//!
//! ### Installation
//!
//! To install `shokunin`, you need to have the Rust toolchain installed on
//! your machine. You can install the Rust toolchain by following the
//! instructions on the [Rust website](https://www.rust-lang.org/learn/get-started).
//!
//! Once you have the Rust toolchain installed, you can install `shokunin`
//! using the following command:
//!
//! ```shell
//! cargo install ssg
//! ```
//!
//! For simplicity, we have given `shokunin` a simple alias `ssg` which can
//! stand for `Shokunin Site Generator` or `Static Site Generator`.
//!
//! You can then run the help command to see the available options:
//!
//! ```shell
//! ssg --help
//! ```
//!
#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/icon/ico-shokunin.svg",
    html_logo_url = "https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/icon/ico-shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

use std::error::Error;
use std::fs;
use std::path::Path;

/// The `args` module contains functions for processing command-line
/// arguments.
pub mod args;
/// The `cli` module contains functions for processing command-line
/// input.
pub mod cli;
/// File module handles file reading and writing.
mod file;
/// Frontmatter module extracts metadata from files.
mod frontmatter;
/// HTML module generates HTML content.
mod html;
/// Template module renders pages with metadata.
mod template;

use file::{add_files, File};
use frontmatter::extract_front_matter;
use html::{generate_html, generate_meta_tags};
use template::render_page;

#[allow(non_camel_case_types)]

/// run() is the main function of the program. It reads files from
pub fn run() -> Result<(), Box<dyn Error>> {
    let title = "Shokunin (職人) 🦀 (v0.0.3)";
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";
    let width = title.len().max(description.len()) + 4;
    let horizontal_line = "─".repeat(width - 2);

    println!("┌{}┐", horizontal_line);
    println!("│{: ^width$}│", title, width = width - 5);
    println!("├{}┤", horizontal_line);
    println!("│{: ^width$}│", description, width = width - 2);
    println!("└{}┘", horizontal_line);

    let result = match cli::build_cli() {
        Ok(matches) => {
            args::process_arguments(&matches)?;
            Ok(())
        }
        Err(e) => Err(format!("❌ Error: {}", e)),
    };

    match result {
        Ok(_) => println!("\n✅ All Done"),
        Err(e) => println!("{}", e),
    }

    // Print the welcome message if no arguments were passed
    if std::env::args().len() == 1 {
        eprintln!(
            "\n\nWelcome to Shokunin (職人) 🦀\n\nLet's get started! Please, run `ssg --help` for more information.\n"
        );
    }

    Ok(())
}

/// create_new_project() is the main function of the program. It
/// reads files from the
/// source directory, compiles them, and writes them to the output
/// directory.
///
pub fn create_new_project(
    src_dir: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    // Constants
    let src_dir = Path::new(src_dir);
    let out_dir = Path::new(out_dir);

    // Delete the output directory
    println!("\n❯ Deleting old files...");
    fs::remove_dir_all(&out_dir)?;
    println!("  Done.\n");

    // Create the output directory
    println!("❯ Creating output directory...");
    fs::create_dir(&out_dir)?;
    println!("  Done.\n");

    // Read the files in the source directory
    println!("❯ Reading files...");
    let files = add_files(&src_dir)?;
    println!("  Found {} files.\n", files.len());

    // Compile the files
    println!("❯ Compiling files...");
    let files_compiled: Vec<File> = files
        .into_iter()
        .map(|file| {
            // Extract metadata from front matter
            let (title, description, keywords, permalink) = extract_front_matter(&file.content);
            let meta = generate_meta_tags(&[("url".to_owned(), permalink)]);

            let content = render_page(
                &title,
                &description,
                &keywords,
                &meta,
                "style.css",
                &generate_html(&file.content, &title, &description),
                "Copyright © 2022-2023 My Company. All rights reserved.",
            )
            .unwrap();

            File {
                name: file.name,
                content,
            }
        })
        .collect();

    println!("  Done.\n");

    // Write the compiled files to the output directory
    println!("❯ Writing files...");
    for file in &files_compiled {
        let out_file = out_dir.join(file.name.replace(".md", ".html"));
        fs::write(&out_file, &file.content)?;
        println!("  Wrote file: {}", out_file.display());
    }
    println!("  Done.\n");

    // Write the index file
    println!("❯ Writing index...");
    let index = format!(
        "<ul>\n{}\n</ul>",
        files_compiled
            .iter()
            .map(|file| {
                format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    file.name.replace(".md", ".html"),
                    file.name.replace(".md", "")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    let index_file = out_dir.join("index.html");
    fs::write(&index_file, index)?;

    // Done
    Ok(())
}
