//! # Command Line Interface Module
//!
//! This module provides a secure and robust command-line interface (CLI) for the
//! **Static Site Generator (SSG)**. It handles argument parsing, configuration management,
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
//! use ssg::cmd::{Cli, SsgConfig};
//!
//! fn main() -> anyhow::Result<()> {
//!     let matches = Cli::build().get_matches();
//!
//!     // Attempt to load configuration from command-line arguments
//!     let mut config = SsgConfig::from_matches(&matches)?;
//!
//!     println!("Configuration loaded: {:?}", config);
//!     // Continue with application logic...
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use thiserror::Error;
use url::Url;

/// Default port for the local development server.
pub const DEFAULT_PORT: u16 = 8000;
/// Default host for the local development server.
pub const DEFAULT_HOST: &str = "127.0.0.1";
/// Reserved names that cannot be used as paths on Windows systems.
pub const RESERVED_NAMES: &[&str] =
    &["con", "aux", "nul", "prn", "com1", "lpt1"];
/// Maximum allowed size in bytes for config files.
pub const MAX_CONFIG_SIZE: usize = 1024 * 1024; // 1MB limit

/// Default site name for the configuration.
pub const DEFAULT_SITE_NAME: &str = "MySsgSite";
/// Default site title for the configuration.
pub const DEFAULT_SITE_TITLE: &str = "My SSG Site";

/// A static default configuration for the SSG site.
pub static DEFAULT_CONFIG: Lazy<Arc<SsgConfig>> = Lazy::new(|| {
    Arc::new(SsgConfig {
        site_name: DEFAULT_SITE_NAME.to_string(),
        content_dir: PathBuf::from("content"),
        output_dir: PathBuf::from("public"),
        template_dir: PathBuf::from("templates"),
        serve_dir: None,
        base_url: format!("http://{}:{}", DEFAULT_HOST, DEFAULT_PORT),
        site_title: DEFAULT_SITE_TITLE.to_string(),
        site_description: "A site built with SSG".to_string(),
        language: "en-GB".to_string(),
    })
});

/// Type-safe representation of a language code.
///
/// # Examples
/// ```
/// use ssg::cmd::LanguageCode;
/// assert!(LanguageCode::new("en-GB").is_ok());
/// assert!(LanguageCode::new("invalid").is_err());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageCode(String);

impl LanguageCode {
    /// Creates a new `LanguageCode` instance from a string.
    pub fn new(code: &str) -> Result<Self, CliError> {
        if code.len() != 5 || code.chars().nth(2) != Some('-') {
            return Err(CliError::ValidationError(
                "Invalid language code format".into(),
            ));
        }

        let (lang, region) = code.split_at(2);
        let region = &region[1..]; // Skip hyphen

        if !lang.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(CliError::ValidationError(
                "Language code must be lowercase".into(),
            ));
        }

        if !region.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CliError::ValidationError(
                "Region code must be uppercase".into(),
            ));
        }

        Ok(Self(code.to_string()))
    }
}

impl std::fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Possible errors that can occur during CLI operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CliError {
    #[error("Invalid path '{field}': {details}")]
    /// Error indicating an invalid path with additional details.
    InvalidPath {
        /// Field name where the path is used.
        field: String,
        /// Additional details about the invalid path.
        details: String,
    },

    #[error("Required argument missing: {0}")]
    /// Error indicating a missing required argument.
    MissingArgument(String),

    #[error("Invalid URL: {0}")]
    /// Error indicating an invalid URL.
    InvalidUrl(String),

    #[error("IO error: {0}")]
    /// Error indicating an I/O error.
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    /// Error indicating a TOML parsing error.
    TomlError(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    /// Error indicating a validation error.
    ValidationError(String),
}

/// Core configuration for the static site generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgConfig {
    /// Name of the site.
    pub site_name: String,
    /// Directory containing content files.
    pub content_dir: PathBuf,
    /// Directory for generated output files.
    pub output_dir: PathBuf,
    /// Directory containing template files.
    pub template_dir: PathBuf,
    /// Optional directory for development server files.
    pub serve_dir: Option<PathBuf>,
    /// Base URL of the site.
    pub base_url: String,
    /// Title of the site.
    pub site_title: String,
    /// Description of the site.
    pub site_description: String,
    /// Language code for the site.
    pub language: String,
}

impl Default for SsgConfig {
    fn default() -> Self {
        DEFAULT_CONFIG.as_ref().clone()
    }
}

impl SsgConfig {
    /// Applies command-line arguments to override defaults.
    fn override_with_cli(
        mut self,
        matches: &ArgMatches,
    ) -> Result<Self, CliError> {
        // If `-n/--new` was used
        if let Some(site_name) = matches.get_one::<String>("new") {
            self.site_name = site_name.to_string();
        }

        // If `-c/--content` was used
        if let Some(content_dir) = matches.get_one::<PathBuf>("content") {
            self.content_dir = content_dir.clone();
        }

        // If `-o/--output` was used
        if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
            self.output_dir = output_dir.clone();
        }

        // If `-t/--template` was used
        if let Some(template_dir) = matches.get_one::<PathBuf>("template") {
            self.template_dir = template_dir.clone();
        }

        // If `-s/--serve` was used
        if let Some(serve_dir) = matches.get_one::<PathBuf>("serve") {
            self.serve_dir = Some(serve_dir.clone());
        }

        // `--watch` flag is handled by the caller (run() in lib.rs)

        // Re-validate after overriding
        self.validate()?;
        Ok(self)
    }
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
    /// let config = SsgConfig::from_matches(&matches)?;
    /// ```
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, CliError> {
        if let Some(config_path) = matches.get_one::<PathBuf>("config") {
            let loaded_config = Self::from_file(config_path)?;
            return Ok(loaded_config);
        }

        // 1) Start with defaults
        let config = Self::default();

        // 2) Override them with CLI flags
        let config = config.override_with_cli(matches)?;

        // 3) Return the result
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
    /// let config = SsgConfig::from_file(Path::new("config.toml"))?;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, CliError> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_CONFIG_SIZE as u64 {
            return Err(CliError::ValidationError(format!(
                "Config file too large (max {} bytes)",
                MAX_CONFIG_SIZE
            )));
        }

        let content = fs::read_to_string(path)?;
        let config: SsgConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Creates a new `SsgConfig` instance from a TOML file.
    pub fn validate(&self) -> Result<(), CliError> {
        debug!("Validating config: {:?}", self);

        if self.site_name.trim().is_empty() {
            error!("site_name cannot be empty");
            return Err(CliError::ValidationError(
                "site_name cannot be empty".into(),
            ));
        }

        if !self.base_url.is_empty() {
            validate_url(&self.base_url)?;
        }

        validate_path_safety(&self.content_dir, "content_dir")?;
        validate_path_safety(&self.output_dir, "output_dir")?;
        validate_path_safety(&self.template_dir, "template_dir")?;
        if let Some(ref serve_dir) = self.serve_dir {
            validate_path_safety(serve_dir, "serve_dir")?;
        }

        info!("Config validation successful");
        Ok(())
    }

    /// Creates a new `SsgConfig` instance from a TOML file.
    pub fn builder() -> SsgConfigBuilder {
        SsgConfigBuilder::default()
    }
}

impl FromStr for SsgConfig {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: SsgConfig = toml::from_str(s)?;
        config.validate()?;
        Ok(config)
    }
}

/// Builder for `SsgConfig`.
#[derive(Debug, Clone, Default)]
pub struct SsgConfigBuilder {
    config: SsgConfig,
}

/// # Examples
/// ```
/// use ssg::cmd::SsgConfig;
/// let config = SsgConfig::builder()
///     .site_name("My Site".to_string())
///     .base_url("http://example.com".to_string())
///     .build()
///     .unwrap();
/// ```
impl SsgConfigBuilder {
    /// Sets the site name for the configuration.
    pub fn site_name(mut self, name: String) -> Self {
        self.config.site_name = name;
        self
    }
    /// Sets the base URL for the configuration.
    pub fn base_url(mut self, url: String) -> Self {
        self.config.base_url = url;
        self
    }
    /// Sets the content directory for the configuration.
    pub fn content_dir(mut self, dir: PathBuf) -> Self {
        self.config.content_dir = dir;
        self
    }
    /// Sets the output directory for the configuration.
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.config.output_dir = dir;
        self
    }
    /// Sets the template directory for the configuration.
    pub fn template_dir(mut self, dir: PathBuf) -> Self {
        self.config.template_dir = dir;
        self
    }
    /// Sets the optional development server directory for the configuration.
    pub fn serve_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.config.serve_dir = dir;
        self
    }
    /// Sets the site title for the configuration.
    pub fn site_title(mut self, title: String) -> Self {
        self.config.site_title = title;
        self
    }
    /// Sets the site description for the configuration.
    pub fn site_description(mut self, desc: String) -> Self {
        self.config.site_description = desc;
        self
    }
    /// Sets the language code for the configuration.
    pub fn language(mut self, lang: String) -> Self {
        self.config.language = lang;
        self
    }
    /// Builds the final `SsgConfig` instance.
    pub fn build(self) -> Result<SsgConfig, CliError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Validates a URL for security and format.
///
/// # Examples
/// ```
/// use ssg::cmd::validate_url;
/// assert!(validate_url("http://example.com").is_ok());
/// assert!(validate_url("javascript:alert(1)").is_err());
/// ```
pub fn validate_url(url: &str) -> Result<(), CliError> {
    let xss_patterns = ["javascript:", "data:", "vbscript:"];
    if xss_patterns.iter().any(|p| url.contains(p)) {
        return Err(CliError::InvalidUrl(
            "URL contains unsafe protocol".into(),
        ));
    }

    if url.contains('<') || url.contains('>') || url.contains('"') {
        return Err(CliError::InvalidUrl(
            "URL contains invalid characters".into(),
        ));
    }

    let parsed_url =
        Url::parse(url).map_err(|_| CliError::InvalidUrl(url.to_string()))?;
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(CliError::InvalidUrl(url.to_string()));
    }
    Ok(())
}

fn validate_path_safety(path: &Path, field: &str) -> Result<(), CliError> {
    // Check for invalid characters and mixed separators
    let path_str = path.to_string_lossy();

    // Basic invalid characters
    let invalid_chars = ["<", ">", "|", "\"", "?", "*"];
    if invalid_chars.iter().any(|&c| path_str.contains(c)) {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains invalid characters".to_string(),
        });
    }

    // Check for mixed/invalid path separators (only on non-Windows)
    #[cfg(not(target_os = "windows"))]
    if path_str.contains('\\') {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains backslashes".to_string(),
        });
    }

    // Parent directory traversal check
    if !path.is_absolute() && path_str.contains("..") {
        return Err(CliError::InvalidPath {
            field: field.to_string(),
            details: "Path contains parent directory traversal".to_string(),
        });
    }

    // Check for Windows reserved names
    if let Some(stem) = path.file_stem() {
        let stem_lower = stem.to_string_lossy().to_lowercase();
        if RESERVED_NAMES.contains(&stem_lower.as_str()) {
            return Err(CliError::InvalidPath {
                field: field.to_string(),
                details: format!("Path uses reserved name '{}'", stem_lower),
            });
        }
    }

    // If path exists, check if it's a symlink
    if path.exists() {
        let metadata = fs::symlink_metadata(path).map_err(|_| {
            CliError::IoError(std::io::Error::other(
                "Failed to get path metadata",
            ))
        })?;

        if metadata.file_type().is_symlink() {
            return Err(CliError::InvalidPath {
                field: field.to_string(),
                details: "Path is a symlink".to_string(),
            });
        }
    }

    Ok(())
}

/// Const validation for compile-time checks.
const _: () = {
    assert!(MAX_CONFIG_SIZE > 0);
    assert!(MAX_CONFIG_SIZE <= 10 * 1024 * 1024); // Max 10MB
};

#[derive(Clone, Copy, Debug, Default)]
/// A simple CLI struct for building the SSG command.
pub struct Cli;

impl Cli {
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
                    .value_name("NAME")
                    .value_parser(clap::value_parser!(String)), // Change from PathBuf to String
            )
            .arg(
                Arg::new("content")
                    .help("Content directory")
                    .long("content")
                    .short('c')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("output")
                    .help("Output directory")
                    .long("output")
                    .short('o')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("template")
                    .help("Template directory")
                    .long("template")
                    .short('t')
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
            .arg(
                Arg::new("drafts")
                    .help("Include draft pages in the build")
                    .long("drafts")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("deploy")
                    .help("Generate deployment config (netlify, vercel, cloudflare, github)")
                    .long("deploy")
                    .value_name("TARGET")
                    .value_parser(clap::value_parser!(String)),
            )
            .arg(
                Arg::new("quiet")
                    .help("Suppress non-error output")
                    .long("quiet")
                    .short('q')
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("verbose")
                    .help("Show detailed build information")
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
    }

    /// Displays the application banner
    pub fn print_banner() {
        let version = env!("CARGO_PKG_VERSION");
        let mut title = String::with_capacity(16 + version.len());
        title.push_str("SSG 🦀 v");
        title.push_str(version);

        let description =
            "A Fast and Flexible Static Site Generator written in Rust";
        let width = title.len().max(description.len()) + 4;
        let line = "─".repeat(width - 2);

        println!("\n┌{}┐", line);
        println!("│{:^width$}│", title.green().bold(), width = width - 3);
        println!("├{}┤", line);
        println!("│{:^width$}│", description.blue().bold(), width = width - 2);
        println!("└{}┘\n", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_language_code() {
        assert!(LanguageCode::new("en-GB").is_ok());
        assert!(LanguageCode::new("en-gb").is_err());
        assert!(LanguageCode::new("EN-GB").is_err());
        assert!(LanguageCode::new("e-GB").is_err());
    }

    #[test]
    fn test_config_validation() {
        let config = SsgConfig::builder().site_name("".to_string()).build();
        assert!(matches!(config, Err(CliError::ValidationError(_))));
    }

    #[test]
    fn test_url_validation() {
        let cmd = Cli::build();
        // Provide the required arguments so Clap won't fail:
        let _matches = cmd.get_matches_from(vec![
            "ssg",
            "--new",
            "dummy_site",
            "--content",
            "dummy_content",
            "--output",
            "dummy_output",
            "--template",
            "dummy_template",
        ]);

        // Now test logic that calls validate_url, etc.
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("javascript:alert(1)").is_err());
        assert!(validate_url("https://example.com<script>").is_err());
    }

    #[test]
    fn test_config_file_size_limit() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("large.toml");
        let mut file = File::create(&config_path).unwrap();

        // Write data larger than MAX_CONFIG_SIZE
        write!(file, "{}", "x".repeat(MAX_CONFIG_SIZE + 1)).unwrap();

        assert!(matches!(
            SsgConfig::from_file(&config_path),
            Err(CliError::ValidationError(_))
        ));
    }

    #[test]
    fn test_path_safety() {
        let valid = Path::new("valid");
        let absolute_valid = std::env::current_dir().unwrap().join(valid);
        assert!(validate_path_safety(&absolute_valid, "test").is_ok());
    }

    #[test]
    fn test_config_from_str() {
        let config_str = r#"
    site_name = "test"
    content_dir = "./examples/content"
    output_dir = "./examples/public"
    template_dir = "./examples/templates"
    base_url = "http://example.com"
    site_title = "Test Site"
    site_description = "Test Description"
    language = "en-GB"
    "#;

        let config: Result<SsgConfig, _> = config_str.parse();
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_builder_all_fields() {
        let temp_dir = tempdir().unwrap();
        let serve_dir = temp_dir.path().join("serve");

        // Create the serve directory
        fs::create_dir_all(&serve_dir).unwrap();

        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url("http://example.com".to_string())
            .content_dir(PathBuf::from("./examples/content"))
            .output_dir(PathBuf::from("./examples/public"))
            .template_dir(PathBuf::from("./examples/templates"))
            .serve_dir(Some(serve_dir))
            .site_title("Test Site".to_string())
            .site_description("Test Desc".to_string())
            .language("en-GB".to_string())
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_banner_display() {
        // Create the expected title
        let version = env!("CARGO_PKG_VERSION");
        let title = format!("SSG 🦀 v{}", version);
        let description =
            "A Fast and Flexible Static Site Generator written in Rust";
        let width = title.len().max(description.len()) + 4;
        let line = "─".repeat(width - 2);

        // Call print_banner and verify output visually
        // Note: We can't easily capture stdout in a test, so we just verify
        // that the function doesn't panic
        Cli::print_banner();

        // Basic sanity check - verify the banner components are formatted correctly
        assert!(!line.is_empty());
        assert!(title.contains("SSG"));
        assert!(title.contains(version));
    }

    #[test]
    fn test_invalid_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        let mut file = File::create(&config_path).unwrap();
        write!(file, "invalid toml content").unwrap();

        assert!(matches!(
            SsgConfig::from_file(&config_path),
            Err(CliError::TomlError(_))
        ));
    }

    #[test]
    fn test_language_code_display() {
        let code = LanguageCode::new("en-GB").unwrap();
        assert_eq!(code.to_string(), "en-GB");
    }

    #[test]
    fn test_from_matches() {
        let matches = Cli::build().get_matches_from(vec!["ssg"]);
        let config = SsgConfig::from_matches(&matches);
        assert!(config.is_ok());
    }

    #[test]
    fn test_language_code_edge_cases() {
        assert!(LanguageCode::new("enGB").is_err());
        assert!(LanguageCode::new("e-G").is_err());
        assert!(LanguageCode::new("").is_err());
    }

    #[test]
    fn test_config_builder_empty_required_fields() {
        let config = SsgConfig::builder()
            .site_name("".to_string())
            .site_title("".to_string())
            .build();
        assert!(matches!(config, Err(CliError::ValidationError(_))));
    }
    #[test]
    fn test_absolute_path_validation() {
        let path = std::env::current_dir().unwrap().join("valid_path");
        assert!(validate_path_safety(&path, "test").is_ok());
    }
    #[test]
    fn test_path_with_separators() {
        // Minimal command that doesn't require any flags:
        let cmd = Command::new("test_no_required_args");
        let _matches = cmd.get_matches_from(vec!["test_no_required_args"]);

        // Now test the function you actually care about:
        let path = Path::new("path/to\\file");
        let result = validate_path_safety(path, "test");
        assert!(result.is_err(), "Expected error for backslashes");
    }

    #[test]
    fn test_symlink_path_validation() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("target");
        let symlink = temp_dir.path().join("symlink");

        // Create target and symlink
        fs::write(&target, "content").unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &symlink).unwrap();

        // Verify paths
        let resolved_path = fs::canonicalize(&symlink).unwrap();
        let normalized_target = fs::canonicalize(&target).unwrap();
        println!("Resolved symlink path: {:?}", resolved_path);
        println!("Normalized target path: {:?}", normalized_target);

        // Validate symlink path - should fail as symlinks are not allowed
        let result = validate_path_safety(&symlink, "symlink");
        assert!(result.is_err(), "Expected error for symlink path");
        assert!(matches!(
            result,
            Err(CliError::InvalidPath { field: _, details }) if details.contains("symlink")
        ));
    }
    #[test]
    fn test_url_edge_cases() {
        assert!(validate_url("http://").is_err());
        assert!(validate_url("https://").is_err());
        assert!(validate_url("http://example.com:65536").is_err());
    }

    #[test]
    fn test_config_file_not_found() {
        let non_existent = Path::new("non_existent.toml");
        assert!(matches!(
            SsgConfig::from_file(non_existent),
            Err(CliError::IoError(_))
        ));
    }

    #[test]
    fn test_from_matches_with_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
site_name = "from-file"
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://example.com"
site_title = "File Site"
site_description = "From file"
language = "en-GB"
"#;
        fs::write(&config_path, config_content).unwrap();

        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec![
            "ssg",
            "--config",
            config_path.to_str().unwrap(),
        ]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert_eq!(config.site_name, "from-file");
    }

    #[test]
    fn test_override_with_cli_all_flags() {
        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec![
            "ssg",
            "--new",
            "cli-site",
            "--content",
            "./examples/content",
            "--output",
            "./examples/public",
            "--template",
            "./examples/templates",
            "--serve",
            "./examples/public",
        ]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert_eq!(config.site_name, "cli-site");
        assert_eq!(config.content_dir, PathBuf::from("./examples/content"));
        assert_eq!(config.output_dir, PathBuf::from("./examples/public"));
        assert_eq!(config.template_dir, PathBuf::from("./examples/templates"));
        assert!(config.serve_dir.is_some());
    }

    #[test]
    fn test_validate_url_ftp_scheme() {
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_validate_path_with_invalid_chars() {
        let result =
            validate_path_safety(Path::new("path<with>invalid"), "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
    }

    #[test]
    fn test_validate_path_with_traversal() {
        let result = validate_path_safety(Path::new("../etc/passwd"), "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
    }

    #[test]
    fn test_validate_path_with_reserved_name() {
        let result = validate_path_safety(Path::new("con"), "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
        let result = validate_path_safety(Path::new("aux"), "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_validate_path_with_backslash() {
        let result =
            validate_path_safety(Path::new("path\\with\\backslash"), "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
    }

    #[test]
    fn test_override_with_watch_flag() {
        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec!["ssg", "--watch"]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        // Watch flag is accepted but is a no-op currently
        assert!(!config.site_name.is_empty());
    }

    #[test]
    fn test_validate_empty_url() {
        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url(String::new())
            .build();
        // Empty URL should be accepted (skips validation)
        assert!(config.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_path_existing_symlink() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("real");
        let link = temp_dir.path().join("link");
        fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = validate_path_safety(&link, "test");
        assert!(matches!(result, Err(CliError::InvalidPath { .. })));
    }
}
