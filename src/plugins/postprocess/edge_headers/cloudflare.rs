// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cloudflare Workers `wrangler.toml [headers]` emitter (issue #550 AC1).
//!
//! Produces a TOML snippet that site authors merge into their
//! `wrangler.toml` under a `[[headers]]` array entry matching every
//! route (`for = "/*"`). The header values themselves come from
//! [`super::merged_headers`] so platform syntax stays in lock-step
//! with the Netlify and Vercel emitters.

use super::PQC_NOTE_LINES;

/// Renders the Cloudflare-flavoured TOML snippet.
///
/// The output is structured as:
///
/// ```toml
/// # PQC docstring (8 lines + URLs) — AC6
/// # How to merge — instructional comment
///
/// [[headers]]
///   for = "/*"
///   [headers.values]
///     "Strict-Transport-Security" = "…"
///     "Content-Security-Policy"   = "…"
///     …
/// ```
///
/// Header values are escaped for TOML basic strings by replacing `"`
/// with `\"`. CSP values can legitimately contain `'self'`,
/// semicolons, and spaces — none of which need escaping — but the
/// `"` guard is kept for forward-compat with override values that
/// may.
#[must_use]
pub(super) fn render(headers: &[(String, String)]) -> String {
    let mut out = String::new();

    // PQC docstring + intent (AC6).
    out.push_str(
        "# ssg edge-headers: Cloudflare Workers wrangler.toml snippet\n",
    );
    out.push_str(
        "# ------------------------------------------------------------\n",
    );
    for line in PQC_NOTE_LINES {
        out.push_str(&format!("# {line}\n"));
    }
    out.push('#');
    out.push('\n');
    out.push_str(
        "# Merge this file into your project's wrangler.toml under a\n",
    );
    out.push_str(
        "# [[headers]] block; the snippet below is self-contained and\n",
    );
    out.push_str("# can be appended verbatim.\n");
    out.push('\n');

    out.push_str("[[headers]]\n");
    out.push_str("  for = \"/*\"\n");
    out.push_str("  [headers.values]\n");
    for (key, value) in headers {
        let escaped = value.replace('"', "\\\"");
        out.push_str(&format!("    \"{key}\" = \"{escaped}\"\n"));
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
        assert!(body.contains("cloudflare.com/ssl/post-quantum"));
    }

    #[test]
    fn render_produces_valid_toml_structure() {
        let body = render(&merged_headers(&BTreeMap::new()));
        assert!(body.contains("[[headers]]"));
        assert!(body.contains("for = \"/*\""));
        assert!(body.contains("[headers.values]"));
    }

    #[test]
    fn render_includes_all_five_baseline_headers() {
        let body = render(&merged_headers(&BTreeMap::new()));
        for key in [
            "Strict-Transport-Security",
            "Content-Security-Policy",
            "X-Content-Type-Options",
            "Referrer-Policy",
            "Permissions-Policy",
        ] {
            assert!(body.contains(key), "missing {key}");
        }
    }

    #[test]
    fn render_parses_as_valid_toml() {
        let body = render(&merged_headers(&BTreeMap::new()));
        let parsed = toml::from_str::<toml::Value>(&body);
        assert!(parsed.is_ok(), "should be valid TOML: {parsed:?}");
    }

    #[test]
    fn render_escapes_embedded_quotes() {
        let headers =
            vec![("X-Custom".to_string(), "value with \"quotes\"".to_string())];
        let body = render(&headers);
        assert!(body.contains("value with \\\"quotes\\\""));
        // Round-trip through the TOML parser to prove escaping works.
        let parsed: toml::Value = toml::from_str(&body).unwrap();
        let v = parsed
            .get("headers")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|h| h.get("values"))
            .and_then(|v| v.get("X-Custom"))
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(v, "value with \"quotes\"");
    }

    #[test]
    fn render_with_empty_headers_emits_only_skeleton() {
        let body = render(&[]);
        assert!(body.contains("[[headers]]"));
        assert!(body.contains("for = \"/*\""));
        assert!(body.contains("[headers.values]"));
        // Skeleton is still valid TOML.
        let _: toml::Value = toml::from_str(&body).unwrap();
    }

    #[test]
    fn render_includes_merge_instruction_comment() {
        let body = render(&merged_headers(&BTreeMap::new()));
        assert!(
            body.contains("Merge this file into your project's wrangler.toml")
        );
    }

    #[test]
    fn render_emits_pqc_note_lines_as_comment_block() {
        let body = render(&[]);
        for line in PQC_NOTE_LINES {
            let formatted = format!("# {line}");
            assert!(body.contains(&formatted), "missing PQC line: {formatted}");
        }
    }

    #[test]
    fn render_preserves_input_header_order() {
        let headers = vec![
            ("Aaa".to_string(), "1".to_string()),
            ("Bbb".to_string(), "2".to_string()),
            ("Ccc".to_string(), "3".to_string()),
        ];
        let body = render(&headers);
        let i_a = body.find("\"Aaa\"").unwrap();
        let i_b = body.find("\"Bbb\"").unwrap();
        let i_c = body.find("\"Ccc\"").unwrap();
        assert!(i_a < i_b && i_b < i_c);
    }

    #[test]
    fn render_round_trips_baseline_headers_through_toml() {
        let baseline = merged_headers(&BTreeMap::new());
        let body = render(&baseline);
        let parsed: toml::Value = toml::from_str(&body).unwrap();
        let values = parsed
            .get("headers")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|h| h.get("values"))
            .and_then(|v| v.as_table())
            .unwrap();
        // Each of the 5 baseline keys must be present.
        for (key, value) in &baseline {
            let parsed_value =
                values.get(key).and_then(|v| v.as_str()).unwrap_or_else(|| {
                    panic!("missing baseline key {key} in TOML output")
                });
            assert_eq!(parsed_value, value);
        }
    }

    #[test]
    fn render_starts_with_module_header_comment() {
        let body = render(&[]);
        assert!(body.starts_with(
            "# ssg edge-headers: Cloudflare Workers wrangler.toml snippet"
        ));
    }
}
