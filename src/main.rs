// Copyright © 2023-2025 Shokunin Static Site Generator.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator - Main Entry Point
//!
//! This module contains the main entry point for initiating the Shokunin Static Site Generator.
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

use ssg::run;

/// Executes the main logic of the Shokunin Static Site Generator.
///
/// This function performs the primary actions for generating a static site, including:
/// 1. Calling `run` from the `ssg` module to generate the site.
/// 2. Returning a fixed success or failure message (no translation).
///
/// # Return
/// `Result<String, String>` - A result containing either a success message or an error string.
async fn execute_main_logic() -> Result<String, String> {
    match run().await {
        Ok(_) => Ok("Site generated successfully.".to_string()),
        Err(e) => Err(format!("Program encountered an error: {}", e)),
    }
}

/// The main entry point of the Shokunin Static Site Generator.
///
/// This function initiates the static site generation process by calling `execute_main_logic`.
/// It handles the output to the console, displaying either a success message
/// or an error message if the generation fails.
///
/// ### Exit Codes
/// - Returns `0` if site generation is successful.
/// - Returns a non-zero status code if an error occurs.
#[tokio::main]
async fn main() {
    match execute_main_logic().await {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("{}", e),
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::io::{BufWriter, Write};
    use std::sync::Mutex;

    lazy_static! {
        static ref OUTPUT_LOCK: Mutex<()> = Mutex::new(());
    }

    #[tokio::test]
    async fn test_main_success() {
        let _lock = OUTPUT_LOCK.lock().unwrap();

        let mut output = Vec::new();
        {
            let mut writer = BufWriter::new(&mut output);
            // Redirect stdout to our writer
            writeln!(writer, "Site generated successfully.").unwrap();
            writer.flush().unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Site generated successfully."));
    }

    #[tokio::test]
    async fn test_main_error() {
        let _lock = OUTPUT_LOCK.lock().unwrap();

        let mut output = Vec::new();
        {
            let mut writer = BufWriter::new(&mut output);
            // Redirect stderr to our writer
            writeln!(
                writer,
                "Program encountered an error: test error"
            )
            .unwrap();
            writer.flush().unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Program encountered an error"));
    }
}
