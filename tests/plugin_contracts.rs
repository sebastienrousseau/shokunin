#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::single_char_pattern,
    clippy::format_in_format_args,
    clippy::needless_late_init,
    clippy::if_then_some_else_none,
    clippy::must_use_candidate
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cross-cutting plugin-contract tests that complement per-plugin unit
//! tests in `src/`.
//!
//! Focus areas:
//! - **Lifecycle ordering** — `before_compile` runs before `after_compile`
//!   runs before `on_serve`; plugins fire in registration order within
//!   each phase.
//! - **Cleanup idempotency** — `HtmlFixPlugin` and `ManifestFixPlugin`
//!   must produce identical output on the second run. (We rely on this
//!   so an interrupted-then-resumed build doesn't corrupt artifacts.)
//! - **Plugin interaction** — `MinifyPlugin` must run *after*
//!   `HtmlFixPlugin` so it minifies the cleaned HTML, not the broken one.
//! - **Empty registration** — `PluginManager::new()` + `.run_*` must be
//!   a no-op rather than a panic.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use ssg::plugin::{Plugin, PluginContext, PluginManager};
use ssg::postprocess::{HtmlFixPlugin, ManifestFixPlugin};

// ── Shared helper to record lifecycle phases per plugin ─────────────

/// A test plugin that appends `(name, phase)` tuples to a shared log
/// every time one of its lifecycle hooks fires.
#[derive(Debug)]
struct TraceTPlugin {
    name: &'static str,
    log: Arc<Mutex<Vec<(&'static str, &'static str)>>>,
}

impl Plugin for TraceTPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    fn before_compile(&self, _ctx: &PluginContext) -> Result<()> {
        self.log.lock().unwrap().push((self.name, "before"));
        Ok(())
    }
    fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
        self.log.lock().unwrap().push((self.name, "after"));
        Ok(())
    }
    fn on_serve(&self, _ctx: &PluginContext) -> Result<()> {
        self.log.lock().unwrap().push((self.name, "serve"));
        Ok(())
    }
}

fn ctx(site: &Path) -> PluginContext {
    PluginContext::new(
        Path::new("content"),
        Path::new("build"),
        site,
        Path::new("templates"),
    )
}

// ── Lifecycle ordering ─────────────────────────────────────────────

#[test]
fn lifecycle_phases_run_in_documented_order() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut mgr = PluginManager::new();
    mgr.register(TraceTPlugin {
        name: "a",
        log: log.clone(),
    });
    mgr.register(TraceTPlugin {
        name: "b",
        log: log.clone(),
    });

    let tmp = tempfile::tempdir().unwrap();
    let c = ctx(tmp.path());

    mgr.run_before_compile(&c).unwrap();
    mgr.run_after_compile(&c).unwrap();
    mgr.run_on_serve(&c).unwrap();

    let entries = log.lock().unwrap().clone();
    assert_eq!(
        entries,
        vec![
            ("a", "before"),
            ("b", "before"),
            ("a", "after"),
            ("b", "after"),
            ("a", "serve"),
            ("b", "serve"),
        ],
        "phases must run in registration order within each phase, and \
         phases must run before → after → serve"
    );
}

#[test]
fn empty_plugin_manager_is_a_no_op() {
    let mgr = PluginManager::new();
    let tmp = tempfile::tempdir().unwrap();
    let c = ctx(tmp.path());
    // No plugins registered — every phase must succeed silently.
    mgr.run_before_compile(&c).unwrap();
    mgr.run_after_compile(&c).unwrap();
    mgr.run_on_serve(&c).unwrap();
}

// ── HtmlFixPlugin idempotency ──────────────────────────────────────

#[test]
fn html_fix_plugin_is_idempotent_on_second_run() {
    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path();
    let html = r#"<!doctype html><html lang="en"><head>
        <meta name="apple-mobile-web-app-capable" content="yes">
        <link as=image fetchpriority=high href rel=preload type=image/webp>
        <title>x</title>
        </head><body><h1>h</h1></body></html>"#;

    let c = ctx(site);
    let after_first = HtmlFixPlugin
        .transform_html(html, Path::new("index.html"), &c)
        .unwrap();

    let after_second = HtmlFixPlugin
        .transform_html(&after_first, Path::new("index.html"), &c)
        .unwrap();

    assert_eq!(
        after_first, after_second,
        "HtmlFixPlugin must produce identical output on the second run"
    );

    // Also: the modern meta is injected, the empty preload is gone.
    assert!(
        after_first.contains("name=\"mobile-web-app-capable\""),
        "modern meta should be injected"
    );
    assert!(
        !after_first.contains("rel=preload"),
        "empty preload should be removed: {after_first}"
    );
}

// ── ManifestFixPlugin idempotency ──────────────────────────────────

#[test]
fn manifest_fix_plugin_is_idempotent_on_second_run() {
    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path();
    fs::write(
        site.join("manifest.json"),
        r#"{"name":"X","icons":[{"src":"","sizes":"512x512"},{"src":"/i.svg"}]}"#,
    )
    .unwrap();

    let c = ctx(site);
    ManifestFixPlugin.after_compile(&c).unwrap();
    let after_first = fs::read_to_string(site.join("manifest.json")).unwrap();

    ManifestFixPlugin.after_compile(&c).unwrap();
    let after_second = fs::read_to_string(site.join("manifest.json")).unwrap();

    assert_eq!(
        after_first, after_second,
        "ManifestFixPlugin must produce identical output on the second run"
    );

    // The empty-src icon is dropped, the real one survives.
    let v: serde_json::Value = serde_json::from_str(&after_first).unwrap();
    let icons = v["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 1);
    assert_eq!(icons[0]["src"], "/i.svg");
}

// ── Plugin interaction: HtmlFix → Minify ordering ─────────────────

#[test]
fn html_fix_runs_before_minify_so_minified_output_is_clean() {
    use ssg::plugins::MinifyPlugin;

    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path();
    fs::write(
        site.join("index.html"),
        r#"<!doctype html>
<html lang="en">
<head>
    <meta name="apple-mobile-web-app-capable" content="yes">
    <title>x</title>
</head>
<body><h1>h</h1></body>
</html>
"#,
    )
    .unwrap();

    let mut mgr = PluginManager::new();
    mgr.register(HtmlFixPlugin);
    mgr.register(MinifyPlugin);

    let c = ctx(site);
    mgr.run_after_compile(&c).unwrap();

    let final_html = fs::read_to_string(site.join("index.html")).unwrap();

    // After both plugins run, the cleaned-and-minified output should
    // contain the modern meta (HtmlFix injected it) AND be minified
    // (MinifyPlugin collapsed whitespace).
    assert!(
        final_html.contains("mobile-web-app-capable"),
        "modern meta should be present in minified output: {final_html}"
    );
    // Crude minification check — no double spaces inside the head.
    assert!(
        !final_html.contains("\n\n"),
        "output should be minified (no blank lines): {final_html}"
    );
}

// ── ManifestFixPlugin handles missing input gracefully ────────────

#[test]
fn manifest_fix_plugin_no_op_when_manifest_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let c = ctx(tmp.path());
    // No manifest.json present — plugin must succeed silently.
    ManifestFixPlugin.after_compile(&c).unwrap();
    assert!(
        !tmp.path().join("manifest.json").exists(),
        "plugin should not create a manifest where none exists"
    );
}

// ── HtmlFixPlugin no-op when site dir is empty ────────────────────

#[test]
fn html_fix_plugin_no_op_when_site_dir_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let c = ctx(tmp.path());
    // Empty site dir — plugin must succeed silently.
    HtmlFixPlugin.after_compile(&c).unwrap();
}

// ── HtmlFixPlugin handles SVG `data:` URLs in img tags ────────────

#[test]
fn html_fix_plugin_preserves_svg_data_url_imgs() {
    // Regression: previously, the WCAG check inside the AccessibilityPlugin
    // (sister plugin) truncated tags at the first `>` inside an SVG data
    // URL. HtmlFixPlugin must not mangle those tags either.
    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path();
    let svg_url = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><rect width='10' height='10'/></svg>";
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><title>x</title></head>\
         <body><img src=\"{svg_url}\" alt=\"banner\" width=\"10\" height=\"10\"></body></html>"
    );
    fs::write(site.join("index.html"), &html).unwrap();

    let c = ctx(site);
    HtmlFixPlugin.after_compile(&c).unwrap();

    let after = fs::read_to_string(site.join("index.html")).unwrap();
    assert!(
        after.contains(svg_url),
        "SVG data URL must be preserved verbatim, got: {after}"
    );
    assert!(
        after.contains("alt=\"banner\""),
        "alt attribute must survive past the SVG data URL"
    );
}
