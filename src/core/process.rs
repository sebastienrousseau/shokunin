// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Argument-driven site processing.
//!
//! Bridges the parsed [`clap::ArgMatches`] from `cmd::Cli` to the build
//! pipeline orchestrated in [`crate::run`]. Responsibilities:
//!
//! - Resolve content / output / template directories from CLI flags or
//!   configuration files, applying sensible defaults when callers omit
//!   them.
//! - Create build and site directories on disk, ensuring distinct paths
//!   so `staticdatagen::compile` can finalise output by renaming.
//!
//! Most binaries should call [`crate::run`] rather than this module
//! directly; the helpers here are exposed for tests and embedders that
//! need a smaller building block than the full pipeline.
//!
//! # Source files are immutable
//!
//! As of issue #543, this module never writes back to any file under
//! `content/`. An earlier `preprocess_content` helper used to rewrite
//! markdown sources in place with a `<!--frontmatter-processed-->`
//! sentinel; that path was destructive (it dirtied users' git working
//! trees on every build and left source files partially transformed if
//! the build crashed mid-pass), was not load-bearing for any active
//! plugin, and has been removed. Front-matter parsing now happens in
//! memory inside [`staticdatagen::compiler::service::compile`].

use clap::ArgMatches;
use std::{fs, path::Path};
/// Represents errors that may occur during argument processing.
///
/// Marked `#[non_exhaustive]` so new error cases can be added in minor
/// versions. Consumers should always include a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum ProcessError {
    /// Occurs when a directory cannot be created.
    ///
    /// # Fields
    /// - `dir_type`: The type of directory (e.g., "content", "output").
    /// - `path`: The file path where the directory creation failed.
    DirectoryCreation {
        /// Type of the directory, such as "content" or "output".
        dir_type: String,
        /// Path where the directory creation failed.
        path: String,
        /// The underlying IO error that occurred.
        source: std::io::Error,
    },

    /// Triggered when a required command-line argument is missing.
    ///
    /// # Fields
    /// - The name of the missing argument.
    MissingArgument(String),

    /// Represents a failure during the compilation process.
    ///
    /// # Fields
    /// - Compilation error message.
    CompilationError(String),

    /// Wraps underlying I/O errors.
    IoError(std::io::Error),

    /// Represents a failure during the frontmatter processing.
    FrontmatterError(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryCreation {
                dir_type,
                path,
                source,
            } => write!(
                f,
                "Failed to create {dir_type} directory at '{path}': {source}"
            ),
            Self::MissingArgument(arg) => {
                write!(f, "Required argument missing: {arg}")
            }
            Self::CompilationError(msg) => {
                write!(f, "Compilation error: {msg}")
            }
            Self::IoError(e) => write!(f, "{e}"),
            Self::FrontmatterError(msg) => {
                write!(f, "Frontmatter processing error: {msg}")
            }
        }
    }
}

impl std::error::Error for ProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DirectoryCreation { source, .. } => Some(source),
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
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
    if path.exists() {
        // Check if the existing path is a directory
        if !path.is_dir() {
            return Err(ProcessError::DirectoryCreation {
                dir_type: dir_type.to_string(),
                path: path.display().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "Path exists but is not a directory",
                ),
            });
        }
    } else {
        fs::create_dir_all(path).map_err(|e| {
            ProcessError::DirectoryCreation {
                dir_type: dir_type.to_string(),
                path: path.display().to_string(),
                source: e,
            }
        })?;
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

/// Processes CLI arguments and executes the corresponding site compilation workflow.
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

    // Compile the site. Note: front-matter is parsed in memory by
    // `staticdatagen::compiler::service::compile`; we deliberately do
    // NOT pre-process / rewrite source `.md` files here (see issue
    // #543 — the previous in-place writer dirtied users' git trees).
    internal_compile(build_path, content_path, site_path, template_path)
        .map_err(ProcessError::CompilationError)?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use anyhow::Result;
    use clap::{arg, Command};
    use std::fs::{self, File};
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
        assert!(matches!(result, Err(ProcessError::MissingArgument(_))));
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

    #[cfg(not(target_os = "windows"))] // Unix-specific: path behaviour / error messages differ on Windows
    #[test]
    fn test_process_error_display() {
        let error = ProcessError::MissingArgument("content".to_string());
        assert_eq!(error.to_string(), "Required argument missing: content");

        let error = ProcessError::DirectoryCreation {
            dir_type: "content".to_string(),
            path: "/invalid/path".to_string(),
            source: std::io::Error::from_raw_os_error(13),
        };
        assert_eq!(
            error.to_string(),
            "Failed to create content directory at '/invalid/path': Permission denied (os error 13)"
        );

        let error =
            ProcessError::CompilationError("Failed to compile".to_string());
        assert_eq!(error.to_string(), "Compilation error: Failed to compile");
    }

    #[test]
    fn test_process_error_io_error() {
        let io_error = std::io::Error::other("an I/O error occurred");
        let error: ProcessError = io_error.into();
        assert!(matches!(error, ProcessError::IoError(_)));
        assert_eq!(error.to_string(), "an I/O error occurred");
    }

    #[test]
    fn test_process_error_io_error_format() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error: ProcessError = io_error.into();
        assert!(matches!(error, ProcessError::IoError(_)));
        assert_eq!(error.to_string(), "File not found");
    }

    #[cfg(unix)]
    #[test]
    fn test_ensure_directory_permission_denied() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let protected_path = temp_dir.path().join("protected_dir");

        // Create the directory and make it read-only
        fs::create_dir(&protected_path).unwrap();
        fs::set_permissions(&protected_path, Permissions::from_mode(0o400))
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
        fs::set_permissions(&protected_path, Permissions::from_mode(0o700))
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
    // NOTE: Tests for the old `preprocess_content` / `process_frontmatter`
    // helpers were removed in issue #543 along with the destructive in-place
    // writer those helpers backed. Source files in `content/` are no longer
    // rewritten during a build; see the new integration test at
    // `tests/build_does_not_mutate_sources.rs` for the regression guard.

    #[test]
    fn test_internal_compile_error_handling() {
        let temp_dir = tempdir().unwrap();
        let result = internal_compile(
            &temp_dir.path().join("build"),
            &temp_dir.path().join("content"),
            &temp_dir.path().join("site"),
            &temp_dir.path().join("template"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_argument_with_empty_value() {
        let matches = Command::new("test")
            .arg(arg!(--"empty" <EMPTY> "Empty value"))
            .get_matches_from(vec!["test", "--empty", ""]);

        let result = get_argument(&matches, "empty");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_ensure_directory_with_existing_file(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("existing_file");

        // Create a file instead of a directory
        let _file = File::create(&file_path)?;

        // Attempt to ensure directory at the same path
        let result = ensure_directory(&file_path, "test");

        // Verify that the operation failed because path exists but is not a directory
        let err = result.unwrap_err();
        match err {
            ProcessError::DirectoryCreation { source, .. } => {
                assert_eq!(source.kind(), std::io::ErrorKind::AlreadyExists);
            }
            other => panic!("Expected DirectoryCreation, got: {other}"),
        }

        Ok(())
    }

    #[test]
    fn test_ensure_directory_with_existing_directory(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let dir_path = temp_dir.path().join("existing_dir");

        // First create the directory
        fs::create_dir(&dir_path)?;

        // Attempt to ensure directory at the same path
        let result = ensure_directory(&dir_path, "test");

        // Should succeed because path exists and is a directory
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_ensure_directory_with_symlink() -> Result<(), ProcessError> {
        let temp_dir = tempdir()?;
        let real_dir = temp_dir.path().join("real_dir");
        let symlink = temp_dir.path().join("symlink_dir");

        fs::create_dir(&real_dir)?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_dir, &symlink)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real_dir, &symlink)?;

        // Should succeed as symlink points to a valid directory
        let result = ensure_directory(&symlink, "symlink");
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_process_error_frontmatter_display() {
        let error = ProcessError::FrontmatterError("bad yaml".to_string());
        assert_eq!(error.to_string(), "Frontmatter processing error: bad yaml");
    }

    #[test]
    fn test_process_error_source_for_directory_creation() {
        use std::error::Error;
        let error = ProcessError::DirectoryCreation {
            dir_type: "output".to_string(),
            path: "/bad".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "denied",
            ),
        };
        assert!(error.source().is_some());
    }

    #[test]
    fn test_process_error_source_for_io_error() {
        use std::error::Error;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let error = ProcessError::IoError(io_err);
        assert!(error.source().is_some());
    }

    #[test]
    fn test_process_error_source_for_missing_argument() {
        use std::error::Error;
        let error = ProcessError::MissingArgument("foo".to_string());
        assert!(error.source().is_none());
    }

    #[test]
    fn test_process_error_source_for_compilation_error() {
        use std::error::Error;
        let error = ProcessError::CompilationError("oops".to_string());
        assert!(error.source().is_none());
    }

    #[test]
    fn test_process_error_source_for_frontmatter_error() {
        use std::error::Error;
        let error = ProcessError::FrontmatterError("bad".to_string());
        assert!(error.source().is_none());
    }

    #[test]
    fn test_process_error_debug() {
        let error = ProcessError::MissingArgument("arg".to_string());
        let debug = format!("{error:?}");
        assert!(debug.contains("MissingArgument"));
    }

    #[test]
    fn test_internal_compile_with_empty_directories() {
        let temp_dir = tempdir().unwrap();

        // Create empty required directories
        let build_dir = temp_dir.path().join("build");
        let content_dir = temp_dir.path().join("content");
        let site_dir = temp_dir.path().join("site");
        let template_dir = temp_dir.path().join("template");

        fs::create_dir_all(&build_dir).unwrap();
        fs::create_dir_all(&content_dir).unwrap();
        fs::create_dir_all(&site_dir).unwrap();
        fs::create_dir_all(&template_dir).unwrap();

        let result = internal_compile(
            &build_dir,
            &content_dir,
            &site_dir,
            &template_dir,
        );

        assert!(result.is_err());
    }
}
