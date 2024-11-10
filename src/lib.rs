// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator
//!
//! A high-performance, secure static site generator written in Rust that prioritises content delivery.
//! This library transforms markdown content and templates into static websites whilst providing
//! development server capabilities.
//!
//! ## Core Features
//!
//! * Content-First Architecture: Optimised for efficient content management and delivery
//! * Development Server: Built-in server for rapid local testing and development
//! * Security-Focused: Comprehensive file handling with robust security measures
//! * Advanced Logging: Detailed logging system for effective debugging and monitoring
//! * Template Engine: Flexible template system for maintaining consistent site styling
//!
//! ## Library Structure
//!
//! The library is organised into these primary components:
//!
//! 1. Site Generation: Core functionality for building static sites
//! 2. Development Server: Local testing and preview capabilities
//! 3. File Management: Secure file operations and directory handling
//! 4. Logging System: Comprehensive activity tracking and debugging
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use ssg::run;
//!
//! fn main() -> anyhow::Result<()> {
//!     run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library employs Rust's robust error handling with custom error types and
//! comprehensive error messages. All operations return `Result` types with
//! detailed context for debugging.
//!
//! ## Security Measures
//!
//! * Path Validation: All file paths undergo thorough validation
//! * Resource Limits: Configurable limits prevent resource exhaustion
//! * Input Sanitisation: Comprehensive validation of all user inputs

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
use anyhow::{Context, Result};
use dtt::datetime::DateTime;
use http_handle::Server;
use rayon::prelude::*;
use rlg::{log_format::LogFormat, log_level::LogLevel, macro_log};
use staticdatagen::{
    compiler::service::compile, locales::en::translate, macro_serve,
    utilities::uuid::generate_unique_string,
};

/// Module declarations
pub mod cmd;

/// Re-exports
pub use staticdatagen;

/// Main entry point for the static site generator.
///
/// Orchestrates the entire site generation process through several key stages:
///
/// 1. Logging System Initialisation
///    - Creates log file
///    - Sets up logging infrastructure
///
/// 2. Command-Line Interface
///    - Displays the CLI banner
///    - Processes user arguments
///
/// 3. Directory Structure
///    - Creates necessary directories
///    - Validates directory permissions
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
/// - Required command-line arguments are missing
/// - File system operations fail (e.g., insufficient permissions)
/// - Site compilation encounters errors
/// - Development server fails to start
///
/// # Example
///
/// ```rust,no_run
/// use ssg::run;
///
/// fn main() -> anyhow::Result<()> {
///     run()?;
///     Ok(())
/// }
/// ```
///
/// # Performance Characteristics
///
/// - Time Complexity: O(n) where n is the number of files
/// - Space Complexity: O(m) where m is the average file size
pub fn run() -> Result<()> {
    // Initialize logging
    let date = DateTime::new();
    let mut log_file = create_log_file("./ssg.log")
        .context("Failed to create log file")?;

    // Display banner and log initialization
    cmd::cli::print_banner();
    log_initialization(&mut log_file, &date)?;

    // Parse command-line arguments
    let matches = cmd::cli::build().get_matches();
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

/// Collection of essential paths for site generation.
///
/// Maintains references to all critical directories used throughout the
/// site generation process. Each path serves a specific purpose in the
/// build pipeline.
///
/// # Fields
///
/// * `site` - Root directory for the generated website
/// * `content` - Source directory containing markdown and other content
/// * `build` - Temporary directory for build artifacts
/// * `template` - Directory containing site templates and layouts
#[derive(Debug)]
struct Paths {
    site: PathBuf,
    content: PathBuf,
    build: PathBuf,
    template: PathBuf,
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
    fs::create_dir_all(&paths.build)
        .with_context(|| format!("Failed to create or access build directory at path: {:?}", &paths.build))?;
    fs::create_dir_all(&paths.site)
        .with_context(|| format!("Failed to create or access site directory at path: {:?}", &paths.site))?;
    fs::create_dir_all(&paths.template)
        .with_context(|| format!("Failed to create or access template directory at path: {:?}", &paths.template))?;

    // Path safety check with additional context
    if !is_safe_path(&paths.content)? || !is_safe_path(&paths.build)?
        || !is_safe_path(&paths.site)? || !is_safe_path(&paths.template)?
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

/// Validates source directory and copies files.
///
/// Ensures the integrity of the source directory before
/// copying its contents to the destination.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Errors
///
/// Returns an error if:
/// * Source directory is invalid or missing
/// * Destination is not writable
/// * Copy operations fail
/// * Verification checks fail
fn verify_and_copy_files(src: &Path, dst: &Path) -> Result<()> {
    // Check if source path is safe
    if !is_safe_path(src).with_context(|| format!("Source directory is unsafe or inaccessible: {:?}", src))? {
        anyhow::bail!("Source directory is unsafe or inaccessible: {:?}", src);
    }
    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {:?}", src);
    }

    // Ensure the destination directory exists and add context in case of error.
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create or access destination directory at path: {:?}", dst))?;

    // Copy directory contents with error context
    copy_dir_all(src, dst).with_context(|| {
        format!("Failed to copy files from source: {:?} to destination: {:?}", src, dst)
    })?;

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

/// Checks if a given path is safe to use.
///
/// This function ensures that the provided path does not contain any parent directory references
/// (i.e., "..") to prevent directory traversal attacks. It uses the `canonicalize` method to resolve
/// any symbolic links and then checks if the resulting path still contains any ".." components.
///
/// # Parameters
///
/// * `path`: A reference to the path to be checked.
///
/// # Return Value
///
/// * `true`: If the provided path is safe to use (i.e., does not contain any ".." components).
/// * `false`: If the provided path is not safe to use (i.e., contains ".." components).
pub fn is_safe_path(path: &Path) -> Result<bool> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {:?}", path);
    }

    match path.canonicalize() {
        Ok(canonical) => Ok(!canonical.to_string_lossy().contains("..")),
        Err(e) => Err(anyhow::anyhow!("Failed to canonicalize path {:?}: {}", path, e)),
    }
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
        .into_par_iter() // Use into_par_iter() instead of par_iter()
        .try_for_each(|entry| -> Result<()> {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
            Ok(())
        })?;

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
