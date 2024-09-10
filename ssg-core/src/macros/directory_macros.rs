// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module provides macros for directory operations, including checking directory existence,
//! creating multiple directories at once, and cleaning up directories.
//!
//! # `macro_check_directory` Macro
//!
//! Checks if a directory exists and creates it if necessary.
//!
//! ## Usage
//!
//! ```rust
//! use ssg::macro_check_directory;
//! use std::path::Path;
//!
//! let path = Path::new("logs");
//! macro_check_directory!(path, "logs");
//! ```
//!
//! ## Arguments
//!
//! * `_dir` - The path to check, as a `std::path::Path`.
//! * `_name` - A string literal representing the directory name. This is used in error messages.
//!
//! ## Behaviour
//!
//! The `macro_check_directory` macro checks if the directory specified by `_dir` exists.
//! If it exists and is not a directory, a panic with an error message is triggered.
//! If the directory doesn't exist, the macro attempts to create it using `std::fs::create_dir_all(_dir)`.
//!
//! # `macro_cleanup_directories` Macro
//!
//! Cleans up multiple directories by invoking the `cleanup_directory` function.
//!
//! ## Usage
//!
//! ```rust
//! use std::path::Path;
//! use ssg::macro_check_directory;
//!
//! let path = Path::new("logs");
//! macro_check_directory!(path, "logs");
//! ```
//!
//! ## Arguments
//!
//! * `$( $_dir:expr ),*` - A comma-separated list of directory paths to clean up.
//!
//! ## Behaviour
//!
//! The `macro_cleanup_directories` macro takes multiple directory paths as arguments
//! and invokes the `cleanup_directory` function for each path.
//!
//! # `macro_create_directories` Macro
//!
//! Creates multiple directories at once.
//!
//! ## Usage
//!
//! ```rust
//! use ssg::{macro_create_directories, macro_cleanup_directories};
//! use std::path::Path;
//!
//! macro_create_directories!("logs", "logs1", "logs2");
//! macro_cleanup_directories!(Path::new("./logs"), Path::new("./logs1"), Path::new("./logs2"));
//! ```
//!
//! ## Arguments
//!
//! * `...` - Variable number of directory paths, each specified as an expression (`expr`).
//!
//! ## Behaviour
//!
//! The `macro_create_directories` macro creates multiple directories at once.
//!
//! ## Example
//!
//! ```rust
//! use ssg::{macro_create_directories, macro_cleanup_directories};
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let test = Path::new("logs");
//!     let test2  = Path::new("logs1");
//!     macro_create_directories!(test, test2)?;
//!     macro_cleanup_directories!(test, test2);
//!     Ok(())
//! }
//! ```
//!
//! # Note
//!
//! These macros assume familiarity with Rust macros and their usage.
//! Users are encouraged to review Rust macro documentation for a better understanding
//! of how to work with macros effectively.

/// # `macro_check_directory` Macro
///
/// Check if a directory exists and create it if necessary.
///
/// ## Usage
///
/// ```rust
/// use ssg::macro_check_directory;
/// use std::path::Path;
///
/// let path = Path::new("logs");
/// macro_check_directory!(path, "logs");
/// ```
///
/// ## Arguments
///
/// * `_dir` - The path to check, as a `std::path::Path`.
/// * `_name` - A string literal representing the directory name. This is used in error messages.
///
/// ## Behaviour
///
/// The `macro_check_directory` macro checks if the directory specified by `_dir` exists. If it exists and is not a directory, a panic with an error message is triggered. If the directory doesn't exist, the macro attempts to create it using `std::fs::create_dir_all(_dir)`. If the creation is successful, no action is taken. If an error occurs during the directory creation, a panic is triggered with an error message indicating the failure.
///
/// Please note that the macro panics on failure. Consider using this macro in scenarios where panicking is an acceptable behaviour, such as during application startup or setup.
///
/// # See Also
///
/// - [`macro_create_directories`] for creating multiple directories
/// - [`macro_cleanup_directories`] for cleaning up directories
///
#[macro_export]
macro_rules! macro_check_directory {
    ($_dir:expr, $_name:expr) => {{
        use std::path::Path;
        let directory: &Path = $_dir;
        let name = $_name;
        if directory.exists() {
            if !directory.is_dir() {
                log::warn!("❌ '{}' is not a directory.", name);
                panic!("❌ '{}' is not a directory.", name);
            }
        } else {
            match std::fs::create_dir_all(directory) {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "❌ Cannot create '{}' directory: {}",
                        name,
                        e
                    );
                    panic!(
                        "❌ Cannot create '{}' directory: {}",
                        name, e
                    )
                }
            }
        }
    }};
}

/// # `macro_cleanup_directories` Macro
///
/// Cleanup multiple directories by invoking the `cleanup_directory` function.
///
/// ## Usage
///
/// ```rust
/// use std::path::Path;
/// use ssg::macro_check_directory;
///
/// let path = Path::new("logs");
/// macro_check_directory!(path, "logs");
/// ```
///
/// ## Arguments
///
/// * `$( $_dir:expr ),*` - A comma-separated list of directory paths to clean up.
///
/// ## Behaviour
///
/// The `macro_cleanup_directories` macro takes multiple directory paths as arguments and invokes the `cleanup_directory` function for each path. It is assumed that the `cleanup_directory` function is available in the crate's utilities module (`$crate::utilities::cleanup_directory`).
///
/// The macro creates an array `directories` containing the provided directory paths and passes it as an argument to `cleanup_directory`. The `cleanup_directory` function is responsible for performing the cleanup operations.
///
/// Please note that the macro uses the `?` operator for error propagation. It expects the `cleanup_directory` function to return a `Result` type. If an error occurs during the cleanup process, it will be propagated up the call stack, allowing the caller to handle it appropriately.
///
/// # See Also
///
/// - [`macro_check_directory`] for checking and creating a single directory
/// - [`macro_create_directories`] for creating multiple directories
///
#[macro_export]
macro_rules! macro_cleanup_directories {
    ($path:expr) => {
        {
            use anyhow::Context;
            std::fs::remove_dir_all($path).with_context(|| format!("Failed to clean up directory: {:?}", $path))
        }
    };
}

/// # `macro_create_directories` Macro
///
/// Create multiple directories at once.
///
/// ## Usage
///
/// ```rust
/// use ssg::{macro_create_directories, macro_cleanup_directories};
/// use std::path::Path;
/// macro_create_directories!("logs", "logs1", "logs2");
/// macro_cleanup_directories!(Path::new("./logs"), Path::new("./logs1"), Path::new("./logs2"));
/// ```
///
/// ## Arguments
///
/// * `...` - Variable number of directory paths, each specified as an expression (`expr`).
///
/// ## Behaviour
///
/// The `macro_create_directories` macro creates multiple directories at once. It takes a variable number of directory paths as arguments and uses the `create_directory` utility function from the `$crate` crate to create the directories.
///
/// The directories are specified as expressions and separated by commas. For example, `macro_create_directories!("logs", "logs1", "logs2")` will attempt to create the `logs`, `logs1`, and `logs2`.
///
/// The macro internally creates a slice of the directory paths and passes it to the `create_directory` function. If any error occurs during the directory creation, the macro returns an `Err` value, indicating the first encountered error. Otherwise, it returns `Ok(())`.
///
/// ## Example
///
/// ```rust
/// use ssg::{macro_create_directories, macro_cleanup_directories};
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let test = Path::new("logs");
///     let test2  = Path::new("logs1");
///     macro_create_directories!(test, test2)?;
///     macro_cleanup_directories!(test, test2);
///     Ok(())
/// }
/// ```
///
/// # See Also
///
/// - [`macro_check_directory`] for checking and creating a single directory
/// - [`macro_cleanup_directories`] for cleaning up directories
///
#[macro_export]
macro_rules! macro_create_directories {
    ($($path:expr),+) => {
        {
            use anyhow::{Result, Context};
            (|| -> Result<()> {
                $(
                    std::fs::create_dir_all($path)
                        .with_context(|| format!("Failed to create directory: {:?}", $path))?;
                )+
                Ok(())
            })()
        }
    };
}
