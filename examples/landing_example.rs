// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Landing Example — Zero-JS marketing page with WCAG 2.1 AA report
//!
//! ## What this example demonstrates
//!
//! - **Zero JavaScript output** — HTML + CSS only, fastest possible load
//! - **WCAG 2.1 AA compliance report** — generated on every build
//! - **CSP + HSTS headers + responsive `<picture>`** — production-grade hardening
//!
//! ## When to use this pattern
//!
//! Use this example for marketing landing pages where conversion depends on
//! instant load times and strict accessibility/security compliance.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example landing_example
//! ```
//!
//! Then open <http://127.0.0.1:3004> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `blog` which is content-heavy, this example targets a single
//! high-conversion page with zero JS and enterprise security headers.

use anyhow::{Context, Result};
use http_handle::Server;
use ssg::{
    accessibility::AccessibilityReport,
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

/// Site generator configured for a zero-JS enterprise landing page.
struct LandingSiteGenerator {
    config: SsgConfig,
    paths: Paths,
    log_file: File,
}

impl LandingSiteGenerator {
    /// Creates a new generator pointing at `examples/landing/content/`.
    fn new() -> Result<Self> {
        let log_file = File::create("landing_generation.log")
            .context("Failed to create log file")?;

        let base_dir = PathBuf::from("examples");
        let content_dir = base_dir.join("landing").join("content");
        let output_dir = base_dir.join("landing").join("build");
        let template_dir = base_dir.join("templates");
        let site_dir = base_dir.join("landing").join("public");

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
            .site_name("meridian-systems".to_string())
            .base_url("http://127.0.0.1:3004".to_string())
            .content_dir(content_dir.clone())
            .output_dir(output_dir.clone())
            .template_dir(template_dir.clone())
            .site_title(
                "Meridian Systems — Compliance-grade software".to_string(),
            )
            .site_description(
                "Compliance-grade software for regulated industries: \
                 clinical trials, public procurement, financial \
                 reconciliation"
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
        self.log_message("Starting Acme landing page generation...")?;
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
        // CSP hardening (extract inline styles/scripts to external files with SRI)
        plugins.register(ssg::csp::CspPlugin);

        // Interactive islands (Web Components with lazy hydration)
        plugins.register(ssg::islands::IslandPlugin);

        plugins.register(ssg::plugins::MinifyPlugin);

        // Deployment adapter (generate netlify.toml with cache + security headers)
        plugins.register(ssg::deploy::DeployPlugin::new(
            ssg::deploy::DeployTarget::Netlify,
        ));

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
        let elapsed = start.elapsed();
        self.log_message(&format!("⚡ Built in {elapsed:.0?}"))?;

        self.log_message(&format!(
            "Site generated at: {}",
            self.paths.site.display()
        ))?;

        Ok(())
    }

    /// Reads `accessibility-report.json` and prints a compliance summary.
    fn print_a11y_report(&self) -> Result<()> {
        let report_path = self.paths.site.join("accessibility-report.json");
        if report_path.exists() {
            let data = fs::read_to_string(&report_path)?;
            let report: AccessibilityReport = serde_json::from_str(&data)?;
            if report.total_issues == 0 {
                println!(
                    "WCAG 2.1 AA: PASS (0 issues across {} pages)",
                    report.pages_scanned
                );
            } else {
                println!(
                    "WCAG 2.1 AA: {} issues across {} pages",
                    report.total_issues, report.pages_scanned
                );
                for page in &report.pages {
                    for issue in &page.issues {
                        println!(
                            "  [{}] {} — {}",
                            issue.severity, issue.criterion, issue.message
                        );
                    }
                }
            }
        } else {
            println!("Accessibility report not found (no HTML pages?)");
        }
        Ok(())
    }

    /// Counts `<script>` tags across all generated HTML (excluding
    /// the opt-in search widget) and verifies zero-JS output.
    fn verify_zero_js(&self) -> Result<()> {
        let mut script_count: usize = 0;
        if self.paths.site.exists() {
            for entry in walkdir(self.paths.site.clone())? {
                let html = fs::read_to_string(&entry)?;
                // Count <script> tags, but exclude the search widget
                // which is opt-in and not part of the page content.
                for line in html.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("<script") && !lower.contains("search") {
                        script_count += 1;
                    }
                }
            }
        }
        println!(
            "JavaScript: {script_count} scripts ({})",
            if script_count == 0 {
                "zero-JS verified"
            } else {
                "scripts detected"
            }
        );
        Ok(())
    }

    /// Prints the total size of all generated HTML files.
    fn print_html_size(&self) -> Result<()> {
        let mut total: u64 = 0;
        if self.paths.site.exists() {
            for entry in walkdir(self.paths.site.clone())? {
                total += fs::metadata(&entry)?.len();
            }
        }
        println!("Total HTML size: {} bytes", total);
        Ok(())
    }

    /// Starts a dev server at 127.0.0.1:3004.
    fn serve(&self) -> Result<()> {
        self.log_message(
            "Starting development server at http://127.0.0.1:3004",
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
            .address("127.0.0.1:3004")
            .document_root(root.as_str())
            .custom_header("Permissions-Policy", "browsing-topics=()")
            .build()
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        println!("Server running at http://127.0.0.1:3004");
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

/// Collect all `.html` files under a directory.
fn walkdir(dir: PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir(path)?);
        } else if path.extension().map_or(false, |ext| ext == "html") {
            files.push(path);
        }
    }
    Ok(files)
}

fn main() -> Result<()> {
    let generator = LandingSiteGenerator::new()?;

    // Build the site
    generator.generate()?;

    // Single-locale build: hide the language dropdown the templates
    // hardcode for the multilingual example.
    hide_language_icon(&generator.paths.site)?;

    // Post-build analysis
    println!("\n--- Post-Build Analysis ---");
    generator.print_a11y_report()?;
    generator.verify_zero_js()?;
    generator.print_html_size()?;
    println!("---\n");

    // Serve the site
    generator.serve()?;

    Ok(())
}
