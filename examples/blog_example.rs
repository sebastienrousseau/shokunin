// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Blog Example — EAA-compliant accessibility-first blog
//!
//! ## What this example demonstrates
//!
//! - **Dual RSS + Atom feeds** — broad reader compatibility from one build
//! - **JSON-LD Person entities** — structured author metadata for search engines
//! - **On-build accessibility report** — European Accessibility Act (EAA) conformance check
//!
//! ## When to use this pattern
//!
//! Use this example when you need a blog that must meet EU accessibility
//! regulations and wants first-class feed support for subscribers.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example blog_example
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `quickstart` which is a generic starter, this example bakes in
//! EAA compliance, responsive images, and dual-feed publishing for blogs.

use anyhow::{Context, Result};
use http_handle::Server;
use ssg::{
    accessibility::AccessibilityReport,
    cmd::SsgConfig,
    execute_build_pipeline,
    plugin::{PluginContext, PluginManager},
    Paths,
};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    // ---------------------------------------------------------------
    // 1. Set up directories
    // ---------------------------------------------------------------
    let base_dir = PathBuf::from("examples").join("blog");
    let content_dir = base_dir.join("content");
    let output_dir = base_dir.join("build");
    let template_dir = PathBuf::from("examples").join("templates");
    let site_dir = base_dir.join("public");

    fs::create_dir_all(&content_dir)?;
    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(&template_dir)?;
    fs::create_dir_all(&site_dir)?;

    let content_dir = fs::canonicalize(&content_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    let template_dir = fs::canonicalize(&template_dir)?;
    let site_dir = fs::canonicalize(&site_dir)?;

    // ---------------------------------------------------------------
    // 2. Build configuration
    // ---------------------------------------------------------------
    let config = SsgConfig::builder()
        .site_name("threshold".to_string())
        .base_url("http://127.0.0.1:3000".to_string())
        .content_dir(content_dir.clone())
        .output_dir(output_dir.clone())
        .template_dir(template_dir.clone())
        .site_title("Threshold — Accessibility & inclusive design".to_string())
        .site_description(
            "An accessibility journal: WCAG, EAA, and inclusive design \
             writing for product teams"
                .to_string(),
        )
        .language("en-GB".to_string())
        .build()
        .context("Failed to build configuration")?;

    let paths = Paths {
        content: content_dir.clone(),
        build: output_dir.clone(),
        site: site_dir.clone(),
        template: template_dir.clone(),
    };

    // ---------------------------------------------------------------
    // 3. Register the full plugin pipeline including accessibility
    // ---------------------------------------------------------------
    let ctx = PluginContext::with_config(
        &config.content_dir,
        &config.output_dir,
        &paths.site,
        &config.template_dir,
        config.clone(),
    );

    let mut plugins = PluginManager::new();
    plugins.register(ssg::shortcodes::ShortcodePlugin);
    #[cfg(feature = "templates")]
    plugins.register(ssg::template_plugin::TemplatePlugin::from_template_dir(
        &config.template_dir,
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
        &config.base_url,
        &config.site_name,
    ));
    plugins.register(ssg::seo::CanonicalPlugin::new(config.base_url.clone()));
    plugins.register(ssg::seo::RobotsPlugin::new(config.base_url.clone()));
    plugins.register(ssg::search::SearchPlugin);
    plugins.register(ssg::accessibility::AccessibilityPlugin);
    plugins.register(ssg::plugins::MinifyPlugin);

    // ---------------------------------------------------------------
    // 4. Build the site
    // ---------------------------------------------------------------
    println!("Building accessibility-first blog...");
    execute_build_pipeline(
        &plugins,
        &ctx,
        &config.output_dir,
        &config.content_dir,
        &paths.site,
        &config.template_dir,
        false,
    )?;
    println!("Build complete.");

    // Single-locale build: hide the language dropdown the templates
    // hardcode for the multilingual example.
    hide_language_icon(&site_dir)?;

    // ---------------------------------------------------------------
    // 5. Print build summary with accessibility report
    // ---------------------------------------------------------------
    // Count generated pages
    let page_count = fs::read_dir(&site_dir).map_or(0, |entries| {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
            .count()
    });

    println!("\n=== Build Summary ===");
    println!("  Pages built:   {page_count}");

    // Read and display accessibility report
    let report_path = site_dir.join("accessibility-report.json");
    if report_path.exists() {
        let report_json = fs::read_to_string(&report_path)?;
        let report: AccessibilityReport = serde_json::from_str(&report_json)?;
        println!(
            "  A11y scanned:  {} page(s), {} issue(s)",
            report.pages_scanned, report.total_issues
        );
        if report.total_issues == 0 {
            println!("  A11y status:   PASS (EAA-compliant)");
        } else {
            println!("  A11y status:   {} issue(s) found", report.total_issues);
            for page in &report.pages {
                for issue in &page.issues {
                    println!(
                        "    [{}/{}] {} — {}",
                        issue.severity,
                        issue.criterion,
                        page.path,
                        issue.message
                    );
                }
            }
        }
    } else {
        println!("  A11y status:   report not generated");
    }

    // Check feeds
    let rss_exists = site_dir.join("rss.xml").exists();
    let atom_exists = site_dir.join("atom.xml").exists();
    println!(
        "  RSS feed:      {}",
        if rss_exists { "rss.xml" } else { "not found" }
    );
    println!(
        "  Atom feed:     {}",
        if atom_exists { "atom.xml" } else { "not found" }
    );

    // Check search index
    let search_path = site_dir.join("search-index.json");
    if search_path.exists() {
        let size = fs::metadata(&search_path)?.len();
        println!("  Search index:  {size} bytes");
    } else {
        println!("  Search index:  not found");
    }
    println!("====================\n");

    // ---------------------------------------------------------------
    // 6. Serve the site
    // ---------------------------------------------------------------
    let doc_root = site_dir
        .to_str()
        .context("Failed to convert site path to string")?
        .to_string();

    // Build the server with a Permissions-Policy header that opts the
    // page out of the Topics API. Suppresses the "Browsing Topics API
    // removed" Chrome console message in dev mode.
    let server = Server::builder()
        .address("127.0.0.1:3000")
        .document_root(doc_root.as_str())
        .custom_header("Permissions-Policy", "browsing-topics=()")
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!("Server running at http://127.0.0.1:3000");
    println!("Document root: {doc_root}");
    println!("Press Ctrl+C to stop.");

    server.start().context("Failed to start dev server")?;

    Ok(())
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
