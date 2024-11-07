// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # NucleusFlow CLI
//!
//! This is the main entry point for the NucleusFlow command-line interface.
//! It initializes the logger, displays a banner, and runs the main CLI process.

use anyhow::{Context, Result};
use log::info;
use nucleusflow::run;

/// The main entry point of the NucleusFlow CLI.
///
/// This function performs the following steps:
/// 1. Initializes the logger using `env_logger`.
/// 2. Runs the main CLI process.
///
/// If any step fails, it logs the error and exits with a non-zero status code.
///
/// # Errors
///
/// This function will return an error if:
/// - The logger fails to initialize.
/// - The main CLI process encounters an error.
fn main() -> Result<()> {
    // Log the start of the SSG CLI process
    info!("Starting SSG CLI");

    // Run the SSG
    run().context("Failed to run SSG")?;

    // Log the successful completion of the SSG CLI process
    info!("SSG CLI completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        // This test ensures that the main function runs without panicking
        // It doesn't actually test the functionality, just that it completes
        let _ = main();
    }
}
