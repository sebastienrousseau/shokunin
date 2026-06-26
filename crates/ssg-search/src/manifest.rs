// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `manifest.json` — the human-readable map from corpus row index to
//! `{url, title, excerpt}`.
//!
//! The manifest is loaded once on the JS side and consulted to render
//! search results. The WASM engine never reads it — boundary stays
//! pure `Float32Array` (AC4).

use serde::{Deserialize, Serialize};

/// One entry in the manifest, indexed by its row position in
/// `embeddings.bin`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Relative URL to the document (e.g. `/blog/post.html`).
    pub url: String,
    /// Page title.
    pub title: String,
    /// Short excerpt (~160 chars) for the search-result snippet.
    pub excerpt: String,
}

/// The full manifest written to `manifest.json`.
///
/// `entries[i]` corresponds to the i-th vector in `embeddings.bin`.
/// The `dim`, `count`, and `model_hash` fields let the loader sanity
/// check the bundle before allocating buffers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Embedding dimensionality. Must match every vector in
    /// `embeddings.bin`.
    pub dim: u32,
    /// Number of vectors in `embeddings.bin`. Must equal
    /// `entries.len()`.
    pub count: u32,
    /// Hex-encoded SHA-256 of `model.bin` — lets the loader bail
    /// fast if the model and embeddings were built from different
    /// encoder versions.
    pub model_hash: String,
    /// Format version stamped into the artifacts.
    pub format_version: u32,
    /// One entry per row of `embeddings.bin`.
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    /// Constructs a new manifest. Asserts (debug-only) that `count`
    /// and `entries.len()` agree.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::len is non-const on stable
    pub fn new(
        dim: u32,
        model_hash: String,
        entries: Vec<ManifestEntry>,
    ) -> Self {
        let count = entries.len() as u32;
        Self {
            dim,
            count,
            model_hash,
            format_version: crate::ARTIFACT_FORMAT_VERSION,
            entries,
        }
    }

    /// Returns true if the manifest is internally consistent.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::is_empty is non-const on stable
    pub fn is_valid(&self) -> bool {
        (self.count as usize) == self.entries.len()
            && self.dim > 0
            && !self.entries.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest::new(
            256,
            "deadbeef".to_string(),
            vec![
                ManifestEntry {
                    url: "/a".to_string(),
                    title: "A".to_string(),
                    excerpt: "ex a".to_string(),
                },
                ManifestEntry {
                    url: "/b".to_string(),
                    title: "B".to_string(),
                    excerpt: "ex b".to_string(),
                },
            ],
        )
    }

    #[test]
    fn manifest_new_sets_count() {
        let m = sample();
        assert_eq!(m.count, 2);
        assert_eq!(m.dim, 256);
        assert_eq!(m.format_version, crate::ARTIFACT_FORMAT_VERSION);
    }

    #[test]
    fn manifest_is_valid_for_well_formed() {
        assert!(sample().is_valid());
    }

    #[test]
    fn manifest_invalid_for_empty_entries() {
        let m = Manifest::new(256, "x".to_string(), vec![]);
        assert!(!m.is_valid());
    }

    #[test]
    fn manifest_invalid_for_zero_dim() {
        let mut m = sample();
        m.dim = 0;
        assert!(!m.is_valid());
    }

    #[test]
    fn manifest_invalid_for_count_mismatch() {
        let mut m = sample();
        m.count = 99;
        assert!(!m.is_valid());
    }

    #[test]
    fn manifest_round_trips_json() {
        let m = sample();
        let json = serde_json::to_string(&m).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn manifest_entry_round_trips_json() {
        let e = ManifestEntry {
            url: "/u".to_string(),
            title: "t".to_string(),
            excerpt: "ex".to_string(),
        };
        let s = serde_json::to_string(&e).unwrap();
        let back: ManifestEntry = serde_json::from_str(&s).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn manifest_entry_clone_is_independent() {
        let e = ManifestEntry {
            url: "/x".to_string(),
            title: "T".to_string(),
            excerpt: "E".to_string(),
        };
        let cloned = e.clone();
        assert_eq!(e, cloned);
        // Equality should hold even though strings are owned copies.
        assert_ne!(e.url.as_ptr(), cloned.url.as_ptr());
    }

    #[test]
    fn manifest_clone_is_value_equal() {
        let m = sample();
        let cloned = m.clone();
        assert_eq!(m, cloned);
    }

    #[test]
    fn manifest_debug_contains_entry_count() {
        let m = sample();
        let d = format!("{m:?}");
        assert!(d.contains("count"));
        assert!(d.contains("entries"));
    }

    #[test]
    fn manifest_json_has_expected_fields() {
        let m = sample();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"dim\":256"));
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"model_hash\":\"deadbeef\""));
        assert!(json.contains("\"format_version\":1"));
        assert!(json.contains("\"entries\""));
    }

    #[test]
    fn manifest_new_with_empty_entries_has_zero_count() {
        let m = Manifest::new(16, "h".to_string(), vec![]);
        assert_eq!(m.count, 0);
        assert!(!m.is_valid());
    }

    #[test]
    fn manifest_with_single_entry_is_valid() {
        let m = Manifest::new(
            8,
            "model".to_string(),
            vec![ManifestEntry {
                url: "/only".into(),
                title: "Only".into(),
                excerpt: "ex".into(),
            }],
        );
        assert!(m.is_valid());
        assert_eq!(m.count, 1);
    }

    #[test]
    fn manifest_invalid_when_count_smaller_than_entries() {
        let mut m = sample();
        m.count = 1; // entries.len() == 2
        assert!(!m.is_valid());
    }

    #[test]
    fn manifest_entry_eq_distinguishes_url() {
        let a = ManifestEntry {
            url: "/a".into(),
            title: "T".into(),
            excerpt: "E".into(),
        };
        let b = ManifestEntry {
            url: "/b".into(),
            title: "T".into(),
            excerpt: "E".into(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn manifest_eq_distinguishes_model_hash() {
        let mut a = sample();
        let mut b = sample();
        assert_eq!(a, b);
        a.model_hash = "x".into();
        b.model_hash = "y".into();
        assert_ne!(a, b);
    }
}
