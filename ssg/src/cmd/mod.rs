// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator Modules
//!
//! This module exposes key submodules of the Shokunin Static Site Generator.
//!
//! ## Module Overview
//!
//! - [`cli`]: Contains functions and structures for handling command-line interface (CLI) input, argument parsing, and configuration management.
//! - [`process`]: Manages argument validation, error handling, and any required processing steps based on user inputs.
//!

/// The `cli` module provides functions and structures to manage the command-line interface (CLI) of the Shokunin Static Site Generator.
///
/// This module utilises the `clap` crate to define and parse command-line arguments, offering secure handling of paths, options, and configuration for the generator's functionality.
///
/// ## Features
/// - **Argument Parsing**: Handles various command-line arguments for customising the site generation process.
/// - **Configuration Management**: Validates and manages user configurations, ensuring all required options are set.
/// - **Secure Path Handling**: Ensures that paths provided by users are safe and valid.
///
pub mod cli;

/// The `process` module contains functions for processing command-line arguments and executing actions accordingly.
///
/// This module handles various tasks such as validating user-provided arguments and performing actions like initialising directories or triggering the static site generation based on the received input.
///
/// ## Features
///
/// - **Argument Validation**: Verifies that required arguments are present and in valid formats.
/// - **Error Handling**: Provides custom errors for missing arguments, directory creation issues, and other potential problems.
/// - **Execution Flow**: Facilitates the primary actions of the static site generator based on parsed command-line arguments.
///
pub mod process;
