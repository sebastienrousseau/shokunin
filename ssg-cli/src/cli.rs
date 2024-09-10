// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use log::debug;

/// # Function: `build`
///
/// This function builds a command-line interface (CLI) using the `clap` crate,
/// parsing the arguments passed to the CLI and returning them as an `ArgMatches` object.
///
/// The CLI supports the following arguments:
/// - `--new` or `-n`: Creates a new project.
/// - `--content` or `-c`: Specifies the location of the content directory.
/// - `--output` or `-o`: Specifies the location of the output directory.
/// - `--template` or `-t`: Specifies the location of the template directory.
/// - `--serve` or `-s`: Serves the public directory on a local web server.
///
/// # Arguments
///
/// This function does not take any parameters.
///
/// # Returns
///
/// - `Result<ArgMatches>`: On success, an `Ok` variant containing the `ArgMatches` struct
///   with the parsed arguments is returned. On failure, an `Err` with the encountered error is returned.
///
/// # Example
///
/// ```
/// use ssg_cli::cli::build;
///
/// let cmd = build().unwrap();
/// ```
///
/// # Errors
///
/// - If an error occurs during CLI building or parsing, an `Err` is returned.
///
/// # Logging
///
/// Debug messages are logged to indicate the progress of the CLI building process.
pub fn build() -> Result<ArgMatches> {
    debug!("Building CLI command");

    let cmd = Command::new("Shokunin Static Site Generator")
        .author("Sebastien Rousseau")
        .about("A fast and flexible static site generator written in Rust.")
        .bin_name("ssg")
        .version("0.0.30")
        .arg(
            Arg::new("new")
                .help("Create a new project.")
                .long("new")
                .short('n')
                .value_name("NEW"),
        )
        .arg(
            Arg::new("content")
                .help("Location of the content directory.")
                .long("content")
                .short('c')
                .value_name("CONTENT"),
        )
        .arg(
            Arg::new("output")
                .help("Location of the output directory.")
                .long("output")
                .short('o')
                .value_name("OUTPUT"),
        )
        .arg(
            Arg::new("template")
                .help("Location of the template directory.")
                .long("template")
                .short('t')
                .value_name("TEMPLATE"),
        )
        .arg(
            Arg::new("serve")
                .help("Serve the public directory on a local web server.")
                .long("serve")
                .short('s')
                .value_name("SERVE"),
        )
        .after_help(
            "\x1b[1;4mDocumentation:\x1b[0m\n\n  https://shokunin.one\n\n\
             \x1b[1;4mLicense:\x1b[0m\n  The project is licensed under the terms of \
             both the MIT license and the Apache License (Version 2.0).",
        )
        .get_matches();

    debug!("CLI command built successfully");
    Ok(cmd)
}

/// # Function: `print_banner`
///
/// Prints a formatted banner to the terminal displaying the title and
/// description of the Shokunin Static Site Generator tool.
///
/// The banner is enclosed in a box with borders and a horizontal separator
/// between the title and the description.
///
/// # Arguments
///
/// This function does not take any parameters.
///
/// # Example
///
/// ```
/// use ssg_cli::cli::print_banner;
///
/// print_banner();
/// ```
///
/// # Output
///
/// The banner is printed directly to the console, providing users with
/// an introduction to the tool.
pub fn print_banner() {
    // Define the title and description
    let title = "Shokunin (ssg) 🦀 v0.0.30";
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";

    // Determine the box width based on the longest string
    let width = title.len().max(description.len()) + 4;

    // Create a horizontal line for the banner
    let horizontal_line = "─".repeat(width - 2);

    // Print the title and description within a box
    println!("\n┌{}┐", horizontal_line);
    println!("│{: ^1$}│", title, width - 3);
    println!("├{}┤", horizontal_line);
    println!("│{: ^1$}│", description, width - 2);
    println!("└{}┘\n", horizontal_line);
}