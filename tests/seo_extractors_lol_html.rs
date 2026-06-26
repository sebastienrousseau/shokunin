#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression suite for issue #539 — parser-driven title / lang /
//! canonical extractors.
//!
//! Verifies that the lol_html-backed `extract_head_meta` and the
//! individual `extract_*` helpers in `ssg::seo::helpers` no longer
//! mistake HTML comments or `<pre>` content for real page metadata.

use ssg::seo::helpers::extract_title;
use ssg::util::head_dom::{extract_head_meta, HeadMeta};

// AC1: Title extracted from real <title>, not from a comment that
// contains a `<title>` literal.
#[test]
fn ac1_title_from_real_title_not_comment() {
    let html = "<html><head>\
                <!-- <title>Old</title> -->\
                <title>Real</title>\
                </head><body></body></html>";
    let meta = extract_head_meta(html);
    assert_eq!(meta.title, "Real");
    assert_eq!(extract_title(html), "Real");
}

// AC2: Lang extracted from <html lang>, not from <pre><html lang=…>.
#[test]
fn ac2_lang_from_html_not_pre() {
    let html = "<html lang=\"en-GB\"><head></head>\
                <body><pre>&lt;html lang=\"fr\"&gt;</pre></body></html>";
    let meta = extract_head_meta(html);
    assert_eq!(meta.lang, "en-GB");
}

// AC3: Canonical detected via the link[rel~="canonical"] selector.
#[test]
fn ac3_canonical_detected() {
    let html =
        r#"<html><head><link rel="canonical" href="https://x"></head></html>"#;
    let meta = extract_head_meta(html);
    assert_eq!(meta.canonical, "https://x");
}

// AC4: Single parser walk — extract_head_meta returns title + lang +
// canonical in one pass.
//
// We don't expose feed() invocation counts (the lol_html API doesn't
// surface them), but the surface area is a single function call that
// returns the combined HeadMeta struct, which is the structural
// invariant the AC4 is testing.
#[test]
fn ac4_single_function_returns_all_three() {
    let html = r#"<html lang="en"><head><title>T</title><link rel="canonical" href="/c"></head><body></body></html>"#;
    let meta = extract_head_meta(html);
    assert_eq!(
        meta,
        HeadMeta {
            title: "T".to_string(),
            lang: "en".to_string(),
            canonical: "/c".to_string(),
        }
    );
}

// Bonus: empty document — every field defaults to empty string.
#[test]
fn empty_document_yields_default_meta() {
    let meta = extract_head_meta("");
    assert_eq!(meta, HeadMeta::default());
}

// Bonus: 200KB document — the parser walk does not panic on a large
// input and still extracts the right metadata.
#[test]
fn large_document_extracts_correctly() {
    // ~200KB of <pre> body content surrounding a tiny head block.
    let pre_text = "a".repeat(200_000);
    let html = format!(
        "<html lang=\"de\"><head><title>Large</title><link rel=\"canonical\" href=\"https://large\"></head><body><pre>{pre_text}</pre></body></html>"
    );
    let meta = extract_head_meta(&html);
    assert_eq!(meta.title, "Large");
    assert_eq!(meta.lang, "de");
    assert_eq!(meta.canonical, "https://large");
}

// Quoting variants.
#[test]
fn quoting_variants_all_extract() {
    let cases = [
        r#"<html lang="fr"><head></head></html>"#,
        "<html lang='fr'><head></head></html>",
        "<html lang=fr><head></head></html>",
    ];
    for html in cases {
        let meta = extract_head_meta(html);
        assert_eq!(meta.lang, "fr", "lang variant: {html}");
    }
}
