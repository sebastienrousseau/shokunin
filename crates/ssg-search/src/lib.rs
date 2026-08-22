// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ssg-search — Browser-native vector semantic search
//!
//! Provides everything needed to ship a privacy-first, fully static
//! semantic search experience for an SSG site:
//!
//! - A **build-side** API ([`Artifacts::from_docs`]) that walks document text, runs the
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
///
/// # Examples
///
/// ```
/// use ssg_search::paths;
///
/// // The four canonical filenames every build emits.
/// assert_eq!(paths::EMBEDDINGS_FILE, "embeddings.bin");
/// assert_eq!(paths::MANIFEST_FILE, "manifest.json");
/// assert_eq!(paths::MODEL_FILE, "model.bin");
/// assert_eq!(paths::TOKENIZER_FILE, "tokenizer.bin");
/// ```
pub mod paths {
    /// Pre-normalised f32 corpus vectors, little-endian, `N × D × 4` bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::paths::EMBEDDINGS_FILE;
    /// assert_eq!(EMBEDDINGS_FILE, "embeddings.bin");
    /// assert!(EMBEDDINGS_FILE.ends_with(".bin"));
    /// ```
    pub const EMBEDDINGS_FILE: &str = "embeddings.bin";
    /// JSON map from row index to `{url, title, excerpt}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::paths::MANIFEST_FILE;
    /// assert_eq!(MANIFEST_FILE, "manifest.json");
    /// assert!(MANIFEST_FILE.ends_with(".json"));
    /// ```
    pub const MANIFEST_FILE: &str = "manifest.json";
    /// Encoder weights (projection matrix or model2vec int8 weights).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::paths::MODEL_FILE;
    /// assert_eq!(MODEL_FILE, "model.bin");
    /// ```
    pub const MODEL_FILE: &str = "model.bin";
    /// Tokeniser configuration (vocab / n-gram bounds / hash seed).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::paths::TOKENIZER_FILE;
    /// assert_eq!(TOKENIZER_FILE, "tokenizer.bin");
    /// ```
    pub const TOKENIZER_FILE: &str = "tokenizer.bin";
}

/// Version stamped into every emitted `model.bin` header. Bumping this
/// invalidates older builds — the [`VectorEngine`] refuses to load
/// mismatched headers rather than silently producing garbage.
///
/// # Examples
///
/// ```
/// use ssg_search::ARTIFACT_FORMAT_VERSION;
///
/// // Format v1 is the first stable release.
/// assert_eq!(ARTIFACT_FORMAT_VERSION, 1);
/// ```
pub const ARTIFACT_FORMAT_VERSION: u32 = 1;

/// The four-byte ASCII magic prefix on every artifact header.
///
/// # Examples
///
/// ```
/// use ssg_search::ARTIFACT_MAGIC;
///
/// assert_eq!(&ARTIFACT_MAGIC, b"SSGS");
/// assert_eq!(ARTIFACT_MAGIC.len(), 4);
/// ```
pub const ARTIFACT_MAGIC: [u8; 4] = *b"SSGS";

/// Default top-K returned by [`VectorEngine::search`] when the caller
/// passes `0`.
///
/// # Examples
///
/// ```
/// use ssg_search::DEFAULT_TOP_K;
///
/// assert_eq!(DEFAULT_TOP_K, 10);
/// ```
pub const DEFAULT_TOP_K: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_magic_is_ssgs_ascii() {
        assert_eq!(&ARTIFACT_MAGIC, b"SSGS");
        assert_eq!(ARTIFACT_MAGIC.len(), 4);
        // Every byte must be printable ASCII for hex dump readability.
        for &b in &ARTIFACT_MAGIC {
            assert!(b.is_ascii_uppercase());
        }
    }

    #[test]
    fn artifact_format_version_is_one() {
        assert_eq!(ARTIFACT_FORMAT_VERSION, 1);
    }

    #[test]
    fn default_top_k_is_ten() {
        assert_eq!(DEFAULT_TOP_K, 10);
    }

    #[test]
    fn paths_constants_have_expected_filenames() {
        assert_eq!(paths::EMBEDDINGS_FILE, "embeddings.bin");
        assert_eq!(paths::MANIFEST_FILE, "manifest.json");
        assert_eq!(paths::MODEL_FILE, "model.bin");
        assert_eq!(paths::TOKENIZER_FILE, "tokenizer.bin");
    }

    #[test]
    fn paths_constants_are_unique() {
        let all = [
            paths::EMBEDDINGS_FILE,
            paths::MANIFEST_FILE,
            paths::MODEL_FILE,
            paths::TOKENIZER_FILE,
        ];
        let mut sorted = all.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), all.len());
    }

    #[test]
    fn re_exported_manifest_constructs() {
        // Exercises the `pub use manifest::{Manifest, ManifestEntry}`
        // path — if the re-export name changed this would not compile.
        let m = Manifest::new(8, "abc".into(), vec![]);
        assert_eq!(m.format_version, ARTIFACT_FORMAT_VERSION);
        let e = ManifestEntry {
            url: "/u".into(),
            title: "t".into(),
            excerpt: "x".into(),
        };
        assert_eq!(e.url, "/u");
    }

    #[test]
    fn re_exported_quantize_round_trips() {
        let q = quantize_int8(&[0.5, -0.5, 1.0]);
        let back = dequantize_int8(&q);
        assert_eq!(back.len(), 3);
    }

    #[test]
    fn re_exported_encoder_constructs_with_dim() {
        // Exercises the `pub use encoder::{Encoder, ProjectionEncoder,
        // EMBEDDING_DIM}` path by constructing an encoder and verifying
        // the embedding dimension matches the re-exported constant.
        let enc = ProjectionEncoder::default();
        let v = <ProjectionEncoder as Encoder>::embed(&enc, "hello world");
        assert_eq!(v.len(), EMBEDDING_DIM);
        assert_eq!(<ProjectionEncoder as Encoder>::dim(&enc), EMBEDDING_DIM);
    }
}
