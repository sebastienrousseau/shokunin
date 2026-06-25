// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Vercel `vercel.json` headers-array emitter (issue #550 AC3).
//!
//! Vercel's deployment config doesn't tolerate JSON comments, so the
//! PQC documentation is carried as a top-level `_pqc_note` array of
//! strings (Vercel ignores keys it doesn't recognise). The
//! canonical `headers` array follows the Vercel schema verbatim:
//!
//! ```json
//! {
//!   "_pqc_note": ["…"],
//!   "headers": [
//!     {
//!       "source": "/(.*)",
//!       "headers": [
//!         { "key": "Strict-Transport-Security", "value": "…" },
//!         …
//!       ]
//!     }
//!   ]
//! }
//! ```

use super::PQC_NOTE_LINES;
use serde_json::{json, Value};

/// Renders the Vercel JSON body.
///
/// # Errors
///
/// Returns `serde_json::Error` only if the produced `Value` fails to
/// serialise, which in practice never happens because the structure
/// is built from owned strings.
pub(super) fn render(
    headers: &[(String, String)],
) -> Result<String, serde_json::Error> {
    let headers_arr: Vec<Value> = headers
        .iter()
        .map(|(k, v)| json!({ "key": k, "value": v }))
        .collect();

    let pqc_note: Vec<Value> = PQC_NOTE_LINES
        .iter()
        .map(|s| Value::String((*s).into()))
        .collect();

    let config = json!({
        "_pqc_note": pqc_note,
        "headers": [
            {
                "source": "/(.*)",
                "headers": headers_arr,
            }
        ]
    });

    serde_json::to_string_pretty(&config)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::postprocess::edge_headers::merged_headers;
    use std::collections::BTreeMap;

    #[test]
    fn render_includes_pqc_note_field() {
        let body = render(&merged_headers(&BTreeMap::new())).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let note = parsed.get("_pqc_note").and_then(|v| v.as_array()).unwrap();
        let joined: String = note
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(joined.contains("X25519+ML-KEM-768"));
        assert!(joined.contains("vercel.com"));
    }

    #[test]
    fn render_uses_canonical_headers_array_shape() {
        let body = render(&merged_headers(&BTreeMap::new())).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let group = parsed
            .get("headers")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .unwrap();
        assert_eq!(group.get("source").and_then(|v| v.as_str()), Some("/(.*)"));
        let arr = group.get("headers").and_then(|v| v.as_array()).unwrap();
        assert_eq!(arr.len(), 5);
        for item in arr {
            assert!(item.get("key").is_some(), "each entry needs a key");
            assert!(item.get("value").is_some(), "each entry needs a value");
        }
    }

    #[test]
    fn render_emits_one_csp_entry_only() {
        // AC7: no duplicate Content-Security-Policy across the array.
        let body = render(&merged_headers(&BTreeMap::new())).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let arr = parsed["headers"][0]["headers"].as_array().unwrap();
        let count = arr
            .iter()
            .filter(|h| {
                h.get("key").and_then(|k| k.as_str()).is_some_and(|s| {
                    s.eq_ignore_ascii_case("Content-Security-Policy")
                })
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn render_keys_are_in_baseline_order() {
        let body = render(&merged_headers(&BTreeMap::new())).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let arr = parsed["headers"][0]["headers"].as_array().unwrap();
        let keys: Vec<&str> =
            arr.iter().map(|h| h["key"].as_str().unwrap()).collect();
        assert_eq!(
            keys,
            vec![
                "Strict-Transport-Security",
                "Content-Security-Policy",
                "X-Content-Type-Options",
                "Referrer-Policy",
                "Permissions-Policy",
            ]
        );
    }
}
