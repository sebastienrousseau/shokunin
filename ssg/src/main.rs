// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The main function of the program.
//!
//! Calls the `run()` function from the `ssg` module to run the static site generator.
//!
//! If an error occurs while running the `run()` function, the function prints an error message
//! to standard error and exits the program with a non-zero status code.

use langweave::translate;
use ssg::run;

fn main() {
    // Select language dynamically, e.g., from an environment variable or command-line argument
    let lang =
        std::env::var("LANGUAGE").unwrap_or_else(|_| "en".to_string());

    match run() {
        Ok(_) => {
            // Translate the message based on the selected language
            match translate(&lang, "main_logger_msg") {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Translation failed: {}", e),
            }
        }
        Err(e) => eprintln!("Program encountered an error: {}", e),
    }
}
