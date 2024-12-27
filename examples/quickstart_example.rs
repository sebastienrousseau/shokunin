// examples/basic_site.rs
//! # Basic Site Generation Example
//!
//! This example demonstrates how to use the Shokunin Static Site Generator (SSG)
//! to create a basic static website. It showcases:
//!
//! - Basic configuration setup
//! - Directory structure validation
//! - Site generation process
//! - Error handling
//! - Progress logging

use anyhow::{Context, Result};
use dtt::datetime::DateTime;
use http_handle::Server;
use rlg::{log_format::LogFormat, log_level::LogLevel, macro_log};
use ssg::{cmd::ShokuninConfig, verify_and_copy_files, Paths};
use staticdatagen::compiler::service::compile;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

/// Represents the configuration for site generation
struct SiteGenerator {
    config: ShokuninConfig,
    paths: Paths,
    log_file: File,
}

impl SiteGenerator {
    /// Creates a new SiteGenerator instance with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `site_name` - Name of the site
    /// * `base_url` - Base URL for the site
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured SiteGenerator or an error
    fn new(site_name: &str, base_url: &str) -> Result<Self> {
        // Create log file
        let log_file = File::create("site_generation.log")
            .context("Failed to create log file")?;

        // Ensure directories exist before configuration validation
        let base_dir = PathBuf::from("examples");
        let content_dir = base_dir.join("content");
        let output_dir = base_dir.join("build");
        let template_dir = base_dir.join("templates");
        let site_dir = base_dir.join("public");

        fs::create_dir_all(&content_dir)
            .context("Failed to create content directory")?;
        fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;
        fs::create_dir_all(&template_dir)
            .context("Failed to create template directory")?;

        // Convert directories to absolute paths
        let content_dir = fs::canonicalize(content_dir)?;
        let output_dir = fs::canonicalize(output_dir)?;
        let template_dir = fs::canonicalize(template_dir)?;
        let site_dir =
            fs::canonicalize(site_dir.clone()).unwrap_or(site_dir);

        // Create configuration
        let config = ShokuninConfig::builder()
            .site_name(site_name.to_string())
            .base_url(base_url.to_string())
            .content_dir(content_dir.clone())
            .output_dir(output_dir.clone())
            .template_dir(template_dir.clone())
            .site_title("Basic Shokunin Site".to_string())
            .site_description(
                "A basic static site built with Shokunin".to_string(),
            )
            .language("en-GB".to_string())
            .build()
            .context("Failed to build configuration")?;

        // Initialize paths
        let paths = Paths {
            content: content_dir,
            build: output_dir,
            site: site_dir,
            template: template_dir,
        };

        Ok(Self {
            config,
            paths,
            log_file,
        })
    }

    /// Ensures all required directories exist and are accessible
    fn prepare_directories(&self) -> Result<()> {
        for (name, path) in [
            ("content", &self.config.content_dir),
            ("build", &self.config.output_dir),
            ("site", &self.paths.site),
            ("template", &self.config.template_dir),
        ] {
            fs::create_dir_all(path).with_context(|| {
                format!("Failed to create {} directory", name)
            })?;
            self.log_message(
                &format!(
                    "Ensured {} directory at: {}",
                    name,
                    path.display()
                ),
                LogLevel::INFO,
            )?;
        }
        Ok(())
    }

    /// Logs a message with timestamp to the log file
    fn log_message(
        &self,
        message: &str,
        level: LogLevel,
    ) -> Result<()> {
        let date = DateTime::new();
        let log_entry = macro_log!(
            &self.config.site_name,
            &date.to_string(),
            &level,
            "process",
            message,
            &LogFormat::CLF
        );

        writeln!(&self.log_file, "{}", log_entry)
            .context("Failed to write to log file")?;

        println!("{}", message);
        Ok(())
    }

    /// Generates the static site
    /// Generates the static site
    fn generate(&mut self) -> Result<()> {
        self.log_message(
            &format!(
                "Starting generation for site: {}",
                self.config.site_name
            ),
            LogLevel::INFO,
        )?;

        // Prepare directories - this ensures they exist but doesn't delete them
        self.prepare_directories()?;

        // Compile the site
        self.log_message("Compiling site...", LogLevel::INFO)?;
        compile(
            &self.config.output_dir,
            &self.config.content_dir,
            &self.paths.site,
            &self.config.template_dir,
        )
        .context("Failed to compile site")?;

        self.log_message("Site compilation completed", LogLevel::INFO)?;

        // First ensure the build directory exists
        if !self.config.output_dir.exists() {
            fs::create_dir_all(&self.config.output_dir)
                .context("Failed to create build directory")?;

            self.log_message(
                &format!(
                    "Created build directory at: {}",
                    self.config.output_dir.display()
                ),
                LogLevel::INFO,
            )?;
        }

        // Copy static files
        self.log_message("Copying static files...", LogLevel::INFO)?;
        verify_and_copy_files(
            &self.config.output_dir,
            &self.paths.site,
        )
        .context("Failed to copy static files")?;

        self.log_message(
            &format!(
                "Site generated successfully at: {}",
                self.paths.site.display()
            ),
            LogLevel::INFO,
        )?;

        Ok(())
    }

    /// Starts a development server to preview the generated site
    fn serve(&self) -> Result<()> {
        self.log_message(
            "Starting development server at http://127.0.0.1:3000",
            LogLevel::INFO,
        )?;

        // Get the site directory as a string for the server
        let example_root: String = self
            .paths
            .site
            .to_str()
            .context("Failed to convert site path to string")?
            .to_string();

        // Create a new server with an address and document root
        let server =
            Server::new("127.0.0.1:3000", example_root.as_str());

        // Start the server
        let _ = server.start();

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut generator =
        SiteGenerator::new("basic-site", "http://127.0.0.1:3000")?;

    // Generate the site
    generator.generate()?;

    // Serve the site (this will block until the server is stopped)
    generator.serve()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_site_generator_creation() -> Result<()> {
        let generator =
            SiteGenerator::new("test-site", "127.0.0.1:3000")?;
        assert_eq!(generator.config.site_name, "test-site");
        assert_eq!(generator.config.base_url, "127.0.0.1:3000");
        Ok(())
    }

    #[test]
    fn test_directory_preparation() -> Result<()> {
        let generator =
            SiteGenerator::new("test-site", "127.0.0.1:3000")?;
        generator.prepare_directories()?;

        assert!(generator.config.content_dir.exists());
        assert!(generator.config.output_dir.exists());
        assert!(generator.config.template_dir.exists());
        assert!(generator.paths.site.exists());

        Ok(())
    }
}
