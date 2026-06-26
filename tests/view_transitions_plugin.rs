// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::view_transitions::ViewTransitionsPlugin`
//! (issue #547).
//!
//! Verifies the build-side contract of the plugin: opt-in via
//! `ssg.toml`, idempotent HTML injection, cross-origin safety, and
//! the script-budget guard. The browser-side AC1/AC2/AC6 behaviour is
//! covered by the Playwright suite in `tests/visual/`.

use ssg::cmd::SsgConfig;
use ssg::plugin::{Plugin, PluginContext};
use ssg::view_transitions::{ViewTransitionsPlugin, VIEW_TRANSITIONS_JS};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Smoke test: the plugin advertises a stable, non-empty name.
#[test]
fn plugin_name_is_stable() {
    assert_eq!(ViewTransitionsPlugin::new().name(), "view-transitions");
}

/// AC opt-in: when `transitions` is `false` (the default), the plugin
/// does not consider itself enabled.
#[test]
fn opt_in_defaults_to_false() {
    let cfg = SsgConfig::builder()
        .site_name("t".into())
        .base_url("http://example.com".into())
        .build()
        .expect("config");
    assert!(!ViewTransitionsPlugin::enabled(&cfg));
}

/// AC opt-in: the builder method flips the flag.
#[test]
fn opt_in_via_builder() {
    let cfg = SsgConfig::builder()
        .site_name("t".into())
        .base_url("http://example.com".into())
        .transitions(true)
        .build()
        .expect("config");
    assert!(ViewTransitionsPlugin::enabled(&cfg));
}

/// AC opt-in: a `ssg.toml` snippet with `transitions = true` is
/// honoured by the same path used by users.
#[test]
fn opt_in_via_toml() {
    let toml = r#"
site_name = "t"
content_dir = "./content"
output_dir = "./public"
template_dir = "./templates"
base_url = "http://example.com"
site_title = "T"
site_description = "D"
language = "en-GB"
transitions = true
"#;
    let cfg: SsgConfig = toml.parse().expect("toml parses");
    assert!(ViewTransitionsPlugin::enabled(&cfg));
}

/// AC opt-in: when `transitions` is omitted, the field defaults to
/// `false` (so existing sites' bundle size doesn't regress).
#[test]
fn omitted_field_defaults_to_disabled() {
    let toml = r#"
site_name = "t"
content_dir = "./content"
output_dir = "./public"
template_dir = "./templates"
base_url = "http://example.com"
site_title = "T"
site_description = "D"
language = "en-GB"
"#;
    let cfg: SsgConfig = toml.parse().expect("toml parses");
    assert!(!ViewTransitionsPlugin::enabled(&cfg));
}

/// AC4/AC6: the injected HTML carries both the persistent-root style
/// hooks and the deferred module script.
#[test]
fn transform_html_emits_script_and_style() {
    let dir = tempdir().unwrap();
    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        dir.path(),
        Path::new("/tmp/t"),
    );

    let html = r"<html><head><title>x</title></head><body><main>x</main></body></html>";
    let out = ViewTransitionsPlugin::new()
        .transform_html(html, Path::new("/page.html"), &ctx)
        .expect("transform");

    assert!(out.contains("/_transitions/ssg-transitions.js"));
    assert!(out.contains("data-ssg-transitions"));
    assert!(out.contains("ssg-header"));
    assert!(out.contains("ssg-footer"));
    assert!(out.contains("ssg-main"));
    assert!(out.contains("prefers-reduced-motion"));
}

/// Idempotency: re-running `transform_html` on already-injected HTML
/// is a no-op, so the fused-transform pass doesn't double-inject.
#[test]
fn transform_html_is_idempotent() {
    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        Path::new("/tmp/s"),
        Path::new("/tmp/t"),
    );
    let plugin = ViewTransitionsPlugin::new();

    let html = r"<html><head></head><body></body></html>";
    let once = plugin
        .transform_html(html, Path::new("/x.html"), &ctx)
        .unwrap();
    let twice = plugin
        .transform_html(&once, Path::new("/x.html"), &ctx)
        .unwrap();

    assert_eq!(once, twice);
    assert_eq!(twice.matches("/_transitions/ssg-transitions.js").count(), 1);
    assert_eq!(twice.matches("data-ssg-transitions-style").count(), 1);
}

/// Partials (HTML fragments without a closing `</body>`) must be left
/// alone — otherwise we'd inject the script into RSS items / shortcode
/// fragments / search snippets.
#[test]
fn transform_html_skips_fragments() {
    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        Path::new("/tmp/s"),
        Path::new("/tmp/t"),
    );
    let html = "<article><p>fragment</p></article>";
    let out = ViewTransitionsPlugin::new()
        .transform_html(html, Path::new("/snippet.html"), &ctx)
        .unwrap();
    assert_eq!(out, html);
}

/// AC: `after_compile` materialises the client script under
/// `<site>/_transitions/ssg-transitions.js`.
#[test]
fn after_compile_writes_client_script() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    fs::create_dir_all(&site).unwrap();

    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        &site,
        Path::new("/tmp/t"),
    );
    ViewTransitionsPlugin::new()
        .after_compile(&ctx)
        .expect("after_compile");

    let path = site.join("_transitions/ssg-transitions.js");
    assert!(path.exists(), "script must be written to {path:?}");

    let body = fs::read_to_string(&path).unwrap();
    assert!(body.contains("startViewTransition"));
    assert!(body.contains("__ssgTransitionsReload"));
}

/// `ssg check` (dry-run mode, #527) must not write the script.
#[test]
fn after_compile_dry_run_is_noop() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    fs::create_dir_all(&site).unwrap();

    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        &site,
        Path::new("/tmp/t"),
    )
    .with_dry_run(true);

    ViewTransitionsPlugin::new()
        .after_compile(&ctx)
        .expect("dry-run after_compile must succeed");

    assert!(!site.join("_transitions").exists());
}

/// The script must stay within its 5 KB budget — this guards against
/// future feature creep silently regressing first-paint cost (AC4).
#[test]
fn script_budget_is_enforced() {
    assert!(
        VIEW_TRANSITIONS_JS.len() <= 5 * 1024,
        "view-transitions script grew to {} bytes (budget 5 KB)",
        VIEW_TRANSITIONS_JS.len()
    );
}

/// AC3: the client script must contain a same-origin guard.
#[test]
fn script_contains_cross_origin_guard() {
    assert!(VIEW_TRANSITIONS_JS.contains("url.origin !== location.origin"));
}

/// AC2: the client script must detect View Transitions support before
/// calling `startViewTransition`.
#[test]
fn script_contains_feature_detection() {
    assert!(VIEW_TRANSITIONS_JS
        .contains("typeof document.startViewTransition === 'function'"));
}

/// AC5: the client script must signal outgoing islands so they can
/// detach event listeners cleanly before the swap.
#[test]
fn script_dispatches_island_detach_event() {
    assert!(VIEW_TRANSITIONS_JS.contains("ssg:detach"));
}

/// AC7: the client script must expose `window.__ssgTransitionsReload`
/// for the livereload client to call.
#[test]
fn script_exposes_hmr_hook() {
    assert!(VIEW_TRANSITIONS_JS.contains("window.__ssgTransitionsReload"));
}

/// End-to-end: a built page goes through `transform_html` then
/// `after_compile`, producing both an injected page and an emitted
/// script that link up correctly.
#[test]
fn end_to_end_build_emits_linkable_assets() {
    let dir = tempdir().unwrap();
    let site = dir.path().join("site");
    fs::create_dir_all(&site).unwrap();

    let html = r"<html><head></head><body><main>hello</main></body></html>";
    let html_path = site.join("index.html");
    fs::write(&html_path, html).unwrap();

    let ctx = PluginContext::new(
        Path::new("/tmp/c"),
        Path::new("/tmp/b"),
        &site,
        Path::new("/tmp/t"),
    );
    let plugin = ViewTransitionsPlugin::new();

    let injected = plugin.transform_html(html, &html_path, &ctx).unwrap();
    fs::write(&html_path, &injected).unwrap();
    plugin.after_compile(&ctx).unwrap();

    // The page references the exact path the script was written to.
    assert!(injected.contains("/_transitions/ssg-transitions.js"));
    assert!(site.join("_transitions/ssg-transitions.js").exists());
}
