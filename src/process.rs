// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
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
    #[error(
        "Failed to create {dir_type} directory at '{path}': {source}"
    )]
    DirectoryCreation {
        /// Type of the directory, such as "content" or "output".
        dir_type: String,
        /// Path where the directory creation failed.
        path: String,
        #[source]
        /// The underlying IO error that occurred.
        source: std::io::Error,
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

    /// Represents a failure during the frontmatter processing.
    #[error("Frontmatter processing error: {0}")]
    FrontmatterError(String),
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

/// Preprocesses markdown files to properly handle frontmatter
fn preprocess_content(content_path: &Path) -> Result<(), ProcessError> {
    if !content_path.exists() {
        return Ok(());
    }

    // Process all .md files in the content directory
    for entry in fs::read_dir(content_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path.extension().map_or(false, |ext| ext == "md")
        {
            let content = fs::read_to_string(&path)?;
            let processed_content = process_frontmatter(&content)
                .map_err(|e| {
                    ProcessError::FrontmatterError(e.to_string())
                })?;
            fs::write(&path, processed_content)?;
        }
    }
    Ok(())
}

/// Processes frontmatter in markdown content to ensure proper rendering
fn process_frontmatter(content: &str) -> Result<String, ProcessError> {
    const DELIMITER: &str = "---";

    let parts: Vec<&str> = content.splitn(3, DELIMITER).collect();
    match parts.len() {
        3 => {
            // Format: ---\nfrontmatter\n---\ncontent
            let frontmatter = parts[1].trim();
            let main_content = parts[2].trim();

            // Add an HTML comment to preserve frontmatter for metadata processing
            // while preventing it from appearing in the rendered output
            Ok(format!(
                "---\n{}\n---\n<!--frontmatter-processed-->\n{}",
                frontmatter, main_content
            ))
        }
        _ => Ok(content.to_string()), // Return unchanged if no frontmatter found
    }
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

    // Preprocess content files to handle frontmatter
    preprocess_content(content_path)?;

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
    use std::fs::Permissions;
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
            source: std::io::Error::from_raw_os_error(13),
        };
        assert_eq!(
            error.to_string(),
            "Failed to create content directory at '/invalid/path': Permission denied (os error 13)"
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
    #[test]
    fn test_process_frontmatter_with_valid_frontmatter(
    ) -> Result<(), ProcessError> {
        let content = "\
---
title: Test Post
date: 2024-01-01
---
# Main Content
This is the main content.";

        let processed = process_frontmatter(content)?;
        assert!(processed.contains("<!--frontmatter-processed-->"));
        assert!(processed.contains("title: Test Post"));
        assert!(processed.contains("# Main Content"));
        Ok(())
    }

    #[test]
    fn test_process_frontmatter_without_frontmatter(
    ) -> Result<(), ProcessError> {
        let content = "# Just Content\nNo frontmatter here.";
        let processed = process_frontmatter(content)?;
        assert_eq!(processed, content);
        Ok(())
    }

    #[test]
    fn test_process_frontmatter_with_empty_frontmatter(
    ) -> Result<(), ProcessError> {
        let content = "---\n---\nContent after empty frontmatter";
        let processed = process_frontmatter(content)?;
        assert!(processed.contains("<!--frontmatter-processed-->"));
        Ok(())
    }

    #[test]
    fn test_preprocess_content_with_multiple_files(
    ) -> Result<(), ProcessError> {
        let temp_dir = tempdir()?;

        // Create multiple markdown files
        let file1_path = temp_dir.path().join("post1.md");
        let file2_path = temp_dir.path().join("post2.md");
        let non_md_path = temp_dir.path().join("other.txt");

        fs::write(&file1_path, "---\ntitle: Post 1\n---\nContent 1")?;
        fs::write(&file2_path, "---\ntitle: Post 2\n---\nContent 2")?;
        fs::write(&non_md_path, "Not a markdown file")?;

        preprocess_content(temp_dir.path())?;

        // Verify markdown files were processed
        let content1 = fs::read_to_string(&file1_path)?;
        let content2 = fs::read_to_string(&file2_path)?;
        let other = fs::read_to_string(&non_md_path)?;

        assert!(content1.contains("<!--frontmatter-processed-->"));
        assert!(content2.contains("<!--frontmatter-processed-->"));
        assert_eq!(other, "Not a markdown file");

        Ok(())
    }

    #[test]
    fn test_preprocess_content_with_non_existent_directory(
    ) -> Result<(), ProcessError> {
        let non_existent = Path::new("non_existent_directory");
        let result = preprocess_content(non_existent);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_preprocess_content_with_invalid_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("readonly.md");

        // Create file with frontmatter
        fs::write(&file_path, "---\ntitle: Test\n---\nContent")
            .unwrap();

        // Make file read-only
        fs::set_permissions(&file_path, Permissions::from_mode(0o444))
            .unwrap();

        let result = preprocess_content(temp_dir.path());
        assert!(result.is_err());

        // Reset permissions for cleanup
        fs::set_permissions(&file_path, Permissions::from_mode(0o666))
            .unwrap();
    }

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
        assert!(result.is_err());
        if let Err(ProcessError::DirectoryCreation { source, .. }) =
            result
        {
            assert_eq!(
                source.kind(),
                std::io::ErrorKind::AlreadyExists
            );
        } else {
            panic!("Expected DirectoryCreation error");
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
    fn test_preprocess_content_with_invalid_utf8() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("invalid.md");

        // Write invalid UTF-8 bytes
        let invalid_bytes = vec![0xFF, 0xFF];
        fs::write(&file_path, invalid_bytes)?;

        let result = preprocess_content(temp_dir.path());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_process_frontmatter_with_multiple_delimiters() -> Result<()>
    {
        let content = "\
---
title: First
---
---
title: Second
---
Content";

        let processed = process_frontmatter(content)?;
        // Should only process the first frontmatter section
        assert!(processed.contains("title: First"));
        assert!(processed.contains("---\ntitle: Second"));
        Ok(())
    }

    #[test]
    fn test_process_frontmatter_with_malformed_delimiters(
    ) -> Result<(), ProcessError> {
        // Test case where there's only one delimiter
        let content = "---\ntitle: Test\nContent";
        let processed = process_frontmatter(content)?;
        assert_eq!(processed, content); // Should remain unchanged with single delimiter

        // Test case with extra spaces in delimiters (this should still be valid frontmatter)
        let content = "---\ntitle: Test\n---\nContent";
        let processed = process_frontmatter(content)?;
        assert!(processed.contains("<!--frontmatter-processed-->"));
        assert!(processed.contains("title: Test"));
        assert!(processed.contains("Content"));

        Ok(())
    }

    #[test]
    fn test_process_frontmatter_with_whitespace(
    ) -> Result<(), ProcessError> {
        // Test with whitespace before first delimiter
        let content = "\n\n---\ntitle: Test\n---\nContent";
        let processed = process_frontmatter(content)?;
        // Should still process valid frontmatter even with leading whitespace
        assert!(processed.contains("<!--frontmatter-processed-->"));
        assert!(processed.contains("title: Test"));
        assert!(processed.contains("Content"));

        // Test with mixed whitespace in frontmatter
        let content =
            "---\n  title: Test  \n  author: Someone  \n---\nContent";
        let processed = process_frontmatter(content)?;
        assert!(processed.contains("<!--frontmatter-processed-->"));
        assert!(processed.contains("title: Test"));
        assert!(processed.contains("author: Someone"));
        assert!(processed.contains("Content"));

        Ok(())
    }

    #[test]
    fn test_process_frontmatter_with_invalid_format(
    ) -> Result<(), ProcessError> {
        // Missing second delimiter completely
        let content = "---\ntitle: Test\nContent";
        let processed = process_frontmatter(content)?;
        assert_eq!(processed, content);

        // Wrong delimiter character
        let content = "===\ntitle: Test\n===\nContent";
        let processed = process_frontmatter(content)?;
        assert_eq!(processed, content);

        // Empty content between delimiters
        let content = "---\n\n---\nContent";
        let processed = process_frontmatter(content)?;
        assert!(processed.contains("<!--frontmatter-processed-->"));

        Ok(())
    }

    #[test]
    fn test_preprocess_content_with_nested_directories(
    ) -> Result<(), ProcessError> {
        let temp_dir = tempdir()?;
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;

        // Create files in both root and nested directory
        let root_file = temp_dir.path().join("root.md");
        let nested_file = nested_dir.join("nested.md");

        fs::write(&root_file, "---\ntitle: Root\n---\nRoot content")?;
        fs::write(
            &nested_file,
            "---\ntitle: Nested\n---\nNested content",
        )?;

        preprocess_content(temp_dir.path())?;

        // Verify only root file was processed (since we don't recurse into subdirectories)
        let root_content = fs::read_to_string(&root_file)?;
        assert!(root_content.contains("<!--frontmatter-processed-->"));

        let nested_content = fs::read_to_string(&nested_file)?;
        assert!(
            !nested_content.contains("<!--frontmatter-processed-->")
        );

        Ok(())
    }

    #[test]
    fn test_preprocess_content_with_empty_files(
    ) -> Result<(), ProcessError> {
        let temp_dir = tempdir()?;
        let empty_file = temp_dir.path().join("empty.md");

        // Create empty markdown file
        fs::write(&empty_file, "")?;

        preprocess_content(temp_dir.path())?;

        // Verify empty file remains unchanged
        let content = fs::read_to_string(&empty_file)?;
        assert!(content.is_empty());

        Ok(())
    }

    #[test]
    fn test_ensure_directory_with_symlink() -> Result<(), ProcessError>
    {
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
