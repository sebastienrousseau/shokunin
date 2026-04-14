// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! # Docs Example — Documentation portal with content schema validation
//!
//! ## What this example demonstrates
//!
//! - **Schema-validated frontmatter** — enforces required `title`, `description`, `category` fields
//! - **TOML-driven schema** — `content.schema.toml` declares the enum of valid categories
//! - **Build-time failures** — invalid docs break the build, never ship
//!
//! ## When to use this pattern
//!
//! Use this example for documentation portals where every page must carry
//! consistent metadata for navigation, search, and category listings.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example docs_example
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `blog` which accepts any frontmatter, this example fails the build
//! whenever a document is missing required fields or uses an unknown category.

use anyhow::{Context, Result};
use http_handle::Server;
use ssg::{
    cmd::SsgConfig,
    content::validate_only,
    execute_build_pipeline,
    plugin::{PluginContext, PluginManager},
    Paths,
};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    // ---------------------------------------------------------------
    // 1. Set up directories
    // ---------------------------------------------------------------
    let base_dir = PathBuf::from("examples");
    let content_dir = base_dir.join("docs").join("content");
    let output_dir = base_dir.join("build");
    let template_dir = base_dir.join("templates");
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
    // ---------------------------------------------------------------
    println!("Validating content schemas...");
    match validate_only(&content_dir) {
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
        .site_name("ssg-docs".to_string())
        .base_url("http://127.0.0.1:3000".to_string())
        .content_dir(content_dir.clone())
        .output_dir(output_dir.clone())
        .template_dir(template_dir.clone())
        .site_title("SSG Documentation".to_string())
        .site_description(
            "Official documentation for the Static Site Generator".to_string(),
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
    #[cfg(feature = "tera-templates")]
    plugins.register(ssg::tera_plugin::TeraPlugin::from_template_dir(
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

    // ---------------------------------------------------------------
    // 6. Print build summary
    // ---------------------------------------------------------------
    let page_count = fs::read_dir(&site_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "html")
                })
                .count()
        })
        .unwrap_or(0);

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

    let server = Server::new("127.0.0.1:3000", doc_root.as_str());

    println!("Server running at http://127.0.0.1:3000");
    println!("Document root: {doc_root}");
    println!("Press Ctrl+C to stop.");

    server.start().context("Failed to start dev server")?;

    Ok(())
}
