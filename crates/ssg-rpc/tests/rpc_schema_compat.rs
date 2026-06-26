// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration test backing AC4, AC5 of issue #548.
//!
//! AC4 — Type-safe TS client: we lock the shape of the emitted
//!       `.d.ts` so the matching JS client (`web/rpc.js`) lines up.
//! AC5 — Schema versioning + breaking-change detection: a golden
//!       snapshot of the emitted TS is committed under
//!       `tests/golden/rpc.d.ts`. Any change to the schema fails
//!       this test with a clear instruction on how to refresh.
//!
//! Refreshing the snapshot is intentionally a manual step — that's
//! exactly the friction AC5 is asking for. To refresh, set
//! `SSG_RPC_UPDATE_SNAPSHOT=1` and rerun the test.

#![allow(clippy::unwrap_used, missing_docs, missing_copy_implementations)]

use std::fs;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ssg_rpc::{
    emit_typescript_for, schema_for, schema_for_result, ssg_rpc, EmitOptions,
    RpcError, RpcSchema,
};

// ---- Stable fixture types ---------------------------------------------------

/// Input for the snapshot fixture RPC.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SnapInput {
    pub post_id: String,
    #[serde(default)]
    pub author: Option<String>,
}

/// Output for the snapshot fixture RPC.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SnapOutput {
    pub likes: u64,
}

#[ssg_rpc]
pub fn snap_like(input: SnapInput) -> Result<SnapOutput, RpcError> {
    let _ = input;
    Ok(SnapOutput { likes: 0 })
}

// ---- Helpers ----------------------------------------------------------------

fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("rpc.d.ts")
}

fn current_snapshot_emission() -> String {
    // Build the schema explicitly rather than relying on the global
    // inventory so the snapshot does not drift when another test in
    // the same binary registers an extra RPC.
    let schema = RpcSchema {
        name: "snap_like",
        input: schema_for::<SnapInput>(),
        output: schema_for_result::<Result<SnapOutput, RpcError>>(),
    };
    let opts = EmitOptions {
        header: String::from(
            "// AUTO-GENERATED snapshot — do not edit by hand.\n",
        ),
        emit_rpc_index: true,
    };
    emit_typescript_for(&[schema], &opts)
}

// ---- Tests ------------------------------------------------------------------

#[test]
fn ac4_emitted_snapshot_describes_input_and_output() {
    let ts = current_snapshot_emission();
    // Input interface contains both fields, with `author` optional.
    assert!(ts.contains("export interface SnapLikeInput"), "{ts}");
    assert!(ts.contains("post_id: string"), "{ts}");
    assert!(
        ts.contains("author?:") || ts.contains("\"null\""),
        "expected optional author field: {ts}"
    );
    // Output interface.
    assert!(ts.contains("export interface SnapLikeOutput"), "{ts}");
    assert!(ts.contains("likes: number"), "{ts}");
    // Rpc index.
    assert!(
        ts.contains("snap_like(input: SnapLikeInput): Promise<SnapLikeOutput>"),
        "{ts}"
    );
}

#[test]
fn ac5_snapshot_matches_golden_file() {
    let actual = current_snapshot_emission();
    let path = snapshot_path();

    if std::env::var("SSG_RPC_UPDATE_SNAPSHOT").is_ok() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, &actual).unwrap();
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing golden snapshot at {}: {e}.\n\
             Re-run with SSG_RPC_UPDATE_SNAPSHOT=1 to create it.",
            path.display()
        )
    });

    if actual != expected {
        // Synthesise a clear diff message naming the offending RPC.
        panic!(
            "RPC schema drift detected in {}.\n\
             ---- expected ----\n{expected}\n\
             ---- actual ----\n{actual}\n\
             Refresh with: SSG_RPC_UPDATE_SNAPSHOT=1 cargo test \
             -p ssg-rpc --test rpc_schema_compat",
            path.display(),
        );
    }
}

#[test]
fn ac5_breaking_change_in_field_set_fails_diff() {
    // Synthesise a "v2" snapshot that drops `post_id`. The diff
    // assertion below stands in for the CI message developers see
    // when they remove or rename a required field.
    let v1 = current_snapshot_emission();
    let v2_schema = RpcSchema {
        name: "snap_like",
        input: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        output: schema_for_result::<Result<SnapOutput, RpcError>>(),
    };
    let v2 = emit_typescript_for(
        &[v2_schema],
        &EmitOptions {
            header: String::from(
                "// AUTO-GENERATED snapshot — do not edit by hand.\n",
            ),
            emit_rpc_index: true,
        },
    );
    assert_ne!(v1, v2, "removing post_id should be observable");
    assert!(v1.contains("post_id"), "v1 should still mention post_id");
    assert!(
        !v2.contains("post_id"),
        "v2 should no longer mention post_id"
    );
}
