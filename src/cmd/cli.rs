// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Command Line Interface Module
//!
//! This module provides a secure and robust command-line interface for the Shokunin
//! Static Site Generator. It handles argument parsing, configuration management,
//! and validation of user inputs.
//!
//! ## Key Features
//!
//! - Safe path handling
//! - Input validation
//! - Secure configuration
//! - Error handling
//!
//! ## Usage Example
//!
//! ## Example Usage
//! ```rust,no_run
//! use ssg::cmd::cli::build;
//! use ssg::cmd::cli::ShokuninConfig;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Initialize the CLI with arguments from `build()`
//!     let matches = build().get_matches();
//!
//!     // Use the matches to configure and run your application
//!     if let Some(config) = ShokuninConfig::from_matches(&matches)?.serve_dir {
//!         println!("Configuration loaded: {:?}", config);
//!         // Continue with application logic...
//!     }
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

/// Possible errors that can occur during CLI operations.
#[derive(Error, Debug)]
pub enum CliError {
    /// Indicates an invalid or unsafe path.
    #[error("Invalid path for {field}: {details}")]
    InvalidPath {
        /// The field name containing the path
        field: String,
        /// Details about why the path is invalid
        details: String,
    },

    /// Indicates that a required argument is missing.
    #[error("Required argument missing: {0}")]
    MissingArgument(String),

    /// Indicates an invalid URL format.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Wraps standard IO errors.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Wraps TOML parsing errors.
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Core configuration for the static site generator.
///
/// This structure holds all settings needed to generate a static site,
/// including paths, metadata, and server options.
///
/// # Security
///
/// All paths undergo validation to prevent:
/// - Directory traversal
/// - Access to system directories
/// - Use of unsafe characters
///
/// # Example
///
/// ```rust,no_run
/// use ssg::cmd::cli::ShokuninConfig;
/// use std::path::PathBuf;
///
/// let config = ShokuninConfig {
///     site_name: String::from("my-site"),
///     content_dir: PathBuf::from("content"),
///     output_dir: PathBuf::from("public"),
///     template_dir: PathBuf::from("templates"),
///     serve_dir: None,
///     base_url: String::from("http://localhost:8000"),
///     site_title: String::from("My Site"),
///     site_description: String::from("A static site"),
///     language: String::from("en-GB"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShokuninConfig {
    /// Project name
    pub site_name: String,
    /// Location of content files
    pub content_dir: PathBuf,
    /// Output directory for generated files
    pub output_dir: PathBuf,
    /// Location of template files
    pub template_dir: PathBuf,
    /// Optional directory for development server
    pub serve_dir: Option<PathBuf>,
    /// Site's base URL
    pub base_url: String,
    /// Site title
    pub site_title: String,
    /// Site description
    pub site_description: String,
    /// Site language (format: xx-XX)
    pub language: String,
}

impl Default for ShokuninConfig {
    fn default() -> Self {
        Self {
            site_name: String::new(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("public"),
            template_dir: PathBuf::from("templates"),
            serve_dir: None,
            base_url: String::from("http://localhost:8000"),
            site_title: String::from("My Shokunin Site"),
            site_description: String::from(
                "A site built with Shokunin",
            ),
            language: String::from("en-GB"),
        }
    }
}

impl ShokuninConfig {
    /// Creates a new configuration from command-line arguments.
    ///
    /// # Arguments
    ///
    /// * `matches` - Parsed command-line arguments
    ///
    /// # Returns
    ///
    /// Returns a Result containing either the validated configuration or an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ssg::cmd::cli::{build, ShokuninConfig};
    ///
    /// let matches = build().get_matches();
    /// let config = ShokuninConfig::from_matches(&matches).unwrap();
    /// ```
    pub fn from_matches(
        matches: &ArgMatches,
    ) -> Result<Self, CliError> {
        if let Some(config_path) = matches.get_one::<PathBuf>("config")
        {
            return Self::from_file(config_path);
        }

        let mut config = Self::default();

        if let Some(site_name) = matches.get_one::<String>("new") {
            config.site_name = site_name.clone();
        }

        if let Some(content_dir) = matches.get_one::<PathBuf>("content")
        {
            config.content_dir =
                validate_path(content_dir, "content_dir")?;
        }

        if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
            config.output_dir =
                validate_path(output_dir, "output_dir")?;
        }

        if let Some(template_dir) =
            matches.get_one::<PathBuf>("template")
        {
            config.template_dir =
                validate_path(template_dir, "template_dir")?;
        }

        if let Some(serve_dir) = matches.get_one::<PathBuf>("serve") {
            config.serve_dir =
                Some(validate_path(serve_dir, "serve_dir")?);
        }

        Ok(config)
    }

    /// Loads configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML file
    ///
    /// # Returns
    ///
    /// Returns a Result containing either the validated configuration or an error.
    pub fn from_file(path: &Path) -> Result<Self, CliError> {
        let content = std::fs::read_to_string(path)?;
        let config: ShokuninConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates all configuration values.
    ///
    /// Checks:
    /// - URL format
    /// - Path safety
    /// - Language format
    fn validate(&self) -> Result<(), CliError> {
        if !self.base_url.is_empty() {
            let parsed_url =
                Url::parse(&self.base_url).map_err(|_| {
                    CliError::InvalidUrl(self.base_url.clone())
                })?;

            // Reject URLs that are not HTTP or HTTPS
            if parsed_url.scheme() != "http"
                && parsed_url.scheme() != "https"
            {
                return Err(CliError::InvalidUrl(
                    self.base_url.clone(),
                ));
            }

            // Additional check: reject URLs with invalid or empty hostname
            if parsed_url.host_str().map_or(true, |host| {
                host.is_empty() || host.starts_with('.')
            }) {
                return Err(CliError::InvalidUrl(
                    self.base_url.clone(),
                ));
            }

            // Reject URLs that contain backslashes
            if self.base_url.contains('\\') {
                return Err(CliError::InvalidUrl(
                    self.base_url.clone(),
                ));
            }
        }

        validate_path_safety(&self.content_dir, "content_dir")?;
        validate_path_safety(&self.output_dir, "output_dir")?;
        validate_path_safety(&self.template_dir, "template_dir")?;

        if let Some(serve_dir) = &self.serve_dir {
            validate_path_safety(serve_dir, "serve_dir")?;
        }

        if !self.language.contains('-') || self.language.len() != 5 {
            return Err(CliError::InvalidPath {
                field: "language".to_string(),
                details: "Language must use format 'xx-XX'".to_string(),
            });
        }

        Ok(())
    }
}

/// Checks a path for security issues.
fn validate_path_safety(
    path: &Path,
    field: &str,
) -> Result<(), CliError> {
    let path_str = path.to_string_lossy();

    if path_str.contains('\0') {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains null byte".to_string(),
        });
    }

    if path_str.contains('\u{202E}') {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains bidirectional text override"
                .to_string(),
        });
    }

    if path_str.contains("..")
        || path.is_absolute()
        || path_str.contains("//")
        || path_str.contains(r"\\")
    {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path traversal or invalid format detected"
                .to_string(),
        });
    }

    // Reject paths containing colons, except for drive letters on Windows (e.g., "C:\path").
    if path_str.contains(':')
        && !(cfg!(windows) && path_str.chars().nth(1) == Some(':'))
    {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains invalid character ':'".to_string(),
        });
    }

    let reserved_names = ["con", "aux", "nul", "prn", "com1", "lpt1"];
    if reserved_names.contains(&path_str.to_lowercase().as_str()) {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Reserved system path name".to_string(),
        });
    }

    Ok(())
}

/// Validates and normalises a path.
fn validate_path(
    path: &Path,
    field: &str,
) -> Result<PathBuf, CliError> {
    validate_path_safety(path, field)?;
    Ok(path.to_path_buf())
}

/// Creates the command-line interface.
pub fn build() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("config")
                .help("Configuration file path")
                .long("config")
                .short('f')
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("new")
                .help("Create new project")
                .long("new")
                .short('n')
                .required_unless_present("config")
                .value_name("NAME")
                .value_parser(clap::value_parser!(String)), // Change from PathBuf to String
        )
        .arg(
            Arg::new("content")
                .help("Content directory")
                .long("content")
                .short('c')
                .required_unless_present("config")
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("output")
                .help("Output directory")
                .long("output")
                .short('o')
                .required_unless_present("config")
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("template")
                .help("Template directory")
                .long("template")
                .short('t')
                .required_unless_present("config")
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("serve")
                .help("Development server directory")
                .long("serve")
                .short('s')
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("watch")
                .help("Watch for changes")
                .long("watch")
                .short('w')
                .action(ArgAction::SetTrue),
        )
}

/// Displays the application banner.
pub fn print_banner() {
    let title =
        format!("Shokunin (ssg) 🦀 v{}", env!("CARGO_PKG_VERSION"));
    let description =
        "A Fast and Flexible Static Site Generator written in Rust";
    let width = title.len().max(description.len()) + 4;
    let line = "─".repeat(width - 2);

    println!("\n┌{}┐", line);
    println!("│{:^width$}│", title);
    println!("├{}┤", line);
    println!("│{:^width$}│", description);
    println!("└{}┘\n", line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::{fs, io::Write};
    use tempfile::TempDir;

    #[test]
    fn test_cli_structure() {
        // Verify that the CLI structure is set up correctly and all required arguments exist
        build().debug_assert();
    }

    /// Creates a temporary directory for testing
    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }

    #[test]
    /// Test default configuration creation
    fn test_default_config() {
        let config = ShokuninConfig::default();
        assert_eq!(config.content_dir, PathBuf::from("content"));
        assert_eq!(config.output_dir, PathBuf::from("public"));
        assert_eq!(config.template_dir, PathBuf::from("templates"));
        assert!(config.serve_dir.is_none());
        assert_eq!(config.base_url, "http://localhost:8000");
        assert_eq!(config.language, "en-GB");
        assert!(config.validate().is_ok());
    }

    #[test]
    /// Test configuration validation with valid settings
    fn test_config_validation_valid() {
        let config = ShokuninConfig {
            site_name: "test".to_string(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("public"),
            template_dir: PathBuf::from("templates"),
            serve_dir: Some(PathBuf::from("serve")),
            base_url: "http://localhost:8000".to_string(),
            site_title: "Test Site".to_string(),
            site_description: "Test Description".to_string(),
            language: "en-GB".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    /// Test configuration validation with invalid URL
    fn test_config_validation_invalid_url() {
        let config = ShokuninConfig {
            base_url: "not a url".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(CliError::InvalidUrl(_))
        ));
    }

    #[test]
    /// Test configuration validation with path traversal attempts
    fn test_config_validation_path_traversal() {
        let config = ShokuninConfig {
            content_dir: PathBuf::from("../content"),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(CliError::InvalidPath { .. })
        ));
    }

    /// Creates a test configuration file
    fn create_test_config(dir: &Path) -> PathBuf {
        let config_path = dir.join("config.toml");
        let config_content = r#"
            site_name = "test-site"
            content_dir = "content"
            output_dir = "public"
            template_dir = "templates"
            base_url = "http://localhost:8000"
            site_title = "Test Site"
            site_description = "A test site"
            language = "en-GB"
        "#;
        fs::write(&config_path, config_content)
            .expect("Failed to write config file");
        config_path
    }

    #[test]
    /// Test configuration loading from TOML file
    fn test_config_from_file() {
        let temp_dir = setup_temp_dir();
        let config_path = create_test_config(temp_dir.path());

        let config = ShokuninConfig::from_file(&config_path);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.site_name, "test-site");
        assert_eq!(config.content_dir, PathBuf::from("content"));
        assert_eq!(config.base_url, "http://localhost:8000");
    }

    #[test]
    /// Test configuration loading from invalid TOML file
    fn test_config_from_invalid_file() {
        let temp_dir = setup_temp_dir();
        let config_path = temp_dir.path().join("invalid.toml");
        fs::write(&config_path, "invalid = { toml")
            .expect("Failed to write file");

        let result = ShokuninConfig::from_file(&config_path);
        assert!(matches!(result, Err(CliError::TomlError(_))));
    }

    #[test]
    /// Test path validation with various invalid paths
    fn test_path_validation_invalid() {
        let invalid_paths = vec![
            "../dangerous",
            "content/../secret",
            "content//hidden",
            r"content\..\..\secret",
            "content/../../etc/passwd",
            "content/./../../secret",
        ];

        for path in invalid_paths {
            assert!(matches!(
                validate_path(Path::new(path), "test"),
                Err(CliError::InvalidPath { .. })
            ));
        }
    }

    #[test]
    /// Test path validation with valid paths
    fn test_path_validation_valid() {
        let valid_paths = vec![
            "content",
            "content/subfolder",
            "templates/layouts",
            "./content",
        ];

        for path in valid_paths {
            assert!(validate_path(Path::new(path), "test").is_ok());
        }
    }

    #[test]
    /// Test CLI argument parsing with various combinations
    fn test_cli_argument_parsing() {
        // Test minimum required arguments
        let result = build().try_get_matches_from(vec![
            "ssg",
            "--new",
            "test-site",
            "--content",
            "content",
            "--output",
            "public",
            "--template",
            "templates",
        ]);
        assert!(result.is_ok());

        // Test with optional serve argument
        let result = build().try_get_matches_from(vec![
            "ssg",
            "--new",
            "test-site",
            "--content",
            "content",
            "--output",
            "public",
            "--template",
            "templates",
            "--serve",
            "serve",
        ]);
        assert!(result.is_ok());

        // Test with watch flag
        let result = build().try_get_matches_from(vec![
            "ssg",
            "--new",
            "test-site",
            "--content",
            "content",
            "--output",
            "public",
            "--template",
            "templates",
            "--watch",
        ]);
        assert!(result.is_ok());
    }

    #[test]
    /// Test CLI argument parsing with missing required arguments
    fn test_cli_argument_parsing_missing_required() {
        let result = build().try_get_matches_from(vec!["ssg"]);
        assert!(result.is_err());

        let result = build().try_get_matches_from(vec![
            "ssg",
            "--new",
            "test-site",
        ]);
        assert!(result.is_err());
    }

    #[test]
    /// Test banner output format
    fn test_banner_output() {
        let mut output = Vec::new();
        {
            let mut stdout = std::io::Cursor::new(&mut output);
            writeln!(stdout, "\n┌{}┐", "─".repeat(53)).unwrap();
            writeln!(
                stdout,
                "│{: ^54}│",
                format!(
                    "Shokunin (ssg) 🦀 v{}",
                    env!("CARGO_PKG_VERSION")
                )
            )
            .unwrap();
            writeln!(stdout, "├{}┤", "─".repeat(53)).unwrap();
            writeln!(
                stdout,
                "│{: ^54}│",
                "A Fast and Flexible Static Site Generator written in Rust"
            )
            .unwrap();
            writeln!(stdout, "└{}┘\n", "─".repeat(53)).unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Shokunin"));
        assert!(output_str.contains(env!("CARGO_PKG_VERSION")));
        assert!(output_str.contains("Static Site Generator"));
    }

    #[test]
    /// Test configuration creation from CLI matches
    fn test_config_from_matches() {
        let matches = build().get_matches_from(vec![
            "ssg",
            "--new",
            "test-site",
            "--content",
            "content",
            "--output",
            "public",
            "--template",
            "templates",
        ]);

        let config = ShokuninConfig::from_matches(&matches);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.site_name, "test-site".to_string()); // Convert PathBuf to String for comparison
        assert_eq!(config.content_dir, PathBuf::from("content"));
        assert_eq!(config.output_dir, PathBuf::from("public"));
        assert_eq!(config.template_dir, PathBuf::from("templates"));
        assert!(config.serve_dir.is_none());
    }

    #[test]
    /// Test configuration with serve directory
    fn test_config_with_serve() {
        let matches = build().get_matches_from(vec![
            "ssg",
            "--new",
            "test-site", // Use as expected by PathBuf
            "--content",
            "content",
            "--output",
            "public",
            "--template",
            "templates",
            "--serve",
            "serve",
        ]);

        let config = ShokuninConfig::from_matches(&matches).unwrap();
        assert_eq!(config.site_name, "test-site".to_string()); // Convert PathBuf to String for comparison
        assert_eq!(config.content_dir, PathBuf::from("content"));
        assert_eq!(config.output_dir, PathBuf::from("public"));
        assert_eq!(config.template_dir, PathBuf::from("templates"));
        assert_eq!(config.serve_dir, Some(PathBuf::from("serve")));
    }

    #[test]
    /// Test error handling for various error cases
    fn test_error_handling() {
        // Test missing argument error
        let error = CliError::MissingArgument("test".to_string());
        assert_eq!(
            error.to_string(),
            "Required argument missing: test"
        );

        // Test invalid path error
        let error = CliError::InvalidPath {
            field: "test".to_string(),
            details: "invalid".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid path for test: invalid");

        // Test invalid URL error
        let error = CliError::InvalidUrl("test".to_string());
        assert_eq!(error.to_string(), "Invalid URL: test");
    }

    #[test]
    /// Test path normalization
    fn test_path_normalization() {
        let temp_dir = setup_temp_dir();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result =
            validate_path(Path::new("content"), "test").unwrap();
        assert_eq!(result, PathBuf::from("content"));

        let result =
            validate_path(Path::new("content/./subfolder"), "test")
                .unwrap();
        assert_eq!(result, PathBuf::from("content/subfolder"));
    }

    #[test]
    fn test_path_validation_edge_cases() {
        let edge_cases = vec![
            "content\0hidden",            // Null byte
            "content\u{202E}hidden",      // Right-to-left override
            "content\\../hidden",         // Mixed separators
            "con",                        // Reserved name
            "content:alternate",          // Alternate data stream
            "C:\\Windows\\System32\\con", // Absolute path
            "/etc/passwd",                // Absolute path
        ];

        for path in edge_cases {
            let result = validate_path(Path::new(path), "test");
            assert!(
            matches!(result, Err(CliError::InvalidPath { .. })),
            "Expected path '{}' to be invalid, but it passed validation",
            path
        );
        }
    }

    #[test]
    fn test_url_validation() {
        let invalid_urls = vec![
            "not-a-url",                     // Not a valid URL format
            "ftp://example.com",             // Non-HTTP(S) scheme
            "http://invalid\u{202E}url.com", // Bidirectional text override
            "http://example.com\\path", // Backslashes instead of forward slashes
            "https://:80",              // Missing domain
            "http://example.com:abc",   // Invalid port
            "http://.com",              // Invalid domain format
        ];

        for url in invalid_urls {
            let config = ShokuninConfig {
                base_url: url.to_string(),
                ..Default::default()
            };

            assert!(
            config.validate().is_err(),
            "Expected URL '{}' to be invalid, but it passed validation",
            url
        );
        }
    }
}
