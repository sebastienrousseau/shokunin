// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Text → dense vector encoder.
//!
//! The default encoder is a deterministic **hashed-n-gram projection**
//! — every n-gram (word + char-trigram) of the input text is mapped to
//! a row of a fixed projection matrix via a stable 64-bit hash; the
//! rows are summed and L2-normalised to a unit vector. This is the
//! same family of "static" encoders as `model2vec` and `fastText`'s
//! `hashing trick`, just without learned weights.
//!
//! It is intentionally lightweight (no `tokenizers`, no `safetensors`,
//! no model file dependency) so the default build of `ssg-search` ships
//! a sub-200 KB WASM module. Swap in a real `model2vec-rs` encoder with
//! the `model2vec` feature when you need true distilled embeddings.
//!
//! ## Determinism
//!
//! For identical `(text, dim, seed)` inputs the encoder produces the
//! same bytes on every platform — verified by the
//! `tests/encoder_determinism.rs` integration test.

use serde::{Deserialize, Serialize};

/// Dimensionality of every vector produced by the default encoder.
///
/// Chosen as a multiple of 4 so SIMD f32x4 dot-products fit cleanly,
/// and small enough that a 1000-doc corpus is well under 1 MB
/// (`1000 × 256 × 4` = 1.0 MB).
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{Encoder, ProjectionEncoder, EMBEDDING_DIM};
///
/// assert_eq!(EMBEDDING_DIM, 256);
/// // The default encoder always produces vectors of this length.
/// let enc = ProjectionEncoder::default();
/// assert_eq!(enc.embed("hello").len(), EMBEDDING_DIM);
/// ```
pub const EMBEDDING_DIM: usize = 256;

/// Deterministic seed baked into the projection matrix. Bumping it
/// invalidates every existing build — keep stable across releases
/// unless you intend to break compatibility.
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{ProjectionEncoder, PROJECTION_SEED};
///
/// // The seed spells "SSGSEARC" in ASCII (little-endian).
/// assert_eq!(PROJECTION_SEED, 0x5353_4753_4541_5243);
/// // It's the default encoder's seed.
/// assert_eq!(ProjectionEncoder::default().seed(), PROJECTION_SEED);
/// ```
pub const PROJECTION_SEED: u64 = 0x5353_4753_4541_5243; // "SSGSEARC"

/// Trait every encoder implements. The build-side and WASM-side share
/// the same instance type — so the runtime query embedding is
/// guaranteed to be in the same vector space as the corpus embeddings.
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{Encoder, ProjectionEncoder, EMBEDDING_DIM};
///
/// // Generic helper that works against any Encoder impl.
/// fn embed_len<E: Encoder>(enc: &E, text: &str) -> usize {
///     enc.embed(text).len()
/// }
///
/// let enc = ProjectionEncoder::default();
/// assert_eq!(embed_len(&enc, "hello"), EMBEDDING_DIM);
/// ```
pub trait Encoder {
    /// Returns the output dimensionality.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{Encoder, ProjectionEncoder, EMBEDDING_DIM};
    ///
    /// let enc = ProjectionEncoder::default();
    /// assert_eq!(<ProjectionEncoder as Encoder>::dim(&enc), EMBEDDING_DIM);
    /// ```
    fn dim(&self) -> usize;

    /// Encodes a UTF-8 string into an L2-normalised vector.
    ///
    /// The output `Vec<f32>` always has length [`Encoder::dim`]. Its
    /// L2 norm is in `[0.999, 1.001]` (verified by AC5).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{Encoder, ProjectionEncoder};
    ///
    /// let enc = ProjectionEncoder::default();
    /// let v = enc.embed("the quick brown fox");
    /// assert_eq!(v.len(), enc.dim());
    ///
    /// // L2 norm is ~1.0 (or 0.0 for empty input).
    /// let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    /// assert!((0.999..=1.001).contains(&norm));
    /// ```
    fn embed(&self, text: &str) -> Vec<f32>;

    /// Serialises encoder weights / config to the byte layout written
    /// into `model.bin`. The format is encoder-specific but always
    /// starts with the four-byte [`crate::ARTIFACT_MAGIC`] and a u32
    /// [`crate::ARTIFACT_FORMAT_VERSION`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{Encoder, ProjectionEncoder};
    /// use ssg_search::{ARTIFACT_MAGIC, ARTIFACT_FORMAT_VERSION};
    ///
    /// let bytes = ProjectionEncoder::default().serialize_model();
    /// assert_eq!(&bytes[0..4], &ARTIFACT_MAGIC);
    /// let v = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    /// assert_eq!(v, ARTIFACT_FORMAT_VERSION);
    /// ```
    fn serialize_model(&self) -> Vec<u8>;

    /// Serialises tokenizer config to the byte layout written into
    /// `tokenizer.bin`. Same header layout as `serialize_model`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{Encoder, ProjectionEncoder};
    /// use ssg_search::ARTIFACT_MAGIC;
    ///
    /// let bytes = ProjectionEncoder::default().serialize_tokenizer();
    /// assert!(bytes.len() > 12);
    /// assert_eq!(&bytes[0..4], &ARTIFACT_MAGIC);
    /// ```
    fn serialize_tokenizer(&self) -> Vec<u8>;
}

/// On-disk representation of the projection encoder — small enough
/// (`< 100 B`) that the deserialiser can construct the full
/// `ProjectionEncoder` lazily from this struct.
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{ProjectionConfig, EMBEDDING_DIM, PROJECTION_SEED};
///
/// // Defaults are tuned for the standard 256-dim encoder.
/// let cfg = ProjectionConfig::default();
/// assert_eq!(cfg.dim, EMBEDDING_DIM as u32);
/// assert_eq!(cfg.seed, PROJECTION_SEED);
/// assert_eq!(cfg.ngram_min, 3);
/// assert_eq!(cfg.ngram_max, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionConfig {
    /// Output dimensionality.
    pub dim: u32,
    /// Seed used to derive the projection matrix.
    pub seed: u64,
    /// Minimum char-n-gram length included.
    pub ngram_min: u8,
    /// Maximum char-n-gram length included.
    pub ngram_max: u8,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            dim: EMBEDDING_DIM as u32,
            seed: PROJECTION_SEED,
            ngram_min: 3,
            ngram_max: 5,
        }
    }
}

/// Deterministic hashed-n-gram projection encoder. Tiny, model-free,
/// reproducible byte-for-byte across platforms.
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{Encoder, ProjectionEncoder, EMBEDDING_DIM};
///
/// let enc = ProjectionEncoder::default();
///
/// // Same input always produces the same vector — bit-for-bit reproducible.
/// let a = enc.embed("static site generator");
/// let b = enc.embed("static site generator");
/// assert_eq!(a, b);
/// assert_eq!(a.len(), EMBEDDING_DIM);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ProjectionEncoder {
    cfg: ProjectionConfig,
}

impl Default for ProjectionEncoder {
    fn default() -> Self {
        Self::new(ProjectionConfig::default())
    }
}

impl ProjectionEncoder {
    /// Constructs a new encoder from a [`ProjectionConfig`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{Encoder, ProjectionConfig, ProjectionEncoder};
    ///
    /// // Build a smaller-than-default encoder for tests.
    /// let cfg = ProjectionConfig { dim: 64, ..ProjectionConfig::default() };
    /// let enc = ProjectionEncoder::new(cfg);
    /// assert_eq!(enc.dim(), 64);
    /// assert_eq!(enc.embed("hello").len(), 64);
    /// ```
    #[must_use]
    pub const fn new(cfg: ProjectionConfig) -> Self {
        Self { cfg }
    }

    /// Returns the [`ProjectionConfig`] for this encoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{ProjectionConfig, ProjectionEncoder};
    ///
    /// let enc = ProjectionEncoder::default();
    /// assert_eq!(enc.config(), ProjectionConfig::default());
    /// ```
    #[must_use]
    pub const fn config(&self) -> ProjectionConfig {
        self.cfg
    }

    /// Returns the seed used to derive the projection matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_search::encoder::{ProjectionEncoder, PROJECTION_SEED};
    ///
    /// let enc = ProjectionEncoder::default();
    /// assert_eq!(enc.seed(), PROJECTION_SEED);
    /// ```
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.cfg.seed
    }

    /// Iterates the (lowercased) whitespace-split words of `text`.
    fn tokens(text: &str) -> impl Iterator<Item = String> + '_ {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
    }

    /// Iterates the char-n-grams of `word` within the configured
    /// `[ngram_min, ngram_max]` window. Adds boundary markers so
    /// `"the"` and `"there"` share fewer trigrams.
    fn char_ngrams<'a>(
        &'a self,
        word: &'a str,
    ) -> impl Iterator<Item = String> + 'a {
        let marked = format!("<{word}>");
        let chars: Vec<char> = marked.chars().collect();
        let lo = self.cfg.ngram_min as usize;
        let hi = self.cfg.ngram_max as usize;
        (lo..=hi).flat_map(move |n| {
            let chars = chars.clone();
            (0..chars.len().saturating_sub(n.saturating_sub(1))).map(move |i| {
                if i + n <= chars.len() {
                    chars[i..i + n].iter().collect::<String>()
                } else {
                    String::new()
                }
            })
        })
    }

    /// Stable FNV-1a 64-bit hash. Identical across all targets —
    /// load-bearing for AC6 (reproducible builds across CI runners).
    fn hash(seed: u64, s: &str) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ seed;
        for &b in s.as_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    /// Splatts the hash of `feature` into the accumulator vector.
    /// Each feature touches `STAMPS_PER_FEATURE` entries with a
    /// deterministic sign (±1), giving a sparse random-projection.
    fn project_into(&self, feature: &str, out: &mut [f32]) {
        const STAMPS_PER_FEATURE: usize = 8;
        let dim = out.len() as u64;
        let base = Self::hash(self.cfg.seed, feature);
        for k in 0..STAMPS_PER_FEATURE {
            // Mix the stamp index into a 64-bit derived hash. xorshift*
            // is cheap and deterministic.
            let mut x = base.wrapping_add(
                0x9E37_79B9_7F4A_7C15u64.wrapping_mul(k as u64 + 1),
            );
            x ^= x >> 30;
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 27;
            x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
            x ^= x >> 31;
            let idx = (x % dim) as usize;
            let sign = if (x >> 63) & 1 == 1 {
                -1.0_f32
            } else {
                1.0_f32
            };
            out[idx] += sign;
        }
    }
}

impl Encoder for ProjectionEncoder {
    fn dim(&self) -> usize {
        self.cfg.dim as usize
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        let dim = self.dim();
        let mut v = vec![0.0_f32; dim];
        let mut feature_count: u32 = 0;
        for word in Self::tokens(text) {
            // Stamp the whole word.
            self.project_into(&word, &mut v);
            feature_count += 1;
            // And every char-n-gram of the word.
            for gram in self.char_ngrams(&word) {
                if gram.is_empty() {
                    continue;
                }
                self.project_into(&gram, &mut v);
                feature_count += 1;
            }
        }

        // Empty input → zero vector → return a zero vector (norm 0).
        // The engine guards against division-by-zero by skipping
        // norm-zero queries before the dot-product loop.
        if feature_count == 0 {
            return v;
        }

        // L2-normalise so similarity == dot product at runtime (AC5).
        let mut sumsq = 0.0_f32;
        for &x in &v {
            sumsq = x.mul_add(x, sumsq);
        }
        if sumsq > 0.0 {
            let inv_norm = 1.0_f32 / sumsq.sqrt();
            for x in &mut v {
                *x *= inv_norm;
            }
        }
        v
    }

    fn serialize_model(&self) -> Vec<u8> {
        // The hashed-projection encoder is fully described by its
        // 24-byte config — no materialised matrix needed. Format:
        //
        //   [0..4]   magic "SSGS"
        //   [4..8]   format_version (u32 LE)
        //   [8..12]  dim (u32 LE)
        //   [12..20] seed (u64 LE)
        //   [20]     ngram_min (u8)
        //   [21]     ngram_max (u8)
        //   [22..24] reserved (0, 0)
        let mut out = Vec::with_capacity(24);
        out.extend_from_slice(&crate::ARTIFACT_MAGIC);
        out.extend_from_slice(&crate::ARTIFACT_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.cfg.dim.to_le_bytes());
        out.extend_from_slice(&self.cfg.seed.to_le_bytes());
        out.push(self.cfg.ngram_min);
        out.push(self.cfg.ngram_max);
        out.extend_from_slice(&[0u8, 0u8]);
        out
    }

    fn serialize_tokenizer(&self) -> Vec<u8> {
        // Tokeniser config is just JSON because it's tiny and
        // human-inspectable. Header is still magic+version.
        let mut out = Vec::new();
        out.extend_from_slice(&crate::ARTIFACT_MAGIC);
        out.extend_from_slice(&crate::ARTIFACT_FORMAT_VERSION.to_le_bytes());
        // Tokenizer is fully determined by the encoder config — but we
        // serialise the n-gram bounds explicitly so an external reader
        // can reconstruct the boundary marker / lowercasing rules.
        let cfg = serde_json::json!({
            "kind": "hashed-ngram",
            "lowercase": true,
            "boundary_markers": "<>",
            "ngram_min": self.cfg.ngram_min,
            "ngram_max": self.cfg.ngram_max,
            "split": "non-alphanumeric",
        });
        let json = serde_json::to_vec(&cfg).unwrap_or_default();
        out.extend_from_slice(&(json.len() as u32).to_le_bytes());
        out.extend_from_slice(&json);
        out
    }
}

/// Deserialise a [`ProjectionEncoder`] from the bytes produced by
/// [`ProjectionEncoder::serialize_model`]. Returns `None` if the magic
/// header doesn't match or the version is unsupported.
///
/// # Examples
///
/// ```
/// use ssg_search::encoder::{
///     deserialize_projection_encoder, Encoder, ProjectionEncoder,
/// };
///
/// let enc = ProjectionEncoder::default();
/// let bytes = enc.serialize_model();
///
/// // Round-trips losslessly.
/// let restored = deserialize_projection_encoder(&bytes).unwrap();
/// assert_eq!(restored.config(), enc.config());
///
/// // Garbage / too-short input fails cleanly.
/// assert!(deserialize_projection_encoder(&[]).is_none());
/// assert!(deserialize_projection_encoder(&[0u8; 8]).is_none());
/// ```
#[must_use]
pub fn deserialize_projection_encoder(
    bytes: &[u8],
) -> Option<ProjectionEncoder> {
    if bytes.len() < 24 {
        return None;
    }
    if bytes[0..4] != crate::ARTIFACT_MAGIC {
        return None;
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    if version != crate::ARTIFACT_FORMAT_VERSION {
        return None;
    }
    let dim = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    let seed = u64::from_le_bytes(bytes[12..20].try_into().ok()?);
    let ngram_min = bytes[20];
    let ngram_max = bytes[21];
    if ngram_min == 0 || ngram_max < ngram_min || dim == 0 {
        return None;
    }
    Some(ProjectionEncoder::new(ProjectionConfig {
        dim,
        seed,
        ngram_min,
        ngram_max,
    }))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons
)]
mod tests {
    use super::*;

    fn unit_norm(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    #[test]
    fn default_dim_is_256() {
        assert_eq!(EMBEDDING_DIM, 256);
        assert_eq!(ProjectionEncoder::default().dim(), 256);
    }

    #[test]
    fn embed_is_l2_normalised() {
        let enc = ProjectionEncoder::default();
        let v = enc.embed("the quick brown fox jumps over the lazy dog");
        let n = unit_norm(&v);
        assert!((0.999..=1.001).contains(&n), "L2 norm out of range: {n}");
    }

    #[test]
    fn embed_dim_matches_config() {
        let enc = ProjectionEncoder::default();
        assert_eq!(enc.embed("hello world").len(), EMBEDDING_DIM);
    }

    #[test]
    fn embed_deterministic() {
        let enc = ProjectionEncoder::default();
        let a = enc.embed("static site generator with semantic search");
        let b = enc.embed("static site generator with semantic search");
        assert_eq!(a, b);
    }

    #[test]
    fn embed_empty_returns_zero_vector() {
        let enc = ProjectionEncoder::default();
        let v = enc.embed("");
        assert_eq!(v.len(), EMBEDDING_DIM);
        assert_eq!(unit_norm(&v), 0.0);
    }

    #[test]
    fn embed_whitespace_only_returns_zero_vector() {
        let enc = ProjectionEncoder::default();
        let v = enc.embed("   \t\n  ");
        assert_eq!(unit_norm(&v), 0.0);
    }

    #[test]
    fn related_docs_score_higher_than_unrelated() {
        let enc = ProjectionEncoder::default();
        let q = enc.embed("rust web assembly");
        let a = enc.embed("rust webassembly module loaded in browser");
        let b = enc.embed("baking sourdough bread");
        let sim_a: f32 = q.iter().zip(&a).map(|(x, y)| x * y).sum();
        let sim_b: f32 = q.iter().zip(&b).map(|(x, y)| x * y).sum();
        assert!(
            sim_a > sim_b,
            "expected related doc to score higher: a={sim_a}, b={sim_b}"
        );
    }

    #[test]
    fn serialize_model_round_trip() {
        let enc = ProjectionEncoder::default();
        let bytes = enc.serialize_model();
        assert_eq!(bytes.len(), 24);
        assert_eq!(&bytes[0..4], &crate::ARTIFACT_MAGIC);
        let de = deserialize_projection_encoder(&bytes).unwrap();
        assert_eq!(de.config(), enc.config());
    }

    #[test]
    fn deserialize_rejects_bad_magic() {
        let mut bytes = ProjectionEncoder::default().serialize_model();
        bytes[0] = b'X';
        assert!(deserialize_projection_encoder(&bytes).is_none());
    }

    #[test]
    fn deserialize_rejects_short_input() {
        assert!(deserialize_projection_encoder(&[]).is_none());
        assert!(deserialize_projection_encoder(&[0u8; 8]).is_none());
    }

    #[test]
    fn deserialize_rejects_bad_version() {
        let mut bytes = ProjectionEncoder::default().serialize_model();
        bytes[4] = 0xFF;
        assert!(deserialize_projection_encoder(&bytes).is_none());
    }

    #[test]
    fn deserialize_rejects_zero_dim() {
        let mut bytes = ProjectionEncoder::default().serialize_model();
        // overwrite dim field (bytes 8..12) with zero
        bytes[8..12].copy_from_slice(&0u32.to_le_bytes());
        assert!(deserialize_projection_encoder(&bytes).is_none());
    }

    #[test]
    fn serialize_tokenizer_starts_with_magic() {
        let bytes = ProjectionEncoder::default().serialize_tokenizer();
        assert!(bytes.len() > 12);
        assert_eq!(&bytes[0..4], &crate::ARTIFACT_MAGIC);
        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(version, crate::ARTIFACT_FORMAT_VERSION);
        let len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        assert_eq!(bytes.len(), 12 + len);
        // Body must be valid JSON
        let json: serde_json::Value =
            serde_json::from_slice(&bytes[12..]).unwrap();
        assert_eq!(json["kind"], "hashed-ngram");
        assert_eq!(json["lowercase"], true);
    }

    #[test]
    fn config_default_is_sensible() {
        let cfg = ProjectionConfig::default();
        assert_eq!(cfg.dim, EMBEDDING_DIM as u32);
        assert_eq!(cfg.ngram_min, 3);
        assert_eq!(cfg.ngram_max, 5);
    }

    #[test]
    fn hash_is_deterministic_across_calls() {
        let h1 = ProjectionEncoder::hash(42, "hello");
        let h2 = ProjectionEncoder::hash(42, "hello");
        assert_eq!(h1, h2);
        let h3 = ProjectionEncoder::hash(43, "hello");
        assert_ne!(h1, h3);
    }

    #[test]
    fn tokens_splits_on_non_alphanumeric() {
        let toks: Vec<_> =
            ProjectionEncoder::tokens("Hello, world! Foo-bar_baz.").collect();
        assert_eq!(toks, vec!["hello", "world", "foo", "bar", "baz"]);
    }

    #[test]
    fn char_ngrams_includes_boundary_markers() {
        let enc = ProjectionEncoder::default();
        let grams: Vec<_> = enc.char_ngrams("hi").collect();
        // "<hi>" — trigrams: "<hi", "hi>" (len 3 windows)
        assert!(grams.iter().any(|g| g == "<hi"));
        assert!(grams.iter().any(|g| g == "hi>"));
    }
}
