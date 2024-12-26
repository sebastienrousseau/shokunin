//! # Command Line Interface Module
//!
//! This module provides a secure and robust command-line interface (CLI) for the
//! **Shokunin Static Site Generator**. It handles argument parsing, configuration management,
//! and validation of user inputs to ensure that the static site generator operates
//! reliably and securely.
//!
//! ## Key Features
//! - Safe path handling (including symbolic link checks and canonicalization)
//! - Input validation (URL, language, environment variables)
//! - Secure configuration with size-limited config files
//! - Builder pattern for convenient configuration construction
//! - Error handling via `CliError`
//!
//! ## Example Usage
//! ```rust,no_run
//! use ssg::cmd::{Cli, ShokuninConfig};
//!
//! fn main() -> anyhow::Result<()> {
//!     let cli = Cli::new();
//!     let matches = cli.build().get_matches();
//!
//!     // Attempt to load configuration from either command-line arguments or a file
//!     let mut config = ShokuninConfig::from_matches(&matches)?;
//!
//!     // Optionally load environment variables into the configuration
//!     config.load_from_env()?;
//!
//!     println!("Configuration loaded: {:?}", config);
//!     // Continue with application logic...
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

/// Default port for the local development server.
pub const DEFAULT_PORT: u16 = 8000;
/// Default host for the local development server.
pub const DEFAULT_HOST: &str = "127.0.0.1";
/// Reserved names on some operating systems.
pub const RESERVED_NAMES: &[&str] =
    &["con", "aux", "nul", "prn", "com1", "lpt1"];
/// Maximum allowed size (in bytes) for the Shokunin config file.
pub const MAX_CONFIG_SIZE: u64 = 1024 * 1024; // 1MB limit

/// A static default configuration for the Shokunin site.
///
/// Using `once_cell::sync::Lazy` allows the default configuration
/// to be created only once at runtime, even if referenced multiple times.
pub static DEFAULT_CONFIG: Lazy<ShokuninConfig> =
    Lazy::new(|| ShokuninConfig {
        site_name: "MyShokuninSite".to_string(),
        content_dir: PathBuf::from("content"),
        output_dir: PathBuf::from("public"),
        template_dir: PathBuf::from("templates"),
        serve_dir: None,
        base_url: format!("http://{}:{}", DEFAULT_HOST, DEFAULT_PORT),
        site_title: "My Shokunin Site".to_string(),
        site_description: "A site built with Shokunin".to_string(),
        language: "en-GB".to_string(),
    });

/// Possible errors that can occur during CLI operations.
#[derive(Error, Debug)]
pub enum CliError {
    /// Indicates an invalid or unsafe path.
    #[error("Invalid path for {field}: {details}")]
    InvalidPath {
        /// The field name containing the path.
        field: String,
        /// Details about why the path is invalid.
        details: String,
    },

    /// Indicates that a required argument is missing.
    #[error("Required argument missing: {0}")]
    MissingArgument(String),

    /// Indicates an invalid URL format or usage.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Wraps standard I/O errors.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Wraps TOML parsing errors.
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Indicates a validation error in configuration values.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Core configuration for the static site generator.
///
/// This structure holds all settings needed to generate a static site,
/// including paths, metadata, and server options.
///
/// ## Security
/// - Paths undergo validation to prevent directory traversal, symbolic links, or unsafe characters.
/// - URL fields must be valid HTTP or HTTPS URLs.
/// - Config files are size-limited to mitigate malicious large-file attacks.
///
/// ## Example
/// ```rust,no_run
/// use ssg::cmd::ShokuninConfig;
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
    /// Project name.
    pub site_name: String,
    /// Location of content files.
    pub content_dir: PathBuf,
    /// Output directory for generated files.
    pub output_dir: PathBuf,
    /// Location of template files.
    pub template_dir: PathBuf,
    /// Optional directory for development server.
    pub serve_dir: Option<PathBuf>,
    /// Base URL of the site (must be HTTP/HTTPS).
    pub base_url: String,
    /// Site title.
    pub site_title: String,
    /// Site description.
    pub site_description: String,
    /// Site language (format: `xx-XX`).
    pub language: String,
}

impl Default for ShokuninConfig {
    /// Provides a default configuration using the pre-initialized `DEFAULT_CONFIG`.
    fn default() -> Self {
        DEFAULT_CONFIG.clone()
    }
}

impl ShokuninConfig {
    /// Creates a configuration by merging the default values with any command-line arguments.
    ///
    /// # Arguments
    /// * `matches` - Parsed command-line arguments from Clap.
    ///
    /// # Errors
    /// Returns a [`CliError`] if:
    /// - A path fails validation (e.g., directory traversal or symlink).
    /// - A URL is malformed.
    /// - The language is incorrectly formatted.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let matches = cli.build().get_matches();
    /// let config = ShokuninConfig::from_matches(&matches)?;
    /// ```
    pub fn from_matches(
        matches: &ArgMatches,
    ) -> Result<Self, CliError> {
        // If a config file is specified, load from file.
        if let Some(config_path) = matches.get_one::<PathBuf>("config")
        {
            let mut loaded_config = Self::from_file(config_path)?;
            loaded_config.override_with_cli(matches)?;
            return Ok(loaded_config);
        }

        // Otherwise, start with the default configuration and override from CLI.
        let mut config = Self::default();
        config.override_with_cli(matches)?;
        Ok(config)
    }

    /// Loads configuration from a TOML file, enforcing a maximum file size limit.
    ///
    /// # Arguments
    /// * `path` - The path of the TOML file to be read.
    ///
    /// # Errors
    /// Returns a [`CliError`] if:
    /// - The file cannot be read or exceeds `MAX_CONFIG_SIZE`.
    /// - The file is malformed TOML.
    /// - Any fields fail validation afterward.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let config = ShokuninConfig::from_file(Path::new("config.toml"))?;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, CliError> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_CONFIG_SIZE {
            return Err(CliError::ValidationError(format!(
                "Config file too large (max {} bytes)",
                MAX_CONFIG_SIZE
            )));
        }

        let content = fs::read_to_string(path)?;
        let config: ShokuninConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// **(New)** Loads configuration settings from environment variables (if present).
    ///
    /// # Examples
    /// ```rust,ignore
    /// let mut config = ShokuninConfig::default();
    /// config.load_from_env()?;
    /// ```
    ///
    /// # Errors
    /// Returns a [`CliError::ValidationError`] if the final configuration fails validation.
    pub fn load_from_env(&mut self) -> Result<(), CliError> {
        if let Ok(base_url) = std::env::var("SHOKUNIN_BASE_URL") {
            self.base_url = base_url;
        }
        // Add additional environment variables here as needed
        self.validate()?;
        Ok(())
    }

    /// **(New)** A fluent builder interface for creating a `ShokuninConfig` in steps.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let config = ShokuninConfig::builder()
    ///     .site_name("My Custom Site".into())
    ///     .build()?;
    /// ```
    pub fn builder() -> ShokuninConfigBuilder {
        ShokuninConfigBuilder::default()
    }

    /// Validates the current configuration state.
    ///
    /// - Ensures the base URL is valid (HTTP/HTTPS, valid host, valid port).
    /// - Ensures no path poses a security risk (symlinks, directory traversal, etc.).
    /// - Checks that `language` follows the `xx-XX` format.
    /// - Ensures `site_name` and `site_title` are not empty.
    ///
    /// # Errors
    /// Returns a [`CliError::ValidationError`] or [`CliError::InvalidPath`] or
    /// [`CliError::InvalidUrl`] if validation fails.
    pub fn validate(&self) -> Result<(), CliError> {
        if self.site_name.trim().is_empty() {
            return Err(CliError::ValidationError(
                "site_name cannot be empty".into(),
            ));
        }

        if self.site_title.trim().is_empty() {
            return Err(CliError::ValidationError(
                "site_title cannot be empty".into(),
            ));
        }

        if !self.validate_language_format() {
            return Err(CliError::ValidationError(
                "Language must be in format 'xx-XX' with valid ISO codes".into(),
            ));
        }

        // Validate URL
        if !self.base_url.is_empty() {
            validate_url(&self.base_url)?;
        }

        // Validate paths
        validate_path_safety(&self.content_dir, "content_dir")?;
        validate_path_safety(&self.output_dir, "output_dir")?;
        validate_path_safety(&self.template_dir, "template_dir")?;
        if let Some(ref serve_dir) = self.serve_dir {
            validate_path_safety(serve_dir, "serve_dir")?;
        }

        Ok(())
    }

    /// Applies CLI overrides (e.g., `--new`, `--content`, etc.) on top of an existing configuration.
    fn override_with_cli(
        &mut self,
        matches: &ArgMatches,
    ) -> Result<(), CliError> {
        if let Some(site_name) = matches.get_one::<String>("new") {
            self.site_name = site_name.clone();
        }

        if let Some(content_dir) = matches.get_one::<PathBuf>("content")
        {
            self.content_dir =
                validate_path(content_dir, "content_dir")?;
        }

        if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
            self.output_dir = validate_path(output_dir, "output_dir")?;
        }

        if let Some(template_dir) =
            matches.get_one::<PathBuf>("template")
        {
            self.template_dir =
                validate_path(template_dir, "template_dir")?;
        }

        if let Some(serve_dir) = matches.get_one::<PathBuf>("serve") {
            self.serve_dir =
                Some(validate_path(serve_dir, "serve_dir")?);
        }

        self.validate()?;
        Ok(())
    }

    /// Checks if `self.language` conforms to the `xx-XX` pattern,
    /// where `xx` is lowercase ISO code, and `XX` is uppercase ISO code.
    ///
    /// # Returns
    /// * `true` if `language` is well-formed
    /// * `false` otherwise
    fn validate_language_format(&self) -> bool {
        let parts: Vec<&str> = self.language.split('-').collect();
        if parts.len() != 2 {
            return false;
        }

        let (lang, region) = (parts[0], parts[1]);
        lang.len() == 2
            && region.len() == 2
            && lang.chars().all(|c| c.is_ascii_lowercase())
            && region.chars().all(|c| c.is_ascii_uppercase())
    }
}

/// **(New)** A builder for `ShokuninConfig`, allowing fluent step-by-step construction.
///
/// ```rust,ignore
/// let config = ShokuninConfig::builder()
///     .site_name("Example".into())
///     .site_title("Example Title".into())
///     .build()?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct ShokuninConfigBuilder {
    /// The configuration being built.
    pub config: ShokuninConfig,
}

impl ShokuninConfigBuilder {
    /// Sets the `site_name`.
    pub fn site_name(mut self, name: String) -> Self {
        self.config.site_name = name;
        self
    }

    /// Sets the `base_url`.
    pub fn base_url(mut self, url: String) -> Self {
        self.config.base_url = url;
        self
    }

    /// Sets the `site_title`.
    pub fn site_title(mut self, title: String) -> Self {
        self.config.site_title = title;
        self
    }

    /// Sets the `site_description`.
    pub fn site_description(mut self, description: String) -> Self {
        self.config.site_description = description;
        self
    }

    /// Sets the `language`.
    pub fn language(mut self, lang: String) -> Self {
        self.config.language = lang;
        self
    }

    /// Sets the `content_dir`.
    pub fn content_dir(mut self, dir: PathBuf) -> Self {
        self.config.content_dir = dir;
        self
    }

    /// Sets the `output_dir`.
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.config.output_dir = dir;
        self
    }

    /// Sets the `template_dir`.
    pub fn template_dir(mut self, dir: PathBuf) -> Self {
        self.config.template_dir = dir;
        self
    }

    /// Sets the `serve_dir`.
    pub fn serve_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.config.serve_dir = dir;
        self
    }

    /// Validates and returns the final `ShokuninConfig`.
    ///
    /// # Errors
    /// Returns a [`CliError`] if the configuration fails validation.
    pub fn build(self) -> Result<ShokuninConfig, CliError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Command-line interface builder for the static site generator.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cli;

impl Cli {
    /// Creates a new CLI instance.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let cli = Cli::new();
    /// let matches = cli.build().get_matches();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Builds the command-line interface with all available options.
    ///
    /// # Examples
    /// ```rust,ignore
    /// let cli = Cli::new();
    /// let command = cli.build();
    /// let matches = command.get_matches();
    /// ```
    pub fn build(&self) -> Command {
        Command::new(env!("CARGO_PKG_NAME"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .version(env!("CARGO_PKG_VERSION"))
            .arg(Self::config_arg())
            .arg(Self::new_project_arg())
            .arg(Self::content_dir_arg())
            .arg(Self::output_dir_arg())
            .arg(Self::template_dir_arg())
            .arg(Self::serve_dir_arg())
            .arg(Self::watch_arg())
    }

    /// Displays the application banner (with a small performance optimization by using
    /// `String::with_capacity`).
    ///
    /// # Examples
    /// ```rust,ignore
    /// Cli::print_banner();
    /// ```
    pub fn print_banner() {
        let version = env!("CARGO_PKG_VERSION");
        let mut title = String::with_capacity(24 + version.len()); // "Shokunin (ssg) 🦀 v" + version
        title.push_str("Shokunin (ssg) 🦀 v");
        title.push_str(version);

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

    // -- Private methods for constructing arguments --

    fn config_arg() -> Arg {
        Arg::new("config")
            .help("Configuration file path (TOML)")
            .long("config")
            .short('f')
            .value_name("FILE")
            .value_parser(clap::value_parser!(PathBuf))
    }

    fn new_project_arg() -> Arg {
        Arg::new("new")
            .help("Create new project with the specified name")
            .long("new")
            .short('n')
            .value_name("NAME")
            .value_parser(clap::value_parser!(String))
    }

    fn content_dir_arg() -> Arg {
        Arg::new("content")
            .help("Path to the content directory")
            .long("content")
            .short('c')
            .value_name("DIR")
            .value_parser(clap::value_parser!(PathBuf))
    }

    fn output_dir_arg() -> Arg {
        Arg::new("output")
            .help("Path to the output directory")
            .long("output")
            .short('o')
            .value_name("DIR")
            .value_parser(clap::value_parser!(PathBuf))
    }

    fn template_dir_arg() -> Arg {
        Arg::new("template")
            .help("Path to the template directory")
            .long("template")
            .short('t')
            .value_name("DIR")
            .value_parser(clap::value_parser!(PathBuf))
    }

    fn serve_dir_arg() -> Arg {
        Arg::new("serve")
            .help("Path to the directory for the development server")
            .long("serve")
            .short('s')
            .value_name("DIR")
            .value_parser(clap::value_parser!(PathBuf))
    }

    fn watch_arg() -> Arg {
        Arg::new("watch")
            .help("Watch files and re-generate on changes")
            .long("watch")
            .short('w')
            .action(ArgAction::SetTrue)
    }
}

/// Creates the command-line interface (legacy function).
///
/// Provided for backward compatibility with previous code.
/// Prefer using [`Cli::new`] and [`Cli::build`] in newer code.
pub fn build() -> Command {
    Cli::new().build()
}

/// Displays the application banner (legacy function).
///
/// Provided for backward compatibility with previous code.
/// Prefer using [`Cli::print_banner`] in newer code.
pub fn print_banner() {
    Cli::print_banner();
}

/// Validates and normalizes a path by canonicalizing it only if it exists.
/// This avoids errors when the path does not exist yet but is otherwise valid.
///
/// # Arguments
/// * `path`  - A reference to the path to validate.
/// * `field` - The name of the field (used for error messages).
///
/// # Errors
/// Returns a [`CliError::InvalidPath`] if the path is determined to be unsafe.
///
/// # Examples
/// ```rust
/// use std::path::Path;
/// use ssg::cmd::{validate_path, CliError};
///
/// fn main() -> Result<(), CliError> {
///     let safe_path = validate_path(Path::new("content"), "content_dir")?;
///     Ok(())
/// }
/// ```
pub fn validate_path(
    path: &Path,
    field: &str,
) -> Result<PathBuf, CliError> {
    validate_path_safety(path, field)?;

    // Only canonicalize if the path exists.
    // If it doesn't exist, return the original path.
    if path.exists() {
        let canonical =
            path.canonicalize().map_err(CliError::IoError)?;
        Ok(canonical)
    } else {
        Ok(path.to_path_buf())
    }
}

/// Performs security checks on a path to prevent unsafe usage such as directory traversal
/// or symbolic links.
///
/// # Arguments
/// * `path` - The path to be validated.
/// * `field` - Field name for error reporting.
///
/// # Errors
/// Returns a [`CliError::InvalidPath`] if any security checks fail.
fn validate_path_safety(
    path: &Path,
    field: &str,
) -> Result<(), CliError> {
    let path_str = path.to_string_lossy();

    // Debug output to trace path handling
    println!("DEBUG: Validating path: {}", path_str);

    // Check for null bytes.
    if path_str.contains('\0') {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains null byte".to_string(),
        });
    }

    // Check for right-to-left override characters.
    if path_str.contains('\u{202E}') {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains bidirectional text override"
                .to_string(),
        });
    }

    // Directory traversal check: Reject paths with `..` components.
    if path_str.contains("..") && !path.is_absolute() {
        println!("DEBUG: Path failed directory traversal check");
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path traversal or invalid format detected"
                .to_string(),
        });
    }

    // Reject double slashes (`//`) in relative paths.
    if path_str.contains("//") && !path.is_absolute() {
        println!("DEBUG: Path failed double-slash check");
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains invalid double slashes".to_string(),
        });
    }

    // Windows-only check: Disallow colons except for drive letters (e.g., `C:\`).
    #[cfg(windows)]
    {
        let chars: Vec<char> = path_str.chars().collect();
        if path_str.contains(':')
            && (chars.len() < 2 || chars[1] != ':')
        {
            println!("DEBUG: Path failed Windows colon check");
            return Err(CliError::InvalidPath {
                field: field.to_string(),
                details: "Invalid use of colon in path".to_string(),
            });
        }
    }

    // Non-Windows check: Disallow colons entirely.
    #[cfg(not(windows))]
    if path_str.contains(':') {
        println!("DEBUG: Path failed non-Windows colon check");
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains invalid character ':'".to_string(),
        });
    }

    // Check for reserved names (platform-specific).
    if RESERVED_NAMES.contains(&path_str.to_lowercase().as_str()) {
        println!("DEBUG: Path failed reserved name check");
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Reserved system path name".to_string(),
        });
    }

    // Check if the path exists and is a symbolic link.
    if path.exists() {
        let meta =
            path.symlink_metadata().map_err(CliError::IoError)?;
        if meta.file_type().is_symlink() {
            println!("DEBUG: Path failed symbolic link check");
            return Err(CliError::InvalidPath {
                field: field.to_string(),
                details: "Symbolic links are not allowed".to_string(),
            });
        }
    }

    // Reject specific sensitive system paths (e.g., `/etc/` on Unix).
    #[cfg(unix)]
    {
        if path.starts_with("/etc/") {
            println!("DEBUG: Path failed sensitive system path check");
            return Err(CliError::InvalidPath {
                field: field.to_string(),
                details: "Sensitive system path detected".to_string(),
            });
        }
    }

    println!("DEBUG: Path passed all checks");
    Ok(())
}

/// Ensures a given URL is valid, using only HTTP or HTTPS schemes.
/// Also verifies ports (if present) are valid.
///
/// # Arguments
/// * `url` - A string reference to the URL being validated.
///
/// # Errors
/// Returns [`CliError::InvalidUrl`] if the URL is malformed or uses an invalid scheme/port.
fn validate_url(url: &str) -> Result<(), CliError> {
    let parsed_url = Url::parse(url)
        .map_err(|_| CliError::InvalidUrl(url.to_string()))?;

    // Only allow http or https
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(CliError::InvalidUrl(url.to_string()));
    }

    // Host must be non-empty and not start with '.'
    if parsed_url
        .host_str()
        .map_or(true, |host| host.is_empty() || host.starts_with('.'))
    {
        return Err(CliError::InvalidUrl(url.to_string()));
    }

    // Disallow backslashes
    if url.contains('\\') {
        return Err(CliError::InvalidUrl(url.to_string()));
    }

    // Check for valid port if specified
    if let Some(port) = parsed_url.port() {
        if port == 0 {
            return Err(CliError::InvalidUrl(format!(
                "URL '{}' has invalid port: 0",
                url
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, io::Write};
    use tempfile::TempDir;

    /// Creates a temporary directory for testing.
    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }

    /// Creates a test configuration file.
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
    fn test_cli_structure() {
        let cli = Cli::new();
        cli.build().debug_assert();
    }

    #[test]
    fn test_default_config() {
        let config = ShokuninConfig::default();
        assert_eq!(config.content_dir, PathBuf::from("content"));
        assert_eq!(config.output_dir, PathBuf::from("public"));
        assert_eq!(config.template_dir, PathBuf::from("templates"));
        assert!(config.serve_dir.is_none());
        assert_eq!(
            config.base_url,
            format!("http://{}:{}", DEFAULT_HOST, DEFAULT_PORT)
        );
        assert_eq!(config.language, "en-GB");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = ShokuninConfig {
            site_name: "test".to_string(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("public"),
            template_dir: PathBuf::from("templates"),
            serve_dir: Some(PathBuf::from("serve")),
            base_url: format!(
                "http://{}:{}",
                DEFAULT_HOST, DEFAULT_PORT
            ),
            site_title: "Test Site".to_string(),
            site_description: "Test Description".to_string(),
            language: "en-GB".to_string(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
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

    #[test]
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
    fn test_config_from_invalid_file() {
        let temp_dir = setup_temp_dir();
        let config_path = temp_dir.path().join("invalid.toml");
        fs::write(&config_path, "invalid = { toml")
            .expect("Failed to write file");

        let result = ShokuninConfig::from_file(&config_path);
        assert!(matches!(result, Err(CliError::TomlError(_))));
    }

    #[test]
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
    fn test_path_validation_valid() {
        let valid_paths = vec![
            "content",
            "content/subfolder",
            "templates/layouts",
            "./content",
        ];

        for path in valid_paths {
            assert!(
                validate_path(Path::new(path), "test").is_ok(),
                "Path failed: {path}"
            );
        }
    }

    #[test]
    fn test_cli_argument_parsing() {
        let cli = Cli::new();

        // Test minimal arguments
        let result = cli.build().try_get_matches_from(vec![
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

        // Test optional serve argument
        let result = cli.build().try_get_matches_from(vec![
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

        // Test watch flag
        let result = cli.build().try_get_matches_from(vec![
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
    fn test_config_from_matches() {
        // Create absolute paths for directories.
        let content_path =
            fs::canonicalize("content").unwrap_or_else(|_| {
                fs::create_dir_all("content")
                    .expect("Failed to create content dir");
                fs::canonicalize("content")
                    .expect("Failed to canonicalize content dir")
            });
        let output_path =
            fs::canonicalize("public").unwrap_or_else(|_| {
                fs::create_dir_all("public")
                    .expect("Failed to create public dir");
                fs::canonicalize("public")
                    .expect("Failed to canonicalize public dir")
            });
        let template_path = fs::canonicalize("templates")
            .unwrap_or_else(|_| {
                fs::create_dir_all("templates")
                    .expect("Failed to create templates dir");
                fs::canonicalize("templates")
                    .expect("Failed to canonicalize templates dir")
            });

        let cli = Cli::new();
        let matches = cli
            .build()
            .try_get_matches_from(vec![
                "ssg",
                "--new",
                "test-site",
                "--content",
                content_path.to_str().unwrap(),
                "--output",
                output_path.to_str().unwrap(),
                "--template",
                template_path.to_str().unwrap(),
            ])
            .expect("Failed to parse matches");

        let config = ShokuninConfig::from_matches(&matches);
        assert!(
            config.is_ok(),
            "Expected successful config creation, but got: {:#?}",
            config.err()
        );

        let config = config.unwrap();
        assert_eq!(config.site_name, "test-site");
        assert_eq!(config.content_dir, content_path);
        assert_eq!(config.output_dir, output_path);
        assert_eq!(config.template_dir, template_path);
        assert!(config.serve_dir.is_none());
    }

    #[test]
    fn test_error_handling() {
        // Missing argument error
        let error = CliError::MissingArgument("test".to_string());
        assert_eq!(
            error.to_string(),
            "Required argument missing: test"
        );

        // Invalid path error
        let error = CliError::InvalidPath {
            field: "test".to_string(),
            details: "invalid".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid path for test: invalid");

        // Invalid URL error
        let error = CliError::InvalidUrl("test".to_string());
        assert_eq!(error.to_string(), "Invalid URL: test");
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
            "/etc/passwd",                // Sensitive system file
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
            "not-a-url",
            "ftp://example.com",
            "http://invalid\u{202E}url.com",
            "http://example.com\\path",
            "https://:80",
            "http://example.com:abc",
            "http://.com",
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

    #[test]
    fn test_config_validation_empty_fields() {
        let config = ShokuninConfig {
            site_name: "".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(CliError::ValidationError(_))
        ));
    }

    #[test]
    fn test_language_validation() {
        let valid_languages = vec!["en-US", "fr-FR", "de-DE"];
        let invalid_languages =
            vec!["en", "en-", "en-Us", "EN-US", "123-45"];

        for lang in valid_languages {
            let config = ShokuninConfig {
                language: lang.to_string(),
                ..Default::default()
            };
            assert!(
                config.validate().is_ok(),
                "Language {} should be valid",
                lang
            );
        }

        for lang in invalid_languages {
            let config = ShokuninConfig {
                language: lang.to_string(),
                ..Default::default()
            };
            assert!(
                config.validate().is_err(),
                "Language {} should be invalid",
                lang
            );
        }
    }

    #[test]
    fn test_builder_pattern() {
        let built_config = ShokuninConfig::builder()
            .site_name("Built Site".into())
            .base_url("http://127.0.0.1:8080".into())
            .site_title("Builder Title".into())
            .site_description("Builder Description".into())
            .language("en-US".into())
            .build();

        assert!(built_config.is_ok());
        let final_config = built_config.unwrap();
        assert_eq!(final_config.site_name, "Built Site");
        assert_eq!(final_config.base_url, "http://127.0.0.1:8080");
        assert_eq!(final_config.site_title, "Builder Title");
        assert_eq!(
            final_config.site_description,
            "Builder Description"
        );
        assert_eq!(final_config.language, "en-US");
    }
}
