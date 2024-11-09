// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use clap::ArgMatches;
use std::{fs, path::Path};
use thiserror::Error;

/// Errors that can occur during argument processing
#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Failed to create {dir_type} directory at '{path}'")]
    /// Directory creation error
    DirectoryCreation {
        /// Directory type
        dir_type: String,
        /// Path
        path: String,
    },

    #[error("Required argument missing: {0}")]
    /// Missing argument error
    MissingArgument(String),

    #[error("Compilation error: {0}")]
    /// Compilation error
    CompilationError(String),

    #[error(transparent)]
    /// IO error wrapper
    IoError(#[from] std::io::Error),
}

/// Gets an argument value from matches
fn get_argument(
    matches: &ArgMatches,
    name: &str,
) -> Result<String, ProcessError> {
    matches
        .get_one::<String>(name)
        .ok_or_else(|| ProcessError::MissingArgument(name.to_string()))
        .map(String::from)
}

/// Ensures a directory exists, creating it if necessary
fn ensure_directory(
    path: &Path,
    dir_type: &str,
) -> Result<(), ProcessError> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            // Convert the IO error to our custom error type
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ProcessError::DirectoryCreation {
                    dir_type: dir_type.to_string(),
                    path: path.display().to_string(),
                }
            } else {
                ProcessError::IoError(e)
            }
        })?;
        println!("Created {} directory: {}", dir_type, path.display());
    }
    Ok(())
}

/// Process command line arguments and compile the project
///
/// This function:
/// 1. Extracts required paths from command line arguments
/// 2. Creates necessary directories if they don't exist
/// 3. Compiles the static site
///
/// # Arguments
///
/// * `matches` - Command line arguments from clap
///
/// # Returns
///
/// * `Result<(), ProcessError>` - Ok if successful, Error otherwise
///
/// # Example
///
/// ```no_run
/// use clap::ArgMatches;
/// use ssg::cmd::process::args;
///
/// fn run(matches: &ArgMatches) {
///     if let Err(e) = args(matches) {
///         eprintln!("Error processing arguments: {}", e);
///     }
/// }
/// ```
pub fn args(matches: &ArgMatches) -> Result<(), ProcessError> {
    // Get required paths
    let content_dir = get_argument(matches, "content")?;
    let output_dir = get_argument(matches, "output")?;
    let site_dir = get_argument(matches, "new")?;
    let template_dir = get_argument(matches, "template")?;

    // Create Path objects
    let content_path = Path::new(&content_dir);
    let build_path = Path::new(&output_dir);
    let site_path = Path::new(&site_dir);
    let template_path = Path::new(&template_dir);

    // Ensure directories exist
    ensure_directory(content_path, "content")?;
    ensure_directory(build_path, "output")?;
    ensure_directory(site_path, "project")?;
    ensure_directory(template_path, "template")?;

    // Compile the site
    staticdatagen::compiler::service::compile(
        build_path,
        content_path,
        site_path,
        template_path,
    )
    .map_err(|e| ProcessError::CompilationError(e.to_string()))?;

    Ok(())
}
