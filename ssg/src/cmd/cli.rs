// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use clap::{Arg, ArgMatches, Command, Error};

/// ## Function: build - returns a Result containing the parsed input options
///
/// Builds a command-line interface (CLI) using the `clap` crate and
/// returns the command-line arguments passed to the CLI as an
/// `ArgMatches` object.
///
/// This function creates a CLI, and adds the following arguments to it:
///
/// - `--new` or `-n`: Creates a new project.
/// - `--content` or `-c`: Specifies the location of the content directory.
/// - `--output` or `-o`: Specifies the location of the output directory.
/// - `--template` or `-t`: Specifies the location of the template directory.
/// - `--serve` or `-s`: Serves the public directory on a local web server.
///
/// If the CLI is successfully built and the command-line arguments are
/// parsed correctly, the function returns an `Ok` result containing the
/// `ArgMatches` object. If an error occurs while building or parsing
/// the CLI, an `Err` result containing the error message is returned.
///
/// # Arguments
///
/// None
///
/// # Returns
///
/// * `Result<ArgMatches, Error>` - A struct containing the parsed
///   command-line arguments and their values, or an error if the arguments
///   could not be parsed.
///
/// # Examples
///
/// ```
/// use staticdatagen::cmd::cli::build;
/// let cmd = build().unwrap();
///
/// ```
pub fn build() -> Result<ArgMatches, Error> {
    let cmd =  Command::new("Shokunin Static Site Generator")
        .author("Sebastien Rousseau")
        .about("")
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
                .value_name("SERVE")
        )
        .after_help(
            "\x1b[1;4mDocumentation:\x1b[0m\n\n  https://shokunin.one\n\n\x1b[1;4mLicense:\x1b[0m\n  The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).",
        )
        .get_matches();
    Ok(cmd)
}

/// # `print_banner` function
///
/// This function prints a banner containing the title and description of
/// the `Shokunin` static site generator tool.
///
/// The banner is printed to the terminal in a box, with a horizontal line
/// separating the title and description. The width of the box is determined
/// by the length of the title and description.
///
/// # Arguments
///
/// This function takes no arguments.
///
/// # Examples
///
/// ```
/// use staticdatagen::cmd::cli::print_banner;
///
/// print_banner();
/// ```
pub fn print_banner() {
    // Set the title and description for the CLI
    let title = "Shokunin (ssg) 🦀 v0.0.30";
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";

    // Set the width of the box to fit the title and description
    let width = title.len().max(description.len()) + 4;

    // Create a horizontal line to separate the box
    let horizontal_line = "─".repeat(width - 2);

    // Print the title and description in a box
    println!("\n┌{}┐", horizontal_line);
    println!("│{: ^1$}│", title, width - 3);
    println!("├{}┤", horizontal_line);
    println!("│{: ^1$}│", description, width - 2);
    println!("└{}┘\n", horizontal_line);
}
