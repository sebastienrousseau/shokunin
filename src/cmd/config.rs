// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SSG site configuration and builder.

use super::error::CliError;
use super::validation::{validate_path_safety, validate_url};
use super::{default_config, MAX_CONFIG_SIZE};
use clap::ArgMatches;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

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
    /// Optional i18n configuration for multi-locale sites.
    #[serde(default)]
    pub i18n: Option<crate::i18n::I18nConfig>,
}

impl Default for SsgConfig {
    fn default() -> Self {
        default_config().as_ref().clone()
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
            self.site_name.clone_from(site_name);
        }

        // If `-c/--content` was used
        if let Some(content_dir) = matches.get_one::<PathBuf>("content") {
            self.content_dir.clone_from(content_dir);
        }

        // If `-o/--output` was used
        if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
            self.output_dir.clone_from(output_dir);
        }

        // If `-t/--template` was used
        if let Some(template_dir) = matches.get_one::<PathBuf>("template") {
            self.template_dir.clone_from(template_dir);
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
                "Config file too large (max {MAX_CONFIG_SIZE} bytes)"
            )));
        }

        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Creates a new `SsgConfig` instance from a TOML file.
    pub fn validate(&self) -> Result<(), CliError> {
        debug!("Validating config: {self:?}");

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
    #[must_use]
    pub fn builder() -> SsgConfigBuilder {
        SsgConfigBuilder::default()
    }
}

impl FromStr for SsgConfig {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s)?;
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
    #[must_use]
    pub fn site_name(mut self, name: String) -> Self {
        self.config.site_name = name;
        self
    }
    /// Sets the base URL for the configuration.
    #[must_use]
    pub fn base_url(mut self, url: String) -> Self {
        self.config.base_url = url;
        self
    }
    /// Sets the content directory for the configuration.
    #[must_use]
    pub fn content_dir(mut self, dir: PathBuf) -> Self {
        self.config.content_dir = dir;
        self
    }
    /// Sets the output directory for the configuration.
    #[must_use]
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.config.output_dir = dir;
        self
    }
    /// Sets the template directory for the configuration.
    #[must_use]
    pub fn template_dir(mut self, dir: PathBuf) -> Self {
        self.config.template_dir = dir;
        self
    }
    /// Sets the optional development server directory for the configuration.
    #[must_use]
    pub fn serve_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.config.serve_dir = dir;
        self
    }
    /// Sets the site title for the configuration.
    #[must_use]
    pub fn site_title(mut self, title: String) -> Self {
        self.config.site_title = title;
        self
    }
    /// Sets the site description for the configuration.
    #[must_use]
    pub fn site_description(mut self, desc: String) -> Self {
        self.config.site_description = desc;
        self
    }
    /// Sets the language code for the configuration.
    #[must_use]
    pub fn language(mut self, lang: String) -> Self {
        self.config.language = lang;
        self
    }
    /// Sets the i18n configuration.
    #[must_use]
    pub fn i18n(mut self, i18n: Option<crate::i18n::I18nConfig>) -> Self {
        self.config.i18n = i18n;
        self
    }
    /// Builds the final `SsgConfig` instance.
    pub fn build(self) -> Result<SsgConfig, CliError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Cli;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_config_validation() {
        let config = SsgConfig::builder().site_name(String::new()).build();
        assert!(matches!(config, Err(CliError::ValidationError(_))));
    }

    #[test]
    fn test_config_file_size_limit() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("large.toml");
        let mut file = File::create(&config_path).unwrap();

        write!(file, "{}", "x".repeat(MAX_CONFIG_SIZE + 1)).unwrap();

        assert!(matches!(
            SsgConfig::from_file(&config_path),
            Err(CliError::ValidationError(_))
        ));
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
    fn test_from_matches() {
        let matches = Cli::build().get_matches_from(vec!["ssg"]);
        let config = SsgConfig::from_matches(&matches);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_builder_empty_required_fields() {
        let config = SsgConfig::builder()
            .site_name(String::new())
            .site_title(String::new())
            .build();
        assert!(matches!(config, Err(CliError::ValidationError(_))));
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
    fn test_override_with_watch_flag() {
        let cmd = Cli::build();
        let matches = cmd.get_matches_from(vec!["ssg", "--watch"]);
        let config = SsgConfig::from_matches(&matches).unwrap();
        assert!(!config.site_name.is_empty());
    }

    #[test]
    fn test_validate_empty_url() {
        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url(String::new())
            .build();
        assert!(config.is_ok());
    }

    // -----------------------------------------------------------------
    // SsgConfig::from_file -- valid TOML
    // -----------------------------------------------------------------

    #[test]
    fn test_config_from_file_valid_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("valid.toml");
        let toml_content = r#"
site_name = "TestSite"
content_dir = "./examples/content"
output_dir = "./examples/public"
template_dir = "./examples/templates"
base_url = "http://test.example.com"
site_title = "Test Title"
site_description = "A test site"
language = "en-GB"
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = SsgConfig::from_file(&config_path).unwrap();
        assert_eq!(config.site_name, "TestSite");
        assert_eq!(config.site_title, "Test Title");
        assert_eq!(config.base_url, "http://test.example.com");
    }
}
