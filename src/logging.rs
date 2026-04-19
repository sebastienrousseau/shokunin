// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Logging infrastructure for the static site generator.

use std::fs::File;
use std::io::Write;

use anyhow::{Context, Result};
use log::{info, LevelFilter};

// Constants for configuration
pub(crate) const DEFAULT_LOG_LEVEL: &str = "info";
pub(crate) const ENV_LOG_LEVEL: &str = "SSG_LOG_LEVEL";

/// Maps a case-insensitive log level string to a `LevelFilter`.
///
/// Unrecognised values fall back to `LevelFilter::Info`. Extracted
/// from `initialize_logging` so it can be unit-tested without
/// installing a global logger (which is one-shot per process).
pub(crate) fn parse_log_level(log_level: &str) -> LevelFilter {
    match log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

/// A minimal logger that writes to stderr.
#[derive(Debug)]
pub(crate) struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "[{} {}] {}",
                record.level(),
                record.module_path().unwrap_or(""),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Initializes the logging system based on environment variables.
pub(crate) fn initialize_logging() -> Result<()> {
    let log_level = std::env::var(ENV_LOG_LEVEL)
        .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

    let level = parse_log_level(&log_level);

    let _ = log::set_logger(&SimpleLogger).map(|()| log::set_max_level(level));

    info!("Logging initialized at level: {log_level}");
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
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = ssg::now_iso();
///
///     log_initialization(&mut log_file, &date)?;
///     println!("System initialisation logged");
///     Ok(())
/// }
/// ```
pub fn log_initialization(log_file: &mut File, date: &str) -> Result<()> {
    writeln!(
        log_file,
        "[{date}] INFO process: System initialization complete"
    )
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
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = ssg::now_iso();
///
///     log_arguments(&mut log_file, &date)?;
///     println!("Arguments logged successfully");
///     Ok(())
/// }
/// ```
pub fn log_arguments(log_file: &mut File, date: &str) -> Result<()> {
    writeln!(log_file, "[{date}] INFO process: Arguments processed")
        .context("Failed to write arguments log")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_level_info() {
        assert_eq!(parse_log_level("info"), LevelFilter::Info);
    }

    #[test]
    fn parse_log_level_debug() {
        assert_eq!(parse_log_level("debug"), LevelFilter::Debug);
    }

    #[test]
    fn parse_log_level_warn() {
        assert_eq!(parse_log_level("warn"), LevelFilter::Warn);
    }

    #[test]
    fn parse_log_level_error() {
        assert_eq!(parse_log_level("error"), LevelFilter::Error);
    }

    #[test]
    fn parse_log_level_trace() {
        assert_eq!(parse_log_level("trace"), LevelFilter::Trace);
    }

    #[test]
    fn parse_log_level_case_insensitive() {
        assert_eq!(parse_log_level("DEBUG"), LevelFilter::Debug);
        assert_eq!(parse_log_level("Warn"), LevelFilter::Warn);
    }

    #[test]
    fn parse_log_level_invalid_defaults_to_info() {
        assert_eq!(parse_log_level("garbage"), LevelFilter::Info);
        assert_eq!(parse_log_level(""), LevelFilter::Info);
    }

    #[test]
    fn create_log_file_in_tempdir() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.log");
        let file = create_log_file(path.to_str().unwrap());
        assert!(file.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn log_initialization_writes_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("init.log");
        let mut file = create_log_file(path.to_str().unwrap()).unwrap();

        log_initialization(&mut file, "2025-01-01T00:00:00Z").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("System initialization complete"));
        assert!(contents.contains("2025-01-01"));
    }

    #[test]
    fn log_arguments_writes_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("args.log");
        let mut file = create_log_file(path.to_str().unwrap()).unwrap();

        log_arguments(&mut file, "2025-06-15T12:00:00Z").unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("Arguments processed"));
    }
}
