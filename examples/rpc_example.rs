#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Edge RPC Example — `#[ssg_rpc]` end-to-end (v0.0.44, issue #548)
//!
//! Demonstrates the `ssg-rpc` crate by:
//!
//! 1. Defining a tiny `like_post` RPC with `#[ssg_rpc]`.
//! 2. Round-tripping a call through the JSON dispatcher.
//! 3. Printing the generated JSON Schema for the input + output types.
//! 4. Printing the generated TypeScript `.d.ts` so site authors can see
//!    what the matching JS client gets.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example rpc_example
//! ```

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ssg_rpc::{
    dispatch, emit_typescript_for, registered_names, schema_for,
    schema_for_result, ssg_rpc, EmitOptions, RpcError, RpcSchema,
};

/// Input payload for the `like_post` RPC.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LikeInput {
    /// Post ID being liked.
    pub post_id: String,
}

/// Output payload for the `like_post` RPC.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct LikeOutput {
    /// New like-count after this call.
    pub likes: u64,
}

/// Tiny demo RPC — accepts a post ID, returns a stub like-count.
#[ssg_rpc]
pub fn like_post(input: LikeInput) -> Result<LikeOutput, RpcError> {
    // The real implementation would update a counter; this example just
    // echoes back a deterministic value so the round-trip is visible.
    let likes = u64::from(input.post_id.len() as u32).saturating_add(1);
    Ok(LikeOutput { likes })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inspect the registry — `like_post` should be discoverable
    //    because the proc-macro registered it via `inventory::submit!`.
    let names = registered_names();
    println!("[rpc] registered RPCs ({}): {names:?}", names.len());

    // 2. JSON-in, JSON-out round trip through the dispatcher.
    let payload = r#"{"post_id":"hello-world"}"#;
    let resp = dispatch("like_post", payload)?;
    println!("[rpc] dispatch like_post({payload}) -> {resp}");

    // 3. Print the JSON Schema for the input and the success branch.
    let in_schema = schema_for::<LikeInput>();
    let out_schema = schema_for_result::<Result<LikeOutput, RpcError>>();
    println!(
        "[rpc] LikeInput  schema:\n{}",
        serde_json::to_string_pretty(&in_schema)?
    );
    println!(
        "[rpc] LikeOutput schema:\n{}",
        serde_json::to_string_pretty(&out_schema)?
    );

    // 4. Emit the TypeScript declarations for just this RPC.
    let schema_bundle = RpcSchema {
        name: "like_post",
        input: in_schema,
        output: out_schema,
    };
    let opts = EmitOptions::default();
    let ts = emit_typescript_for(&[schema_bundle], &opts);
    println!("[rpc] generated TypeScript .d.ts:\n{ts}");

    Ok(())
}
