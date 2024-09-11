// main.rs - Entry point for the Shokunin Static Site Generator CLI
// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::{Context, Result};
use log::info;
use ssg_cli::cli::{build, print_banner};
use ssg_cli::process::args;

/// # Function: `main`
///
/// The main entry point of the Shokunin Static Site Generator (SSG) command-line interface (CLI).
///
/// This function is responsible for:
/// 1. Initializing the logger using the `env_logger` crate.
/// 2. Displaying the CLI banner via the `print_banner` function.
/// 3. Building and parsing command-line arguments using the `build` function.
/// 4. Processing the parsed arguments by invoking the `args` function from the `ssg_cli::process` module.
///
/// # Returns
///
/// - `Result<()>`: If the program executes successfully, an `Ok(())` is returned.
///   If an error occurs during argument processing or command execution, an `Err` is returned, encapsulating the error.
///
/// # Errors
///
/// This function may return an error in the following scenarios:
/// - If there is an issue during the initialization of the environment logger.
/// - If any command-line argument parsing fails in the `build` function.
/// - If the argument processing function (`args`) encounters an error.
///
/// # Logging
///
/// - Informational logs are emitted during the program's execution, indicating the start and successful completion of the CLI process.
///
/// # Example
///
/// Run the following command in your terminal to execute the CLI:
///
/// ```bash
/// cargo run --bin ssg -- --new --content "content" --output "public" --serve
/// ```
///
/// This will execute the CLI with the provided arguments.
fn main() -> Result<()> {
    // Initialize the environment logger for logging purposes.
    env_logger::init();

    // Display the CLI banner for the Shokunin Static Site Generator.
    print_banner();

    // Log the start of the SSG CLI process.
    info!("Starting SSG CLI");

    // Build the CLI and parse the command-line arguments.
    let matches = build().get_matches();

    // Process the parsed arguments.
    args(&matches).context("Failed to process arguments")?;

    // Log the successful completion of the SSG CLI process.
    info!("SSG CLI completed successfully");

    // Return a successful result.
    Ok(())
}
