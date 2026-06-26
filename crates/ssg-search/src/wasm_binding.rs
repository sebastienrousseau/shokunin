// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WASM / JS binding for [`crate::VectorEngine`].
//!
//! The exposed surface is intentionally thin:
//!
//! - `WasmVectorEngine::new(model, tokenizer, embeddings, count)` —
//!   construct from raw `Uint8Array` artifacts.
//! - `WasmVectorEngine::search(query: string, top_k: u32) -> Float32Array` —
//!   embed + dot-product + top-K in one boundary crossing.
//! - `WasmVectorEngine::search_vec(query_vec: Float32Array, top_k: u32) -> Float32Array` —
//!   skip embedding (caller already has the query vector).
//! - `WasmVectorEngine::embed(query: string) -> Float32Array` — for
//!   the JS shim that wants to cache embeddings client-side.
//!
//! **AC4** — every boundary call is a scalar, string, or `Float32Array`.
//! No JS objects. No nested arrays. No JSON.

use crate::engine::{EngineError, VectorEngine};
use wasm_bindgen::prelude::*;

/// JS-visible vector engine. Wraps the pure-Rust [`VectorEngine`].
///
/// # Examples
///
/// JS-side construction once the WASM module is loaded:
///
/// ```ignore
/// // (JS) — for reference; the doctest is `ignore` because
/// // `WasmVectorEngine` lives behind the wasm-bindgen boundary and is
/// // only callable from JavaScript.
/// import init, { WasmVectorEngine } from "./pkg/ssg_search.js";
/// await init();
/// const engine = new WasmVectorEngine(model, tokenizer, embeddings, count);
/// const top = engine.search("rust wasm", 10);
/// ```
#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmVectorEngine {
    inner: VectorEngine,
}

#[wasm_bindgen]
impl WasmVectorEngine {
    /// Constructs a new engine from the four artifact buffers.
    ///
    /// `count` is passed explicitly (rather than derived from
    /// `embeddings.len() / dim / 4`) so a malformed bundle fails fast
    /// with a clear error rather than silently truncating.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — called as a constructor across the wasm-bindgen
    /// // boundary; not invocable from native Rust.
    /// const engine = new WasmVectorEngine(model, tokenizer, embeddings, count);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a `JsError` wrapping an [`EngineError`] when the
    /// artifact bytes are malformed or inconsistent.
    #[wasm_bindgen(constructor)]
    pub fn new(
        model: &[u8],
        tokenizer: &[u8],
        embeddings: &[u8],
        count: u32,
    ) -> Result<Self, JsError> {
        let inner =
            VectorEngine::new(model, tokenizer, embeddings, count as usize)
                .map_err(map_err)?;
        Ok(Self { inner })
    }

    /// Number of indexed documents.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — read as a property of the JS-side instance.
    /// const n = engine.count;
    /// ```
    #[allow(clippy::missing_const_for_fn)] // #[wasm_bindgen] forbids const fn
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> u32 {
        self.inner.count() as u32
    }

    /// Vector dimensionality.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — read as a property of the JS-side instance.
    /// const d = engine.dim; // 256 with the default encoder
    /// ```
    #[allow(clippy::missing_const_for_fn)] // #[wasm_bindgen] forbids const fn
    #[wasm_bindgen(getter)]
    pub fn dim(&self) -> u32 {
        self.inner.dim() as u32
    }

    /// Embed a query string and return the unit-norm vector.
    ///
    /// Returned as `Float32Array` — never as a JS array of numbers
    /// (AC4).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — caller receives a Float32Array, length === engine.dim.
    /// const q = engine.embed("rust wasm");
    /// console.assert(q instanceof Float32Array);
    /// console.assert(q.length === engine.dim);
    /// ```
    #[wasm_bindgen]
    pub fn embed(&self, query: &str) -> js_sys::Float32Array {
        profile_boundary("embed");
        let v = self.inner.embed_query(query);
        js_sys::Float32Array::from(v.as_slice())
    }

    /// Run the full embed → search pipeline and return the top-K
    /// `[idx, score, idx, score, …]` interleaved as a `Float32Array`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — single boundary crossing for full search.
    /// const out = engine.search("rust wasm", 10);
    /// // 10 results × 2 floats per result = 20.
    /// console.assert(out.length === 20);
    /// const [idx0, score0] = [out[0], out[1]];
    /// ```
    #[wasm_bindgen]
    pub fn search(&self, query: &str, top_k: u32) -> js_sys::Float32Array {
        profile_boundary("search");
        let v = self.inner.search(query, top_k as usize);
        js_sys::Float32Array::from(v.as_slice())
    }

    /// Skip the embed step — caller supplies the query vector
    /// directly (useful when re-running the same query against
    /// multiple engines).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // (JS) — embed once, search many.
    /// const q = engine.embed("rust");
    /// const a = engineA.search_vec(q, 10);
    /// const b = engineB.search_vec(q, 10);
    /// ```
    #[wasm_bindgen]
    pub fn search_vec(
        &self,
        query_vec: &[f32],
        top_k: u32,
    ) -> js_sys::Float32Array {
        profile_boundary("search_vec");
        let v = self.inner.search_vec(query_vec, top_k as usize);
        js_sys::Float32Array::from(v.as_slice())
    }
}

fn map_err(e: EngineError) -> JsError {
    JsError::new(&e.to_string())
}

/// Process-wide boundary-crossing counter (AC4 verification).
///
/// Off by default — only mutates under `--features wasm-profiling`.
/// Read via [`boundary_crossings`].
#[cfg(feature = "wasm-profiling")]
static BOUNDARY_COUNTER: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(0);

/// Boundary-crossing audit hook (AC4 / `wasm-profiling`).
///
/// When the `wasm-profiling` feature is **off** (the default), this is
/// inlined to a no-op. When **on**, it bumps a process-wide counter
/// the JS test harness can read via [`boundary_crossings`].
#[cfg_attr(not(feature = "wasm-profiling"), inline(always))]
fn profile_boundary(_op: &'static str) {
    #[cfg(feature = "wasm-profiling")]
    {
        let _ = BOUNDARY_COUNTER
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
}

/// JS-visible getter for the per-process boundary-crossing count.
///
/// Always exposed (returning 0 when `wasm-profiling` is off) so the
/// JS bundle has a stable API surface across builds.
///
/// # Examples
///
/// ```ignore
/// // (JS) — exposed even without the `wasm-profiling` feature, in
/// // which case it always returns 0.
/// import { boundary_crossings } from "./pkg/ssg_search.js";
/// console.assert(typeof boundary_crossings() === "number");
/// ```
#[wasm_bindgen]
pub fn boundary_crossings() -> u32 {
    #[cfg(feature = "wasm-profiling")]
    {
        BOUNDARY_COUNTER.load(core::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(feature = "wasm-profiling"))]
    0
}
