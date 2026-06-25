// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration test covering AC1, AC2, AC3 of issue #548.
//!
//! AC1 — `#[ssg_rpc]` macro generates registration + schema.
//! AC2 — Single dispatcher handles all RPCs.
//! AC3 — Unknown RPC name returns 404 without leaking names.
//!
//! AC1 is also exercised by `rpc_schema_compat.rs` (which checks
//! the schema emission); this file focuses on the dispatch side.

#![allow(clippy::unwrap_used, missing_docs, missing_copy_implementations)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ssg_rpc::{dispatch, registered_names, ssg_rpc, RpcError};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LikeInput {
    pub post_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LikeOutput {
    pub likes: u64,
}

#[ssg_rpc]
pub fn like_post(input: LikeInput) -> Result<LikeOutput, RpcError> {
    if input.post_id.is_empty() {
        return Err(RpcError::BadRequest("post_id required".into()));
    }
    Ok(LikeOutput { likes: 42 })
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EchoIn {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EchoOut {
    pub msg: String,
}

#[ssg_rpc]
pub fn echo(input: EchoIn) -> Result<EchoOut, RpcError> {
    Ok(EchoOut { msg: input.msg })
}

#[test]
fn ac1_registered_names_includes_macro_functions() {
    let names = registered_names();
    assert!(
        names.contains(&"like_post"),
        "missing like_post in {names:?}"
    );
    assert!(names.contains(&"echo"), "missing echo in {names:?}");
}

#[test]
fn ac2_dispatch_routes_to_registered_function() {
    let body = dispatch("like_post", r#"{"post_id":"abc"}"#).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["likes"], 42);
}

#[test]
fn ac2_handler_error_propagates() {
    let err = dispatch("like_post", r#"{"post_id":""}"#).unwrap_err();
    assert!(matches!(err, RpcError::BadRequest(_)));
    assert_eq!(err.status_code(), 400);
}

#[test]
fn ac2_malformed_payload_becomes_bad_request() {
    let err = dispatch("like_post", "{not json").unwrap_err();
    assert!(matches!(err, RpcError::BadRequest(_)));
}

#[test]
fn ac3_unknown_rpc_returns_not_found() {
    let err = dispatch("nonexistent", "{}").unwrap_err();
    assert!(matches!(err, RpcError::NotFound));
    assert_eq!(err.status_code(), 404);
    // Crucial: the wire body must NOT include the registry names.
    let wire = err.to_wire_body();
    for name in registered_names() {
        assert!(
            !wire.contains(name),
            "wire body leaks registered name {name}: {wire}"
        );
    }
}

#[test]
fn ac3_wire_body_is_terse_not_found_string() {
    let err = dispatch("nope", "{}").unwrap_err();
    let body = err.to_wire_body();
    assert_eq!(body, r#"{"error":"not found"}"#);
}

#[test]
fn ac2_echo_round_trips_payload() {
    let body = dispatch("echo", r#"{"msg":"hello"}"#).unwrap();
    let parsed: EchoOut = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed.msg, "hello");
}

#[test]
fn registered_names_are_sorted() {
    let names = registered_names();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
}
