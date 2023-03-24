// Copyright © 2023 shokunin. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//! # Shokunin (職人) 🦀
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
//! ## Examples and Usage 📚
//!
//! Check out the examples folder for helpful snippets of code that
//! demonstrate how to use the `shokunin` library. You can also check
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

use file::{add, File};
use frontmatter::extract;
use html::generate_html;
use json::manifest;
use metatags::generate_metatags;
use std::{error::Error, fs, path::Path};
use template::render_page;

use crate::json::ManifestOptions;
use crate::template::{create_template_folder, PageOptions};

/// The `cli` module contains functions for the command-line interface.
pub mod cli;
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
/// The `parser` module contains functions for parsing command-line
/// arguments and options.
pub mod parser;
/// The `template` module renders the HTML content using the pre-defined
/// template.
pub mod template;
/// The `directory` function ensures that a directory
/// exists.
pub mod utilities;

#[allow(non_camel_case_types)]

/// Runs the static site generator command-line tool. This function
/// prints a banner containing the title and description of the tool,
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
    let title = "Shokunin (職人) 🦀 (version 0.0.8)";
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";
    let width = title.len().max(description.len()) + 4;
    let horizontal_line = "─".repeat(width - 2);

    println!("\n┌{}┐", horizontal_line);
    println!("│{: ^width$}│", title, width = width - 5);
    println!("├{}┤", horizontal_line);
    println!("│{: ^width$}│", description, width = width - 2);
    println!("└{}┘", horizontal_line);

    let result = match cli::build() {
        Ok(matches) => {
            parser::args(&matches)?;
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

/// Generates a navigation menu as an unordered list of links to the
/// compiled HTML files.
///
/// # Arguments
///
/// * `files` - A slice of `File` structs containing the compiled HTML
///             files.
///
/// # Returns
///
/// A string containing the HTML code for the navigation menu. The
/// string is wrapped in a `<ul>` element with the class `nav`.
/// Each file is wrapped in a `<li>` element, and each link is wrapped
/// in an `<a>` element.
///
/// The `href` attribute of the `<a>` element is set to `../` to move
/// up one level in the directory hierarchy, followed by the name of
/// the directory for the file (generated based on the file name), and
/// finally `index.html`. The text of the link is set to the name of
/// the file with the `.md` extension removed.
///
/// The files are sorted alphabetically by their names.
/// The function returns an empty string if the slice is empty.
/// The function returns an error if the file name does not contain
/// the `.md` extension.
///
/// If multiple files have the same base directory name, the function
/// generates unique directory names by appending a suffix to the
/// directory name for each file.
///
/// If a file's name already contains a directory path
/// (e.g., `path/to/file.md`), the function generates a directory name
/// that preserves the structure of the path, without duplicating the
/// directory names (e.g., `path_to/file.md` becomes `path_to_file`).
///
pub fn generate_navigation(files: &[File]) -> String {
    let mut files_sorted = files.to_vec();
    files_sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let nav_links = files_sorted.iter().map(|file| {
        let mut dir_name = file.name.replace(".md", "");

        // Handle special case for files in the same base directory
        if let Some((index, _)) = dir_name.match_indices('/').next() {
            let base_dir = &dir_name[..index];
            let file_name = &dir_name[index + 1..];
            dir_name = format!("{}{}", base_dir, file_name.replace(base_dir, ""));
        }

        format!(
            "<li><a href=\"../{}/index.html\" role=\"navitem\">{}</a></li>",
            dir_name,
            file.name.replace(".md", "")
        )
    }).collect::<Vec<_>>().join("\n");

    format!("<ul class=\"nav\">\n{}\n</ul>", nav_links)
}

/// Compiles files in a source directory, generates HTML pages from
/// them, and writes the resulting pages to an output directory. Also
/// generates an index page containing links to the generated pages.
/// This function takes in the paths to the source and output
/// directories as arguments.
///
/// The function reads in all Markdown files in the source directory and
/// extracts metadata from them.
/// The metadata is used to generate appropriate meta tags for the
/// resulting HTML pages. The Markdown content of the files is then
/// converted to HTML using the `generate_html` function and rendered
/// into complete HTML pages using the `render_page` function. The
/// resulting pages are written to the output directory as HTML files
/// with the same names as the original Markdown files.
///
/// Finally, the function generates an index HTML page containing links
/// to all the generated pages. The resulting index page is written to
/// the output directory as "index.html".
///
/// If any errors occur during the process (e.g. a file cannot be read
/// or written), an error is returned. Otherwise, `Ok(())` is returned.
pub fn compile(
    src_dir: &Path,
    out_dir: &Path,
    template_path: Option<&String>,
    site_name: String,
) -> Result<(), Box<dyn Error>> {
    // Constants
    let src_dir = Path::new(src_dir);
    let out_dir = Path::new(out_dir);

    println!("❯ Generating a new site: \"{}\"", site_name);

    // Delete the output directory
    println!("\n❯ Deleting any previous directory...");
    fs::remove_dir_all(out_dir)?;
    fs::remove_dir(Path::new(&site_name))?;
    println!("  Done.\n");

    // Creating the template directory
    println!("\n❯ Creating template directory...");
    let template_path = create_template_folder(template_path)
        .expect("❌ Error: Could not create template directory");
    println!("  Done.\n");

    // Create the output directory
    println!("❯ Creating output directory...");
    fs::create_dir(out_dir)?;
    println!("  Done.\n");

    // Read the files in the source directory
    println!("❯ Reading files...");
    let files = add(src_dir)?;
    println!("  Found {} files.\n", files.len());

    // Compile the files
    println!("❯ Compiling files...");

    // Generate the HTML code for the navigation menu
    let navigation = generate_navigation(&files);

    let files_compiled: Vec<File> = files
        .into_iter()
        .map(|file| {
            // Extract metadata from front matter
            let (title, date, description, keywords, permalink) =
                extract(&file.content);
            let meta =
                generate_metatags(&[("url".to_owned(), permalink)]);

            // Generate HTML
            let content = render_page(&PageOptions {
                title: &title,
                date: &date,
                description: &description,
                keywords: &keywords,
                meta: &meta,
                lang: "en-GB",
                content: &generate_html(&file.content, &title, &description),
                copyright: format!(
                    "Copyright © {} 2023. All rights reserved.",
                    site_name
                )
                .as_str(),
                css: "style.css",
                navigation: &navigation,
            }, &template_path)
            .unwrap();

            // Generate JSON
            let options = ManifestOptions {
                background_color: "#000".to_string(),
                description,
                dir: "/".to_string(),
                display: "fullscreen".to_string(),
                icons: "{ \"src\": \"icon/lowres.webp\", \"sizes\": \"64x64\", \"type\": \"image/webp\" }, { \"src\": \"icon/lowres.png\", \"sizes\": \"64x64\" }".to_string(),
                identity: "/".to_string(),
                lang: "en-GB".to_string(),
                name: title,
                orientation: "any".to_string(),
                scope: "/".to_string(),
                short_name: "/".to_string(),
                start_url: "/".to_string(),
                theme_color: "#fff".to_string(),
            };
            let json_data = manifest(&options);


            File {
                name: file.name,
                content,
                json: json_data,
            }
        })
        .collect();

    // Generate the HTML code for the navigation menu
    generate_navigation(&files_compiled);

    println!("  Done.\n");

    // Write the compiled files to the output directory
    println!("❯ Writing files...");
    for file in &files_compiled {
        let file_name = file.name.replace(".md", "");
        let dir_name = out_dir.join(file_name.clone());
        fs::create_dir_all(&dir_name)?;

        let out_file = dir_name.join("index.html");
        let out_json_file = dir_name.join("manifest.json");

        fs::write(&out_file, &file.content)?;
        fs::write(&out_json_file, &file.json)?;

        println!("  - {}", out_file.display());
        println!("  - {}", out_json_file.display());
    }
    println!("  Done.\n");

    // Write the index file
    println!("❯ Writing index...");
    let index = format!(
    "<html><head></head><body><ul class=\"nav\">\n{}\n</ul></body></html>",
    files_compiled
        .iter()
        .map(|file| {
            format!(
                "<li><a href=\"{}/index.html\">{}</a></li>",
                file.name.replace(".md", ""),
                file.name.replace(".md", "")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
);

    let out_index_file = out_dir.join("index.html");
    fs::write(&out_index_file, index)?;
    println!("  - {}", out_index_file.display());
    println!("  Done.\n");

    // Move the output directory to the public directory
    println!("❯ Moving output directory...");
    let public_dir = Path::new("public");
    fs::remove_dir_all(public_dir)?;
    let site_name = site_name.replace(' ', "_");
    let new_project_dir = public_dir.join(site_name);
    fs::create_dir_all(&new_project_dir)?;
    fs::rename(out_dir, &new_project_dir)?;
    println!("  Done.\n");

    Ok(())
}
