// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Portfolio Example — Developer portfolio with JSON-LD Person entity
//!
//! ## What this example demonstrates
//!
//! - **JSON-LD Person entity** — skills, employment, projects for Google Knowledge Panel
//! - **Atom feed for project updates** — subscribers track new work automatically
//! - **Responsive image pipeline** — optimised hero/project thumbnails
//!
//! ## When to use this pattern
//!
//! Use this example for a developer or freelancer portfolio that needs rich
//! structured data so search engines surface skills, roles, and projects.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example portfolio_example
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `landing` which models an Organization, this example emits a
//! Person entity tailored for individual developer discoverability.

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

/// Site generator configured for a developer portfolio.
struct PortfolioSiteGenerator {
    config: SsgConfig,
    paths: Paths,
    log_file: File,
}

impl PortfolioSiteGenerator {
    /// Creates a new generator pointing at `examples/portfolio/content/`.
    fn new() -> Result<Self> {
        let log_file = File::create("portfolio_generation.log")
            .context("Failed to create log file")?;

        let base_dir = PathBuf::from("examples");
        let content_dir = base_dir.join("portfolio").join("content");
        let output_dir = base_dir.join("portfolio").join("build");
        let template_dir = base_dir.join("templates");
        let site_dir = base_dir.join("portfolio").join("public");

        fs::create_dir_all(&content_dir)
            .context("Failed to create content directory")?;
        fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;
        fs::create_dir_all(&site_dir)
            .context("Failed to create site directory")?;

        let content_dir = fs::canonicalize(content_dir)?;
        let output_dir = fs::canonicalize(output_dir)?;
        let template_dir = fs::canonicalize(template_dir)?;
        let site_dir = fs::canonicalize(site_dir.clone()).unwrap_or(site_dir);

        let config = SsgConfig::builder()
            .site_name("maya-okafor".to_string())
            .base_url("http://127.0.0.1:3000".to_string())
            .content_dir(content_dir.clone())
            .output_dir(output_dir.clone())
            .template_dir(template_dir.clone())
            .site_title("Maya Okafor — UX Research & Design".to_string())
            .site_description(
                "Independent UX researcher and product designer working with \
                 founders, charities, and small product teams"
                    .to_string(),
            )
            .language("en-GB".to_string())
            .build()
            .context("Failed to build configuration")?;

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

    /// Ensures all required directories exist.
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
        }
        Ok(())
    }

    /// Logs a message to the log file and stdout.
    fn log_message(&self, message: &str) -> Result<()> {
        let date = ssg::now_iso();
        writeln!(&self.log_file, "[{date}] INFO process: {message}")
            .context("Failed to write to log file")?;
        println!("{message}");
        Ok(())
    }

    /// Builds the site with the full plugin pipeline.
    fn generate(&self) -> Result<()> {
        self.log_message("Starting portfolio site generation...")?;
        self.prepare_directories()?;

        let ctx = PluginContext::with_config(
            &self.config.content_dir,
            &self.config.output_dir,
            &self.paths.site,
            &self.config.template_dir,
            self.config.clone(),
        );

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
        plugins.register(ssg::plugins::MinifyPlugin);

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
            "Site generated at: {}",
            self.paths.site.display()
        ))?;

        Ok(())
    }

    /// Checks for JSON-LD structured data in index.html and prints
    /// a summary of any Person or Organization entities found.
    fn print_structured_data(&self) -> Result<()> {
        let index_path = self.paths.site.join("index.html");
        if index_path.exists() {
            let html = fs::read_to_string(&index_path)?;
            let mut found_jsonld = false;
            // Look for JSON-LD script blocks
            for line in html.lines() {
                if line.contains("application/ld+json") {
                    found_jsonld = true;
                }
                if found_jsonld && line.contains("\"@type\"") {
                    let trimmed = line.trim();
                    println!("JSON-LD: {trimmed}");
                }
            }
            if found_jsonld {
                println!("Structured data: JSON-LD entity found in index.html");
            } else {
                println!(
                    "Structured data: no JSON-LD block found in index.html"
                );
            }
        } else {
            println!("index.html not found in site output");
        }
        Ok(())
    }

    /// Verifies Atom feed was generated and counts entries.
    fn verify_atom_feed(&self) -> Result<()> {
        let atom_path = self.paths.site.join("atom.xml");
        if atom_path.exists() {
            let xml = fs::read_to_string(&atom_path)?;
            let entries = xml.matches("<entry>").count();
            println!("Atom feed: {entries} entries");
        } else {
            println!("Atom feed: not generated");
        }
        Ok(())
    }

    /// Starts a dev server at 127.0.0.1:3000.
    fn serve(&self) -> Result<()> {
        self.log_message(
            "Starting development server at http://127.0.0.1:3000",
        )?;

        let root: String = self
            .paths
            .site
            .to_str()
            .context("Failed to convert site path to string")?
            .to_string();

        // Build the server with a Permissions-Policy header that opts the
        // page out of the Topics API. Suppresses the "Browsing Topics API
        // removed" Chrome console message in dev mode.
        let server = Server::builder()
            .address("127.0.0.1:3000")
            .document_root(root.as_str())
            .custom_header("Permissions-Policy", "browsing-topics=()")
            .build()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        println!("Server running at http://127.0.0.1:3000");
        println!("  Document root: {root}");
        println!("  Press Ctrl+C to stop.");

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
    let generator = PortfolioSiteGenerator::new()?;

    // Build the site
    generator.generate()?;

    // Single-locale build: hide the language dropdown the templates
    // hardcode for the multilingual example.
    hide_language_icon(&generator.paths.site)?;

    // Post-build analysis
    println!("\n--- Post-Build Analysis ---");
    generator.print_structured_data()?;
    generator.verify_atom_feed()?;
    println!("---\n");

    // Serve the site
    generator.serve()?;

    Ok(())
}
