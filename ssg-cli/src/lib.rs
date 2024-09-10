// lib.rs - The main library module for the Shokunin Static Site Generator CLI.
// This file provides the main entry points for the CLI functionality, including the
// command-line interface (CLI) and process management.

// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// # Module: `cli`
///
/// This module contains functionality related to building and handling the command-line
/// interface (CLI) for the Shokunin Static Site Generator (SSG).
///
/// The module defines functions for constructing the CLI with argument parsing using the
/// `clap` crate. It handles user input for various commands such as creating new projects,
/// specifying content directories, and more.
///
/// See [`cli.rs`](./cli.rs) for more details.
pub mod cli;

/// # Module: `process`
///
/// This module processes the parsed command-line arguments and handles core actions
/// such as directory management and project compilation.
///
/// The `process` module provides utility functions to validate directories, manage
/// content paths, and trigger the static site generation using the `ssg` library.
///
/// See [`process.rs`](./process.rs) for more details.
pub mod process;
