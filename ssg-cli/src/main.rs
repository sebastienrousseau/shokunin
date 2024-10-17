//! Entry point for the Shokunin Static Site Generator CLI.
//!
//! This file contains the `main` function, which serves as the entry point
//! for the Shokunin Static Site Generator (SSG) command-line interface (CLI).

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::{Context, Result};
use log::info;
use ssg_cli::cli::{build, print_banner};
use ssg_cli::process::args;

/// The main entry point of the Shokunin Static Site Generator (SSG) command-line interface (CLI).
///
/// This function orchestrates the execution of the SSG CLI by performing the following steps:
/// 1. Initializes the logger using the `env_logger` crate.
/// 2. Displays the CLI banner using the `print_banner` function.
/// 3. Builds and parses command-line arguments using the `build` function.
/// 4. Processes the parsed arguments by invoking the `args` function from the `ssg_cli::process` module.
///
/// # Returns
///
/// Returns `Ok(())` if the program executes successfully. If an error occurs during execution,
/// an `Err` is returned, encapsulating the error details.
///
/// # Errors
///
/// This function may return an error in the following scenarios:
/// - Initialization of the environment logger fails.
/// - Command-line argument parsing fails in the `build` function.
/// - The argument processing function (`args`) encounters an error.
///
/// # Logging
///
/// Informational logs are emitted during the program's execution, indicating the start
/// and successful completion of the CLI process.
///
/// # Example
///
/// To execute the CLI with sample arguments, run the following command in your terminal:
///
/// ```bash
/// cargo run --bin ssg -- --new --content "content" --output "public" --serve
/// ```
///
/// This command will execute the CLI with the provided arguments to create a new project,
/// specify content and output directories, and start the development server.
fn main() -> Result<()> {
    // Initialize the environment logger
    env_logger::init();

    // Display the CLI banner
    print_banner();

    // Log the start of the SSG CLI process
    info!("Starting SSG CLI");

    // Build the CLI and parse the command-line arguments
    let matches = build().get_matches();

    // Process the parsed arguments
    args(&matches).context("Failed to process arguments")?;

    // Log the successful completion of the SSG CLI process
    info!("SSG CLI completed successfully");

    Ok(())
}
