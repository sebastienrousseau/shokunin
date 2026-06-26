// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WASM bridge for the Edge RPC dispatcher (issue #548).
//!
//! The TypeScript Worker imports `rpc_dispatch(name, payload)` and
//! gets back a `{ status, body }` pair. The Worker then decides on
//! HTTP framing — content-type, headers, CORS — keeping all
//! HTTP-level concerns on the TypeScript side and all Rust-level
//! routing on the Rust side.
//!
//! The dispatcher itself lives in the `ssg-rpc` crate (alongside the
//! `#[ssg_rpc]` proc-macro); we expose a slim wasm-bindgen façade
//! here so users only need to add `ssg-wasm` to their Worker.

use wasm_bindgen::prelude::*;

use ssg_rpc::dispatch;

/// Wire-format response from `rpc_dispatch`.
///
/// Returned as a JSON string from WASM. Fields match the shape the
/// JS client expects:
///
/// ```json
/// { "status": 200, "body": "{\"likes\": 1}" }
/// ```
///
/// On error, `body` is the standard `{"error": "..."}` JSON.
///
/// # Examples
///
/// ```
/// let resp = ssg_wasm::rpc_dispatch_impl("__missing__", "{}");
/// assert_eq!(resp.status, 404);
/// assert!(resp.body.contains("not found"));
/// ```
#[derive(Debug)]
pub struct RpcResponse {
    /// HTTP status code the Worker should respond with.
    pub status: u16,
    /// JSON response body.
    pub body: String,
}

/// Pure-Rust dispatcher used by both the WASM façade and the
/// native integration tests.
///
/// Maps a registered RPC name + JSON payload to an HTTP-shaped
/// response. Always returns a body — even on `Err`, the body is the
/// canonical `{"error": "..."}` document.
///
/// # Examples
///
/// ```
/// let resp = ssg_wasm::rpc_dispatch_impl("__nope__", "{}");
/// assert_eq!(resp.status, 404);
/// let v: serde_json::Value =
///     serde_json::from_str(&resp.body).unwrap();
/// assert_eq!(v["error"], "not found");
/// ```
#[must_use]
pub fn rpc_dispatch_impl(name: &str, payload: &str) -> RpcResponse {
    match dispatch(name, payload) {
        Ok(body) => RpcResponse { status: 200, body },
        Err(err) => RpcResponse {
            status: err.status_code(),
            body: err.to_wire_body(),
        },
    }
}

/// JS-facing entrypoint. Returns a JSON string of the form
/// `{"status": <int>, "body": "<json>"}` which the Worker parses
/// to choose the HTTP status + Content-Type.
///
/// # Examples
///
/// ```
/// let raw = ssg_wasm::rpc_dispatch("__nope__", "{}");
/// let env: serde_json::Value = serde_json::from_str(&raw).unwrap();
/// assert_eq!(env["status"], 404);
/// let body_str = env["body"].as_str().unwrap();
/// let body: serde_json::Value =
///     serde_json::from_str(body_str).unwrap();
/// assert_eq!(body["error"], "not found");
/// ```
#[wasm_bindgen]
pub fn rpc_dispatch(name: &str, payload: &str) -> String {
    let resp = rpc_dispatch_impl(name, payload);
    // Hand-rolled to avoid a serde_json round-trip; the body is
    // already JSON so we splice it in directly.
    format!(
        "{{\"status\":{status},\"body\":{body}}}",
        status = resp.status,
        // `body` is itself JSON — wrap it in a JSON string so the
        // outer envelope is a single parseable document. We escape
        // backslashes + quotes; everything else is JSON-safe by
        // construction.
        body = json_string_literal(&resp.body),
    )
}

fn json_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Canonical body for an HTTP 405 reply.
///
/// AC6 (method whitelist): the Worker performs the actual method
/// check in TypeScript, but exposing the canonical body here keeps
/// the wire format identical across all error paths
/// (400/401/404/405/500).
///
/// # Examples
///
/// ```
/// let body = ssg_wasm::rpc::method_not_allowed_body();
/// let v: serde_json::Value = serde_json::from_str(body).unwrap();
/// assert_eq!(v["error"], "method not allowed");
/// ```
#[must_use]
pub const fn method_not_allowed_body() -> &'static str {
    "{\"error\":\"method not allowed\"}"
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn unknown_rpc_returns_404_envelope() {
        let resp = rpc_dispatch_impl("__missing__", "{}");
        assert_eq!(resp.status, 404);
        let v: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
        assert_eq!(v["error"], "not found");
    }

    #[test]
    fn json_string_literal_escapes_quotes_and_backslashes() {
        let lit = json_string_literal(r#"a"b\c"#);
        assert_eq!(lit, r#""a\"b\\c""#);
    }

    #[test]
    fn json_string_literal_escapes_control_chars() {
        let lit = json_string_literal("a\nb\tc");
        assert!(lit.contains("\\n"), "{lit}");
        assert!(lit.contains("\\t"), "{lit}");
    }

    #[test]
    fn rpc_dispatch_envelope_is_valid_json() {
        let raw = rpc_dispatch("__missing__", "{}");
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["status"], 404);
        // body is itself JSON-as-a-string in the envelope.
        let body_str = v["body"].as_str().unwrap();
        let body: serde_json::Value = serde_json::from_str(body_str).unwrap();
        assert_eq!(body["error"], "not found");
    }

    #[test]
    fn method_not_allowed_is_canonical() {
        let body = method_not_allowed_body();
        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(v["error"], "method not allowed");
    }
}
