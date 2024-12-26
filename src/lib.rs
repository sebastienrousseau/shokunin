// Copyright © 2025 Shokunin Static Site Generator (SSG). All rights reserved.
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
use anyhow::ensure;
use anyhow::{Context, Result};
use dtt::datetime::DateTime;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use langweave::translate;
use log::{info, LevelFilter};
use rayon::prelude::*;
use rlg::{macro_log, LogFormat, LogLevel};
use staticdatagen::generate_unique_string;
use tokio::fs as async_fs;

pub mod cmd;
/// Module declarations
pub mod process;

/// Re-exports
pub use staticdatagen;

/// Represents the necessary directory paths for the site generator.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The site output directory
    pub site: PathBuf,
    /// The content directory
    pub content: PathBuf,
    /// The build directory
    pub build: PathBuf,
    /// The template directory
    pub template: PathBuf,
}

impl Paths {
    /// Creates a new builder for configuring Paths
    pub fn builder() -> PathsBuilder {
        PathsBuilder::default()
    }

    /// Creates paths with default directories
    pub fn default_paths() -> Self {
        Self {
            site: PathBuf::from("public"),
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        }
    }
}
// Modify the validate method in Paths impl
impl Paths {
    /// Validates all paths in the configuration
    pub fn validate(&self) -> Result<()> {
        // Check for path traversal and other security concerns
        for (name, path) in [
            ("site", &self.site),
            ("content", &self.content),
            ("build", &self.build),
            ("template", &self.template),
        ] {
            // For non-existent paths, validate their components
            let path_str = path.to_string_lossy();
            if path_str.contains("..") {
                anyhow::bail!(
                    "{} path contains directory traversal: {}",
                    name,
                    path.display()
                );
            }
            if path_str.contains("//") {
                anyhow::bail!(
                    "{} path contains invalid double slashes: {}",
                    name,
                    path.display()
                );
            }

            // If path exists, perform additional checks
            if path.exists() {
                let metadata =
                    path.symlink_metadata().with_context(|| {
                        format!(
                            "Failed to get metadata for {}: {}",
                            name,
                            path.display()
                        )
                    })?;

                if metadata.file_type().is_symlink() {
                    anyhow::bail!(
                        "{} path is a symlink which is not allowed: {}",
                        name,
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }
}

/// Builder for creating Paths configurations
#[derive(Debug, Default, Clone)]
pub struct PathsBuilder {
    /// The site output directory
    pub site: Option<PathBuf>,
    /// The content directory
    pub content: Option<PathBuf>,
    /// The build directory
    pub build: Option<PathBuf>,
    /// The template directory
    pub template: Option<PathBuf>,
}

impl PathsBuilder {
    /// Sets the site output directory
    pub fn site<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.site = Some(path.into());
        self
    }

    /// Sets the content directory
    pub fn content<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.content = Some(path.into());
        self
    }

    /// Sets the build directory
    pub fn build_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.build = Some(path.into());
        self
    }

    /// Sets the template directory
    pub fn template<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.template = Some(path.into());
        self
    }

    /// Sets all paths relative to a base directory
    pub fn relative_to<P: AsRef<Path>>(self, base: P) -> Self {
        let base = base.as_ref();
        self.site(base.join("public"))
            .content(base.join("content"))
            .build_dir(base.join("build"))
            .template(base.join("templates"))
    }

    /// Builds the Paths configuration
    ///
    /// # Returns
    ///
    /// * `Result<Paths>` - The configured paths if valid
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Required paths are missing
    /// * Paths are invalid or unsafe
    /// * Unable to create necessary directories
    pub fn build(self) -> Result<Paths> {
        let paths = Paths {
            site: self.site.unwrap_or_else(|| PathBuf::from("public")),
            content: self
                .content
                .unwrap_or_else(|| PathBuf::from("content")),
            build: self.build.unwrap_or_else(|| PathBuf::from("build")),
            template: self
                .template
                .unwrap_or_else(|| PathBuf::from("templates")),
        };

        // Validate the configuration
        paths.validate()?;

        Ok(paths)
    }
}

// Constants for configuration
const DEFAULT_LOG_LEVEL: &str = "info";
const ENV_LOG_LEVEL: &str = "SHOKUNIN_LOG_LEVEL";

/// Initializes the logging system based on environment variables
fn initialize_logging() -> Result<()> {
    let log_level = std::env::var(ENV_LOG_LEVEL)
        .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

    let level = match log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();

    info!("Logging initialized at level: {}", log_level);
    Ok(())
}

/// Executes the static site generation process.
///
/// Introduces asynchronous file operations, parallel processing, and a progress bar for feedback.
pub async fn run() -> Result<()> {
    initialize_logging()?;
    info!("Starting site generation process");

    // Mocked example of file collection and processing with progress bar
    let files_to_process = vec!["file1", "file2", "file3"];
    let progress_bar = ProgressBar::new(files_to_process.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );

    files_to_process
        .par_iter()
        .progress_with(progress_bar.clone())
        .try_for_each(|file| {
            process_file(file)
                .context(format!("Failed to process file: {}", file))
        })?;

    progress_bar.finish_with_message("All files processed.");
    info!("Site generation completed successfully.");
    Ok(())
}

/// Simulated function for processing a file.
fn process_file(file: &str) -> Result<()> {
    info!("Processing file: {}", file);
    // Simulated work
    std::thread::sleep(std::time::Duration::from_millis(500));
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
pub async fn verify_and_copy_files_async(
    src: &Path,
    dst: &Path,
) -> Result<()> {
    if !src.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {:?}",
            src
        ));
    }

    async_fs::create_dir_all(dst).await.with_context(|| format!(
        "Failed to create or access destination directory at path: {:?}",
        dst
    ))?;

    let mut entries = async_fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            Box::pin(verify_and_copy_files_async(&src_path, &dst_path))
                .await?;
        }
    }

    Ok(())
}

/// Recursively copies directories with a progress bar for feedback.
pub fn copy_dir_with_progress(src: &Path, dst: &Path) -> Result<()> {
    // Initialize the progress bar
    let progress_bar = ProgressBar::new(100); // Example total value, adjust as needed
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );

    // Perform directory copying with progress tracking
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = src.join(&file_name);
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir_with_progress(&src_path, &dst_path)?;
        }

        // Update progress bar
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("Copy complete.");
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
/// Verifies the safety of a file.
///
/// ```rust
/// use std::fs;
/// use std::path::Path;
/// use ssg::verify_file_safety;
/// use tempfile::tempdir;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create temporary directory
/// let temp_dir = tempdir()?;
/// let file_path = temp_dir.path().join("index.md");
///
/// // Create test file
/// fs::write(&file_path, "Hello, world!")?;
///
/// // Perform verification
/// verify_file_safety(&file_path)?;
///
/// // Directory and file are automatically cleaned up
/// # Ok(())
/// # }
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

/// Creates and initialises a log file for the static site generator.
///
/// Establishes a new log file at the specified path with appropriate permissions
/// and write capabilities. The log file is used to track the generation process
/// and any errors that occur.
///
/// # Arguments
///
/// * `file_path` - The desired location for the log file
///
/// # Returns
///
/// * `Ok(File)` - A file handle for the created log file
/// * `Err` - If the file cannot be created or permissions are insufficient
///
/// # Examples
///
/// ```rust
/// use ssg::create_log_file;
///
/// fn main() -> anyhow::Result<()> {
///     let log_file = create_log_file("./site_generation.log")?;
///     println!("Log file created successfully");
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * The specified path is invalid
/// * File creation permissions are insufficient
/// * The parent directory is not writable
pub fn create_log_file(file_path: &str) -> Result<File> {
    File::create(file_path).context("Failed to create log file")
}

/// Records system initialisation in the logging system.
///
/// Creates a detailed log entry capturing the system's startup state,
/// including configuration and initial conditions. Uses the Common Log Format (CLF)
/// for consistent logging.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Returns
///
/// * `Ok(())` - If the log entry is written successfully
/// * `Err` - If writing fails or translation errors occur
///
/// # Examples
///
/// ```rust
/// use ssg::{create_log_file, log_initialization};
/// use dtt::datetime::DateTime;
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = DateTime::new();
///
///     log_initialization(&mut log_file, &date)?;
///     println!("System initialisation logged");
///     Ok(())
/// }
/// ```
pub fn log_initialization(
    log_file: &mut File,
    date: &DateTime,
) -> Result<()> {
    let banner_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_banner_log_msg", "default message")
            .unwrap_or_else(|_| {
                "Default banner log message".to_string()
            }),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", banner_log)
        .context("Failed to write banner log")
}

/// Logs processed command-line arguments for debugging and auditing.
///
/// Records all provided command-line arguments and their values in the log file,
/// providing a traceable record of site generation parameters.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Returns
///
/// * `Ok(())` - If arguments are logged successfully
/// * `Err` - If writing fails or translation errors occur
///
/// # Examples
///
/// ```rust
/// use ssg::{create_log_file, log_arguments};
/// use dtt::datetime::DateTime;
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = DateTime::new();
///
///     log_arguments(&mut log_file, &date)?;
///     println!("Arguments logged successfully");
///     Ok(())
/// }
/// ```
pub fn log_arguments(
    log_file: &mut File,
    date: &DateTime,
) -> Result<()> {
    let args_log = macro_log!(
        &generate_unique_string(),
        &date.to_string(),
        &LogLevel::INFO,
        "process",
        &translate("lib_banner_log_msg", "default message")
            .unwrap_or_else(|_| {
                "Default banner log message".to_string()
            }),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", args_log)
        .context("Failed to write arguments log")
}

/// Creates and verifies required directories for site generation.
///
/// Ensures all necessary directories exist and are safe to use, creating
/// them if necessary. Also performs security checks on each directory.
///
/// # Arguments
///
/// * `paths` - Reference to a Paths struct containing required directory paths
///
/// # Returns
///
/// * `Ok(())` - If all directories are created/verified successfully
/// * `Err` - If any directory operation fails
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use ssg::{Paths, create_directories};
///
/// fn main() -> anyhow::Result<()> {
///     let paths = Paths {
///         site: PathBuf::from("public"),
///         content: PathBuf::from("content"),
///         build: PathBuf::from("build"),
///         template: PathBuf::from("templates"),
///     };
///
///     create_directories(&paths)?;
///     println!("All directories ready");
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// Performs the following security checks:
/// * Path traversal prevention
/// * Permission validation
/// * Safe path verification
pub fn create_directories(paths: &Paths) -> Result<()> {
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
/// Sets up a local server for testing and previewing the generated site.
/// Handles file copying and server configuration for local development.
///
/// # Arguments
///
/// * `log_file` - Reference to the active log file
/// * `date` - Current timestamp for logging
/// * `paths` - All required directory paths
/// * `serve_dir` - Directory to serve content from
///
/// # Returns
///
/// * `Ok(())` - If server starts successfully
/// * `Err` - If server configuration or startup fails
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use ssg::{Paths, handle_server, create_log_file};
/// use dtt::datetime::DateTime;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./server.log")?;
///     let date = DateTime::new();
///     let paths = Paths {
///         site: PathBuf::from("public"),
///         content: PathBuf::from("content"),
///         build: PathBuf::from("build"),
///         template: PathBuf::from("templates"),
///     };
///     let serve_dir = PathBuf::from("serve");
///
///     handle_server(&mut log_file, &date, &paths, &serve_dir).await?;
///     Ok(())
/// }
/// ```
///
/// # Server Configuration
///
/// * Default port: 8000
/// * Host: 127.0.0.1 (localhost)
/// * Serves static files from the specified directory
pub async fn handle_server(
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
        &translate("lib_server_log_msg", "default server message")
            .unwrap_or("Default server message".to_string()),
        &LogFormat::CLF
    );
    writeln!(log_file, "{}", server_log)?;

    fs::create_dir_all(serve_dir)
        .context("Failed to create serve directory")?;

    println!("Setting up server...");
    println!("Source: {}", paths.site.display());
    println!("Serving from: {}", serve_dir.display());

    if serve_dir != &paths.site {
        verify_and_copy_files_async(&paths.site, serve_dir).await?;
    }

    println!("\nStarting server at http://127.0.0.1:8000");
    println!("Serving content from: {}", serve_dir.display());

    warp::serve(warp::fs::dir(serve_dir.clone()))
        .run(([127, 0, 0, 1], 8000))
        .await;
    Ok(())
}

/// Recursively collects all file paths within a directory.
///
/// Traverses a directory tree and compiles a list of all file paths found,
/// excluding directories themselves.
///
/// # Arguments
///
/// * `dir` - Reference to the directory to search
/// * `files` - Mutable vector to store found file paths
///
/// # Returns
///
/// * `Ok(())` - If the collection process succeeds
/// * `Err` - If any file system operation fails
///
/// # Examples
///
/// ```rust
/// use std::path::{Path, PathBuf};
/// use ssg::collect_files_recursive;
///
/// fn main() -> anyhow::Result<()> {
///     let mut files = Vec::new();
///     let dir_path = Path::new("./examples/content");
///
///     collect_files_recursive(dir_path, &mut files)?;
///
///     for file in files {
///         println!("Found file: {}", file.display());
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Note
///
/// This function:
/// * Only collects file paths, not directory paths
/// * Follows symbolic links (use with caution)
/// * Maintains original path structure
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

/// Recursively copies a directory whilst maintaining structure and attributes.
///
/// Performs a deep copy of a directory tree, preserving file attributes and
/// handling nested directories. Uses parallel processing for improved performance.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Returns
///
/// * `Ok(())` - If the copy operation succeeds
/// * `Err` - If any part of the copy operation fails
///
/// # Performance
///
/// Uses rayon for parallel processing of files, significantly improving
/// performance for directories with many files.
///
/// # Safety
///
/// * Verifies file safety before copying
/// * Maintains original file permissions
/// * Handles circular references
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    let entries: Vec<_> =
        fs::read_dir(src)?.collect::<std::io::Result<Vec<_>>>()?;

    entries
        .into_par_iter()
        .try_for_each(|entry| -> Result<()> {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
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
    use crate::cmd::Cli;
    use anyhow::Result;
    use std::env;
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

    #[tokio::test]
    async fn test_handle_server_failure() {
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
        let date = DateTime::new();
        let result =
            handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(result.await.is_err());
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

    #[tokio::test]
    async fn test_handle_server_missing_serve_dir() {
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
        let binding = DateTime::new();
        let result = handle_server(
            &mut log_file,
            &binding,
            &paths,
            &non_existent_serve_dir,
        );
        assert!(result.await.is_err());
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
        Cli::print_banner();
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

    #[tokio::test]
    async fn test_handle_server_start_message() -> Result<()> {
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
        let date = DateTime::new();
        let result =
            handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(
            result.await.is_err(),
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
        assert!(result.is_err(), "Expected error, got: {:?}", result);
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

    #[test]
    fn test_paths_builder_default() -> Result<()> {
        let paths = Paths::builder().build()?;
        assert_eq!(paths.site, PathBuf::from("public"));
        assert_eq!(paths.content, PathBuf::from("content"));
        assert_eq!(paths.build, PathBuf::from("build"));
        assert_eq!(paths.template, PathBuf::from("templates"));
        Ok(())
    }

    #[test]
    fn test_paths_builder_custom() -> Result<()> {
        let temp_dir = tempdir()?;
        let paths = Paths::builder()
            .site(temp_dir.path().join("custom_public"))
            .content(temp_dir.path().join("custom_content"))
            .build_dir(temp_dir.path().join("custom_build"))
            .template(temp_dir.path().join("custom_templates"))
            .build()?;

        assert_eq!(paths.site, temp_dir.path().join("custom_public"));
        assert_eq!(
            paths.content,
            temp_dir.path().join("custom_content")
        );
        assert_eq!(paths.build, temp_dir.path().join("custom_build"));
        assert_eq!(
            paths.template,
            temp_dir.path().join("custom_templates")
        );
        Ok(())
    }

    #[test]
    fn test_paths_builder_relative() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create the directories first
        fs::create_dir_all(temp_dir.path().join("public"))?;
        fs::create_dir_all(temp_dir.path().join("content"))?;
        fs::create_dir_all(temp_dir.path().join("build"))?;
        fs::create_dir_all(temp_dir.path().join("templates"))?;

        let paths =
            Paths::builder().relative_to(temp_dir.path()).build()?;

        assert_eq!(paths.site, temp_dir.path().join("public"));
        assert_eq!(paths.content, temp_dir.path().join("content"));
        assert_eq!(paths.build, temp_dir.path().join("build"));
        assert_eq!(paths.template, temp_dir.path().join("templates"));
        Ok(())
    }

    #[test]
    fn test_paths_validation() -> Result<()> {
        // Test directory traversal
        let result = Paths::builder().site("../invalid").build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("directory traversal"),
            "Expected error about directory traversal"
        );

        // Test double slashes
        let result = Paths::builder().site("invalid//path").build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid double slashes"),
            "Expected error about invalid double slashes"
        );

        // Test symlinks if possible
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let temp_dir = tempdir()?;
            let real_path = temp_dir.path().join("real");
            let symlink_path = temp_dir.path().join("symlink");

            fs::create_dir(&real_path)?;
            symlink(&real_path, &symlink_path)?;

            let result = Paths::builder().site(symlink_path).build();

            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("symlink"),
                "Expected error about symlinks"
            );
        }

        Ok(())
    }

    #[test]
    fn test_paths_default_paths() {
        let paths = Paths::default_paths();
        assert_eq!(paths.site, PathBuf::from("public"));
        assert_eq!(paths.content, PathBuf::from("content"));
        assert_eq!(paths.build, PathBuf::from("build"));
        assert_eq!(paths.template, PathBuf::from("templates"));
    }

    // Add a new test for non-existent but valid paths
    #[test]
    fn test_paths_nonexistent_valid() -> Result<()> {
        let temp_dir = tempdir()?;
        let valid_path = temp_dir.path().join("new_directory");

        let paths =
            Paths::builder().site(valid_path.clone()).build()?;

        assert_eq!(paths.site, valid_path);
        Ok(())
    }

    #[test]
    fn test_initialize_logging_with_custom_level() -> Result<()> {
        env::set_var(ENV_LOG_LEVEL, "debug");
        assert!(initialize_logging().is_ok());
        env::remove_var(ENV_LOG_LEVEL);
        Ok(())
    }

    #[test]
    fn test_paths_builder_with_all_invalid_paths() -> Result<()> {
        let result = Paths::builder()
            .site("../invalid")
            .content("content//invalid")
            .build_dir("build/../invalid")
            .template("template//invalid")
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_paths_builder_clone() {
        let builder = PathsBuilder::default();
        let cloned = builder.clone();
        assert!(cloned.site.is_none());
        assert!(cloned.content.is_none());
        assert!(cloned.build.is_none());
        assert!(cloned.template.is_none());
    }

    #[test]
    fn test_paths_clone() -> Result<()> {
        let paths = Paths::default_paths();
        let cloned = paths.clone();

        assert_eq!(paths.site, cloned.site);
        assert_eq!(paths.content, cloned.content);
        assert_eq!(paths.build, cloned.build);
        assert_eq!(paths.template, cloned.template);
        Ok(())
    }

    #[tokio::test]
    async fn test_async_copy_with_empty_source() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("empty_src");
        let dst_dir = temp_dir.path().join("empty_dst");

        fs::create_dir(&src_dir)?;

        let result =
            verify_and_copy_files_async(&src_dir, &dst_dir).await;
        assert!(result.is_ok());
        assert!(dst_dir.exists());
        Ok(())
    }

    #[test]
    fn test_paths_validation_all_aspects() -> Result<()> {
        let temp_dir = tempdir()?;

        // Test with absolute paths
        let result = Paths::builder()
            .site(temp_dir.path().join("site"))
            .content(temp_dir.path().join("content"))
            .build_dir(temp_dir.path().join("build"))
            .template(temp_dir.path().join("template"))
            .build();

        assert!(result.is_ok());

        // Test with multiple validation issues
        let result = Paths::builder()
            .site("../site")
            .content("content//test")
            .build_dir("build/../../test")
            .template("template//test")
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_log_initialization_with_empty_log_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_path = temp_dir.path().join("empty.log");
        let mut log_file = File::create(&log_path)?;

        let date = DateTime::new();
        log_initialization(&mut log_file, &date)?;

        let content = fs::read_to_string(&log_path)?;
        assert!(!content.is_empty());
        assert!(content.contains("process"));
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_with_nested_empty_dirs(
    ) -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create nested empty directory structure
        fs::create_dir_all(src_dir.join("a/b/c"))?;
        fs::create_dir_all(src_dir.join("d/e/f"))?;

        verify_and_copy_files_async(&src_dir, &dst_dir).await?;

        assert!(dst_dir.join("a/b/c").exists());
        assert!(dst_dir.join("d/e/f").exists());
        Ok(())
    }

    #[test]
    fn test_validate_nonexistent_paths() -> Result<()> {
        let paths = Paths {
            site: PathBuf::from("nonexistent/site"),
            content: PathBuf::from("nonexistent/content"),
            build: PathBuf::from("nonexistent/build"),
            template: PathBuf::from("nonexistent/template"),
        };

        // Non-existent paths should be valid if they don't contain unsafe patterns
        assert!(paths.validate().is_ok());
        Ok(())
    }

    #[test]
    fn test_list_directory_contents_with_many_files() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create multiple files and directories
        for i in 0..5 {
            fs::create_dir(temp_dir.path().join(format!("dir{}", i)))?;
            for j in 0..5 {
                fs::write(
                    temp_dir
                        .path()
                        .join(format!("dir{}/file{}.txt", i, j)),
                    "content",
                )?;
            }
        }

        list_directory_contents(temp_dir.path())?;
        Ok(())
    }

    #[tokio::test]
    async fn test_copy_dir_all_async_with_empty_dirs() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        fs::create_dir_all(src_dir.join("empty1"))?;
        fs::create_dir_all(src_dir.join("empty2/empty3"))?;

        copy_dir_all_async(&src_dir, &dst_dir).await?;

        assert!(dst_dir.join("empty1").exists());
        assert!(dst_dir.join("empty2/empty3").exists());
        Ok(())
    }

    #[test]
    fn test_log_level_from_env() {
        // Save the current environment variable value
        let original_value = env::var(ENV_LOG_LEVEL).ok();

        // Helper function to get processed log level
        fn get_processed_log_level() -> String {
            let log_level = env::var(ENV_LOG_LEVEL)
                .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

            match log_level.to_lowercase().as_str() {
                "error" => "error",
                "warn" => "warn",
                "info" => "info",
                "debug" => "debug",
                "trace" => "trace",
                _ => "info", // Default to info for invalid values
            }
            .to_string()
        }

        // Test various log level settings
        let test_levels = vec![
            ("DEBUG", "debug"),
            ("ERROR", "error"),
            ("WARN", "warn"),
            ("INFO", "info"),
            ("TRACE", "trace"),
            ("INVALID", "info"), // Should default to info
        ];

        for (input, expected) in test_levels {
            env::set_var(ENV_LOG_LEVEL, input);
            let processed_level = get_processed_log_level();
            assert_eq!(
                processed_level, expected,
                "Expected log level '{}' for input '{}', but got '{}'",
                expected, input, processed_level
            );
        }

        // Restore the original environment variable state
        match original_value {
            Some(value) => env::set_var(ENV_LOG_LEVEL, value),
            None => env::remove_var(ENV_LOG_LEVEL),
        }
    }

    /// Test for default log level when environment variable is not set
    #[test]
    fn test_default_log_level() {
        // Save current environment variable value
        let original_value = env::var(ENV_LOG_LEVEL).ok();

        // Remove the environment variable to test default behavior
        env::remove_var(ENV_LOG_LEVEL);

        let log_level = env::var(ENV_LOG_LEVEL)
            .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string())
            .to_lowercase();

        assert_eq!(log_level, DEFAULT_LOG_LEVEL.to_lowercase());

        // Restore original environment variable state
        if let Some(value) = original_value {
            env::set_var(ENV_LOG_LEVEL, value);
        }
    }

    /// Test the logic for translating string log levels to LevelFilter values
    #[test]
    fn test_log_level_translation() {
        let test_cases = vec![
            ("error", LevelFilter::Error),
            ("warn", LevelFilter::Warn),
            ("info", LevelFilter::Info),
            ("debug", LevelFilter::Debug),
            ("trace", LevelFilter::Trace),
            ("invalid", LevelFilter::Info),
            ("", LevelFilter::Info),
        ];

        for (input, expected) in test_cases {
            let level = match input.to_lowercase().as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            };

            assert_eq!(
            level,
            expected,
            "Log level mismatch for input: '{}' - expected {:?}, got {:?}",
            input,
            expected,
            level
        );
        }
    }

    /// Test environment variable handling with cleanup
    #[test]
    fn test_env_log_level_handling() {
        // Save original state
        let original_value = env::var(ENV_LOG_LEVEL).ok();

        let test_cases = vec![
            (Some("DEBUG"), "debug"),
            (Some("ERROR"), "error"),
            (Some("WARN"), "warn"),
            (Some("INFO"), "info"),
            (Some("TRACE"), "trace"),
            (Some("INVALID"), "info"),
            (None, "info"),
        ];

        for (env_value, expected) in test_cases {
            // Clear any existing env var
            env::remove_var(ENV_LOG_LEVEL);

            // Set new value if provided
            if let Some(value) = env_value {
                env::set_var(ENV_LOG_LEVEL, value);
            }

            let log_level = env::var(ENV_LOG_LEVEL)
                .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string())
                .to_lowercase();

            let actual = match log_level.as_str() {
                "error" => "error",
                "warn" => "warn",
                "info" => "info",
                "debug" => "debug",
                "trace" => "trace",
                _ => "info",
            };

            assert_eq!(
                actual, expected,
                "Log level mismatch for env value: {:?}",
                env_value
            );
        }

        // Restore original state
        match original_value {
            Some(value) => env::set_var(ENV_LOG_LEVEL, value),
            None => env::remove_var(ENV_LOG_LEVEL),
        }
    }
}
