// Copyright © 2023 shokunin. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//!
//! # Shokunin (職人) 🦀
//!
//! A Fast and Flexible Static Site Generator written in Rust 🦀
//!
//! [![shokunin](https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/shokunin/logo/logo-shokunin.svg)](https://shokunin.one)
//!
//! [![Rust](https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust)](https://www.rust-lang.org)
//! [![Crates.io](https://img.shields.io/crates/v/ssg.svg?style=for-the-badge&color=success&labelColor=27A006)](https://crates.io/crates/ssg)
//! [![Lib.rs](https://img.shields.io/badge/lib.rs-v0.0.5-success.svg?style=for-the-badge&color=8A48FF&labelColor=6F36E4)](https://lib.rs/crates/ssg)
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
/// The `file` module contains functions for file management.
pub mod file;
/// The `frontmatter` module contains functions for frontmatter management.
pub mod frontmatter;
/// The `html` module contains functions for HTML generation.
pub mod html;
/// The `json` module contains functions for JSON generation.
pub mod json;
/// The `metatags` module contains functions for metatags generation.
pub mod metatags;
/// The `parser` module contains functions for parsing arguments.
pub mod parser;
/// The `template` module contains functions for template management.
pub mod template;
/// The `utilities` module contains functions for utilities.
pub mod utilities;

/// Runs the static site generator command-line tool. This function
/// prints a banner containing the title and description of the tool,
/// and then processes any command-line arguments passed to it. If no
/// arguments are passed, it prints a welcome message and instructions
/// on how to use the tool.
///
/// The function uses the `build` function from the `cli` module to
/// create the command-line interface for the tool. It then processes
/// any arguments passed to it using the `parser` function
/// from the `args` module.
///
/// If any errors occur during the process (e.g. an invalid argument is
/// passed), an error message is printed and returned. Otherwise,
/// `Ok(())` is returned.
pub fn run() -> Result<(), Box<dyn Error>> {
    let title = "Shokunin (職人) 🦀 (v0.0.5)";
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";
    print_welcome_message(title, description)?;

    let result = cli::build().map(parser::args);
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "
                Error: {}",
                e
            );
        }
    }

    println!("\n✅ All Done");

    print_welcome_message_on_no_args()?;

    Ok(())
}
/// Prints a welcome message containing the title and description of the
/// tool. The message is printed in a box with a horizontal line at the
/// top and bottom.
pub fn print_welcome_message(
    title: &str,
    description: &str,
) -> Result<(), Box<dyn Error>> {
    let width = title.len().max(description.len()) + 4;
    let horizontal_line = "─".repeat(width - 2);

    println!("\n┌{}┐", horizontal_line);
    println!("│{: ^width$}│", title, width = width - 5);
    println!("├{}┤", horizontal_line);
    println!("│{: ^width$}│", description, width = width - 2);
    println!("└{}┘", horizontal_line);
    Ok(())
}
/// Prints a welcome message if no arguments are passed to the tool.
/// The message contains instructions on how to use the tool.
pub fn print_welcome_message_on_no_args() -> Result<(), Box<dyn Error>>
{
    if std::env::args().len() == 1 {
        eprintln!(
            "\n\nWelcome to Shokunin (職人) 🦀\n\nLet's get started! Please, run `ssg --help` for more information.\n"
        );
    }
    Ok(())
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
    println!("❯ Generating a new site: \"{}\"", site_name);

    delete_previous_directory(out_dir, &site_name)?;
    let template_path = create_template_directory(template_path)?;

    create_output_directory(out_dir)?;
    let files = read_files(src_dir)?;
    let files_compiled =
        compile_files(files, &template_path, &site_name)?;

    write_files(&files_compiled, out_dir)?;
    write_index(&files_compiled, out_dir)?;
    move_output_directory(out_dir, &site_name)?;

    Ok(())
}
/// Deletes any previous directory with the same name as the site.
pub fn delete_previous_directory(
    out_dir: &Path,
    site_name: &str,
) -> Result<(), Box<dyn Error>> {
    println!("\n❯ Deleting any previous directory...");
    fs::remove_dir_all(out_dir)?;
    fs::remove_dir(Path::new(site_name))?;
    println!("  Done.\n");
    Ok(())
}
/// Creates template directory.
pub fn create_template_directory(
    template_path: Option<&String>,
) -> Result<String, Box<dyn Error>> {
    println!("\n❯ Creating template directory...");
    let template_path = create_template_folder(template_path)
        .expect("❌ Error: Could not create template directory");
    println!("  Done.\n");
    Ok(template_path)
}
/// Creates output directory.
pub fn create_output_directory(
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("❯ Creating output directory...");
    fs::create_dir(out_dir)?;
    println!("  Done.\n");
    Ok(())
}
/// Reads all files in a directory and returns a vector of `files`.
pub fn read_files(src_dir: &Path) -> Result<Vec<File>, Box<dyn Error>> {
    println!("❯ Reading files...");
    let files = add(src_dir)?;
    println!("  Found {} files.\n", files.len());
    Ok(files)
}

fn compile_files(
    files: Vec<File>,
    template_path: &str,
    site_name: &str,
) -> Result<Vec<File>, Box<dyn Error>> {
    println!("❯ Compiling files...");

    let navigation = generate_navigation(&files);
    let template_path_string = template_path.to_owned(); // Convert &str to String

    let files_compiled: Vec<File> = files
        .into_iter()
        .map(|file| {
            let (title, description, keywords, permalink) = extract(&file.content);
            let meta = generate_metatags(&[("url".to_owned(), permalink)]);
            let content = render_page(
                &PageOptions {
                    title: &title,
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
                },
                &template_path_string, // Pass the reference to the String instead of &str
            )
            .unwrap();

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

    println!("  Done.\n");
    Ok(files_compiled)
}

fn write_files(
    files_compiled: &[File],
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("❯ Writing files...");
    for file in files_compiled {
        let out_file = out_dir.join(file.name.replace(".md", ".html"));
        let out_json_file =
            out_dir.join(file.name.replace(".md", ".webmanifest"));
        fs::write(&out_file, &file.content)?;
        fs::write(&out_json_file, &file.json)?;
        println!("  - {}", out_file.display());
        println!("  - {}", out_json_file.display());
    }
    println!("  Done.\n");
    Ok(())
}

fn write_index(
    files_compiled: &[File],
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    println!("❯ Writing index...");
    let index = format!(
        "<ul class=\"nav\">\n{}\n</ul>",
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
    fs::write(index_file, index)?;
    println!(" Done.\n");
    Ok(())
}

fn move_output_directory(
    out_dir: &Path,
    site_name: &str,
) -> Result<(), Box<dyn Error>> {
    println!("❯ Moving output directory...");
    let public_dir = Path::new("public");
    fs::remove_dir_all(public_dir)?;
    let site_name = site_name.replace(' ', "_");
    let new_project_dir = public_dir.join(site_name);
    fs::create_dir_all(&new_project_dir)?;
    fs::rename(out_dir, &new_project_dir)?;
    println!(" Done.\n");
    Ok(())
}

fn generate_navigation(files_compiled: &[File]) -> String {
    files_compiled
        .iter()
        .map(|file| {
            format!(
                "<li><a href=\"{}\">{}</a></li>",
                file.name.replace(".md", ".html"),
                file.name.replace(".md", "")
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}
