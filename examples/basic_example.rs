#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Basic Example — Minimal single-locale site with search
//!
//! ## What this example demonstrates
//!
//! - **Direct compile** — invokes `staticdatagen::compiler::service::compile`
//! - **Functional search** — registers `SearchPlugin` so `Ctrl+K` works
//! - **Single-locale layout** — strips the language dropdown for a clean UI
//!
//! ## When to use this pattern
//!
//! Use this example as the smallest functional starting point: one language,
//! one binary, one server. No SEO meta-injection, no JSON-LD, no a11y report.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example basic_example
//! ```
//!
//! Then open <http://127.0.0.1:3000> in your browser.
//!
//! ## What makes this different from other examples
//!
//! Unlike `quickstart` which wires the full pipeline (SEO + JSON-LD + a11y +
//! minify), `basic` registers only the search plugin and a CSS-injection step
//! that hides the language dropdown so the UI matches the single-locale intent.

use anyhow::Result;
use http_handle::Server;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::search::SearchPlugin;
use staticdatagen::compiler::service::compile;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let build_dir = Path::new("./examples/build");
    let site_dir = Path::new("./examples/public");
    let content_dir = Path::new("./examples/content/en");
    let template_dir = Path::new("./examples/templates");

    // 1. Compile content → HTML
    match compile(build_dir, content_dir, site_dir, template_dir) {
        Ok(()) => println!("    ✅ Successfully compiled static site"),
        Err(e) => {
            println!("    ❌ Error compiling site: {e:?}");
            return Err(e);
        }
    }

    // 2. Run only the SearchPlugin so the search UI actually works
    let mut plugins = PluginManager::new();
    plugins.register(SearchPlugin);
    let ctx =
        PluginContext::new(content_dir, build_dir, site_dir, template_dir);
    plugins.run_after_compile(&ctx)?;
    println!("    🔍 Search index generated");

    // 3. Hide the language dropdown — basic is single-locale, so the
    //    icon serves no purpose. Inject a small CSS rule into every page.
    hide_language_icon(site_dir)?;
    println!("    🌐 Language dropdown hidden (single-locale site)");

    // 4. Serve
    let example_root: String = site_dir.to_str().unwrap().to_string();
    let server = Server::new("127.0.0.1:3000", example_root.as_str());

    println!("\n❯ Server is now running at http://127.0.0.1:3000");
    println!("  Document root: {example_root}");
    println!("  Press Ctrl+C to stop the server.");

    server.start().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

/// Injects a tiny `<style>` block before `</head>` that hides the language
/// dropdown trigger and its menu. Idempotent.
fn hide_language_icon(site_dir: &Path) -> Result<()> {
    const MARKER: &str = "/* ssg-basic: hide lang */";
    const STYLE: &str =
        "<style>/* ssg-basic: hide lang */.lang-btn,.lang-dropdown,.mobile-lang{display:none!important}</style>";

    fn walk(dir: &Path, marker: &str, style: &str) -> Result<()> {
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
