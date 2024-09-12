//! # ssg-cli
//!
//! `ssg-cli` is the command-line interface (CLI) for the Shokunin Static Site Generator.
//! This crate provides the necessary functionality to interact with the Shokunin SSG
//! from the command line, allowing users to create, manage, and build static websites.
//!
//! ## Features
//!
//! - Command-line argument parsing for various SSG operations
//! - Project creation and management
//! - Content directory specification
//! - Output directory configuration
//! - Built-in development server
//!
//! ## Main Components
//!
//! - `cli`: Module for building and handling the command-line interface
//! - `process`: Module for processing parsed arguments and executing core actions
//! - `run`: Main function to execute the CLI
//!
//! ## Usage
//!
//! To use the SSG CLI in your Rust project, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ssg-cli = "0.1.0"  # Replace with the actual version
//! ```
//!
//! Then, in your Rust code:
//!
//! ```rust,no_run
//! use ssg_cli::run;
//! use std::env;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Set up mock arguments for testing
//!     env::set_var("RUST_LOG", "info");
//!     let args = vec!["program", "--new", "my_project", "--content", "content", "--output", "public"];
//!     env::set_var("CARGO_PKG_VERSION", "0.1.0");
//!
//!     // In a real scenario, you would use the actual command-line arguments:
//!     // let args: Vec<String> = env::args().collect();
//!
//!     run()
//! }
//! ```
//!
//! For more detailed information on each module, please refer to their respective documentation.

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Module for building and handling the command-line interface (CLI).
///
/// This module contains functionality related to constructing the CLI with argument
/// parsing using the `clap` crate. It handles user input for various commands such
/// as creating new projects, specifying content directories, and more.
///
/// See [`cli.rs`](./cli.rs) for more details.
pub mod cli;

/// Module for processing parsed command-line arguments and executing core actions.
///
/// This module provides utility functions to validate directories, manage content
/// paths, and trigger the static site generation using the `ssg` library.
///
/// See [`process.rs`](./process.rs) for more details.
pub mod process;

pub use cli::{build, print_banner};
pub use process::args;

/// Run the SSG CLI.
///
/// This function initializes the logger, prints the CLI banner, builds and parses
/// command-line arguments, and processes them to execute the requested SSG operations.
///
/// # Returns
///
/// Returns `Ok(())` if the CLI execution is successful, or an `Err` containing
/// the error information if any step fails.
///
/// # Errors
///
/// This function may return an error if:
/// - The logger initialization fails
/// - Command-line argument parsing fails
/// - The requested SSG operation encounters an error
///
/// # Example
///
/// ```rust,no_run
/// use ssg_cli::run;
/// use std::env;
///
/// // Set up mock arguments for testing
/// env::set_var("RUST_LOG", "info");
/// let args = vec!["program", "--new", "my_project", "--content", "content", "--output", "public"];
/// env::set_var("CARGO_PKG_VERSION", "0.1.0");
///
/// // Run the CLI with mock arguments
/// run().unwrap();
/// ```
pub fn run() -> anyhow::Result<()> {
    env_logger::init();
    print_banner();
    let matches = build().get_matches();
    args(&matches)?;
    Ok(())
}
