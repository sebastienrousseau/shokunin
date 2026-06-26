// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression suite for issue #541 — the CSP plugin must preserve
//! every attribute on rewritten `<script>` tags when extracting their
//! body to an external file.
//!
//! Before the fix in `src/plugins/csp.rs`, the rewriter rebuilt the
//! opening tag from scratch with only `src`/`integrity`/`crossorigin`,
//! silently dropping `type="module"`, `async`/`defer`, `data-*`, and
//! any other author-supplied attributes. That broke ES modules,
//! deferred loading, and analytics configuration on every site using
//! the plugin.
//!
//! Each test below maps to one acceptance criterion in #541.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ssg::csp::CspPlugin;
use ssg::plugin::{Plugin, PluginContext};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Build a `PluginContext` whose `site_dir` is a fresh tempdir.
fn ctx_in_tempdir() -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let site = dir.path().join("site");
    fs::create_dir_all(&site).unwrap();
    (dir, site)
}

fn rewrite(html: &str, site: &Path) -> String {
    let ctx = PluginContext::new(site, site, site, site);
    CspPlugin.after_compile(&ctx).unwrap();
    CspPlugin
        .transform_html(html, &site.join("index.html"), &ctx)
        .unwrap()
}

#[test]
fn ac1_type_module_is_preserved() {
    let (_g, site) = ctx_in_tempdir();
    let html = r#"<html><body><script type="module">import x from '/m.js';</script></body></html>"#;

    let out = rewrite(html, &site);

    assert!(
        out.contains(r#"type="module""#),
        "type=module must be preserved on the rewritten <script>; got: {out}"
    );
    assert!(
        out.contains("integrity="),
        "the rewritten <script> must still carry SRI; got: {out}"
    );
    assert!(
        !out.contains("import x"),
        "inline body must be extracted; got: {out}"
    );
}

#[test]
fn ac2_async_and_defer_booleans_are_preserved() {
    let (_g, site) = ctx_in_tempdir();
    // Two inline blocks back-to-back; one async, one defer.
    let html = r#"<html><body><script async>a();</script><script defer>d();</script></body></html>"#;

    let out = rewrite(html, &site);

    assert!(out.contains("async"), "async must survive; got: {out}");
    assert!(out.contains("defer"), "defer must survive; got: {out}");
    // The two scripts must each carry their own integrity hash.
    let integrity_count = out.matches("integrity=").count();
    assert_eq!(
        integrity_count, 2,
        "both inline scripts must be extracted with SRI; got: {out}"
    );
}

#[test]
fn ac3_data_attributes_are_preserved() {
    let (_g, site) = ctx_in_tempdir();
    let html = r#"<html><body><script data-domain="example.com">window.plausible=1;</script></body></html>"#;

    let out = rewrite(html, &site);

    assert!(
        out.contains(r#"data-domain="example.com""#),
        "data-domain must be preserved verbatim; got: {out}"
    );
}

#[test]
fn ac4_integrity_and_crossorigin_external_scripts_are_left_untouched() {
    let (_g, site) = ctx_in_tempdir();
    // External script (has src=…): the CSP plugin must not rewrite it
    // at all — its existing integrity and crossorigin stay verbatim.
    let html = r#"<html><body><script src="/a.js" integrity="sha384-foo" crossorigin="anonymous"></script></body></html>"#;

    let out = rewrite(html, &site);

    assert!(
        out.contains(r#"integrity="sha384-foo""#),
        "author-supplied integrity must survive; got: {out}"
    );
    assert!(
        out.contains(r#"crossorigin="anonymous""#),
        "author-supplied crossorigin must survive; got: {out}"
    );
    assert!(
        out.contains(r#"src="/a.js""#),
        "author-supplied src must survive; got: {out}"
    );
}

#[test]
fn ac5_nonce_injection_path_still_extracts_inline_script() {
    // Smoke check that the rewrite still happens (i.e. we didn't
    // accidentally skip every inline script when attribute parsing
    // returns nothing).
    let (_g, site) = ctx_in_tempdir();
    let html = "<html><body><script>console.log('hi');</script></body></html>";

    let out = rewrite(html, &site);

    assert!(out.contains("<script "), "script tag rewritten; got: {out}");
    assert!(out.contains("integrity="), "SRI present; got: {out}");
    assert!(
        !out.contains("console.log"),
        "inline body extracted; got: {out}"
    );
}
