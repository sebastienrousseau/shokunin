// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Spec B4 acceptance tests — per-page CSP wired from the CSP plugin
//! into the edge-headers platform files (v0.0.47 plan §3 item 2.4).
//!
//! Pipeline ordering contract under test: `after_compile` emits the
//! global-policy files first; the fused transform pass then runs
//! `CspPlugin::transform_html` (inline extraction) *before*
//! `EdgeHeadersPlugin::transform_html` (per-page CSP recording), per
//! their registration order in `register_default_plugins`.

use ssg::cmd::{EdgeHeadersConfig, SriAlgorithm, SsgConfig};
use ssg::csp::CspPlugin;
use ssg::plugin::{Plugin, PluginContext};
use ssg::postprocess::EdgeHeadersPlugin;
use std::fs;
use std::path::PathBuf;

/// Builds a context whose config enables the given edge targets.
fn edge_ctx(
    dir: &std::path::Path,
    targets: &[&str],
) -> (PluginContext, PathBuf) {
    let site = dir.join("site");
    fs::create_dir_all(&site).unwrap();
    let mut edge = EdgeHeadersConfig::default();
    edge.targets = targets.iter().map(|t| (*t).to_string()).collect();
    let cfg = SsgConfig::builder()
        .site_name("t".to_string())
        .base_url("https://example.com".to_string())
        .edge_headers(edge)
        .build()
        .unwrap();
    let ctx = PluginContext::with_config(dir, dir, &site, dir, cfg);
    (ctx, site)
}

/// Acceptance (spec B4): a fixture page with inline JSON-LD produces
/// `_headers` and vercel-headers.json entries for that page's path
/// whose `script-src` carries the exact sha256 of the inline block.
#[test]
fn jsonld_page_gets_per_path_csp_with_exact_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, site) = edge_ctx(dir.path(), &["netlify", "vercel"]);

    let plugin = EdgeHeadersPlugin::new();
    plugin.after_compile(&ctx).unwrap();

    let jsonld = r#"{"@context":"https://schema.org","@type":"BlogPosting"}"#;
    let html = format!(
        r#"<html><head><title>P</title><script type="application/ld+json">{jsonld}</script></head><body>b</body></html>"#
    );
    let page = site.join("blog/2026-07-01-post/index.html");
    let out = plugin.transform_html(&html, &page, &ctx).unwrap();
    assert_eq!(out, html, "edge-headers transform must not rewrite HTML");

    let expected = SriAlgorithm::Sha256.integrity(jsonld.as_bytes());

    // Netlify `_headers`: per-path group with the exact hash.
    let headers = fs::read_to_string(site.join("_headers")).unwrap();
    assert!(
        headers.contains("/blog/2026-07-01-post/\n"),
        "page path missing from _headers:\n{headers}"
    );
    assert!(
        headers.contains(&format!("script-src 'self' '{expected}'")),
        "exact sha256 source missing from _headers:\n{headers}"
    );

    // Vercel JSON: route entry with the same policy.
    let vercel =
        fs::read_to_string(site.join(".ssg/edge/vercel-headers.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&vercel).unwrap();
    let groups = parsed["headers"].as_array().unwrap();
    let entry = groups
        .iter()
        .find(|g| g["source"].as_str() == Some("/blog/2026-07-01-post/"))
        .expect("per-page vercel route entry");
    let value = entry["headers"][0]["value"].as_str().unwrap();
    assert!(value.contains(&format!("'{expected}'")));
}

/// test_csp_strict-style gate (spec B4): with hashes present the
/// emitted per-page policy is hash-strict — no `'unsafe-inline'`
/// anywhere in either platform file.
#[test]
fn per_page_policy_is_hash_strict_without_unsafe_inline() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, site) = edge_ctx(dir.path(), &["netlify", "vercel"]);
    let plugin = EdgeHeadersPlugin::new();
    plugin.after_compile(&ctx).unwrap();

    let html = r#"<html><head><script type="application/ld+json">{"@type":"Thing"}</script></head><body>x</body></html>"#;
    let _ = plugin
        .transform_html(html, &site.join("index.html"), &ctx)
        .unwrap();

    for artefact in ["_headers", ".ssg/edge/vercel-headers.json"] {
        let body = fs::read_to_string(site.join(artefact)).unwrap();
        assert!(
            body.contains("'sha256-"),
            "{artefact} must carry sha256 sources:\n{body}"
        );
        assert!(
            !body.contains("unsafe-inline"),
            "{artefact} must be hash-strict:\n{body}"
        );
    }
}

/// End-to-end interplay with the CSP plugin: an executable inline
/// script is externalized by `CspPlugin` (upstream in the fused
/// pass), so only the JSON-LD block — which extraction skips —
/// contributes a hash to the per-page policy.
#[test]
fn csp_extraction_runs_first_so_only_jsonld_hash_remains() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, site) = edge_ctx(dir.path(), &["netlify"]);

    let edge = EdgeHeadersPlugin::new();
    edge.after_compile(&ctx).unwrap();
    CspPlugin.after_compile(&ctx).unwrap();

    let jsonld = r#"{"@type":"Thing"}"#;
    let inline_js = "console.log('will be externalized');";
    let html = format!(
        r#"<html><head><script type="application/ld+json">{jsonld}</script></head><body><script>{inline_js}</script></body></html>"#
    );
    let page = site.join("post/index.html");

    // Fused-pass registration order: csp before edge-headers.
    let after_csp = CspPlugin.transform_html(&html, &page, &ctx).unwrap();
    assert!(
        after_csp.contains("<script src="),
        "inline script should be externalized by csp"
    );
    let _ = edge.transform_html(&after_csp, &page, &ctx).unwrap();

    let jsonld_hash = SriAlgorithm::Sha256.integrity(jsonld.as_bytes());
    let js_hash = SriAlgorithm::Sha256.integrity(inline_js.as_bytes());
    let headers = fs::read_to_string(site.join("_headers")).unwrap();
    assert!(
        headers.contains(&format!("'{jsonld_hash}'")),
        "JSON-LD hash must be present:\n{headers}"
    );
    assert!(
        !headers.contains(&format!("'{js_hash}'")),
        "externalized script must not be hashed:\n{headers}"
    );
}

/// Pages without inline blocks stay on the global `/*` policy — no
/// per-path entry is added for them.
#[test]
fn plain_pages_fall_back_to_global_policy() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, site) = edge_ctx(dir.path(), &["netlify"]);
    let plugin = EdgeHeadersPlugin::new();
    plugin.after_compile(&ctx).unwrap();

    let html = "<html><head><title>t</title></head><body>plain</body></html>";
    let _ = plugin
        .transform_html(html, &site.join("about/index.html"), &ctx)
        .unwrap();

    let headers = fs::read_to_string(site.join("_headers")).unwrap();
    assert!(headers.contains("/*\n"), "global group must remain");
    assert!(
        !headers.contains("/about/"),
        "no per-path entry without inline hashes:\n{headers}"
    );
    assert_eq!(
        headers
            .lines()
            .filter(|l| l.contains("Content-Security-Policy:"))
            .count(),
        1,
        "exactly the global CSP line"
    );
}

/// Deterministic output: per-page entries render in sorted path
/// order regardless of the order pages were transformed in.
#[test]
fn per_page_entries_are_sorted_for_determinism() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, site) = edge_ctx(dir.path(), &["netlify"]);
    let plugin = EdgeHeadersPlugin::new();
    plugin.after_compile(&ctx).unwrap();

    let html = |body: &str| {
        format!(
            r#"<html><head><script type="application/ld+json">{body}</script></head></html>"#
        )
    };
    // Insert in reverse-alphabetical order.
    for rel in ["zeta/index.html", "mid/index.html", "alpha/index.html"] {
        let _ = plugin
            .transform_html(&html(r#"{"a":1}"#), &site.join(rel), &ctx)
            .unwrap();
    }

    let headers = fs::read_to_string(site.join("_headers")).unwrap();
    let ia = headers.find("/alpha/").unwrap();
    let im = headers.find("/mid/").unwrap();
    let iz = headers.find("/zeta/").unwrap();
    assert!(
        ia < im && im < iz,
        "entries must be path-sorted:\n{headers}"
    );
}
