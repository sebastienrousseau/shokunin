// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Netlify `_headers` emitter (issue #550 AC2).
//!
//! Netlify's `_headers` file is a plain-text format with route-prefix
//! groups (`/*` matches every path) and indented `Header: value`
//! lines. Comments (`#`) and blank lines are tolerated by Netlify's
//! parser, so the PQC documentation block sits at the top of the
//! file and the route group follows underneath.

use super::PQC_NOTE_LINES;
use std::collections::BTreeMap;

/// Renders the Netlify `_headers` body.
///
/// Format:
///
/// ```text
/// # PQC docstring (AC6)
/// /*
///   Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
///   Content-Security-Policy: …
///   …
///
/// /blog/post/
///   Content-Security-Policy: … 'sha256-…' …
/// ```
///
/// `page_csp` adds one route group per page that carries a
/// hash-strict per-page CSP (spec B4); groups render in the map's
/// sorted order so output is deterministic. **Platform caveat:**
/// Netlify merges headers from *every* matching rule into one
/// comma-separated value, and a comma-joined CSP is enforced as
/// multiple policies (all must pass). Deployments that rely on the
/// per-page hashes should drop the global `Content-Security-Policy`
/// via `[edge_headers.overrides]` or dedupe at the platform level;
/// the per-path entries themselves are correct and self-contained.
#[must_use]
pub(super) fn render(
    headers: &[(String, String)],
    page_csp: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();

    // PQC docstring (AC6).
    out.push_str("# ssg edge-headers: Netlify _headers\n");
    out.push_str("# ----------------------------------\n");
    for line in PQC_NOTE_LINES {
        out.push_str(&format!("# {line}\n"));
    }
    out.push('\n');

    out.push_str("/*\n");
    for (key, value) in headers {
        out.push_str(&format!("  {key}: {value}\n"));
    }

    // Per-page CSP route groups (spec B4) — sorted BTreeMap order.
    for (path, policy) in page_csp {
        out.push('\n');
        out.push_str(&format!("{path}\n"));
        out.push_str(&format!("  Content-Security-Policy: {policy}\n"));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postprocess::edge_headers::merged_headers;
    use std::collections::BTreeMap;

    #[test]
    fn render_includes_pqc_docstring() {
        let body = render(&merged_headers(&BTreeMap::new()), &BTreeMap::new());
        assert!(body.contains("X25519+ML-KEM-768"));
        assert!(body.contains("netlify.com"));
    }

    #[test]
    fn render_uses_wildcard_route_group() {
        let body = render(&merged_headers(&BTreeMap::new()), &BTreeMap::new());
        let route_line = body
            .lines()
            .find(|l| !l.starts_with('#') && !l.trim().is_empty());
        assert_eq!(route_line, Some("/*"));
    }

    #[test]
    fn render_indents_headers_under_route() {
        let body = render(&merged_headers(&BTreeMap::new()), &BTreeMap::new());
        assert!(body.contains("  Strict-Transport-Security:"));
        assert!(body.contains("  Content-Security-Policy:"));
        assert!(body.contains("  X-Content-Type-Options: nosniff"));
        assert!(
            body.contains("  Referrer-Policy: strict-origin-when-cross-origin")
        );
        assert!(body.contains("  Permissions-Policy:"));
    }

    #[test]
    fn render_has_no_duplicate_csp_header() {
        // AC7: even with overrides set on a non-CSP header, only one
        // Content-Security-Policy must be emitted.
        let body = render(&merged_headers(&BTreeMap::new()), &BTreeMap::new());
        let csp_count = body
            .lines()
            .filter(|l| l.contains("Content-Security-Policy:"))
            .count();
        assert_eq!(csp_count, 1);
    }

    #[test]
    fn render_with_empty_headers_still_emits_route_group() {
        let body = render(&[], &BTreeMap::new());
        assert!(body.contains("/*\n"));
        assert!(body.starts_with("# ssg edge-headers: Netlify _headers"));
    }

    #[test]
    fn render_preserves_input_header_order() {
        let headers = vec![
            ("First".to_string(), "1".to_string()),
            ("Second".to_string(), "2".to_string()),
            ("Third".to_string(), "3".to_string()),
        ];
        let body = render(&headers, &BTreeMap::new());
        let i_first = body.find("First:").unwrap();
        let i_second = body.find("Second:").unwrap();
        let i_third = body.find("Third:").unwrap();
        assert!(i_first < i_second && i_second < i_third);
    }

    #[test]
    fn render_starts_with_docstring_then_blank_line_then_route() {
        let body = render(&[], &BTreeMap::new());
        let lines: Vec<&str> = body.lines().collect();
        // First line must be a comment.
        assert!(lines[0].starts_with('#'));
        // Find the first blank line — it must precede `/*`.
        let blank_idx = lines
            .iter()
            .position(|l| l.is_empty())
            .expect("expected a blank line between docstring and route");
        assert_eq!(lines[blank_idx + 1], "/*");
    }

    #[test]
    fn render_emits_pqc_note_lines_as_comment_block() {
        let body = render(&[], &BTreeMap::new());
        for line in PQC_NOTE_LINES {
            let formatted = format!("# {line}");
            assert!(body.contains(&formatted), "missing PQC line: {formatted}");
        }
    }

    #[test]
    fn render_includes_link_to_netlify_docs() {
        let body = render(&[], &BTreeMap::new());
        assert!(body.contains("docs.netlify.com"));
    }

    #[test]
    fn render_appends_per_page_csp_groups_sorted() {
        // spec B4: pages with inline hashes get their own route group.
        let mut pages: BTreeMap<String, String> = BTreeMap::new();
        let _ = pages.insert(
            "/blog/post/".to_string(),
            "script-src 'self' 'sha256-abc'".to_string(),
        );
        let _ = pages.insert(
            "/about/".to_string(),
            "script-src 'self' 'sha256-def'".to_string(),
        );
        let body = render(&merged_headers(&BTreeMap::new()), &pages);
        let i_about = body.find("/about/\n").unwrap();
        let i_blog = body.find("/blog/post/\n").unwrap();
        assert!(i_about < i_blog, "groups must render in sorted order");
        assert!(body
            .contains("/blog/post/\n  Content-Security-Policy: script-src 'self' 'sha256-abc'"));
    }

    #[test]
    fn render_without_pages_has_no_extra_route_groups() {
        let body = render(&merged_headers(&BTreeMap::new()), &BTreeMap::new());
        let route_groups = body
            .lines()
            .filter(|l| l.starts_with('/') && !l.starts_with("/*"))
            .count();
        assert_eq!(route_groups, 0);
    }

    #[test]
    fn render_with_override_value_appears_in_output() {
        let mut overrides: BTreeMap<String, String> = BTreeMap::new();
        let _ = overrides.insert(
            "Permissions-Policy".to_string(),
            "geolocation=(self)".to_string(),
        );
        let body = render(&merged_headers(&overrides), &BTreeMap::new());
        assert!(body.contains("Permissions-Policy: geolocation=(self)"));
    }
}
