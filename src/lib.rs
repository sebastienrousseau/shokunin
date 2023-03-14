// Copyright © 2023 shokunin. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//! # A Fast and Flexible Static Site Generator written in Rust 🦀
//! [![shokunin](https://via.placeholder.com/1500x500.png/000000/FFFFFF?text=shokunin)](https://shokunin.one)
//!
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org)
//! [![Crates.io](https://img.shields.io/crates/v/shokunin.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/shokunin)
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.1-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/shokunin)
//! [![License](https://img.shields.io/crates/l/shokunin.svg?style=for-the-badge&color=007EC6&labelColor=03589B)](MIT OR Apache-2.0)
//!
//! ## Features
//!
//! - Serialization and deserialization of data structures to JSON format
//! - ...
//! - ...
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! shokunin = "0.0.1"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! ```
//!
#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc(
    html_favicon_url = "",
    html_logo_url = "",
    html_root_url = "https://docs.rs/shokunin"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

use std::error::Error;
use std::fs;
// use std::io;
use std::path::Path;

mod file;
mod frontmatter;
mod html;
mod template;

use file::{add_files, File};
use frontmatter::extract_front_matter;
use html::{generate_html, generate_meta_tags};
use template::render_page;

#[allow(non_camel_case_types)]
/// This is the main entry point for the my_library library.
pub fn run() -> Result<(), Box<dyn Error>> {
    // Constants
    let src_dir = Path::new("content");
    let out_dir = Path::new("public");

    // Delete the output directory
    println!("Deleting old files...");
    fs::remove_dir_all(&out_dir)?;
    println!("Done.");

    // Create the output directory
    println!("Creating output directory...");
    fs::create_dir(&out_dir)?;
    println!("Done.");

    // Read the files in the source directory
    println!("Reading files...");
    let files = add_files(&src_dir)?;
    println!("Found {} files.", files.len());

    // Compile the files
    println!("Compiling files...");
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

    println!("Done.");

    // Write the compiled files to the output directory
    println!("Writing files...");
    for file in &files_compiled {
        let out_file = out_dir.join(file.name.replace(".md", ".html"));
        fs::write(&out_file, &file.content)?;
        println!("Wrote file: {}", out_file.display());
    }
    println!("Done.");

    // Write the index file
    println!("Writing index...");
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
    println!("Done.");

    // Done
    println!("All done!");
    Ok(())
}
