#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression suite for issue #538 — parser-driven canonical removal.
//!
//! Each test maps to one of the acceptance criteria listed on the
//! issue. The shared helper under test is
//! `ssg::util::head_dom::remove_canonical_links`; the public seam is
//! `CanonicalPlugin::transform_html`, which composes it with the
//! injection helper.

use ssg::plugin::{Plugin, PluginContext};
use ssg::seo::CanonicalPlugin;
use ssg::util::head_dom::remove_canonical_links;
use std::path::Path;
use tempfile::tempdir;

fn ctx(site: &Path) -> PluginContext {
    PluginContext::new(
        Path::new("content"),
        Path::new("build"),
        site,
        Path::new("templates"),
    )
}

// AC1: Canonical link removed from <head>; surrounding elements
// unchanged.
#[test]
fn ac1_canonical_removed_surrounding_head_unchanged() {
    let html = r#"<html><head><meta charset="utf-8"><link rel="canonical" href="https://old.example/"><title>T</title></head><body></body></html>"#;
    let out = remove_canonical_links(html);
    assert!(!out.contains("canonical"));
    assert!(out.contains("<meta charset=\"utf-8\">"));
    assert!(out.contains("<title>T</title>"));
}

// AC2: Literal text in <pre> not corrupted.
#[test]
fn ac2_pre_block_literal_untouched() {
    let html = "<html><head><title>T</title></head>\
                <body><pre><code>&lt;link rel=\"canonical\"&gt;</code></pre></body></html>";
    let out = remove_canonical_links(html);
    assert!(
        out.contains("<pre><code>&lt;link rel=\"canonical\"&gt;</code></pre>"),
        "pre block content must be byte-identical: {out}"
    );
}

// AC3: Non-canonical <link> preserved.
#[test]
fn ac3_non_canonical_link_preserved() {
    let html = r#"<head><link rel="stylesheet" href="/x.css"></head>"#;
    let out = remove_canonical_links(html);
    assert_eq!(out, html);
}

// AC4: Multiple canonicals all removed.
#[test]
fn ac4_multiple_canonicals_all_removed() {
    let html = r#"<head><link rel="canonical" href="/a"><link rel="canonical" href="/b"><link rel="canonical" href="/c"></head>"#;
    let out = remove_canonical_links(html);
    assert!(!out.contains("canonical"));
    assert!(!out.contains("/a"));
    assert!(!out.contains("/b"));
    assert!(!out.contains("/c"));
}

// AC5: Quoting style does not matter.
#[test]
fn ac5_quoting_style_irrelevant() {
    let cases = [
        "<head><link rel=\"canonical\" href=\"/x\"></head>",
        "<head><link rel='canonical' href='/x'></head>",
        "<head><link rel=canonical href=/x></head>",
    ];
    for html in cases {
        let out = remove_canonical_links(html);
        assert!(
            !out.contains("canonical"),
            "quoting variant should remove canonical: {html} → {out}"
        );
    }
}

// CanonicalPlugin integration — the historical idempotency contract
// survives the lol_html port.
#[test]
fn canonical_plugin_remains_idempotent_after_port() {
    let dir = tempdir().unwrap();
    let plugin = CanonicalPlugin::new("https://example.com");
    let c = ctx(dir.path());
    let html = "<html><head><title>P</title></head><body></body></html>";
    let page_path = dir.path().join("page.html");

    let first = plugin.transform_html(html, &page_path, &c).unwrap();
    let second = plugin.transform_html(&first, &page_path, &c).unwrap();
    assert_eq!(first, second, "second run must not duplicate canonical");
    assert_eq!(first.matches("canonical").count(), 1);
}

// Bonus: rel token set match (issue text mentions `rel="canonical
// other-token"`).
#[test]
fn rel_token_set_match_handles_multi_value_rel() {
    let html = r#"<head><link rel="canonical other-token" href="/x"></head>"#;
    let out = remove_canonical_links(html);
    assert!(
        !out.contains("href=\"/x\""),
        "rel token-set match must catch `canonical` alongside other tokens: {out}"
    );
}
