// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Shokunin Static Site Generator
//!
//! A high-performance, secure static site generator written in Rust that puts content first.
//! This library provides functionality for generating static websites from markdown content
//! and templates, with built-in development server capabilities.
//!
//! ## Features
//!
//! * Content-focused static site generation
//! * Built-in development server
//! * Secure file handling
//! * Comprehensive logging
//! * Template support
//!
//! ## Example
//!
//! ```rust,no_run
//! use ssg::run;
//!
//! fn main() -> anyhow::Result<()> {
//!     run()?;
//!     Ok(())
//! }
//! ```

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
/// This function orchestrates the entire site generation process:
///
/// 1. Sets up logging infrastructure
/// 2. Displays the CLI banner
/// 3. Processes command-line arguments
/// 4. Creates necessary directories
/// 5. Compiles the static site
/// 6. Optionally starts a development server
///
/// # Errors
///
/// Returns an error if:
/// - Required arguments are missing
/// - File system operations fail
/// - Site compilation fails
/// - Server startup fails
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

/// Structure holding all necessary paths for site generation
#[derive(Debug)]
struct Paths {
    site: PathBuf,
    content: PathBuf,
    build: PathBuf,
    template: PathBuf,
}

/// Creates a log file at the specified path
///
/// # Errors
///
/// Returns an error if the file cannot be created
fn create_log_file(file_path: &str) -> Result<File> {
    File::create(file_path).context("Failed to create log file")
}

/// Logs initialization information
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

/// Logs argument information
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

/// Extracts and validates paths from command-line arguments
///
/// # Errors
///
/// Returns an error if required paths are missing or invalid.
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

/// Creates all necessary directories for the static site generator.
///
/// # Errors
///
/// Returns an error if directory creation fails.
fn create_directories(paths: &Paths) -> Result<()> {
    fs::create_dir_all(&paths.content)
        .context("Failed to create content directory")?;
    fs::create_dir_all(&paths.build)
        .context("Failed to create build directory")?;
    fs::create_dir_all(&paths.site)
        .context("Failed to create site directory")?;
    fs::create_dir_all(&paths.template)
        .context("Failed to create template directory")?;
    Ok(())
}

/// Handles the development server setup and startup.
///
/// # Arguments
///
/// * `log_file` - Reference to the log file for logging messages.
/// * `date` - The current date and time for logging.
/// * `paths` - Struct holding all paths for the site.
/// * `serve_dir` - PathBuf reference to the directory for serving files.
///
/// # Errors
///
/// Returns an error if the server setup or startup fails.
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

/// Verifies the source directory and copies files to the destination directory.
///
/// # Arguments
///
/// * `src` - Path reference to the source directory.
/// * `dst` - Path reference to the destination directory.
///
/// # Errors
///
/// Returns an error if the directory does not exist or copying fails.
fn verify_and_copy_files(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        anyhow::bail!(
            "Source directory does not exist: {}",
            src.display()
        );
    }

    copy_dir_all(src, dst)?;

    // Verify destination has content
    if !dst.exists() || dst.read_dir()?.next().is_none() {
        anyhow::bail!(
            "Destination directory is empty or does not exist: {}",
            dst.display()
        );
    }

    list_directory_contents(dst)?;
    Ok(())
}

/// Recursively copies all files and directories from the source to the destination.
///
/// # Arguments
///
/// * `src` - Path reference to the source directory.
/// * `dst` - Path reference to the destination directory.
///
/// # Errors
///
/// Returns an error if file or directory copying fails.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Lists the contents of the directory recursively, useful for debugging and verification.
///
/// # Arguments
///
/// * `dir` - Path reference to the directory to list.
///
/// # Errors
///
/// Returns an error if directory reading fails.
fn list_directory_contents(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            list_directory_contents(&path)?;
        }
    }
    Ok(())
}
