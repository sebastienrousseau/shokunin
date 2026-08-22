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
/// # Examples
///
/// ```rust
/// use clap::{Arg, Command};
/// use ssg::process::get_argument;
///
/// let matches = Command::new("t")
///     .arg(Arg::new("name").long("name"))
///     .get_matches_from(vec!["t", "--name", "value"]);
/// assert_eq!(get_argument(&matches, "name").unwrap(), "value");
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
/// # Examples
///
/// ```rust
/// use ssg::process::ensure_directory;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let new = dir.path().join("created");
/// ensure_directory(&new, "output").unwrap();
/// assert!(new.is_dir());
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
/// # Examples
///
/// ```rust
/// use clap::{Arg, Command};
/// use ssg::process::args;
///
/// // Missing required arguments ⇒ `MissingArgument` error.
/// let matches = Command::new("t")
///     .arg(Arg::new("content").long("content"))
///     .get_matches_from(vec!["t"]);
/// assert!(args(&matches).is_err());
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

    // Compile the site. Note: front-matter is parsed in memory by
    // `staticdatagen::compiler::service::compile`; we deliberately do
    // NOT pre-process / rewrite source `.md` files here (see issue
    // #543 — the previous in-place writer dirtied users' git trees).
    internal_compile(build_path, content_path, site_path, template_path)
        .map_err(ProcessError::CompilationError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use clap::{arg, Command};
    use std::fs::{self, File};
    use tempfile::tempdir;

    /// Variant predicates used instead of inline `matches!` /
    /// `match … => panic!` so both the matching and non-matching arms
    /// are exercised (see `variant_helpers_reject_non_matching_values`).
    fn is_missing_argument(
        r: &Result<String, ProcessError>,
        name: &str,
    ) -> bool {
        matches!(r, Err(ProcessError::MissingArgument(arg)) if arg == name)
    }

    fn is_missing_argument_unit(
        r: &Result<(), ProcessError>,
        name: &str,
    ) -> bool {
        matches!(r, Err(ProcessError::MissingArgument(arg)) if arg == name)
    }

    fn is_directory_creation(r: &Result<(), ProcessError>) -> bool {
        matches!(r, Err(ProcessError::DirectoryCreation { .. }))
    }

    fn is_io_error(e: &ProcessError) -> bool {
        matches!(e, ProcessError::IoError(_))
    }

    fn is_input_error(r: &Result<(), ProcessError>) -> bool {
        matches!(
            r,
            Err(ProcessError::CompilationError(_)
                | ProcessError::DirectoryCreation { .. })
        )
    }

    fn directory_creation_source_kind(
        e: ProcessError,
    ) -> Option<std::io::ErrorKind> {
        match e {
            ProcessError::DirectoryCreation { source, .. } => {
                Some(source.kind())
            }
            _ => None,
        }
    }

    #[test]
    fn variant_helpers_reject_non_matching_values() {
        assert!(!is_missing_argument(&Ok("v".to_string()), "content"));
        assert!(!is_missing_argument(
            &Err(ProcessError::MissingArgument("a".to_string())),
            "b"
        ));
        assert!(!is_missing_argument_unit(&Ok(()), "content"));
        assert!(!is_missing_argument_unit(
            &Err(ProcessError::MissingArgument("a".to_string())),
            "b"
        ));
        assert!(!is_directory_creation(&Ok(())));
        assert!(is_input_error(&Err(ProcessError::CompilationError(
            "x".to_string()
        ))));
        assert!(!is_input_error(&Ok(())));
        assert!(!is_io_error(&ProcessError::FrontmatterError(
            "f".to_string()
        )));
        assert!(
            directory_creation_source_kind(ProcessError::MissingArgument(
                "m".to_string()
            ))
            .is_none()
        );
    }

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
        assert!(is_missing_argument(&result, "config"));
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = tempdir().unwrap();
        let result = ensure_directory(temp_dir.path(), "temp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_args_missing_content_argument() {
        // Mirrors `test_args_missing_template_argument` but for the
        // first `?` in `args()` — exercises the early-return path for
        // a missing `content` argument specifically through `args()`
        // (not just through `get_argument` in isolation).
        let matches = Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--output",
                "output",
                "--new",
                "new_site",
                "--template",
                "template",
            ]);
        let result = args(&matches);
        assert!(is_missing_argument_unit(&result, "content"));
    }

    #[test]
    fn test_args_missing_output_argument() {
        let matches = Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--content",
                "content",
                "--new",
                "new_site",
                "--template",
                "template",
            ]);
        let result = args(&matches);
        assert!(is_missing_argument_unit(&result, "output"));
    }

    #[test]
    fn test_args_missing_new_argument() {
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
                "--template",
                "template",
            ]);
        let result = args(&matches);
        assert!(is_missing_argument_unit(&result, "new"));
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
        assert!(is_missing_argument_unit(&result, "template"));
    }

    #[test]
    fn test_ensure_directory_already_exists() {
        let temp_dir = tempdir().unwrap();
        ensure_directory(temp_dir.path(), "existing").unwrap();
        assert!(temp_dir.path().exists());
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
        assert!(is_io_error(&error));
        assert_eq!(error.to_string(), "an I/O error occurred");
    }

    #[test]
    fn test_process_error_io_error_format() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error: ProcessError = io_error.into();
        assert!(is_io_error(&error));
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
        assert!(is_directory_creation(&result));

        // Reset permissions for cleanup
        fs::set_permissions(&protected_path, Permissions::from_mode(0o700))
            .unwrap();
    }

    #[test]
    fn test_args_all_required_arguments() {
        // v0.0.46: staticdatagen 0.0.10's recursive `add()` returns
        // an empty file list (not an error) for nonexistent paths, so
        // we have to pass a real *file* where the content directory
        // is expected — `read_dir` fails on a non-directory and that
        // bubbles up as a `CompilationError`.
        let temp_dir = tempdir().unwrap();
        let content_file = temp_dir.path().join("content_file");
        fs::write(&content_file, "not a directory").unwrap();
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
                content_file.to_str().unwrap(),
                "--output",
                output_dir.to_str().unwrap(),
                "--new",
                site_dir.to_str().unwrap(),
                "--template",
                template_dir.to_str().unwrap(),
            ]);

        let result = args(&matches);
        // v0.0.46: `args()` runs `ensure_directory` against each path
        // before reaching the compile pipeline, so the invalid
        // `content_file` (a regular file, not a dir) now surfaces as
        // `ProcessError::DirectoryCreation` rather than wrapping into
        // `ProcessError::CompilationError`. Either variant indicates
        // a correctly-propagated input error.
        assert!(
            is_input_error(&result),
            "Expected DirectoryCreation or CompilationError from args, got: {result:?}"
        );
    }

    /// Builds `ArgMatches` pointing at the four given directory paths.
    fn matches_for_paths(
        content: &Path,
        output: &Path,
        site: &Path,
        template: &Path,
    ) -> ArgMatches {
        Command::new("test")
            .arg(arg!(--"content" <CONTENT> "Content directory"))
            .arg(arg!(--"output" <OUTPUT> "Output directory"))
            .arg(arg!(--"new" <NEW> "New site directory"))
            .arg(arg!(--"template" <TEMPLATE> "Template directory"))
            .get_matches_from(vec![
                "test",
                "--content",
                content.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--new",
                site.to_str().unwrap(),
                "--template",
                template.to_str().unwrap(),
            ])
    }

    #[test]
    fn test_args_succeeds_with_empty_content_and_templates() {
        // staticdatagen treats empty content + empty templates as
        // "no work to do", so `args` runs the full pipeline — all
        // four ensure_directory calls plus a successful compile.
        let temp_dir = tempdir().unwrap();
        let content = temp_dir.path().join("content");
        let output = temp_dir.path().join("output");
        let site = temp_dir.path().join("new_site");
        let template = temp_dir.path().join("template");

        let matches = matches_for_paths(&content, &output, &site, &template);
        let result = args(&matches);
        assert!(result.is_ok(), "expected success, got: {result:?}");
        assert!(content.is_dir(), "content dir should have been created");
        assert!(template.is_dir(), "template dir should have been created");
    }

    #[test]
    fn test_args_output_directory_creation_failure() {
        // Content is fine, but the output path nests under a file so
        // the second ensure_directory call fails.
        let temp_dir = tempdir().unwrap();
        let content = temp_dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let blocker = temp_dir.path().join("blocker");
        fs::write(&blocker, "file").unwrap();

        let matches = matches_for_paths(
            &content,
            &blocker.join("output"),
            &temp_dir.path().join("site"),
            &temp_dir.path().join("template"),
        );
        assert!(is_directory_creation(&args(&matches)));
    }

    #[test]
    fn test_args_site_directory_creation_failure() {
        let temp_dir = tempdir().unwrap();
        let content = temp_dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let blocker = temp_dir.path().join("blocker");
        fs::write(&blocker, "file").unwrap();

        let matches = matches_for_paths(
            &content,
            &temp_dir.path().join("output"),
            &blocker.join("site"),
            &temp_dir.path().join("template"),
        );
        assert!(is_directory_creation(&args(&matches)));
    }

    #[test]
    fn test_args_template_directory_creation_failure() {
        let temp_dir = tempdir().unwrap();
        let content = temp_dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let blocker = temp_dir.path().join("blocker");
        fs::write(&blocker, "file").unwrap();

        let matches = matches_for_paths(
            &content,
            &temp_dir.path().join("output"),
            &temp_dir.path().join("site"),
            &blocker.join("template"),
        );
        assert!(is_directory_creation(&args(&matches)));
    }

    #[cfg(unix)]
    #[test]
    fn test_args_compilation_error_from_unreadable_content() {
        use std::os::unix::fs::PermissionsExt;

        // All four directories pass ensure_directory, but content is
        // unreadable so staticdatagen's read_dir fails and args maps
        // it into ProcessError::CompilationError.
        let temp_dir = tempdir().unwrap();
        let content = temp_dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::set_permissions(&content, fs::Permissions::from_mode(0o000))
            .unwrap();

        let matches = matches_for_paths(
            &content,
            &temp_dir.path().join("output"),
            &temp_dir.path().join("site"),
            &temp_dir.path().join("template"),
        );
        let result = args(&matches);

        // Restore permissions so tempdir cleanup succeeds.
        fs::set_permissions(&content, fs::Permissions::from_mode(0o755))
            .unwrap();

        assert!(
            result.is_err(),
            "expected CompilationError, got: {result:?}"
        );
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Compilation error"), "got: {msg}");
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
    fn test_ensure_directory_with_existing_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("existing_file");

        // Create a file instead of a directory
        let _file = File::create(&file_path).unwrap();

        // Attempt to ensure directory at the same path
        let result = ensure_directory(&file_path, "test");

        // Verify that the operation failed because path exists but is not a directory
        let err = result.unwrap_err();
        let kind = directory_creation_source_kind(err)
            .expect("expected DirectoryCreation error");
        assert_eq!(kind, std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn test_ensure_directory_with_existing_directory() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("existing_dir");

        // First create the directory
        fs::create_dir(&dir_path).unwrap();

        // Attempt to ensure directory at the same path
        let result = ensure_directory(&dir_path, "test");

        // Should succeed because path exists and is a directory
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_directory_with_symlink() {
        let temp_dir = tempdir().unwrap();
        let real_dir = temp_dir.path().join("real_dir");
        let symlink = temp_dir.path().join("symlink_dir");

        fs::create_dir(&real_dir).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_dir, &symlink).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real_dir, &symlink).unwrap();

        // Should succeed as symlink points to a valid directory
        let result = ensure_directory(&symlink, "symlink");
        assert!(result.is_ok());
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
        // v0.0.46: staticdatagen 0.0.10 treats empty content + empty
        // templates as "no work to do", so this test now asserts
        // error PROPAGATION (not raw "empty inputs fail"). Pass a
        // real file where `content_dir` is expected — the underlying
        // `read_dir` fails on a non-directory.
        let temp_dir = tempdir().unwrap();

        let build_dir = temp_dir.path().join("build");
        let content_file = temp_dir.path().join("content_file");
        let site_dir = temp_dir.path().join("site");
        let template_dir = temp_dir.path().join("template");

        fs::create_dir_all(&build_dir).unwrap();
        fs::write(&content_file, "not a directory").unwrap();
        fs::create_dir_all(&site_dir).unwrap();
        fs::create_dir_all(&template_dir).unwrap();

        let result = internal_compile(
            &build_dir,
            &content_file,
            &site_dir,
            &template_dir,
        );

        assert!(
            result.is_err(),
            "internal_compile should propagate the io error when \
             content_dir is a file, got: {result:?}"
        );
    }
}
