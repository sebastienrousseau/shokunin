// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The main function of the program.
//!
//! Calls the `run()` function from the `ssg` module to run the static site generator.
//!
//! If an error occurs while running the `run()` function, the function prints an error message
//! to standard error and exits the program with a non-zero status code.

use ssg::languages::translate;

fn main() {
    if let Err(err) = ssg::run() {
        eprintln!("{}", err);
        eprintln!("{}", translate("en", "main_logger_msg"));
        std::process::exit(1);
    }
}
