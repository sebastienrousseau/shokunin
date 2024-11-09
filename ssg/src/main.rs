// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator - Main Entry Point
//!
//! This module contains the main function, which initiates the static site generation process
//! by calling the `run` function from the `ssg` module. This function is responsible for:
//! - Handling any errors that may arise during execution.
//! - Translating log messages based on the user's language preference.
//!
//! ## Behaviour
//! If `run` executes successfully, a confirmation message is displayed in the user’s preferred language.
//! Otherwise, an error message is printed to `stderr`, and the program exits with a non-zero status code.
//!
//! ## Example Usage
//! ```rust,no_run
//! use ssg::run;
//! use std::env;
//!
//! let lang = env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());
//! match run() {
//!     Ok(_) => println!("Site generated successfully."),
//!     Err(e) => eprintln!("Error encountered: {}", e),
//! }
//! ```

use langweave::translate;
use ssg::run;

/// The main entry point of the Shokunin Static Site Generator.
///
/// This function retrieves the user’s preferred language (defaulting to English if not set),
/// executes the `run` function to generate the site, and translates a success message based
/// on the selected language.
///
/// ## Language Support
/// The language is determined from the `LANGUAGE` environment variable. If translation fails,
/// an error message indicating the failure is displayed.
///
/// ## Example
/// ```rust,no_run
/// // Run the program with LANGUAGE environment variable set to the desired language
/// use std::env::set_var("LANGUAGE", "en");
/// main();
/// ```
///
/// ## Exit Codes
/// - Returns `0` on success.
/// - Returns a non-zero status code if an error occurs.
///
/// # Errors
/// If `run` encounters an error during site generation, the error message is displayed, and
/// the program exits with a non-zero code.
fn main() {
    // Determine the user's language preference, defaulting to English ("en") if unset.
    let lang =
        std::env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());

    match run() {
        Ok(_) => {
            // Translate and display a success message in the chosen language.
            match translate(&lang, "main_logger_msg") {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Translation failed: {}", e),
            }
        }
        // Display an error message if `run` encounters an issue.
        Err(e) => eprintln!("Program encountered an error: {}", e),
    }
}
