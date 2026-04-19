#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Plugins Example — Annotated lifecycle walkthrough
//!
//! ## What this example is
//!
//! A **developer reference**, not a starter template. It walks through every
//! stage of the SSG plugin pipeline so you can see exactly when each
//! lifecycle hook fires, in what order plugins run, and what each one
//! produces.
//!
//! ## What it demonstrates
//!
//! - **Lifecycle hooks** — `before_compile`, `after_compile`, and `on_serve`
//!   in the order they actually execute
//! - **Plugin composition** — SEO + canonical + robots + search + minify +
//!   live-reload, with the *order* annotated (minify runs last so it sees
//!   final HTML)
//! - **Incremental build cache** — `BuildCache::changed_files()` skips work
//!   when no content has changed
//!
//! ## When to use this pattern
//!
//! When you want to understand the plugin API internals before authoring
//! your own plugin or customising the pipeline ordering. Read the source
//! alongside the console output as a teaching pair.
//!
//! ## When NOT to use this pattern
//!
//! When you want a clone-and-edit site. Use `examples/quickstart` instead
//! (production-ready starter) or `examples/basic` (small studio template).
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example plugins
//! ```
//!
//! This example does not start a server — it is a pipeline demonstration
//! only. Inspect `examples/plugins/public/` after running.

use anyhow::Result;
use ssg::cache::BuildCache;
use ssg::livereload::LiveReloadPlugin;
use ssg::plugin::{PluginContext, PluginManager};
use ssg::plugins::MinifyPlugin;
use ssg::postprocess::{HtmlFixPlugin, ManifestFixPlugin};
use ssg::search::SearchPlugin;
use ssg::seo::{CanonicalPlugin, RobotsPlugin, SeoPlugin};
use staticdatagen::compiler::service::compile;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let build_dir = Path::new("./examples/plugins/build");
    let site_dir = Path::new("./examples/plugins/public");
    let content_dir = Path::new("./examples/plugins/content");
    let template_dir = Path::new("./examples/templates");
    // Cache file lives under `target/` so it doesn't pollute the repo root.
    let cache_dir = Path::new("target/.ssg-cache");
    fs::create_dir_all(cache_dir)?;
    let cache_path = &cache_dir.join("plugins.json");

    // Ensure per-example output directories exist before compile.
    fs::create_dir_all(build_dir)?;
    fs::create_dir_all(site_dir)?;

    // ── Step 1: Incremental build check ──────────────────────────
    //
    // Force a rebuild when the output dir is empty, even if the cache
    // says no source files changed — otherwise a clean checkout (or a
    // test run that wiped public/) would silently produce no output.
    let mut cache = BuildCache::load(cache_path)?;
    let changed = cache.changed_files(content_dir)?;
    let output_missing = !site_dir.join("index.html").exists();

    if changed.is_empty() && !output_missing {
        println!("  ✅ No content changes detected — skipping build");
    } else {
        if output_missing && changed.is_empty() {
            println!("  🔨 Output missing — forcing rebuild");
        }
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

    // Cleanup bundle — browser-compat fixes (modern PWA meta, drop empty
    // preloads, drop empty manifest icons). Run early so subsequent
    // plugins see clean HTML.
    plugins.register(HtmlFixPlugin);
    plugins.register(ManifestFixPlugin);

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
    plugins.run_fused_transforms(&ctx)?;
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

    // ── Step 5: Demonstrate remaining API modules ────────────────

    // File system operations (fs_ops)
    let safe = ssg::fs_ops::is_safe_path(site_dir)?;
    println!(
        "  \u{1f512} Path safety check: {}",
        if safe { "PASS" } else { "FAIL" }
    );

    // Streaming I/O (stream)
    println!(
        "  \u{1f4be} Stream buffer: {} KB",
        ssg::stream::STREAM_BUFFER_SIZE / 1024
    );

    // File watching (watch) — classify changes for selective reload
    let css_kind = ssg::watch::classify_change(Path::new("style.css"));
    let md_kind = ssg::watch::classify_change(Path::new("post.md"));
    println!(
        "  \u{1f441} Watch: style.css → {css_kind:?}, post.md → {md_kind:?}"
    );

    // Markdown extensions (markdown_ext)
    let md = "| Col A | Col B |\n|-------|-------|\n| 1 | 2 |";
    let html = ssg::markdown_ext::expand_gfm(md);
    println!("  \u{1f4dd} GFM table \u{2192} {} bytes HTML", html.len());

    // Schema generation (schema)
    let schema = ssg::schema::generate_schema();
    println!(
        "  \u{1f4cb} Config schema: {} properties",
        schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map_or(0, |o| o.len())
    );

    // Scaffold (project generation)
    println!("  \u{1f3d7} Scaffold: ssg::scaffold::scaffold_project(\"my-site\") creates a starter");

    // Logging
    println!("  \u{1f4c4} Logging: ssg::logging::create_log_file(\"build.log\") for structured logs");

    // Process (CLI argument handling)
    println!("  \u{2699} Process: ssg::process::args() handles CLI \u{2192} compilation workflow");

    println!("\n  Done. Site ready at {}", site_dir.display());
    Ok(())
}
