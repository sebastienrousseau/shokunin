// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Browser-side integration tests for the `ssg-search` engine via
//! `wasm-bindgen-test`. Verifies the Float32Array boundary contract
//! (AC4), the no-division short-circuit (AC5), and end-to-end query
//! behaviour against a small fixture corpus.

#![allow(
    missing_docs,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown
)]
#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use ssg_search::artifacts::{Artifacts, InputDoc};
use ssg_search::wasm_binding::WasmVectorEngine;

fn fixture_bytes() -> (Vec<u8>, Vec<u8>, Vec<u8>, u32) {
    let docs = vec![
        InputDoc {
            url: "/wasm".into(),
            title: "WASM in Rust".into(),
            body: "rust webassembly compiles to portable browser modules"
                .into(),
            excerpt: "rust wasm".into(),
        },
        InputDoc {
            url: "/bread".into(),
            title: "Sourdough Bread".into(),
            body: "baking sourdough starter flour water salt".into(),
            excerpt: "bread".into(),
        },
        InputDoc {
            url: "/simd".into(),
            title: "SIMD in WASM".into(),
            body: "rust simd intrinsics target webassembly vectorisation"
                .into(),
            excerpt: "simd".into(),
        },
    ];
    let arts = Artifacts::from_docs(&docs);
    let count = arts.count() as u32;
    (arts.model, arts.tokenizer, arts.embeddings, count)
}

#[wasm_bindgen_test]
fn engine_constructs_from_artifacts() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n)
        .expect("engine should construct from valid artifacts");
    assert_eq!(engine.count(), 3);
    // Default encoder dim is 256.
    assert_eq!(engine.dim(), 256);
}

#[wasm_bindgen_test]
fn engine_rejects_bad_model_bytes() {
    let bad = vec![0u8; 24];
    let res = WasmVectorEngine::new(&bad, &[], &[], 0);
    assert!(res.is_err(), "engine should reject zeroed-out model bytes");
}

#[wasm_bindgen_test]
fn search_returns_float32array_of_idx_score_pairs() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n).unwrap();
    let out = engine.search("rust wasm", 3);
    // AC4: result is a Float32Array (verified by the type signature),
    // and contains 2 * top_k entries.
    assert_eq!(out.length(), 6);
}

#[wasm_bindgen_test]
fn embed_returns_unit_norm_float32array() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n).unwrap();
    let v = engine.embed("rust webassembly");
    assert_eq!(v.length(), 256);
    // Compute L2 norm on the JS side (it's just a Float32Array).
    let mut sumsq: f32 = 0.0;
    let copy = v.to_vec();
    for x in copy {
        sumsq += x * x;
    }
    let norm = sumsq.sqrt();
    assert!(
        (0.999..=1.001).contains(&norm),
        "embed should L2-normalise, got norm {norm}"
    );
}

#[wasm_bindgen_test]
fn search_ranks_related_doc_first() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n).unwrap();
    let out = engine.search("rust wasm simd", 3).to_vec();
    // Top hit must be index 0 (rust/webassembly) or 2 (rust/simd),
    // never the bread doc at index 1.
    let top_idx = out[0] as u32;
    assert!(top_idx == 0 || top_idx == 2);
}

#[wasm_bindgen_test]
fn search_vec_round_trip() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n).unwrap();
    let query = engine.embed("rust wasm");
    let out = engine.search_vec(&query.to_vec(), 2);
    assert_eq!(out.length(), 4);
}

#[wasm_bindgen_test]
fn search_empty_query_returns_no_nan() {
    let (model, tok, emb, n) = fixture_bytes();
    let engine = WasmVectorEngine::new(&model, &tok, &emb, n).unwrap();
    let out = engine.search("", 3).to_vec();
    for x in out {
        assert!(!x.is_nan(), "no NaN in output");
    }
}
