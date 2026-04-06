#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Multilingual Static Site Generator Example
//!
//! This example demonstrates how to generate a multilingual static site
//! with a language selector at the root of the `public` directory.

use anyhow::Context;
use anyhow::Result;
use http_handle::Server;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::search::SearchPlugin;
use ssg::seo::SeoPlugin;
use staticdatagen::compiler::service::compile;
use std::fs::{self, write};
use std::path::Path;

fn main() -> Result<()> {
    // Define supported languages
    let languages = vec!["en", "fr"];

    // Root directory for public files
    let public_root = Path::new("./examples/public");
    fs::create_dir_all(public_root)?;

    // Generate sites for all languages
    for lang in &languages {
        println!("Processing language: {}", lang);

        // Define paths specific to the language
        let build_dir = Path::new("./examples/build").join(lang);
        let site_dir = public_root.join(lang);
        let content_dir = Path::new("./examples/content").join(lang);
        let template_dir = Path::new("./examples/templates").join(lang);

        // Call the compile function to generate the website
        println!("    🔍 Compiling content for language: {lang}...");
        match compile(&build_dir, &content_dir, &site_dir, &template_dir) {
            Ok(()) => println!(
                "    ✅ Successfully compiled static site for language: {lang}"
            ),
            Err(e) => {
                println!("    ❌ Error compiling site for {lang}: {e:?}");
                return Err(e);
            }
        }

        // Run plugins (SEO + Search) for this language
        let mut plugins = PluginManager::new();
        plugins.register(SeoPlugin);
        plugins.register(SearchPlugin);
        let ctx = PluginContext::new(
            &content_dir,
            &build_dir,
            &site_dir,
            &template_dir,
        );
        plugins.run_after_compile(&ctx)?;
        println!("    🔌 Plugins complete for {lang}");
    }

    // Copy shared assets (manifest.json, rss.xml) to root so
    // absolute paths emitted by staticdatagen resolve correctly.
    for asset in &[
        "manifest.json",
        "rss.xml",
        "robots.txt",
        "sitemap.xml",
        "search-index.json",
    ] {
        // Prefer the English version as the root copy
        let src = public_root.join("en").join(asset);
        if src.exists() {
            let dst = public_root.join(asset);
            if !dst.exists() {
                let _ = fs::copy(&src, &dst)?;
            }
        }
    }

    // Generate the root `index.html` with language links
    generate_language_selector(public_root, &languages)?;

    // Serve the root public directory
    let server = Server::new("127.0.0.1:3000", public_root.to_str().unwrap());
    println!("Serving site at http://127.0.0.1:3000");
    let _ = server.start();

    Ok(())
}

/// Generates a root `index.html` file using the `templates/selector.html` template
fn generate_language_selector(
    public_root: &Path,
    languages: &[&str],
) -> Result<()> {
    // Read the selector.html template
    let template_path = Path::new("./examples/templates/selector.html");
    let template = fs::read_to_string(template_path)
        .context("Failed to read selector.html template")?;

    // Replace the placeholder with the language links
    let mut language_links = String::new();
    for lang in languages {
        let link = format!(
            "<li><a href=\"./{}/\">{}</a></li>\n",
            lang,
            lang.to_uppercase()
        );
        language_links.push_str(&link);
    }
    let output_html = template.replace("{{LANGUAGE_LINKS}}", &language_links);

    // Write the generated HTML to `public/index.html`
    let index_path = public_root.join("index.html");
    write(index_path, output_html)
        .context("Failed to write language selector index.html")?;
    println!(
        "    ✅ Generated language selector at root index.html using template"
    );

    Ok(())
}
