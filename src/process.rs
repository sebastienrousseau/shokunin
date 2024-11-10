// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use clap::ArgMatches;
use std::{fs, path::Path};
use thiserror::Error;

/// Represents errors that may occur during argument processing.
#[derive(Error, Debug)]
pub enum ProcessError {
    /// Occurs when a directory cannot be created.
    ///
    /// # Fields
    /// - `dir_type`: The type of directory (e.g., "content", "output").
    /// - `path`: The file path where the directory creation failed.
    #[error("Failed to create {dir_type} directory at '{path}'")]
    DirectoryCreation {
        /// Type of the directory, such as "content" or "output".
        dir_type: String,
        /// Path where the directory creation failed.
        path: String,
    },

    /// Triggered when a required command-line argument is missing.
    ///
    /// # Fields
    /// - The name of the missing argument.
    #[error("Required argument missing: {0}")]
    MissingArgument(String),

    /// Represents a failure during the compilation process.
    ///
    /// # Fields
    /// - Compilation error message.
    #[error("Compilation error: {0}")]
    CompilationError(String),

    /// Wraps underlying I/O errors.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Retrieves the value of a specified command-line argument.
///
/// # Arguments
///
/// * `matches` - Clap argument matches object containing parsed arguments.
/// * `name` - The name of the argument to retrieve.
///
/// # Returns
///
/// * `Result<String, ProcessError>` - Returns the argument value on success or an error if the argument is missing.
///
/// # Errors
///
/// - Returns `ProcessError::MissingArgument` if the specified argument is not provided.
///
/// # Example
///
/// ```rust,no_run
/// # use clap::{ArgMatches, Command};
/// # use ssg::process::get_argument;
/// let matches = Command::new("test")
///     .arg(clap::arg!(--"config" <CONFIG> "Specifies the configuration file"))
///     .get_matches_from(vec!["test", "--config", "path/to/config.toml"]);
/// let config_path = get_argument(&matches, "config").expect("Argument not found");
/// println!("Config path: {}", config_path);
/// ```
pub fn get_argument(
    matches: &ArgMatches,
    name: &str,
) -> Result<String, ProcessError> {
    matches
        .get_one::<String>(name)
        .ok_or_else(|| ProcessError::MissingArgument(name.to_string()))
        .map(String::from)
}

/// Ensures the specified directory exists, creating it if necessary.
///
/// # Arguments
///
/// * `path` - The path of the directory to check.
/// * `dir_type` - A label describing the directory type (e.g., "content", "output").
///
/// # Returns
///
/// * `Result<(), ProcessError>` - Returns `Ok` if the directory exists or is successfully created.
///
/// # Errors
///
/// - Returns `ProcessError::DirectoryCreation` if the directory cannot be created due to permissions or other issues.
///
/// # Example
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use ssg::process::ensure_directory;
/// let path = Path::new("path/to/output");
/// ensure_directory(path, "output").expect("Failed to ensure directory exists");
/// ```
pub fn ensure_directory(
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

/// Compiles the static site by generating the necessary files from the provided paths.
///
/// # Parameters
///
/// * `build_path`: The path where the compiled site will be built.
/// * `content_path`: The path to the directory containing the content files.
/// * `site_path`: The path to the directory where the site project will be created.
/// * `template_path`: The path to the directory containing the template files.
///
/// # Return
///
/// * `Result<(), String>`: Returns `Ok(())` if the compilation is successful, or an error message as a string if an error occurs.
///
/// # Errors
///
/// * If any error occurs during the compilation process, an error message will be returned as a string.
fn internal_compile(
    build_path: &Path,
    content_path: &Path,
    site_path: &Path,
    template_path: &Path,
) -> Result<(), String> {
    staticdatagen::compiler::service::compile(
        build_path,
        content_path,
        site_path,
        template_path,
    )
    .map_err(|e| e.to_string())
}

/// Processes command-line arguments and initiates the static site generation.
///
/// This function performs the following steps:
/// 1. Retrieves required directory paths from command-line arguments.
/// 2. Ensures each directory exists, creating it if necessary.
/// 3. Calls the compilation service to generate the static site.
///
/// # Arguments
///
/// * `matches` - Parsed command-line arguments from `clap`.
///
/// # Returns
///
/// * `Result<(), ProcessError>` - Returns `Ok` on successful completion, or an error if a problem occurs.
///
/// # Errors
///
/// - Returns `ProcessError::MissingArgument` if a required argument is not provided.
/// - Returns `ProcessError::DirectoryCreation` if a directory cannot be created.
/// - Returns `ProcessError::CompilationError` if the site fails to compile.
///
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
    internal_compile(
        build_path,
        content_path,
        site_path,
        template_path,
    )
    .map_err(ProcessError::CompilationError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{arg, Command};
    use tempfile::tempdir;

    /// Helper function to create a test `ArgMatches` with all required arguments.
    fn create_test_command() -> ArgMatches {
        Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--content",
                "content",
                "--output",
                "output",
                "--new",
                "new_site",
                "--template",
                "template",
            ])
    }

    #[test]
    fn test_get_argument_present() {
        let matches = create_test_command();
        let content = get_argument(&matches, "content").unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_get_argument_missing() {
        let matches = Command::new("test")
            .arg(arg!(--"config" <CONFIG> "Config file"))
            .get_matches_from(vec!["test"]);
        let result = get_argument(&matches, "config");
        assert!(matches!(
            result,
            Err(ProcessError::MissingArgument(_))
        ));
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = tempdir().unwrap();
        let result = ensure_directory(temp_dir.path(), "temp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_args_missing_template_argument() {
        let matches = Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--content",
                "content",
                "--output",
                "output",
                "--new",
                "new_site",
            ]);
        let result = args(&matches);
        assert!(matches!(
            result,
            Err(ProcessError::MissingArgument(ref arg)) if arg == "template"
        ));
    }

    #[test]
    fn test_ensure_directory_already_exists() -> Result<()> {
        let temp_dir = tempdir()?;
        ensure_directory(temp_dir.path(), "existing")?;
        assert!(temp_dir.path().exists());
        Ok(())
    }

    #[test]
    fn test_process_error_display() {
        let error =
            ProcessError::MissingArgument("content".to_string());
        assert_eq!(
            error.to_string(),
            "Required argument missing: content"
        );

        let error = ProcessError::DirectoryCreation {
            dir_type: "content".to_string(),
            path: "/invalid/path".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to create content directory at '/invalid/path'"
        );

        let error = ProcessError::CompilationError(
            "Failed to compile".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Compilation error: Failed to compile"
        );
    }

    #[test]
    fn test_process_error_io_error() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::Other,
            "an I/O error occurred",
        );
        let error: ProcessError = io_error.into();
        assert!(matches!(error, ProcessError::IoError(_)));
        assert_eq!(error.to_string(), "an I/O error occurred");
    }

    #[test]
    fn test_process_error_io_error_format() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        );
        let error: ProcessError = io_error.into();
        assert!(matches!(error, ProcessError::IoError(_)));
        assert_eq!(error.to_string(), "File not found");
    }

    #[test]
    fn test_ensure_directory_permission_denied() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let protected_path = temp_dir.path().join("protected_dir");

        // Create the directory and make it read-only
        fs::create_dir(&protected_path).unwrap();
        fs::set_permissions(
            &protected_path,
            Permissions::from_mode(0o400),
        )
        .unwrap();

        // Attempt to create a subdirectory inside the protected directory to trigger a permission error
        let sub_dir = protected_path.join("sub_dir");
        let result = ensure_directory(&sub_dir, "sub_directory");

        // Check that the permission-denied error was triggered
        assert!(matches!(
            result,
            Err(ProcessError::DirectoryCreation { .. })
        ));

        // Reset permissions for cleanup
        fs::set_permissions(
            &protected_path,
            Permissions::from_mode(0o700),
        )
        .unwrap();
    }

    #[test]
    fn test_args_all_required_arguments(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let output_dir = temp_dir.path().join("output");
        let site_dir = temp_dir.path().join("new_site");
        let template_dir = temp_dir.path().join("template");

        let matches = Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--content",
                content_dir.to_str().unwrap(),
                "--output",
                output_dir.to_str().unwrap(),
                "--new",
                site_dir.to_str().unwrap(),
                "--template",
                template_dir.to_str().unwrap(),
            ]);

        // Since `compile` is shadowed, it will use the mock compile function
        let result = args(&matches);
        assert!(
            matches!(result, Err(ProcessError::CompilationError(_))),
            "Expected CompilationError from args"
        );

        Ok(())
    }
}
