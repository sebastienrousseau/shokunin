#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression suite for issue #540 — parser-driven `</head>` injection.
//!
//! Verifies that `ssg::util::head_dom::inject_before_head_close`
//! injects the payload only at the real `</head>` element and never at
//! a literal `</head>` string that happens to appear inside a `<pre>`
//! block or an HTML comment. Also verifies that every plugin that used
//! to call `str::replace("</head>", …)` has been migrated.

use ssg::util::head_dom::inject_before_head_close;

const PAYLOAD: &str = "<meta name=\"unique-marker\" content=\"x\">";

// AC1: Injection at the real </head> exactly once.
#[test]
fn ac1_injection_at_real_head_close() {
    let html = "<html><head><title>T</title></head><body></body></html>";
    let out = inject_before_head_close(html, PAYLOAD);

    assert_eq!(out.matches(PAYLOAD).count(), 1, "exactly one injection");
    let payload_pos = out.find(PAYLOAD).unwrap();
    let head_close_pos = out.find("</head>").unwrap();
    assert!(
        payload_pos < head_close_pos,
        "payload must come before </head>: {out}"
    );
}

// AC2: <pre> block containing </head> literal is untouched.
#[test]
fn ac2_pre_block_literal_untouched() {
    let html = "<html><head><title>T</title></head>\
                <body><pre>&lt;/head&gt;</pre></body></html>";
    let out = inject_before_head_close(html, PAYLOAD);

    assert_eq!(out.matches(PAYLOAD).count(), 1);
    assert!(
        out.contains("<pre>&lt;/head&gt;</pre>"),
        "pre block must be byte-stable"
    );
}

// AC3: Comment containing </head> literal is untouched.
#[test]
fn ac3_comment_literal_untouched() {
    let html =
        "<html><head><title>T</title></head><body><!-- </head> --></body></html>";
    let out = inject_before_head_close(html, PAYLOAD);

    assert_eq!(out.matches(PAYLOAD).count(), 1);
    assert!(
        out.contains("<!-- </head> -->"),
        "comment must be byte-stable"
    );
}

// AC4: every injector migrated.
//
// The migration removed every `str::replace("</head>", …)` call from
// the plugins inventory; this test guards against regression by
// re-scanning the source tree. If you add a new plugin that needs to
// inject before `</head>`, route it through inject_before_head_close
// (or replace_canonical_link) — do not add a new str::replace.
#[test]
fn ac4_no_remaining_replace_calls_in_production_code() {
    let candidates = [
        "src/plugins/og_image.rs",
        "src/plugins/llm.rs",
        "src/plugins/ai.rs",
        "src/plugins/i18n.rs",
        "src/plugins/highlight.rs",
        "src/plugins/sbom.rs",
        "src/plugins/postprocess/atom.rs",
        "src/plugins/postprocess/json_feed.rs",
        "src/plugins/postprocess/html_fix.rs",
        "src/plugins/seo/canonical.rs",
        "src/plugins/seo/seo_plugin.rs",
        "src/plugins/seo/jsonld.rs",
    ];
    for rel in candidates {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        // Strip line and block comments so doc-comments that mention
        // `</head>` don't trip the check.
        let mut prod = String::with_capacity(src.len());
        for line in src.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }
            prod.push_str(line);
            prod.push('\n');
        }
        // Tests inside #[cfg(test)] modules may still use find("</head>")
        // for assertions — we only care about production paths.
        let lower = prod.to_ascii_lowercase();
        let prod_section = lower.split("#[cfg(test)]").next().unwrap();
        assert!(
            !prod_section.contains(".replace(\"</head>\""),
            "{rel} still calls str::replace(\"</head>\", …) in production code"
        );
        assert!(
            !prod_section.contains(".insert_str(pos, &tag)"),
            "{rel} still uses string-index insertion against `</head>`"
        );
    }
}

// AC5 / AC6: empty payload is a no-op, no </head> is a no-op.
#[test]
fn empty_payload_returns_input_verbatim() {
    let html = "<html><head></head></html>";
    assert_eq!(inject_before_head_close(html, ""), html);
}

#[test]
fn no_head_returns_input_verbatim() {
    let html = "<html><body>no head</body></html>";
    assert_eq!(inject_before_head_close(html, PAYLOAD), html);
}

// Bonus: documents with multiple injection sources all converge on the
// correct location.
#[test]
fn multiple_successive_injections_all_land_before_head_close() {
    let html = "<html><head><title>T</title></head><body></body></html>";
    let mut out = html.to_string();
    for tag in [
        "<meta name=\"a\">",
        "<meta name=\"b\">",
        "<meta name=\"c\">",
    ] {
        out = inject_before_head_close(&out, tag);
    }
    assert!(out.contains("<meta name=\"a\">"));
    assert!(out.contains("<meta name=\"b\">"));
    assert!(out.contains("<meta name=\"c\">"));
    let close_pos = out.find("</head>").unwrap();
    for marker in [
        "<meta name=\"a\">",
        "<meta name=\"b\">",
        "<meta name=\"c\">",
    ] {
        assert!(
            out.find(marker).unwrap() < close_pos,
            "marker {marker} must be inside <head>"
        );
    }
}
