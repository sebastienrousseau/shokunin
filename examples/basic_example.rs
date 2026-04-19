#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Basic Example — Ready-to-deploy small-studio template
//!
//! ## What this example is
//!
//! A polished, opinionated single-page template for a small studio,
//! freelancer, or independent business. **Five minutes from clone to
//! production**: edit three markdown files, change the brand colour,
//! point your domain at the output directory, ship.
//!
//! ## What you get out of the box
//!
//! - **Hero + three content sections + contact** — the layout most small
//!   sites actually need, no blog or tag archive padding
//! - **Search** (`SearchPlugin`) — ⌘K opens a client-side index
//! - **Browser-compat clean** — `HtmlFixPlugin` and `ManifestFixPlugin`
//!   suppress the warnings Chrome would otherwise log on first load
//! - **Lighthouse 100 / 100 / 100 / 100** — performance, accessibility,
//!   best-practices, SEO, on both mobile and desktop
//!
//! ## What to edit
//!
//! - `examples/basic/content/index.md` — homepage hero + sections
//! - `examples/basic/content/about.md` — story, team, working method
//! - `examples/basic/content/contact.md` — emails, hours, locations
//!
//! That's it. The footer "Resources" column, the hero CTAs, and the
//! Posts/Tags nav items are trimmed away in post-build because a small
//! studio site doesn't need them. (`tags.md` + `posts.md` stay on disk
//! because `staticdatagen::compile` requires them, but they're hidden
//! from the nav.)
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example basic
//! ```
//!
//! Then open <http://127.0.0.1:3001> in your browser.
//!
//! ## How this differs from `quickstart`
//!
//! `quickstart` is the **developer reference** — a kitchen-sink wiring
//! that walks through the 16-plugin pipeline so you can pick what you
//! need. `basic` is the **ready-made template** — a small fixed set of
//! plugins, a small fixed page tree, polished copy you replace in place.

use anyhow::Result;
use http_handle::Server;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::search::SearchPlugin;
use staticdatagen::compiler::service::compile;
use std::time::Instant;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let build_dir = Path::new("./examples/basic/build");
    let site_dir = Path::new("./examples/basic/public");
    let content_dir = Path::new("./examples/basic/content");
    let template_dir = Path::new("./examples/templates");

    // Ensure per-example output directories exist before compile.
    fs::create_dir_all(build_dir)?;
    fs::create_dir_all(site_dir)?;

    // 1. Compile content → HTML
    let start = Instant::now();
    match compile(build_dir, content_dir, site_dir, template_dir) {
        Ok(()) => println!("    ✅ Successfully compiled static site"),
        Err(e) => {
            println!("    ❌ Error compiling site: {e:?}");
            return Err(e);
        }
    }

    // 2. Run SearchPlugin so the search UI actually works, plus the
    //    HtmlFix + ManifestFix cleanup plugins to suppress browser
    //    console warnings (empty <link rel=preload>, deprecated apple
    //    meta, manifest icon with empty src).
    let mut plugins = PluginManager::new();
    plugins.register(ssg::postprocess::HtmlFixPlugin);
    plugins.register(ssg::postprocess::ManifestFixPlugin);
    plugins.register(SearchPlugin);
    let ctx =
        PluginContext::new(content_dir, build_dir, site_dir, template_dir);
    plugins.run_after_compile(&ctx)?;
    println!("    🔍 Search index generated");
    println!("    🧹 Browser-compat cleanups applied");
    let elapsed = start.elapsed();
    println!("    ⚡ Built in {elapsed:.0?}");

    // Markdown GFM support
    let gfm_sample = ssg::markdown_ext::expand_gfm("**bold** and ~~strike~~");
    println!("    \u{1f4dd} GFM: {} chars processed", gfm_sample.len());

    // 3. Hide template UI that doesn't apply to this template:
    //    - the language dropdown (single-locale site)
    //    - the footer "Blog" + "RSS Feed" links (no blog in this template)
    //    - the homepage hero CTAs (a single landing page doesn't need them)
    apply_template_trim(site_dir)?;
    println!("    🌐 Template trimmed for single-page studio layout");

    // 4. Serve
    let example_root: String = site_dir.to_str().unwrap().to_string();
    // Build the server with a Permissions-Policy header that opts the
    // page out of the Topics API. Suppresses the "Browsing Topics API
    // removed" Chrome console message in dev mode.
    let server = Server::builder()
        .address("127.0.0.1:3001")
        .document_root(example_root.as_str())
        .custom_header("Permissions-Policy", "browsing-topics=()")
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    println!("\n❯ Server is now running at http://127.0.0.1:3001");
    println!("  Document root: {example_root}");
    println!("  Press Ctrl+C to stop the server.");

    server.start().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

/// Injects a small `<style>` block before `</head>` that trims template UI
/// the shared template hardcodes for richer examples (multilingual, blog,
/// quickstart) but isn't relevant for this single-page studio layout:
///
/// - language dropdown (single-locale site)
/// - footer Resources column (no blog or RSS feed)
/// - hero CTAs (no second page to deep-link)
///
/// Idempotent: each page is patched at most once thanks to a marker comment.
fn apply_template_trim(site_dir: &Path) -> Result<()> {
    const MARKER: &str = "/* ssg-basic: trim */";
    const STYLE: &str = "<style>/* ssg-basic: trim */\
        .lang-btn,.lang-dropdown,.mobile-lang{display:none!important}\
        .footer-cols .footer-col:nth-child(2){display:none!important}\
        .hero-actions{display:none!important}\
        .nav-item:has(a[href*=\"/posts/\"]),\
        .nav-item:has(a[href*=\"/tags/\"]){display:none!important}\
        </style>";

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
