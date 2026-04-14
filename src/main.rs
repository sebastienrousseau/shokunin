// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Static Site Generator - Main Entry Point
//!
//! This module contains the main entry point for initiating the Static Site Generator.
//! It defines the `main` function and an `execute_main_logic` helper function, which together
//! handle the core execution flow, including error handling.
//!
//! ## Core Behaviour
//! - **Execution Flow**: Calls `run` from the `ssg` module to generate the site.
//! - **Exit Status**: On success, outputs a fixed confirmation message. On failure, outputs an
//!   error message and exits with a non-zero status code.
//!
//! ## Example Usage
//! ```rust,no_run
//! use ssg::run;
//! // Just call `run` and handle success or error.
//! match run() {
//!     Ok(_) => println!("Site generated successfully."),
//!     Err(e) => eprintln!("Error encountered: {}", e),
//! }
//! ```

/// The main entry point of the Static Site Generator.
///
/// Delegates to [`ssg::run`] and maps the result to an exit code.
///
/// ### Exit Codes
/// - Returns `0` if site generation is successful.
/// - Returns a non-zero status code if an error occurs.
fn main() {
    match ssg::run() {
        Ok(()) => println!("Site generated successfully."),
        Err(e) => {
            eprintln!("Program encountered an error: {e}");
            std::process::exit(1);
        }
    }
}
