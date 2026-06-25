#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::must_use_candidate
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the PQC-aware edge-runtime header emitter
//! (`EdgeHeadersPlugin`, issue #550).
//!
//! Verifies all seven acceptance criteria:
//!
//! - AC1: Cloudflare target emits `wrangler.toml [headers]` snippet
//!   at `dist/.ssg/edge/wrangler-headers.toml`.
//! - AC2: Netlify target emits `dist/_headers`.
//! - AC3: Vercel target emits `dist/.ssg/edge/vercel-headers.json`.
//! - AC4: Multi-target emit produces all three files with the same
//!   logical header set.
//! - AC5: User override of any header is honoured.
//! - AC6: PQC documentation comment present in every file.
//! - AC7: No CSP clash — single, plugin-sourced Content-Security-Policy.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use ssg::cmd::{EdgeHeadersConfig, SsgConfig};
use ssg::plugin::{Plugin, PluginContext};
use ssg::postprocess::EdgeHeadersPlugin;

// ── Fixture builders ─────────────────────────────────────────────

/// Build an `SsgConfig` with the supplied edge-headers config.
fn config_with(edge: EdgeHeadersConfig) -> SsgConfig {
    SsgConfig::builder()
        .site_name("test".to_string())
        .base_url("http://example.com".to_string())
        .edge_headers(edge)
        .build()
        .expect("config")
}

/// Build a `PluginContext` against a fresh `site_dir`.
fn ctx_with(
    site_dir: &Path,
    targets: &[&str],
    overrides: BTreeMap<String, String>,
) -> PluginContext {
    let edge = EdgeHeadersConfig {
        targets: targets.iter().map(|s| (*s).to_string()).collect(),
        overrides,
    };
    let cfg = config_with(edge);
    PluginContext::with_config(
        site_dir.parent().unwrap_or(site_dir),
        site_dir.parent().unwrap_or(site_dir),
        site_dir,
        site_dir.parent().unwrap_or(site_dir),
        cfg,
    )
}

/// Run the plugin against a fresh tempdir and return `(site_dir, _td)`
/// — the second element keeps the `TempDir` alive for the duration of
/// the test.
fn run_emit(
    targets: &[&str],
    overrides: BTreeMap<String, String>,
) -> (PathBuf, tempfile::TempDir) {
    let td = tempfile::tempdir().expect("tempdir");
    let site = td.path().join("dist");
    fs::create_dir_all(&site).expect("mkdir site");
    let ctx = ctx_with(&site, targets, overrides);
    EdgeHeadersPlugin
        .after_compile(&ctx)
        .expect("emitter must succeed");
    (site, td)
}

// ── AC1 — Cloudflare ─────────────────────────────────────────────

#[test]
fn ac1_cloudflare_emits_wrangler_headers_toml() {
    let (site, _td) = run_emit(&["cloudflare"], BTreeMap::new());
    let out = site.join(".ssg/edge/wrangler-headers.toml");
    assert!(out.exists(), "expected {out:?} to exist");

    let body = fs::read_to_string(&out).unwrap();

    // Five baseline headers must all be present.
    for key in [
        "Strict-Transport-Security",
        "Content-Security-Policy",
        "X-Content-Type-Options",
        "Referrer-Policy",
        "Permissions-Policy",
    ] {
        assert!(body.contains(key), "missing {key} in cloudflare emit");
    }

    // Must be valid TOML and merge-friendly.
    let parsed = toml::from_str::<toml::Value>(&body)
        .expect("emitted file must be valid TOML");
    let headers = parsed
        .get("headers")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("[[headers]] block missing");
    assert_eq!(
        headers.get("for").and_then(|v| v.as_str()),
        Some("/*"),
        "wildcard route required for merge convenience"
    );
}

// ── AC2 — Netlify ───────────────────────────────────────────────

#[test]
fn ac2_netlify_emits_dist_headers() {
    let (site, _td) = run_emit(&["netlify"], BTreeMap::new());
    let out = site.join("_headers");
    assert!(out.exists(), "expected {out:?} to exist");

    let body = fs::read_to_string(&out).unwrap();
    // Route group "/*" must be present at column 0.
    assert!(
        body.lines().any(|l| l == "/*"),
        "expected `/*` route group: {body}"
    );
    // Each baseline header indented under the group.
    for key in [
        "Strict-Transport-Security:",
        "Content-Security-Policy:",
        "X-Content-Type-Options:",
        "Referrer-Policy:",
        "Permissions-Policy:",
    ] {
        let indented = format!("  {key}");
        assert!(
            body.contains(&indented),
            "missing indented `{indented}` in netlify emit"
        );
    }
}

// ── AC3 — Vercel ────────────────────────────────────────────────

#[test]
fn ac3_vercel_emits_vercel_headers_json() {
    let (site, _td) = run_emit(&["vercel"], BTreeMap::new());
    let out = site.join(".ssg/edge/vercel-headers.json");
    assert!(out.exists(), "expected {out:?} to exist");

    let body = fs::read_to_string(&out).unwrap();
    let parsed: Value =
        serde_json::from_str(&body).expect("emitted file must be valid JSON");
    let headers = parsed["headers"]
        .as_array()
        .expect("top-level `headers` array");
    let group = headers.first().unwrap();
    assert_eq!(
        group["source"].as_str(),
        Some("/(.*)"),
        "canonical Vercel match-all source"
    );
    let arr = group["headers"].as_array().unwrap();
    assert_eq!(arr.len(), 5, "five baseline headers");
    for entry in arr {
        assert!(entry.get("key").is_some());
        assert!(entry.get("value").is_some());
    }
}

// ── AC4 — Multi-target ──────────────────────────────────────────

#[test]
fn ac4_multi_target_produces_all_three_files_consistently() {
    let (site, _td) =
        run_emit(&["cloudflare", "netlify", "vercel"], BTreeMap::new());

    let cf = site.join(".ssg/edge/wrangler-headers.toml");
    let nl = site.join("_headers");
    let vc = site.join(".ssg/edge/vercel-headers.json");
    assert!(cf.exists(), "cloudflare file missing");
    assert!(nl.exists(), "netlify file missing");
    assert!(vc.exists(), "vercel file missing");

    let cf_body = fs::read_to_string(&cf).unwrap();
    let nl_body = fs::read_to_string(&nl).unwrap();
    let vc_body = fs::read_to_string(&vc).unwrap();

    // Same logical headers — all five keys present in all three
    // emitted formats, with the same values.
    let expected_values = [
        (
            "Strict-Transport-Security",
            "max-age=63072000; includeSubDomains; preload",
        ),
        ("Content-Security-Policy", ssg::csp::computed_policy()),
        ("X-Content-Type-Options", "nosniff"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        (
            "Permissions-Policy",
            "camera=(), geolocation=(), microphone=()",
        ),
    ];

    for (k, v) in expected_values {
        assert!(
            cf_body.contains(k) && cf_body.contains(v),
            "cloudflare missing {k}={v}: {cf_body}"
        );
        assert!(
            nl_body.contains(k) && nl_body.contains(v),
            "netlify missing {k}={v}: {nl_body}"
        );
        assert!(
            vc_body.contains(k) && vc_body.contains(v),
            "vercel missing {k}={v}: {vc_body}"
        );
    }
}

// ── AC5 — User override ─────────────────────────────────────────

#[test]
fn ac5_user_override_replaces_value_and_preserves_other_defaults() {
    let mut overrides = BTreeMap::new();
    let _ = overrides.insert(
        "permissions-policy".to_string(),
        "geolocation=(self)".to_string(),
    );

    let (site, _td) = run_emit(&["cloudflare", "netlify", "vercel"], overrides);

    // Each emitter must reflect the override.
    let cf = fs::read_to_string(site.join(".ssg/edge/wrangler-headers.toml"))
        .unwrap();
    let nl = fs::read_to_string(site.join("_headers")).unwrap();
    let vc =
        fs::read_to_string(site.join(".ssg/edge/vercel-headers.json")).unwrap();

    assert!(
        cf.contains("geolocation=(self)"),
        "cloudflare override missing"
    );
    assert!(
        nl.contains("Permissions-Policy: geolocation=(self)"),
        "netlify override missing"
    );
    let vc_json: Value = serde_json::from_str(&vc).unwrap();
    let arr = vc_json["headers"][0]["headers"].as_array().unwrap();
    let pp = arr
        .iter()
        .find(|h| h["key"].as_str() == Some("Permissions-Policy"))
        .unwrap();
    assert_eq!(pp["value"].as_str(), Some("geolocation=(self)"));

    // Defaults preserved for non-overridden headers.
    assert!(nl.contains("X-Content-Type-Options: nosniff"));
    assert!(nl.contains("Referrer-Policy: strict-origin-when-cross-origin"));
    assert!(nl.contains(
        "Strict-Transport-Security: max-age=63072000; includeSubDomains; preload"
    ));
}

// ── AC6 — PQC docstring ─────────────────────────────────────────

#[test]
fn ac6_pqc_docstring_present_in_every_emitted_file() {
    let (site, _td) =
        run_emit(&["cloudflare", "netlify", "vercel"], BTreeMap::new());

    let cf = fs::read_to_string(site.join(".ssg/edge/wrangler-headers.toml"))
        .unwrap();
    let nl = fs::read_to_string(site.join("_headers")).unwrap();
    let vc =
        fs::read_to_string(site.join(".ssg/edge/vercel-headers.json")).unwrap();

    // Algorithm name in each.
    assert!(
        cf.contains("X25519+ML-KEM-768"),
        "cloudflare missing PQC algorithm name"
    );
    assert!(
        nl.contains("X25519+ML-KEM-768"),
        "netlify missing PQC algorithm name"
    );
    assert!(
        vc.contains("X25519+ML-KEM-768"),
        "vercel missing PQC algorithm name"
    );

    // Each file must link to its own platform docs.
    assert!(
        cf.contains("cloudflare.com/ssl/post-quantum"),
        "cloudflare missing platform link"
    );
    assert!(nl.contains("netlify.com"), "netlify missing platform link");
    // Vercel link is in the JSON note field.
    let vc_json: Value = serde_json::from_str(&vc).unwrap();
    let note_joined = vc_json["_pqc_note"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        note_joined.contains("vercel.com"),
        "vercel missing platform link"
    );

    // Documentation must be carried as comments / sidecar data, not
    // active config — cloudflare uses `#`, netlify uses `#`, vercel
    // uses an unrelated JSON key.
    assert!(
        cf.lines()
            .any(|l| l.starts_with("# ") && l.contains("X25519")),
        "cloudflare PQC note must be in a comment"
    );
    assert!(
        nl.lines()
            .any(|l| l.starts_with("# ") && l.contains("X25519")),
        "netlify PQC note must be in a comment"
    );
    assert!(
        vc_json.get("_pqc_note").is_some(),
        "vercel PQC note must live in `_pqc_note` (Vercel ignores unknown keys)"
    );
}

// ── AC7 — No CSP duplication ────────────────────────────────────

#[test]
fn ac7_no_duplicate_csp_and_value_comes_from_csp_plugin() {
    let (site, _td) =
        run_emit(&["cloudflare", "netlify", "vercel"], BTreeMap::new());

    let cf = fs::read_to_string(site.join(".ssg/edge/wrangler-headers.toml"))
        .unwrap();
    let nl = fs::read_to_string(site.join("_headers")).unwrap();
    let vc =
        fs::read_to_string(site.join(".ssg/edge/vercel-headers.json")).unwrap();

    // Count `Content-Security-Policy` occurrences in the *header
    // declarations* (excluding any comment lines that might mention
    // the header name in docs).
    let cf_csp = cf
        .lines()
        .filter(|l| {
            !l.trim_start().starts_with('#')
                && l.contains("Content-Security-Policy")
        })
        .count();
    assert_eq!(
        cf_csp, 1,
        "cloudflare must emit exactly one CSP declaration: {cf}"
    );

    let nl_csp = nl
        .lines()
        .filter(|l| {
            !l.trim_start().starts_with('#')
                && l.contains("Content-Security-Policy:")
        })
        .count();
    assert_eq!(nl_csp, 1, "netlify must emit exactly one CSP declaration");

    let vc_json: Value = serde_json::from_str(&vc).unwrap();
    let arr = vc_json["headers"][0]["headers"].as_array().unwrap();
    let vc_csp = arr
        .iter()
        .filter(|h| {
            h.get("key").and_then(|k| k.as_str()).is_some_and(|s| {
                s.eq_ignore_ascii_case("Content-Security-Policy")
            })
        })
        .count();
    assert_eq!(vc_csp, 1, "vercel must emit exactly one CSP entry");

    // And the CSP value must equal what the CSP plugin computes —
    // there is no second hardcoded copy anywhere in the emitter.
    let csp_from_plugin = ssg::csp::computed_policy();
    assert!(
        cf.contains(csp_from_plugin),
        "cloudflare CSP must equal csp::computed_policy()"
    );
    assert!(
        nl.contains(csp_from_plugin),
        "netlify CSP must equal csp::computed_policy()"
    );
    let csp_entry = arr
        .iter()
        .find(|h| h["key"].as_str() == Some("Content-Security-Policy"))
        .unwrap();
    assert_eq!(
        csp_entry["value"].as_str(),
        Some(csp_from_plugin),
        "vercel CSP must equal csp::computed_policy()"
    );
}

// ── Bonus: unknown target is logged + skipped, no panic ─────────

#[test]
fn unknown_target_is_skipped_without_error() {
    let (site, _td) = run_emit(&["fastly"], BTreeMap::new());
    // No files emitted, plugin returns Ok.
    assert!(!site.join("_headers").exists());
    assert!(!site.join(".ssg/edge").exists());
}
