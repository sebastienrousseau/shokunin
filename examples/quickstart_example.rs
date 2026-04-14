// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Quickstart Example — Full plugin pipeline, production-ready template
//!
//! ## What this example demonstrates
//!
//! - **SEO + JSON-LD + canonical** — search-engine-ready metadata out of the box
//! - **Search index + accessibility report** — client-side search and WCAG checks
//! - **HTML/CSS/JS minification** — optimised output assets on every build
//!
//! ## When to use this pattern
//!
//! Use this example as the starting template for most real-world projects —
//! it wires together the plugins you typically want enabled from day one.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example quickstart_example
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `basic` which skips plugins entirely, this example composes the
//! SEO, search, accessibility, and minification plugins into a single pipeline.

use anyhow::{Context, Result};
use http_handle::Server;
use ssg::{
    cmd::SsgConfig,
    execute_build_pipeline,
    plugin::{PluginContext, PluginManager},
    Paths,
};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

/// Represents the configuration for site generation
struct SiteGenerator {
    config: SsgConfig,
    paths: Paths,
    log_file: File,
}

impl SiteGenerator {
    /// Creates a new `SiteGenerator` instance with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `site_name` - Name of the site
    /// * `base_url` - Base URL for the site
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured `SiteGenerator` or an error
    fn new(site_name: &str, base_url: &str) -> Result<Self> {
        // Create log file
        let log_file = File::create("site_generation.log")
            .context("Failed to create log file")?;

        // Ensure directories exist before configuration validation
        let base_dir = PathBuf::from("examples");
        let content_dir = base_dir.join("content").join("en");
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
        let site_dir = fs::canonicalize(site_dir.clone()).unwrap_or(site_dir);

        // Create configuration
        let config = SsgConfig::builder()
            .site_name(site_name.to_string())
            .base_url(base_url.to_string())
            .content_dir(content_dir.clone())
            .output_dir(output_dir.clone())
            .template_dir(template_dir.clone())
            .site_title("Basic SSG Site".to_string())
            .site_description("A basic static site built with SSG".to_string())
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
                format!("Failed to create {name} directory")
            })?;
            self.log_message(&format!(
                "Ensured {} directory at: {}",
                name,
                path.display()
            ))?;
        }
        Ok(())
    }

    /// Logs a message with timestamp to the log file
    fn log_message(&self, message: &str) -> Result<()> {
        let date = ssg::now_iso();
        writeln!(&self.log_file, "[{date}] INFO process: {message}")
            .context("Failed to write to log file")?;

        println!("{message}");
        Ok(())
    }

    /// Generates the static site using the full plugin pipeline.
    fn generate(&self) -> Result<()> {
        self.log_message(&format!(
            "Starting generation for site: {}",
            self.config.site_name
        ))?;

        // Prepare directories
        self.prepare_directories()?;

        // Build plugin context with config for SEO/canonical/search
        let ctx = PluginContext::with_config(
            &self.config.content_dir,
            &self.config.output_dir,
            &self.paths.site,
            &self.config.template_dir,
            self.config.clone(),
        );

        // Register the default plugin pipeline
        let mut plugins = PluginManager::new();
        plugins.register(ssg::shortcodes::ShortcodePlugin);
        #[cfg(feature = "tera-templates")]
        plugins.register(ssg::tera_plugin::TeraPlugin::from_template_dir(
            &self.config.template_dir,
        ));
        plugins.register(ssg::postprocess::SitemapFixPlugin);
        plugins.register(ssg::postprocess::NewsSitemapFixPlugin);
        plugins.register(ssg::postprocess::RssAggregatePlugin);
        plugins.register(ssg::postprocess::AtomFeedPlugin);
        plugins.register(ssg::postprocess::ManifestFixPlugin);
        plugins.register(ssg::postprocess::HtmlFixPlugin);
        plugins.register(ssg::highlight::HighlightPlugin::default());
        plugins.register(ssg::seo::SeoPlugin);
        plugins.register(ssg::seo::JsonLdPlugin::from_site(
            &self.config.base_url,
            &self.config.site_name,
        ));
        plugins.register(ssg::seo::CanonicalPlugin::new(
            self.config.base_url.clone(),
        ));
        plugins.register(ssg::seo::RobotsPlugin::new(
            self.config.base_url.clone(),
        ));
        plugins.register(ssg::search::SearchPlugin);
        plugins.register(ssg::accessibility::AccessibilityPlugin);
        plugins.register(ssg::plugins::MinifyPlugin);

        // Run the full pipeline: before_compile → compile → after_compile
        self.log_message("Compiling site with full plugin pipeline...")?;
        execute_build_pipeline(
            &plugins,
            &ctx,
            &self.config.output_dir,
            &self.config.content_dir,
            &self.paths.site,
            &self.config.template_dir,
            false,
        )?;

        self.log_message(&format!(
            "Site generated successfully at: {}",
            self.paths.site.display()
        ))?;

        Ok(())
    }

    /// Starts a development server to preview the generated site
    fn serve(&self) -> Result<()> {
        self.log_message(
            "Starting development server at http://127.0.0.1:3000",
        )?;

        // Get the site directory as a string for the server
        let example_root: String = self
            .paths
            .site
            .to_str()
            .context("Failed to convert site path to string")?
            .to_string();

        // Create a new server with an address and document root
        let server = Server::new("127.0.0.1:3000", example_root.as_str());

        println!("❯ Server is now running at http://127.0.0.1:3000");
        println!("  Document root: {example_root}");
        println!("  Press Ctrl+C to stop the server.");

        // Start the server (blocks until stopped)
        server.start().context("Failed to start dev server")?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let generator = SiteGenerator::new("basic-site", "http://127.0.0.1:3000")?;

    // Generate the site
    generator.generate()?;

    // Serve the site (this will block until the server is stopped)
    generator.serve()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_generator_creation() -> Result<()> {
        let generator =
            SiteGenerator::new("test-site", "http://127.0.0.1:3000")?;
        assert_eq!(generator.config.site_name, "test-site");
        assert_eq!(generator.config.base_url, "http://127.0.0.1:3000");
        Ok(())
    }

    #[test]
    fn test_directory_preparation() -> Result<()> {
        let generator =
            SiteGenerator::new("test-site", "http://127.0.0.1:3000")?;
        generator.prepare_directories()?;

        assert!(generator.config.content_dir.exists());
        assert!(generator.config.output_dir.exists());
        assert!(generator.config.template_dir.exists());
        assert!(generator.paths.site.exists());

        Ok(())
    }
}
