// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ssg-search — Browser-native vector semantic search
//!
//! Provides everything needed to ship a privacy-first, fully static
//! semantic search experience for an SSG site:
//!
//! - A **build-side** API ([`build`]) that walks document text, runs the
//!   embedder, L2-normalises every vector, and serialises the corpus
//!   into a flat little-endian `f32` blob (`embeddings.bin`) together
//!   with a `manifest.json`, `model.bin`, and `tokenizer.bin`.
//! - A **WASM-side** [`VectorEngine`] that mmaps the blob, runs the
//!   same embedder on user queries, and returns the top-K results as
//!   an interleaved `[idx, score, idx, score, …]` `Float32Array`.
//! - A deterministic, model-free hashed-n-gram projection encoder
//!   ([`encoder::ProjectionEncoder`]) used as the default. With the
//!   `model2vec` feature it switches to a real `model2vec-rs` encoder.
//!
//! ## Architectural invariants
//!
//! 1. **JS/WASM boundary is `Float32Array` only.** No nested objects,
//!    no JSON across the wire. The boundary contract is enforced (and
//!    verified by `--features wasm-profiling`).
//! 2. **All vectors are pre-normalised** at build time so the runtime
//!    similarity reduces to a pure dot product (no division, no sqrt).
//!    The `VectorEngine` performs an explicit short-circuit and never
//!    calls `f32::sqrt`.
//! 3. **Builds are reproducible.** Given identical input bytes and
//!    identical encoder weights, `embeddings.bin` is byte-identical.
//! 4. **Single-threaded by design.** Browser WASM is single-threaded;
//!    the engine carries no `Arc`/`Mutex`/`Rayon` overhead.

pub mod artifacts;
pub mod encoder;
pub mod engine;
pub mod manifest;
pub mod quantize;

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub mod wasm_binding;

pub use artifacts::{Artifacts, ArtifactsBuilder};
pub use encoder::{Encoder, ProjectionEncoder, EMBEDDING_DIM};
pub use engine::VectorEngine;
pub use manifest::{Manifest, ManifestEntry};
pub use quantize::{dequantize_int8, quantize_int8};

/// File layout written under `<site>/search/` by the build step.
pub mod paths {
    /// Pre-normalised f32 corpus vectors, little-endian, `N × D × 4` bytes.
    pub const EMBEDDINGS_FILE: &str = "embeddings.bin";
    /// JSON map from row index to `{url, title, excerpt}`.
    pub const MANIFEST_FILE: &str = "manifest.json";
    /// Encoder weights (projection matrix or model2vec int8 weights).
    pub const MODEL_FILE: &str = "model.bin";
    /// Tokeniser configuration (vocab / n-gram bounds / hash seed).
    pub const TOKENIZER_FILE: &str = "tokenizer.bin";
}

/// Version stamped into every emitted `model.bin` header. Bumping this
/// invalidates older builds — the [`VectorEngine`] refuses to load
/// mismatched headers rather than silently producing garbage.
pub const ARTIFACT_FORMAT_VERSION: u32 = 1;

/// The four-byte ASCII magic prefix on every artifact header.
pub const ARTIFACT_MAGIC: [u8; 4] = *b"SSGS";

/// Default top-K returned by [`VectorEngine::search`] when the caller
/// passes `0`.
pub const DEFAULT_TOP_K: usize = 10;
