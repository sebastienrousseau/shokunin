// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Docs Example — Generic developer-tool documentation template
//!
//! ## What this example is
//!
//! A neutral, opinionated **documentation template** you can clone and
//! adapt for any developer tool, library, or API. The placeholder content
//! is "Polaris" — a fictional CLI/API tool — so nothing in the example
//! commits you to a specific domain.
//!
//! ## What's in the template
//!
//! - **Welcome / overview** — what the tool is, what's in the docs
//! - **Getting started** — install + first-request walkthrough
//! - **Configuration reference** — every option in a scannable table
//! - **API reference** — endpoint table + request/response examples
//! - **Release notes** — chronological changelog page
//! - **Browse by topic** — auto-aggregated tag index
//! - **Contact / support** — routing for unanswered questions
//! - **Privacy** — short, honest data-handling statement
//!
//! ## What it also demonstrates
//!
//! - **Schema-validated frontmatter** — enforces required fields via
//!   `content.schema.toml`, so invalid docs break the build, never ship
//! - **Full plugin pipeline** — the same 16 plugins as `quickstart`
//!
//! ## What to edit
//!
//! - `examples/docs/content/*.md` — replace the placeholder Polaris copy
//! - `examples/docs_example.rs` — change `site_name`, `site_title`,
//!   `site_description` in the `SsgConfig::builder()` call
//! - Frontmatter `name` / `subtitle` on `index.md` for the hero copy
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example docs
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## How this differs from `blog`
//!
//! `blog` is content-led (chronological posts with tags). `docs` is
//! reference-led (stable pages organised by topic, with strict
//! frontmatter validation so docs stay scannable as the team grows).

use anyhow::{Context, Result};
use http_handle::Server;
use ssg::{
    cmd::SsgConfig,
    content::validate_with_schema,
    execute_build_pipeline,
    plugin::{PluginContext, PluginManager},
    Paths,
};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    // ---------------------------------------------------------------
    // 1. Set up directories
    // ---------------------------------------------------------------
    let base_dir = PathBuf::from("examples").join("docs");
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
    // 2. Validate content schemas before building
    //
    // The schema lives at examples/docs/content.schema.toml — *outside*
    // content/ — because staticdatagen::compile reads every file in
    // content_dir and would fail to parse the schema TOML as Markdown.
    // ---------------------------------------------------------------
    let schema_path = base_dir.join("content.schema.toml");
    println!("Validating content schemas...");
    match validate_with_schema(&content_dir, &schema_path) {
        Ok(()) => println!("Schema validation: all pages valid, 0 errors"),
        Err(e) => {
            eprintln!("Schema validation failed: {e}");
            return Err(e);
        }
    }

    // ---------------------------------------------------------------
    // 3. Build configuration
    // ---------------------------------------------------------------
    let config = SsgConfig::builder()
        .site_name("polaris-docs".to_string())
        .base_url("http://127.0.0.1:3000".to_string())
        .content_dir(content_dir.clone())
        .output_dir(output_dir.clone())
        .template_dir(template_dir.clone())
        .site_title("Polaris Documentation".to_string())
        .site_description(
            "Documentation template for any developer tool, library, or API"
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
    // 4. Register the full plugin pipeline
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
    // 5. Build the site
    // ---------------------------------------------------------------
    println!("\nBuilding documentation portal...");
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
    // 6. Print build summary
    // ---------------------------------------------------------------
    let page_count = fs::read_dir(&site_dir).map_or(0, |entries| {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
            .count()
    });

    println!("\n=== Build Summary ===");
    println!("  Pages built:   {page_count}");

    // Search index
    let search_path = site_dir.join("search-index.json");
    if search_path.exists() {
        let size = fs::metadata(&search_path)?.len();
        println!("  Search index:  {size} bytes");
    } else {
        println!("  Search index:  not found");
    }

    // Feeds
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
    println!("====================\n");

    // ---------------------------------------------------------------
    // 7. Serve the site
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
