#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Plugins Example — Plugin pipeline lifecycle demonstration
//!
//! ## What this example demonstrates
//!
//! - **Lifecycle hooks** — `before_compile`, `after_compile`, and `on_serve` in action
//! - **22 built-in plugins** — how SEO, search, minify, and live-reload compose together
//! - **Incremental build cache** — skips work when no content has changed
//!
//! ## When to use this pattern
//!
//! Use this example when you want to understand the plugin API internals before
//! authoring your own plugin or customising the pipeline ordering.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example plugins_example
//! ```
//!
//! This example does not start a server — it is a pipeline demonstration only.
//!
//! ## What makes this different from other examples
//!
//! Unlike `quickstart` which just uses plugins, this example walks through
//! each lifecycle stage so you can see exactly where each plugin runs.

use anyhow::Result;
use ssg::cache::BuildCache;
use ssg::livereload::LiveReloadPlugin;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::plugins::MinifyPlugin;
use ssg::search::SearchPlugin;
use ssg::seo::{CanonicalPlugin, RobotsPlugin, SeoPlugin};
use staticdatagen::compiler::service::compile;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let build_dir = Path::new("./examples/build");
    let site_dir = Path::new("./examples/public");
    let content_dir = Path::new("./examples/content/en");
    let template_dir = Path::new("./examples/templates/en");
    let cache_path = Path::new(".ssg-cache.json");

    // ── Step 1: Incremental build check ──────────────────────────
    let mut cache = BuildCache::load(cache_path)?;
    let changed = cache.changed_files(content_dir)?;

    if changed.is_empty() {
        println!("  ✅ No content changes detected — skipping build");
    } else {
        println!("  🔨 {} file(s) changed — rebuilding", changed.len());
        for path in &changed {
            println!("     ↳ {}", path.display());
        }

        // ── Step 2: Compile the site ─────────────────────────────
        match compile(build_dir, content_dir, site_dir, template_dir) {
            Ok(()) => println!("  ✅ Site compiled successfully"),
            Err(e) => {
                println!("  ❌ Compilation error: {e:?}");
                return Ok(());
            }
        }

        // Update the build cache after successful compilation
        cache.update(content_dir)?;
        cache.save()?;
        println!("  💾 Build cache updated ({} entries)", cache.len());
    }

    // ── Step 3: Run the plugin pipeline ──────────────────────────
    let base_url = "https://example.com";

    let mut plugins = PluginManager::new();

    // SEO bundle — inject meta tags, canonical URLs, robots.txt
    plugins.register(SeoPlugin);
    plugins.register(CanonicalPlugin::new(base_url));
    plugins.register(RobotsPlugin::new(base_url));

    // Search — generate index and inject search UI (Ctrl+K)
    plugins.register(SearchPlugin);

    // Minify — collapse HTML whitespace (run last, after injections)
    plugins.register(MinifyPlugin);

    // Live reload — inject WebSocket script for dev mode
    plugins.register(LiveReloadPlugin::new());

    let ctx =
        PluginContext::new(content_dir, build_dir, site_dir, template_dir);

    println!("\n  🔌 Running {} plugins:", plugins.len());
    for name in plugins.names() {
        println!("     ↳ {name}");
    }

    plugins.run_before_compile(&ctx)?;
    plugins.run_after_compile(&ctx)?;
    plugins.run_on_serve(&ctx)?;

    println!("\n  ✅ Plugin pipeline complete");

    // ── Step 4: Report what was generated ────────────────────────
    let index_path = site_dir.join("search-index.json");
    if index_path.exists() {
        let size = fs::metadata(&index_path)?.len();
        println!("  🔍 Search index: {size} bytes");
    }
    let robots_path = site_dir.join("robots.txt");
    if robots_path.exists() {
        println!("  🤖 robots.txt generated");
    }

    println!("\n  Done. Site ready at {}", site_dir.display());
    Ok(())
}
