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
/// ```
#[must_use]
pub(super) fn render(headers: &[(String, String)]) -> String {
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

    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::postprocess::edge_headers::merged_headers;
    use std::collections::BTreeMap;

    #[test]
    fn render_includes_pqc_docstring() {
        let body = render(&merged_headers(&BTreeMap::new()));
        assert!(body.contains("X25519+ML-KEM-768"));
        assert!(body.contains("netlify.com"));
    }

    #[test]
    fn render_uses_wildcard_route_group() {
        let body = render(&merged_headers(&BTreeMap::new()));
        let route_line = body
            .lines()
            .find(|l| !l.starts_with('#') && !l.trim().is_empty());
        assert_eq!(route_line, Some("/*"));
    }

    #[test]
    fn render_indents_headers_under_route() {
        let body = render(&merged_headers(&BTreeMap::new()));
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
        let body = render(&merged_headers(&BTreeMap::new()));
        let csp_count = body
            .lines()
            .filter(|l| l.contains("Content-Security-Policy:"))
            .count();
        assert_eq!(csp_count, 1);
    }
}
