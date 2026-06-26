#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the 14 native audit gates (issue #549).
//!
//! Per AC8 every gate has at least one positive (passing site) and
//! one negative (intentionally-failing site) test. AC1 / AC2 / AC3 /
//! AC4 / AC5 / AC6 / AC7 / AC8 / AC9 are exercised separately at the
//! bottom of the file.

use ssg::audit::{
    gates, AuditConfig, AuditGate, AuditOptions, AuditRunner, Severity, Site,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =====================================================================
// Test fixtures
// =====================================================================

/// Writes a one-page site into a new tempdir and returns the dir + the
/// loaded [`Site`]. The `TempDir` guards the filesystem lifetime —
/// keep it bound in the caller.
fn one_page(html: &str) -> (TempDir, Site) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let p = root.join("index.html");
    fs::write(&p, html).unwrap();
    let site = Site::load(&root).unwrap();
    (tmp, site)
}

fn write_file(root: &Path, rel: &str, body: &[u8]) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&p, body).unwrap();
}

const PASSING_PAGE: &str = "<!doctype html><html lang=\"en\"><head>\
<meta charset=\"utf-8\">\
<title>Hello</title>\
<meta name=\"description\" content=\"d\">\
<meta property=\"og:title\" content=\"Hello\">\
<meta property=\"og:type\" content=\"website\">\
<meta property=\"og:image\" content=\"/og.png\">\
<meta name=\"twitter:card\" content=\"summary\">\
<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
</head><body><main><h1>Hello</h1><img src=\"/a.jpg\" alt=\"a\" width=\"10\" height=\"10\"></main></body></html>";

// =====================================================================
// 14 gates × 2 (positive + negative) = 28 baseline tests
// =====================================================================

#[test]
fn wcag_positive_clean_page_has_no_findings() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let f = gates::wcag::WcagGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "expected clean WCAG, got {f:?}");
}

#[test]
fn wcag_negative_missing_alt_is_flagged() {
    let html = "<!doctype html><html lang=en><head><title>x</title></head><body><main><h1>x</h1><img src=a.jpg></main></body></html>";
    let (_tmp, site) = one_page(html);
    let f = gates::wcag::WcagGate.run(&site, &AuditOptions::default());
    assert!(f.iter().any(|x| x.code.as_deref() == Some("WCAG-1.1.1")));
}

#[test]
fn jsonld_positive_valid_organization_passes() {
    let html = "<html><head><script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Organization\",\"name\":\"X\",\"url\":\"https://x.test\"}</script></head><body></body></html>";
    let (_tmp, site) = one_page(html);
    let f = gates::jsonld::JsonLdGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn jsonld_negative_unparseable_json_is_flagged() {
    let html = r#"<html><head><script type="application/ld+json">{ not json }</script></head><body></body></html>"#;
    let (_tmp, site) = one_page(html);
    let f = gates::jsonld::JsonLdGate.run(&site, &AuditOptions::default());
    assert!(f.iter().any(|x| matches!(x.severity, Severity::Error)));
}

#[test]
fn hreflang_positive_reciprocal_pair_is_clean() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "en/index.html",
        b"<html><head><link rel=\"alternate\" hreflang=\"en\" href=\"/en/index.html\" data-self=\"true\"><link rel=\"alternate\" hreflang=\"fr\" href=\"/fr/index.html\"></head></html>",
    );
    write_file(
        tmp.path(),
        "fr/index.html",
        b"<html><head><link rel=\"alternate\" hreflang=\"fr\" href=\"/fr/index.html\" data-self=\"true\"><link rel=\"alternate\" hreflang=\"en\" href=\"/en/index.html\"></head></html>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::hreflang::HreflangGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn hreflang_negative_missing_reciprocal_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "en/index.html",
        b"<html><head><link rel=\"alternate\" hreflang=\"en\" href=\"/en/index.html\" data-self=\"true\"><link rel=\"alternate\" hreflang=\"fr\" href=\"/fr/index.html\"></head></html>",
    );
    write_file(
        tmp.path(),
        "fr/index.html",
        b"<html><head><link rel=\"alternate\" hreflang=\"fr\" href=\"/fr/index.html\" data-self=\"true\"></head></html>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::hreflang::HreflangGate.run(&site, &AuditOptions::default());
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("HREFLANG-NO-RECIPROCAL")));
}

#[test]
fn csp_sri_positive_well_secured_page_is_clean() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let f = gates::csp_sri::CspSriGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn csp_sri_negative_missing_csp_is_flagged() {
    let html = "<html><head><script src=\"https://cdn.test/x.js\"></script></head><body></body></html>";
    let (_tmp, site) = one_page(html);
    let f = gates::csp_sri::CspSriGate.run(&site, &AuditOptions::default());
    assert!(f.iter().any(|x| x.code.as_deref() == Some("CSP-MISSING")));
    assert!(f.iter().any(|x| x.code.as_deref() == Some("SRI-MISSING")));
}

#[test]
fn pqc_tls_positive_good_headers_passes() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "_headers",
        b"/*\n  Strict-Transport-Security: max-age=31536000\n  TLS: TLSv1.3\n",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::pqc_tls::PqcTlsGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn pqc_tls_negative_short_hsts_is_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "_headers",
        b"/*\n  Strict-Transport-Security: max-age=3600\n  TLS: TLSv1.3\n",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::pqc_tls::PqcTlsGate.run(&site, &AuditOptions::default());
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("PQC-HSTS-SHORT")));
}

#[test]
fn html5_positive_well_formed_page_passes() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let f = gates::html5::Html5Gate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn html5_negative_skeletal_page_is_flagged() {
    let html = "<html><body><p>x</p></body></html>";
    let (_tmp, site) = one_page(html);
    let f = gates::html5::Html5Gate.run(&site, &AuditOptions::default());
    let codes: Vec<_> = f.iter().filter_map(|x| x.code.as_deref()).collect();
    assert!(codes.contains(&"HTML5-DOCTYPE"));
    assert!(codes.contains(&"HTML5-H1-MISSING"));
    assert!(codes.contains(&"HTML5-TITLE-MISSING"));
    assert!(codes.contains(&"HTML5-CHARSET"));
}

#[test]
fn links_positive_resolving_internal_link_is_clean() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "index.html",
        b"<html><body><a href=\"/about/\">a</a></body></html>",
    );
    write_file(
        tmp.path(),
        "about/index.html",
        b"<html><body></body></html>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::broken_links::BrokenLinksGate
        .run(&site, &AuditOptions::default());
    let errors: Vec<_> = f
        .iter()
        .filter(|x| matches!(x.severity, Severity::Error))
        .collect();
    assert!(errors.is_empty(), "got {errors:?}");
}

#[test]
fn links_negative_dangling_internal_link_flagged() {
    let html = r#"<html><body><a href="/missing/">x</a></body></html>"#;
    let (_tmp, site) = one_page(html);
    let f = gates::broken_links::BrokenLinksGate
        .run(&site, &AuditOptions::default());
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("LINK-INTERNAL-MISSING")));
}

#[test]
fn metadata_positive_complete_og_passes() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let f = gates::metadata::MetadataGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn metadata_negative_missing_og_trio_flagged() {
    let html = "<html><head><title>x</title></head><body></body></html>";
    let (_tmp, site) = one_page(html);
    let f = gates::metadata::MetadataGate.run(&site, &AuditOptions::default());
    let codes: Vec<_> = f.iter().filter_map(|x| x.code.as_deref()).collect();
    assert!(codes.contains(&"OG-TITLE"));
    assert!(codes.contains(&"OG-TYPE"));
    assert!(codes.contains(&"OG-IMAGE"));
    assert!(codes.contains(&"META-DESCRIPTION"));
}

#[test]
fn markdownlint_positive_clean_md_is_clean() {
    let tmp = tempfile::tempdir().unwrap();
    let site_root = tmp.path().join("public");
    let content = tmp.path().join("content");
    fs::create_dir_all(&site_root).unwrap();
    fs::create_dir_all(&content).unwrap();
    fs::write(content.join("a.md"), "# Title\n\nBody.\n").unwrap();
    let site = Site::load(&site_root).unwrap();
    let f = gates::markdownlint::MarkdownlintGate
        .run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn markdownlint_negative_md_with_bad_styles_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let site_root = tmp.path().join("public");
    let content = tmp.path().join("content");
    fs::create_dir_all(&site_root).unwrap();
    fs::create_dir_all(&content).unwrap();
    fs::write(
        content.join("a.md"),
        "## not h1\nhttps://bare-url.test\n\ttab line\ntrailing ws \n",
    )
    .unwrap();
    let site = Site::load(&site_root).unwrap();
    let f = gates::markdownlint::MarkdownlintGate
        .run(&site, &AuditOptions::default());
    let codes: Vec<_> = f.iter().filter_map(|x| x.code.as_deref()).collect();
    assert!(codes.contains(&"MD041"));
    assert!(codes.contains(&"MD034"));
    assert!(codes.contains(&"MD010"));
    assert!(codes.contains(&"MD009"));
}

#[test]
fn performance_positive_tiny_page_under_budget() {
    let (_tmp, site) = one_page("<html></html>");
    let f = gates::performance::PerformanceGate
        .run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn performance_negative_oversized_html_flagged() {
    let html = "<html>".to_string() + &"x".repeat(10_000) + "</html>";
    let (_tmp, site) = one_page(&html);
    let f = gates::performance::PerformanceGate.run(
        &site,
        &AuditOptions {
            page_weight_budget: 100,
            ..AuditOptions::default()
        },
    );
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("PERF-PAGE-OVER")));
}

#[test]
fn ai_discovery_positive_full_set_passes() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), "llms.txt", b"# llms\n");
    write_file(tmp.path(), "agents.txt", b"# agents\n");
    write_file(
        tmp.path(),
        ".well-known/ai-plugin.json",
        br#"{"schema_version":"v1"}"#,
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::ai_discovery::AiDiscoveryGate
        .run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn ai_discovery_negative_e8_files_missing_emits_info() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), "llms.txt", b"# llms\n");
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::ai_discovery::AiDiscoveryGate
        .run(&site, &AuditOptions::default());
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("AI-AGENTS-MISSING")));
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("AI-PLUGIN-JSON-MISSING")));
    // Issue contract: missing E8 files emit info (not error).
    let e8 = f.iter().filter(|x| {
        matches!(
            x.code.as_deref(),
            Some("AI-AGENTS-MISSING" | "AI-PLUGIN-JSON-MISSING")
        )
    });
    for finding in e8 {
        assert!(matches!(finding.severity, Severity::Info));
    }
}

#[test]
fn feeds_positive_valid_rss_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let body = b"<?xml version=\"1.0\"?>\n<rss version=\"2.0\"><channel><title>x</title><link>https://x</link><description>d</description><item><title>i</title></item></channel></rss>";
    write_file(tmp.path(), "rss.xml", body);
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::feeds::FeedsGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn feeds_negative_empty_rss_channel_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "rss.xml",
        b"<?xml version=\"1.0\"?><rss version=\"2.0\"><channel></channel></rss>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::feeds::FeedsGate.run(&site, &AuditOptions::default());
    assert!(f.iter().any(|x| x.code.as_deref() == Some("RSS-TITLE")));
}

#[test]
fn images_positive_alt_and_modern_format_passes() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.jpg"), vec![0u8; 10]).unwrap();
    fs::write(tmp.path().join("a.webp"), vec![0u8; 10]).unwrap();
    write_file(
        tmp.path(),
        "index.html",
        b"<html><body><img src=\"/a.jpg\" alt=\"a\" width=\"10\" height=\"10\"></body></html>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::images::ImagesGate.run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn images_negative_missing_alt_flags_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.jpg"), vec![0u8; 10]).unwrap();
    fs::write(tmp.path().join("a.webp"), vec![0u8; 10]).unwrap();
    write_file(
        tmp.path(),
        "index.html",
        b"<html><body><img src=\"/a.jpg\" width=\"10\" height=\"10\"></body></html>",
    );
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::images::ImagesGate.run(&site, &AuditOptions::default());
    assert!(f.iter().any(|x| x.code.as_deref() == Some("IMG-ALT")));
}

#[test]
fn search_index_positive_matching_hash_passes() {
    use sha2::{Digest, Sha256};
    let tmp = tempfile::tempdir().unwrap();
    let search = tmp.path().join("search");
    fs::create_dir_all(&search).unwrap();
    let body = b"vectors";
    fs::write(search.join("embeddings.bin"), body).unwrap();
    let mut h = Sha256::new();
    h.update(body);
    let mut hex = String::with_capacity(64);
    for byte in h.finalize() {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    fs::write(
        search.join("manifest.json"),
        format!(r#"{{"embeddings_sha256":"{hex}"}}"#),
    )
    .unwrap();
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::search_index::SearchIndexGate
        .run(&site, &AuditOptions::default());
    assert!(f.is_empty(), "got {f:?}");
}

#[test]
fn search_index_negative_hash_mismatch_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let search = tmp.path().join("search");
    fs::create_dir_all(&search).unwrap();
    fs::write(search.join("embeddings.bin"), b"x").unwrap();
    fs::write(
        search.join("manifest.json"),
        r#"{"embeddings_sha256":"deadbeef"}"#,
    )
    .unwrap();
    let site = Site::load(tmp.path()).unwrap();
    let f = gates::search_index::SearchIndexGate
        .run(&site, &AuditOptions::default());
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("SEARCH-HASH-MISMATCH")));
}

// =====================================================================
// Acceptance criteria — end-to-end scenarios
// =====================================================================

#[test]
fn ac1_audit_runs_all_fourteen_gates() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let report = AuditRunner::new(AuditConfig::new()).run(&site);
    assert_eq!(report.gates.len(), 14, "must run 14 gates");
}

#[test]
fn ac2_single_gate_filter_executes_only_that_gate() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let cfg = AuditConfig {
        only: Some("hreflang".to_string()),
        ..AuditConfig::new()
    };
    let report = AuditRunner::new(cfg).run(&site);
    let executed: Vec<_> = report.gates.iter().filter(|g| !g.skipped).collect();
    assert_eq!(executed.len(), 1);
    assert_eq!(executed[0].name, "hreflang");
}

#[test]
fn ac3_json_output_has_stable_shape() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let report = AuditRunner::new(AuditConfig::new()).run(&site);
    let json = ssg::audit::output::json::format(&report).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["gates"].is_array());
    assert_eq!(v["gates"].as_array().unwrap().len(), 14);
    let first = &v["gates"][0];
    assert!(first["name"].is_string());
    assert!(first["severity_counts"]["error"].is_number());
    assert!(first["findings"].is_array());
}

#[test]
fn ac4_severity_floor_suppresses_lower_findings() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    // Run with severity_floor = Error: only errors should appear.
    let cfg = AuditConfig {
        severity_floor: Severity::Error,
        ..AuditConfig::new()
    };
    let report = AuditRunner::new(cfg).run(&site);
    for gate in &report.gates {
        for f in &gate.findings {
            assert!(
                matches!(f.severity, Severity::Error),
                "non-error finding survived: {f:?}"
            );
        }
    }
}

#[test]
fn ac5_disabled_gate_via_config_is_skipped_with_info_line() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let mut disabled = BTreeSet::new();
    let _ = disabled.insert("markdownlint".to_string());
    let cfg = AuditConfig {
        disabled,
        ..AuditConfig::new()
    };
    let report = AuditRunner::new(cfg).run(&site);
    let g = report
        .gates
        .iter()
        .find(|g| g.name == "markdownlint")
        .unwrap();
    assert!(g.skipped);
    assert!(g.skip_reason.as_deref().unwrap_or("").contains("disabled"));
}

#[test]
fn ac6_fail_on_warn_returns_should_fail_true() {
    // Construct a synthetic report containing exactly one warn.
    use ssg::audit::{AuditReport, GateResult, SeverityCounts};
    let report = AuditReport {
        gates: vec![GateResult {
            name: "x".to_string(),
            skipped: false,
            skip_reason: None,
            severity_counts: SeverityCounts {
                info: 0,
                warn: 1,
                error: 0,
            },
            findings: vec![],
        }],
    };
    assert!(report.should_fail(Severity::Warn));
    assert!(!report.should_fail(Severity::Error));
}

#[test]
fn ac7_junit_output_is_valid_xml_with_per_gate_testsuite() {
    let (_tmp, site) = one_page(PASSING_PAGE);
    let report = AuditRunner::new(AuditConfig::new()).run(&site);
    let xml = ssg::audit::output::junit::format(&report);
    assert!(xml.starts_with("<?xml version=\"1.0\""));
    assert_eq!(xml.matches("<testsuite ").count(), 14);
    assert!(xml.contains("</testsuites>"));
}

#[test]
fn ac8_every_gate_has_an_explainer() {
    for gate in gates::all() {
        assert!(
            !gate.explain().trim().is_empty(),
            "gate `{}` has empty explainer",
            gate.name()
        );
    }
}

#[test]
fn ac9_skip_network_results_in_zero_http_requests() {
    // The broken-link gate is the only network-touching gate. When
    // skip_network=true it must emit the LINK-EXTERNAL-SKIPPED info
    // line for any external href encountered, never issuing HTTP.
    let html =
        r#"<html><body><a href="https://example.com">x</a></body></html>"#;
    let (_tmp, site) = one_page(html);
    let f = gates::broken_links::BrokenLinksGate.run(
        &site,
        &AuditOptions {
            skip_network: true,
            ..AuditOptions::default()
        },
    );
    assert!(f
        .iter()
        .any(|x| x.code.as_deref() == Some("LINK-EXTERNAL-SKIPPED")));
    // No error findings should appear from network probing failures.
    for finding in &f {
        assert!(!matches!(finding.severity, Severity::Error));
    }
}

#[test]
fn ac10_explainer_text_for_named_gates() {
    for name in [
        "wcag",
        "jsonld",
        "hreflang",
        "csp_sri",
        "pqc_tls",
        "html5",
        "links",
        "metadata",
        "markdownlint",
        "performance",
        "ai_discovery",
        "feeds",
        "images",
        "search_index",
    ] {
        let g = gates::all()
            .into_iter()
            .find(|g| g.name() == name)
            .unwrap_or_else(|| panic!("gate `{name}` not registered"));
        assert!(g.explain().len() > 20, "gate `{name}` explainer too short");
    }
}
