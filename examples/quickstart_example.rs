// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Quickstart Example — Heron Coffee small-business starter
//!
//! ## What this example is
//!
//! A complete, **clone-and-edit** small-business site styled as a fictional
//! London coffee roastery (*Heron Coffee*). Real-feeling copy, three
//! journal posts with tags, working contact / privacy / 404 / offline.
//! Wires the **full 16-plugin pipeline** so you get SEO, JSON-LD,
//! canonical URLs, sitemaps, RSS + Atom feeds, search, accessibility
//! report, and minification with no further configuration.
//!
//! ## What you get
//!
//! - **Polished homepage** — hero, this-week's-roasts, location, journal teaser
//! - **3 substantive journal posts** with their own frontmatter tags so the
//!   tags page auto-aggregates a real index
//! - **Curated `/posts/` listing** that links to each post with descriptions
//! - **Real `/contact/` page** with cafe address, wholesale, subscriptions
//! - **Generated artifacts**: sitemap.xml, news-sitemap.xml, rss.xml,
//!   atom.xml, manifest.json, robots.txt, search-index.json,
//!   accessibility-report.json
//!
//! ## What to edit to repurpose
//!
//! 1. `examples/quickstart/content/index.md` — homepage hero + sections
//! 2. `examples/quickstart/content/{post-name}.md` — individual posts
//! 3. `examples/quickstart/content/{contact,privacy}.md` — supporting pages
//! 4. `SiteGenerator::new()` in `examples/quickstart_example.rs` — site
//!    name, title, description
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example quickstart
//! ```
//!
//! Then open <http://127.0.0.1:3007>.
//!
//! ## How this differs from `basic`
//!
//! `basic` is a single-page studio template: one fixed page tree, the
//! minimum plugins needed for clean output. `quickstart` is the
//! **kitchen-sink starter** — every plugin you'd typically want from day
//! one is wired up, and the demo content shows the full pipeline working
//! against a real-feeling site.

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
    time::Instant,
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
    fn new(_site_name: &str, _base_url: &str) -> Result<Self> {
        // The example overrides the caller-supplied identifiers with its
        // own opinionated branding (Heron Coffee — a small London
        // roastery) so the output looks like a real site, not a generic
        // demo. Replace these strings to repurpose the starter.
        let site_name = "heron-coffee";
        let base_url = "http://127.0.0.1:3007";
        // Create log file
        let log_file = File::create("site_generation.log")
            .context("Failed to create log file")?;

        // Ensure directories exist before configuration validation
        let base_dir = PathBuf::from("examples").join("quickstart");
        let content_dir = base_dir.join("content");
        let output_dir = base_dir.join("build");
        let template_dir = PathBuf::from("examples").join("templates");
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
            .site_title(
                "Heron Coffee — single-origin roaster, Bermondsey".to_string(),
            )
            .site_description(
                "Heron Coffee — single-origin coffee roasted on Druid \
                 Street every Tuesday morning"
                    .to_string(),
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
        #[cfg(feature = "templates")]
        plugins.register(
            ssg::template_plugin::TemplatePlugin::from_template_dir(
                &self.config.template_dir,
            ),
        );
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
        #[cfg(feature = "image-optimization")]
        plugins.register(ssg::image_plugin::ImageOptimizationPlugin::default());
        plugins.register(ssg::ai::AiPlugin);
        plugins.register(ssg::taxonomy::TaxonomyPlugin);
        plugins.register(ssg::pagination::PaginationPlugin::default());
        plugins.register(ssg::drafts::DraftPlugin::new(true));
        plugins.register(ssg::assets::FingerprintPlugin);
        // CSP hardening (extract inline styles/scripts to external files with SRI)
        plugins.register(ssg::csp::CspPlugin);

        // Interactive islands (Web Components with lazy hydration)
        plugins.register(ssg::islands::IslandPlugin);

        plugins.register(ssg::plugins::MinifyPlugin);

        // Run the full pipeline: before_compile → compile → after_compile
        self.log_message("Compiling site with full plugin pipeline...")?;
        let start = Instant::now();
        execute_build_pipeline(
            &plugins,
            &ctx,
            &self.config.output_dir,
            &self.config.content_dir,
            &self.paths.site,
            &self.config.template_dir,
            false,
        )?;
        println!(
            "    📦 Streaming: {} MB memory budget available",
            ssg::streaming::DEFAULT_MEMORY_BUDGET_MB
        );
        let elapsed = start.elapsed();
        println!("    \u{26a1} Built in {elapsed:.0?}");

        // Show streaming I/O buffer size
        println!(
            "    \u{1f4e6} Streaming I/O buffer: {} KB",
            ssg::stream::STREAM_BUFFER_SIZE / 1024
        );

        self.log_message(&format!(
            "Site generated successfully at: {}",
            self.paths.site.display()
        ))?;

        Ok(())
    }

    /// Starts a development server to preview the generated site
    fn serve(&self) -> Result<()> {
        self.log_message(
            "Starting development server at http://127.0.0.1:3007",
        )?;

        // Get the site directory as a string for the server
        let example_root: String = self
            .paths
            .site
            .to_str()
            .context("Failed to convert site path to string")?
            .to_string();

        // Create a new server with an address and document root
        // Build the server with a Permissions-Policy header that opts the
        // page out of the Topics API. Suppresses the "Browsing Topics API
        // removed" Chrome console message in dev mode.
        let server = Server::builder()
            .address("127.0.0.1:3007")
            .document_root(example_root.as_str())
            .custom_header("Permissions-Policy", "browsing-topics=()")
            .build()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        println!("❯ Server is now running at http://127.0.0.1:3007");
        println!("  Document root: {example_root}");
        println!("  Press Ctrl+C to stop the server.");

        // Start the server (blocks until stopped)
        server.start().context("Failed to start dev server")?;

        Ok(())
    }
}

/// Injects a tiny `<style>` block before `</head>` that hides the language
/// dropdown trigger and its menu. Idempotent. Used by single-locale examples
/// to suppress the language switcher the shared template hardcodes for the
/// multilingual demo.
fn hide_language_icon(site_dir: &std::path::Path) -> Result<()> {
    const MARKER: &str = "/* ssg-single-locale: hide lang */";
    const STYLE: &str =
        "<style>/* ssg-single-locale: hide lang */.lang-btn,.lang-dropdown,.mobile-lang{display:none!important}</style>";

    fn walk(dir: &std::path::Path, marker: &str, style: &str) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, marker, style)?;
            } else if path.extension().is_some_and(|e| e == "html") {
                let html = fs::read_to_string(&path)?;
                if html.contains(marker) {
                    continue;
                }
                if let Some(pos) = html.find("</head>") {
                    let new_html =
                        format!("{}{}{}", &html[..pos], style, &html[pos..]);
                    fs::write(&path, new_html)?;
                }
            }
        }
        Ok(())
    }
    walk(site_dir, MARKER, STYLE)
}

fn main() -> Result<()> {
    // The site_name and base_url passed here are overridden inside
    // SiteGenerator::new() for the demo branding. Replace them and the
    // hard-coded values inside `new()` to repurpose this starter.
    let generator =
        SiteGenerator::new("heron-coffee", "http://127.0.0.1:3007")?;

    // Generate the site
    generator.generate()?;

    // Single-locale build: hide the language dropdown the templates
    // hardcode for the multilingual example.
    hide_language_icon(&generator.paths.site)?;

    // File watching — classify changes for selective reload
    println!(
        "    \u{1f441} Watch: .css → {:?}, .md → {:?}",
        ssg::watch::classify_change(std::path::Path::new("x.css")),
        ssg::watch::classify_change(std::path::Path::new("x.md"))
    );

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
            SiteGenerator::new("test-site", "http://127.0.0.1:3007")?;
        assert_eq!(generator.config.site_name, "test-site");
        assert_eq!(generator.config.base_url, "http://127.0.0.1:3007");
        Ok(())
    }

    #[test]
    fn test_directory_preparation() -> Result<()> {
        let generator =
            SiteGenerator::new("test-site", "http://127.0.0.1:3007")?;
        generator.prepare_directories()?;

        assert!(generator.config.content_dir.exists());
        assert!(generator.config.output_dir.exists());
        assert!(generator.config.template_dir.exists());
        assert!(generator.paths.site.exists());

        Ok(())
    }
}
