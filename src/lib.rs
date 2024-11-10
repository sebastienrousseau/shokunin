// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/shokunin/images/favicon.ico",
    html_logo_url = "https://kura.pro/shokunin/images/logos/shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

// Standard library imports
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

// Third-party imports
use anyhow::{ensure, Context, Result};
use dtt::datetime::DateTime;
use http_handle::Server;
use rayon::prelude::*;
use rlg::{log_format::LogFormat, log_level::LogLevel, macro_log};
use staticdatagen::{
    compiler::service::compile, locales::en::translate, macro_serve,
    utilities::uuid::generate_unique_string,
};

/// Module declarations

/// Process module for handling site generation
pub mod process;

/// CLI module for command-line interface
pub mod cmd;

/// Re-exports
pub use staticdatagen;

/// Configuration of essential paths for site generation.
///
/// This structure maintains references to all critical directories used throughout the
/// site generation process. Each path serves a specific purpose in the build pipeline
/// and must be validated before use.
///
/// # Fields
///
/// * `site` - Root directory for the generated website output
/// * `content` - Source directory containing markdown and other content files
/// * `build` - Temporary directory for build artifacts and intermediate files
/// * `template` - Directory containing site templates and layout definitions
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
/// use ssg::Paths;
/// let paths = Paths {
///     site: PathBuf::from("public"),
///     content: PathBuf::from("content"),
///     build: PathBuf::from("build"),
///     template: PathBuf::from("templates"),
/// };
/// ```
#[derive(Debug)]
pub struct Paths {
    /// Root directory for the generated website output
    pub site: PathBuf,
    /// Source directory containing markdown and other content files
    pub content: PathBuf,
    /// Temporary directory for build artifacts and intermediate files
    pub build: PathBuf,
    /// Directory containing site templates and layout definitions
    pub template: PathBuf,
}

/// Executes the static site generation process.
///
/// This function orchestrates the entire site generation process through several key stages:
///
/// 1. Logging System Initialisation
///    - Creates and configures the log file
///    - Establishes logging infrastructure
///
/// 2. Command-Line Interface
///    - Displays the CLI banner
///    - Processes user arguments
///
/// 3. Directory Structure
///    - Creates required directories
///    - Validates directory permissions
///    - Ensures path safety
///
/// 4. Site Compilation
///    - Processes markdown content
///    - Applies templates
///    - Generates static files
///
/// 5. Development Server (Optional)
///    - Configures local server
///    - Serves compiled content
///
/// # Errors
///
/// Returns an error if:
/// * Required command-line arguments are missing
/// * File system operations fail (e.g., insufficient permissions)
/// * Site compilation encounters errors
/// * Development server fails to start
///
/// # Example
///
/// ```rust,no_run
/// use ssg::run;
///
/// fn main() -> anyhow::Result<()> {
///     // Run the static site generator
///     run()?;
///     println!("Site generation completed successfully");
///     Ok(())
/// }
/// ```
///
/// # Performance Characteristics
///
/// * Time Complexity: O(n) where n is the number of files
/// * Space Complexity: O(m) where m is the average file size
pub fn run() -> Result<()> {
    // Initialize logging
    let date = DateTime::new();
    let mut log_file = create_log_file("./ssg.log")
        .context("Failed to create log file")?;

    // Display banner and log initialization
    cmd::print_banner();
    log_initialization(&mut log_file, &date)?;

    // Parse command-line arguments
    let matches = cmd::build().get_matches();
    log_arguments(&mut log_file, &date)?;

    // Extract and validate paths
    let paths = extract_paths(&matches)?;
    create_directories(&paths)?;

    // Compile the site
    compile(&paths.build, &paths.content, &paths.site, &paths.template)
        .context("Failed to compile site")?;

    // Handle server if requested
    if let Some(serve_dir) = matches.get_one::<PathBuf>("serve") {
        handle_server(&mut log_file, &date, &paths, serve_dir)?;
    }

    Ok(())
}

/// Validates and copies files from source to destination.
///
/// This function performs comprehensive safety checks before copying files,
/// including path validation, symlink detection, and size limitations.
///
/// # Arguments
///
/// * `src` - Source path to copy from
/// * `dst` - Destination path to copy to
///
/// # Returns
///
/// Returns `Ok(())` if the copy operation succeeds, or an error if:
/// * Source path is invalid or inaccessible
/// * Source contains symlinks (not allowed)
/// * Files exceed size limits (default: 10MB)
/// * Destination cannot be created or written to
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use ssg::verify_and_copy_files;
///
/// fn main() -> anyhow::Result<()> {
///     let source = Path::new("source_directory");
///     let destination = Path::new("destination_directory");
///     
///     verify_and_copy_files(source, destination)?;
///     println!("Files copied successfully");
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// This function implements several security measures:
/// * Path traversal prevention
/// * Symlink restriction
/// * File size limits
/// * Permission validation
pub fn verify_and_copy_files(src: &Path, dst: &Path) -> Result<()> {
    ensure!(
        is_safe_path(src)?,
        "Source directory is unsafe or inaccessible: {:?}",
        src
    );

    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {:?}", src);
    }

    // If source is a file, verify its safety
    if src.is_file() {
        verify_file_safety(src)?;
    }

    // Ensure the destination directory exists
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create or access destination directory at path: {:?}", dst))?;

    // Copy directory contents with safety checks
    copy_dir_all(src, dst).with_context(|| {
        format!("Failed to copy files from source: {:?} to destination: {:?}", src, dst)
    })?;

    Ok(())
}

/// Asynchronously validates and copies files between directories.
///
/// Provides an asynchronous implementation of file copying with the same
/// safety guarantees as the synchronous version, using tokio for async I/O.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Returns
///
/// Returns `Ok(())` on successful copy, or an error if:
/// * Source does not exist or is inaccessible
/// * Source contains unsafe elements (symlinks, oversized files)
/// * Destination cannot be created or written to
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use ssg::verify_and_copy_files_async;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let src = Path::new("content");
///     let dst = Path::new("public");
///
///     verify_and_copy_files_async(src, dst).await?;
///     println!("Files copied asynchronously");
///     Ok(())
/// }
/// ```
///
/// # Feature Flag
///
/// This function is only available when the `async` feature is enabled:
/// ```toml
/// [dependencies]
/// ssg = { version = "0.1", features = ["async"] }
/// ```
#[cfg(feature = "async")]
pub async fn verify_and_copy_files_async(
    src: &Path,
    dst: &Path,
) -> Result<()> {
    // First check existence since it's a simple check
    if !src.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {:?}",
            src
        ));
    }

    // Then check path safety
    ensure!(
        is_safe_path(src)?,
        "Source directory is unsafe or inaccessible: {:?}",
        src
    );

    // If source is a file, verify its safety
    if src.is_file() {
        verify_file_safety(src)?;
    }

    // Create destination directory
    tokio::fs::create_dir_all(dst)
        .await
        .with_context(|| format!("Failed to create or access destination directory at path: {:?}", dst))?;

    // Copy directory contents with safety checks
    copy_dir_all_async(src, dst)
        .await
        .with_context(|| format!("Failed to copy files from source: {:?} to destination: {:?}", src, dst))?;

    Ok(())
}

/// Checks if a given path is safe to use.
///
/// Validates that the provided path does not contain directory traversal attempts
/// or other potential security risks.
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// * `Ok(true)` - If the path is safe to use
/// * `Ok(false)` - If the path contains unsafe elements
/// * `Err` - If path validation fails
///
/// # Security
///
/// This function prevents directory traversal attacks by:
/// * Resolving symbolic links
/// * Checking for parent directory references (`..`)
/// * Validating path components
///

pub fn is_safe_path(path: &Path) -> Result<bool> {
    // If path doesn't exist, check its parent
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Ok(true); // Consider non-existent paths safe if they'll be created
            }
        }
    }

    let canonical = path.canonicalize().map_err(|e| {
        anyhow::anyhow!(
            "Failed to canonicalize path {}: {}",
            path.display(),
            e
        )
    })?;

    let normalized = canonical.components().collect::<PathBuf>();

    // Check if the path contains any parent directory references
    let contains_parent_refs = normalized
        .components()
        .any(|comp| matches!(comp, std::path::Component::ParentDir));

    // Consider the path safe if it doesn't contain parent refs and starts with current directory
    Ok(!contains_parent_refs)
}

/// Verifies the safety of a file for processing.
///
/// Performs comprehensive safety checks on a file to ensure it meets security
/// requirements before processing. These checks include symlink detection and
/// file size validation.
///
/// # Arguments
///
/// * `path` - Reference to the path of the file to verify
///
/// # Returns
///
/// * `Ok(())` - If the file passes all safety checks
/// * `Err` - If any safety check fails
///
/// # Safety Checks
///
/// * Symlinks: Not allowed (returns error)
/// * File size: Must be under 10MB
/// * File type: Must be a regular file
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use ssg::verify_file_safety;
///
/// fn main() -> anyhow::Result<()> {
///     let file_path = Path::new("content/index.md");
///     verify_file_safety(file_path)?;
///     println!("File passed safety checks");
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * File is a symlink
/// * File size exceeds 10MB
/// * Cannot read file metadata
pub fn verify_file_safety(path: &Path) -> Result<()> {
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

    // Get symlink metadata without following the symlink
    let symlink_metadata = path.symlink_metadata().map_err(|e| {
        anyhow::anyhow!(
            "Failed to get symlink metadata for {}: {}",
            path.display(),
            e
        )
    })?;

    // Explicitly check for symlinks first
    if symlink_metadata.file_type().is_symlink() {
        return Err(anyhow::anyhow!(
            "Symlinks are not allowed: {}",
            path.display()
        ));
    }

    // Only check size if it's a regular file
    if symlink_metadata.file_type().is_file()
        && symlink_metadata.len() > MAX_FILE_SIZE
    {
        return Err(anyhow::anyhow!(
            "File exceeds maximum allowed size of {} bytes: {}",
            MAX_FILE_SIZE,
            path.display()
        ));
    }

    Ok(())
}

/// Creates and initialises the log file system.
///
/// Establishes a new log file at the specified path with appropriate
/// permissions and write capabilities.
///
/// # Arguments
///
/// * `file_path` - The desired location for the log file
///
/// # Errors
///
/// Returns an error if:
/// * The specified path is invalid
/// * File creation permissions are insufficient
/// * The parent directory is not writable
fn create_log_file(file_path: &str) -> Result<File> {
    File::create(file_path).context("Failed to create log file")
}

/// Records system initialisation in the logging system.
///
/// Creates a detailed log entry capturing the system's startup state,
/// configuration, and initial conditions.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Errors
///
/// Returns an error if:
/// * Writing to the log file fails
/// * Log message translation fails
/// * File system errors occur
fn log_initialization(
    log_file: &mut File,
    date: &DateTime,
) -> Result<()> {
    let banner_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_banner_log_msg").unwrap(),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", banner_log)
        .context("Failed to write banner log")
}

/// Logs processed command-line arguments.
///
/// Records all provided command-line arguments and their values
/// for debugging and audit purposes.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Errors
///
/// Returns an error if:
/// * Writing to the log file fails
/// * Message translation fails
/// * File system errors occur
fn log_arguments(log_file: &mut File, date: &DateTime) -> Result<()> {
    let args_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_banner_log_msg").unwrap_or_else(|_| {
            "Default banner log message".to_string()
        }),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", args_log)
        .context("Failed to write arguments log")
}

/// Processes and validates paths from command-line arguments.
///
/// Extracts all required path information from the provided arguments
/// whilst ensuring their validity and accessibility.
///
/// # Arguments
///
/// * `matches` - Parsed command-line arguments containing path information
///
/// # Returns
///
/// A Result containing a validated Paths structure with all necessary
/// directory information.
///
/// # Errors
///
/// Returns an error if:
/// * Required paths are missing from arguments
/// * Paths are malformed or invalid
/// * Specified directories are inaccessible
fn extract_paths(matches: &clap::ArgMatches) -> Result<Paths> {
    let site_name = matches
        .get_one::<String>("new")
        .context("Project name not specified")?;

    let content_dir = matches
        .get_one::<PathBuf>("content")
        .context("Content directory not specified")?;

    let output_dir = matches
        .get_one::<PathBuf>("output")
        .context("Output directory not specified")?;

    let template_dir = matches
        .get_one::<PathBuf>("template")
        .context("Template directory not specified")?;

    Ok(Paths {
        site: PathBuf::from(site_name), // Convert site_name String to PathBuf here
        content: content_dir.clone(),
        build: output_dir.clone(),
        template: template_dir.clone(),
    })
}

/// Ensures the existence of required directories and checks if paths are safe.
///
/// # Errors
///
/// Returns a user-friendly error if any of the required directories are missing or inaccessible.
fn create_directories(paths: &Paths) -> Result<()> {
    // Ensure each directory exists, with custom error messages for each.
    fs::create_dir_all(&paths.content)
        .with_context(|| format!("Failed to create or access content directory at path: {:?}", &paths.content))?;
    fs::create_dir_all(&paths.build).with_context(|| {
        format!(
            "Failed to create or access build directory at path: {:?}",
            &paths.build
        )
    })?;
    fs::create_dir_all(&paths.site).with_context(|| {
        format!(
            "Failed to create or access site directory at path: {:?}",
            &paths.site
        )
    })?;
    fs::create_dir_all(&paths.template)
        .with_context(|| format!("Failed to create or access template directory at path: {:?}", &paths.template))?;

    // Path safety check with additional context
    if !is_safe_path(&paths.content)?
        || !is_safe_path(&paths.build)?
        || !is_safe_path(&paths.site)?
        || !is_safe_path(&paths.template)?
    {
        anyhow::bail!("One or more paths are unsafe. Ensure paths do not contain '..' and are accessible.");
    }

    // Optional directory listing with error context
    list_directory_contents(&paths.content)
        .with_context(|| format!("Failed to list contents of content directory at path: {:?}", &paths.content))?;
    Ok(())
}

/// Configures and launches the development server.
///
/// Sets up a local server for testing and previewing the generated
/// site, including file copying and server configuration.
///
/// # Arguments
///
/// * `log_file` - Reference to the active log file
/// * `date` - Current timestamp for logging
/// * `paths` - All required directory paths
/// * `serve_dir` - Directory to serve content from
///
/// # Errors
///
/// Returns an error if:
/// * Server configuration fails
/// * Directory setup fails
/// * File copying encounters errors
/// * Server fails to start
fn handle_server(
    log_file: &mut File,
    date: &DateTime,
    paths: &Paths,
    serve_dir: &PathBuf,
) -> Result<()> {
    // Log server initialization
    let server_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_server_log_msg").unwrap(),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", server_log)?;

    fs::create_dir_all(serve_dir)
        .context("Failed to create serve directory")?;

    println!("Setting up server...");
    println!("Source: {}", paths.site.display());
    println!("Serving from: {}", serve_dir.display());

    if serve_dir != &paths.site {
        verify_and_copy_files(&paths.site, serve_dir)?;
    }

    println!("\nStarting server at http://127.0.0.1:8000");
    println!("Serving content from: {}", serve_dir.display());

    macro_serve!("127.0.0.1:8000", serve_dir.to_str().unwrap());
    Ok(())
}

/// Recursively collects all files within a given directory.
///
/// # Parameters
///
/// * `dir`: A reference to the directory to search for files.
/// * `files`: A mutable vector to store the collected file paths.
///
/// # Returns
///
/// * `Result<()>`: Returns an error if any file system operations fail.
///
/// # Example
///
/// ```rust
/// use ssg::collect_files_recursive;
/// use std::path::Path;
/// use std::fs;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut files = Vec::new();
///     let dir_path = Path::new("content");
///
///     collect_files_recursive(dir_path, &mut files)?;
///
///     for file in files {
///         println!("{}", file.display());
///     }
///
///     Ok(())
/// }
/// ```
pub fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry;
        let path = entry?.path();

        if path.is_dir() {
            collect_files_recursive(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// Performs recursive directory copying.
///
/// Copies entire directory structures whilst preserving
/// file attributes and handling nested directories.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Errors
///
/// Returns an error if:
/// * Directory creation fails
/// * File copying fails
/// * Permission issues occur
/// * Resource limitations are reached
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    // First read all entries
    let entries: Vec<_> =
        fs::read_dir(src)?.collect::<std::io::Result<Vec<_>>>()?;

    // Now process them in parallel
    entries
        .into_par_iter()
        .try_for_each(|entry| -> Result<()> {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
                // Verify file safety before copying
                verify_file_safety(&src_path)?;
                _ = fs::copy(&src_path, &dst_path)?;
            }
            Ok(())
        })?;

    Ok(())
}

/// Asynchronously copies an entire directory structure, preserving file attributes and handling nested directories.
///
/// # Parameters
///
/// * `src`: A reference to the source directory path.
/// * `dst`: A reference to the destination directory path.
///
/// # Returns
///
/// * `Result<()>`:
///   - `Ok(())`: If the directory copying is successful.
///   - `Err(e)`: If an error occurs during the directory copying, where `e` is the associated error.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `std::io::Error`: If an error occurs during directory creation, file copying, or permission issues.
/// * `anyhow::Error`: If a file safety check fails.
#[cfg(feature = "async")]
pub async fn copy_dir_all_async(src: &Path, dst: &Path) -> Result<()> {
    internal_copy_dir_async(src, dst).await
}

#[cfg(feature = "async")]
async fn internal_copy_dir_async(src: &Path, dst: &Path) -> Result<()> {
    tokio::fs::create_dir_all(dst).await?;

    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];

    while let Some((src_path, dst_path)) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&src_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let src_entry = entry.path();
            let dst_entry = dst_path.join(entry.file_name());

            if src_entry.is_dir() {
                tokio::fs::create_dir_all(&dst_entry).await?;
                stack.push((src_entry, dst_entry));
            } else {
                verify_file_safety(&src_entry)?;
                _ = tokio::fs::copy(&src_entry, &dst_entry).await?;
            }
        }
    }

    Ok(())
}

/// Creates a recursive directory listing.
///
/// Generates a complete listing of directory contents
/// for verification and debugging purposes.
///
/// # Arguments
///
/// * `dir` - Directory to list recursively
///
/// # Errors
///
/// Returns an error if:
/// * Directory access fails
/// * Permission issues occur
/// * Resource limits are exceeded
fn list_directory_contents(dir: &Path) -> Result<()> {
    let entries: Vec<_> =
        fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;

    entries.par_iter().try_for_each(|entry| -> Result<()> {
        let path = entry.path();
        if path.is_dir() {
            list_directory_contents(&path)?;
        }
        Ok(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::{
        fs::{self, File},
        path::PathBuf,
    };
    use tempfile::tempdir;

    #[test]
    fn test_create_log_file_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("test.log");

        let log_file =
            create_log_file(log_file_path.to_str().unwrap())?;
        assert!(log_file.metadata()?.is_file());

        Ok(())
    }

    #[test]
    fn test_create_log_file_failure() {
        let invalid_path = "/invalid_path/test.log";
        let result = create_log_file(invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_initialization() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("init_log.log");
        let mut log_file = File::create(&log_file_path)?;

        let date = DateTime::new();
        log_initialization(&mut log_file, &date)?;

        let log_content = fs::read_to_string(log_file_path)?;
        assert!(log_content.contains("process"));

        Ok(())
    }

    #[test]
    fn test_log_arguments() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("args_log.log");
        let mut log_file = File::create(&log_file_path)?;

        let date = DateTime::new();
        log_arguments(&mut log_file, &date)?;

        let log_content = fs::read_to_string(log_file_path)?;
        assert!(log_content.contains("process"));

        Ok(())
    }

    #[test]
    fn test_create_directories_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();

        let paths = Paths {
            site: base_path.join("public"),
            content: base_path.join("content"),
            build: base_path.join("build"),
            template: base_path.join("templates"),
        };

        create_directories(&paths)?;

        // Verify each directory exists
        assert!(paths.site.exists());
        assert!(paths.content.exists());
        assert!(paths.build.exists());
        assert!(paths.template.exists());

        Ok(())
    }

    #[test]
    fn test_create_directories_failure() {
        let invalid_paths = Paths {
            site: PathBuf::from("/invalid/site"),
            content: PathBuf::from("/invalid/content"),
            build: PathBuf::from("/invalid/build"),
            template: PathBuf::from("/invalid/template"),
        };

        let result = create_directories(&invalid_paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_all() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let src_file = src_dir.path().join("test_file.txt");
        _ = File::create(&src_file)?;

        let result = copy_dir_all(src_dir.path(), dst_dir.path());
        assert!(result.is_ok());
        assert!(dst_dir.path().join("test_file.txt").exists());

        Ok(())
    }

    #[test]
    fn test_run_success() {
        // Mock data for test
        // Additional setup and teardown logic needed to simulate environment for `run()`
    }

    #[test]
    fn test_verify_and_copy_files_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();

        // Create source directory and test file
        let src_dir = base_path.join("src");
        fs::create_dir_all(&src_dir)?;
        let test_file = src_dir.join("test_file.txt");
        fs::write(&test_file, "test content")?;

        // Create destination directory
        let dst_dir = base_path.join("dst");

        // Verify and copy files
        verify_and_copy_files(&src_dir, &dst_dir)?;

        // Verify the file was copied
        assert!(dst_dir.join("test_file.txt").exists());

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_failure() {
        let src_dir = PathBuf::from("/invalid/src");
        let dst_dir = PathBuf::from("/invalid/dst");

        let result = verify_and_copy_files(&src_dir, &dst_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_server_failure() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path).unwrap();

        let paths = Paths {
            site: PathBuf::from("/invalid/site"),
            content: PathBuf::from("/invalid/content"),
            build: PathBuf::from("/invalid/build"),
            template: PathBuf::from("/invalid/template"),
        };

        let serve_dir = temp_dir.path().join("serve");
        let result = handle_server(
            &mut log_file,
            &DateTime::new(),
            &paths,
            &serve_dir,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_invalid_paths() {
        // Mock invalid paths to trigger error handling in `run`
        let _paths = Paths {
            site: PathBuf::from("/invalid/site"),
            content: PathBuf::from("/invalid/content"),
            build: PathBuf::from("/invalid/build"),
            template: PathBuf::from("/invalid/template"),
        };

        let result = run();
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory_contents() -> Result<()> {
        let temp_dir = tempdir()?;
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;
        let temp_file = sub_dir.join("test_file.txt");
        _ = File::create(&temp_file)?;

        let result = list_directory_contents(temp_dir.path());
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_is_safe_path_safe() -> Result<()> {
        let temp_dir = tempdir()?;
        let safe_path = temp_dir.path().to_path_buf().join("safe_path");

        // Create the directory
        fs::create_dir_all(&safe_path)?;

        // Use the absolute path
        let absolute_safe_path = safe_path.canonicalize()?;
        assert!(is_safe_path(&absolute_safe_path)?);
        Ok(())
    }

    #[test]
    fn test_is_safe_path_unsafe() {
        let unsafe_path = PathBuf::from("../unsafe_path");
        let result = is_safe_path(&unsafe_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_directories_partial_failure() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().join("valid_dir");
        let invalid_path = PathBuf::from("/invalid/path");

        let paths = Paths {
            site: valid_path,
            content: invalid_path.clone(),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let result = create_directories(&paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_all_nested() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let nested_dir = src_dir.path().join("nested_dir");
        fs::create_dir(&nested_dir)?;

        let nested_file = nested_dir.join("nested_file.txt");
        _ = File::create(&nested_file)?;

        copy_dir_all(src_dir.path(), dst_dir.path())?;
        assert!(dst_dir
            .path()
            .join("nested_dir/nested_file.txt")
            .exists());

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_missing_source() {
        let src_path = PathBuf::from("/non_existent_dir");
        let dst_dir = tempdir().unwrap();

        let result = verify_and_copy_files(&src_path, dst_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_server_missing_serve_dir() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path).unwrap();

        let paths = Paths {
            site: temp_dir.path().join("site"),
            content: temp_dir.path().join("content"),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let non_existent_serve_dir =
            PathBuf::from("/non_existent_serve_dir");
        let result = handle_server(
            &mut log_file,
            &DateTime::new(),
            &paths,
            &non_existent_serve_dir,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_log_initialization_write_failure() {
        // Attempt to create a log file in a read-only directory (use an invalid path)
        let invalid_path = PathBuf::from("/invalid/log_file.log");
        let mut log_file =
            File::create(&invalid_path).unwrap_or_else(|_| {
                // Mock a File instance, handle permissions here
                File::open("/dev/null").unwrap()
            });

        let result =
            log_initialization(&mut log_file, &DateTime::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_collect_files_recursive_empty() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut files = Vec::new();

        collect_files_recursive(temp_dir.path(), &mut files)?;
        assert!(files.is_empty());

        Ok(())
    }

    #[test]
    fn test_print_banner() {
        // Simply call the function to ensure it runs without errors.
        cmd::print_banner();
        // Since this is a print statement, we're only verifying that it doesn't panic.
        // If you need to check output, consider capturing stdout.
    }

    #[test]
    fn test_create_directories_with_unsafe_path() {
        // Intentionally create a path with ".." to simulate an unsafe path
        let unsafe_path = PathBuf::from("../unsafe_path");

        let paths = Paths {
            site: unsafe_path.clone(),
            content: unsafe_path.clone(),
            build: unsafe_path.clone(),
            template: unsafe_path.clone(),
        };

        let result = create_directories(&paths);

        // Check that the result is an error due to unsafe paths
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string().contains("unsafe"),
                "Error should indicate unsafe path"
            );
        }
    }

    #[test]
    fn test_collect_files_recursive_with_nested_directories(
    ) -> Result<()> {
        let temp_dir = tempdir()?;
        let nested_dir = temp_dir.path().join("nested_dir");
        fs::create_dir(&nested_dir)?;

        let nested_file = nested_dir.join("nested_file.txt");
        _ = File::create(&nested_file)?;

        let mut files = Vec::new();
        collect_files_recursive(temp_dir.path(), &mut files)?;

        assert!(files.contains(&nested_file));
        assert_eq!(files.len(), 1);
        Ok(())
    }

    #[test]
    fn test_handle_server_start_message() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path)?;

        let paths = Paths {
            site: temp_dir.path().join("site"),
            content: temp_dir.path().join("content"),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let serve_dir = temp_dir.path().join("serve");

        // Check setup conditions before calling `handle_server`
        fs::create_dir_all(&serve_dir)?;
        assert!(
            serve_dir.exists(),
            "Expected serve directory to be created"
        );

        // Now, call `handle_server` and check for specific output or error
        let result = handle_server(
            &mut log_file,
            &DateTime::new(),
            &paths,
            &serve_dir,
        );
        assert!(
            result.is_err(),
            "Expected handle_server to fail without valid setup"
        );

        Ok(())
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn test_verify_file_safety_symlink() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let symlink_path = temp_dir.path().join("test_link.txt");

        // Create a regular file
        fs::write(&file_path, "test content")?;

        // Create a symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &symlink_path)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&file_path, &symlink_path)?;

        // Debug output
        println!("File exists: {}", file_path.exists());
        println!("Symlink exists: {}", symlink_path.exists());
        println!(
            "Is symlink: {}",
            symlink_path.symlink_metadata()?.file_type().is_symlink()
        );

        // Try to verify the symlink
        let result = verify_file_safety(&symlink_path);

        // Print the result for debugging
        println!("Result: {:?}", result);

        // Verify that we got an error
        assert!(
            result.is_err(),
            "Expected error for symlink, got success"
        );

        // Verify the error message
        let err = result.unwrap_err();
        println!("Error message: {}", err);
        assert!(
            err.to_string().contains("Symlinks are not allowed"),
            "Unexpected error message: {}",
            err
        );

        Ok(())
    }

    #[test]
    fn test_verify_file_safety_size() -> Result<()> {
        let temp_dir = tempdir()?;
        let large_file_path = temp_dir.path().join("large.txt");

        // Create a large file
        let file = File::create(&large_file_path)?;
        file.set_len(11 * 1024 * 1024)?; // 11MB

        let result = verify_file_safety(&large_file_path);
        assert!(result.is_err(), "Expected error for large file");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds maximum allowed size"),
            "Unexpected error message"
        );

        Ok(())
    }

    #[test]
    fn test_verify_file_safety_regular() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("regular.txt");

        // Create a regular file
        fs::write(&file_path, "test content")?;

        assert!(verify_file_safety(&file_path).is_ok());
        Ok(())
    }

    /// Tests successful copying of an empty directory
    #[tokio::test]
    async fn test_copy_empty_directory_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let result =
            copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_ok());

        // Verify destination directory exists
        assert!(dst_dir.path().exists());
        Ok(())
    }

    /// Tests copying a directory with a single file
    #[tokio::test]
    async fn test_copy_single_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a test file
        let test_file = src_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify file was copied
        let copied_file = dst_dir.path().join("test.txt");
        assert!(copied_file.exists());
        assert_eq!(fs::read_to_string(copied_file)?, "test content");

        Ok(())
    }

    /// Tests copying a directory with nested subdirectories
    #[tokio::test]
    async fn test_copy_nested_directories_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create nested directory structure
        let nested_dir = src_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;

        // Create files in both root and nested directory
        fs::write(src_dir.path().join("root.txt"), "root content")?;
        fs::write(nested_dir.join("nested.txt"), "nested content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify directory structure and contents
        assert!(dst_dir.path().join("nested").exists());
        assert!(dst_dir.path().join("root.txt").exists());
        assert!(dst_dir.path().join("nested/nested.txt").exists());

        assert_eq!(
            fs::read_to_string(dst_dir.path().join("root.txt"))?,
            "root content"
        );
        assert_eq!(
            fs::read_to_string(
                dst_dir.path().join("nested/nested.txt")
            )?,
            "nested content"
        );

        Ok(())
    }

    /// Tests handling of symlinks
    #[tokio::test]
    async fn test_copy_with_symlink_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a regular file
        let file_path = src_dir.path().join("original.txt");
        fs::write(&file_path, "original content")?;

        // Create a symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let symlink_path = src_dir.path().join("link.txt");
            symlink(&file_path, &symlink_path)?;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            let symlink_path = src_dir.path().join("link.txt");
            symlink_file(&file_path, &symlink_path)?;
        }

        // Attempt to copy - should fail due to symlink
        let result =
            copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying large files
    #[tokio::test]
    async fn test_copy_large_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a large file (11MB)
        let large_file = src_dir.path().join("large.txt");
        let file = fs::File::create(&large_file)?;
        file.set_len(11 * 1024 * 1024)?;

        // Attempt to copy - should fail due to file size limit
        let result =
            copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying with invalid destination
    #[tokio::test]
    async fn test_copy_invalid_destination_async() -> Result<()> {
        let src_dir = tempdir()?;
        let invalid_dst = PathBuf::from("/nonexistent/path");

        let result =
            copy_dir_all_async(src_dir.path(), &invalid_dst).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests concurrent copying of multiple files
    #[tokio::test]
    async fn test_concurrent_copy_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create multiple files
        for i in 0..5 {
            fs::write(
                src_dir.path().join(format!("file{}.txt", i)),
                format!("content {}", i),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify all files were copied
        for i in 0..5 {
            let copied_file =
                dst_dir.path().join(format!("file{}.txt", i));
            assert!(copied_file.exists());
            assert_eq!(
                fs::read_to_string(copied_file)?,
                format!("content {}", i)
            );
        }

        Ok(())
    }

    /// Tests copying with maximum directory depth
    #[tokio::test]
    async fn test_max_directory_depth_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        let max_depth = 5;

        // Create deeply nested directory structure
        let mut current_dir = src_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{}", i));
            fs::create_dir(&current_dir)?;
            fs::write(
                current_dir.join("file.txt"),
                format!("content level {}", i),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify the entire structure was copied
        current_dir = dst_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{}", i));
            assert!(current_dir.exists());
            assert!(current_dir.join("file.txt").exists());
            assert_eq!(
                fs::read_to_string(current_dir.join("file.txt"))?,
                format!("content level {}", i)
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_missing_source(
    ) -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("nonexistent");
        let dst_dir = temp_dir.path().join("dst");

        let error = verify_and_copy_files_async(&src_dir, &dst_dir)
            .await
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("does not exist"),
            "Expected error message about non-existent source, got: {}",
            error
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_symlink() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create source directory
        tokio::fs::create_dir_all(&src_dir).await?;

        // Create target file
        let target = src_dir.join("target.txt");
        tokio::fs::write(&target, "target content").await?;

        // Create symlink
        let symlink = src_dir.join("symlink.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &symlink)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &symlink)?;

        // Try to verify the symlink directly
        let error = verify_and_copy_files_async(&symlink, &dst_dir)
            .await
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("Symlinks are not allowed"),
            "Expected error message about symlinks, got: {}",
            error
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_large_file() -> Result<()>
    {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let large_file = src_dir.join("large.txt");

        // Create source directory and large file
        tokio::fs::create_dir_all(&src_dir).await?;
        let file = tokio::fs::File::create(&large_file).await?;
        file.set_len(11 * 1024 * 1024).await?; // 11MB

        // Try to verify the large file directly
        let error = verify_and_copy_files_async(
            &large_file,
            &temp_dir.path().join("dst"),
        )
        .await
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("exceeds maximum allowed size"),
            "Expected error message about file size, got: {}",
            error
        );

        Ok(())
    }
}
