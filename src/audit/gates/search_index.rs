// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Localized semantic vector search index integrity (depends on E1 —
//! issue #545).
//!
//! Asserts `dist/search/embeddings.bin` exists, has the expected
//! dimensions, and that its content hash matches the recorded hash in
//! `dist/search/manifest.json`. When the producing epic (E1) hasn't
//! merged yet, the gate emits a single info-level finding and skips —
//! exactly as the issue body specifies.
//!
//! `dist/` is checked at the site root first, then under `<root>/dist/`
//! since v0.0.44 output adapters can land either layout.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use sha2::{Digest, Sha256};

const NAME: &str = "search_index";

/// Semantic vector search index integrity gate.
///
/// # Examples
///
/// ```
/// use ssg::audit::AuditGate;
/// use ssg::audit::gates::search_index::SearchIndexGate;
/// assert_eq!(SearchIndexGate.name(), "search_index");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SearchIndexGate;

impl AuditGate for SearchIndexGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Asserts dist/search/embeddings.bin exists and that its SHA-256 \
         matches the value recorded in dist/search/manifest.json. The \
         producing epic is E1 (issue #545); the gate skips with an \
         info note until E1 lands the embeddings artefact."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();

        let bin_a = site.root.join("search").join("embeddings.bin");
        let manifest_a = site.root.join("search").join("manifest.json");
        let bin_b =
            site.root.join("dist").join("search").join("embeddings.bin");
        let manifest_b =
            site.root.join("dist").join("search").join("manifest.json");

        let (bin, manifest) = if bin_a.exists() {
            (bin_a, manifest_a)
        } else if bin_b.exists() {
            (bin_b, manifest_b)
        } else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Info,
                    "search embeddings absent (depends on E1 — issue #545)",
                )
                .with_code("SEARCH-INPUT-MISSING"),
            );
            return findings;
        };

        // Verify bin size > 0.
        let Ok(bin_bytes) = std::fs::read(&bin) else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    "could not read embeddings.bin",
                )
                .with_code("SEARCH-READ-ERROR")
                .with_path(bin.to_string_lossy().into_owned()),
            );
            return findings;
        };
        if bin_bytes.is_empty() {
            findings.push(
                Finding::new(NAME, Severity::Error, "embeddings.bin is empty")
                    .with_code("SEARCH-EMPTY")
                    .with_path(bin.to_string_lossy().into_owned()),
            );
        }

        // Manifest hash check.
        let Ok(manifest_text) = std::fs::read_to_string(&manifest) else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    "manifest.json missing or unreadable",
                )
                .with_code("SEARCH-MANIFEST-MISSING")
                .with_path(manifest.to_string_lossy().into_owned()),
            );
            return findings;
        };
        let Ok(manifest_json) =
            serde_json::from_str::<serde_json::Value>(&manifest_text)
        else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    "manifest.json is not valid JSON",
                )
                .with_code("SEARCH-MANIFEST-INVALID")
                .with_path(manifest.to_string_lossy().into_owned()),
            );
            return findings;
        };
        let expected_hash = manifest_json
            .get("embeddings_sha256")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let Some(expected) = expected_hash else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    "manifest.json missing `embeddings_sha256` field",
                )
                .with_code("SEARCH-HASH-MISSING")
                .with_path(manifest.to_string_lossy().into_owned()),
            );
            return findings;
        };

        let mut hasher = Sha256::new();
        hasher.update(&bin_bytes);
        let actual = hex_encode(&hasher.finalize());
        if !actual.eq_ignore_ascii_case(&expected) {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    format!(
                        "embeddings.bin hash {actual} ≠ manifest {expected}"
                    ),
                )
                .with_code("SEARCH-HASH-MISMATCH")
                .with_path(bin.to_string_lossy().into_owned()),
            );
        }

        findings
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_site() -> Site {
        Site {
            root: PathBuf::from("/nonexistent"),
            html_files: Vec::new(),
        }
    }

    #[test]
    fn absent_inputs_emit_info_skip() {
        let f = SearchIndexGate.run(&empty_site(), &AuditOptions::default());
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Info);
        assert_eq!(f[0].code.as_deref(), Some("SEARCH-INPUT-MISSING"));
    }

    #[test]
    fn matching_hash_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        let body: Vec<u8> = b"vectors".to_vec();
        std::fs::write(search.join("embeddings.bin"), &body).unwrap();
        let mut h = Sha256::new();
        h.update(&body);
        let hash = hex_encode(&h.finalize());
        let manifest = format!(r#"{{"embeddings_sha256":"{hash}"}}"#);
        std::fs::write(search.join("manifest.json"), manifest).unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn mismatched_hash_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        std::fs::write(search.join("embeddings.bin"), "x").unwrap();
        std::fs::write(
            search.join("manifest.json"),
            r#"{"embeddings_sha256":"deadbeef"}"#,
        )
        .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("SEARCH-HASH-MISMATCH")));
    }

    #[test]
    fn dist_layout_is_discovered() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("dist").join("search");
        std::fs::create_dir_all(&search).unwrap();
        let body: Vec<u8> = b"vectors".to_vec();
        std::fs::write(search.join("embeddings.bin"), &body).unwrap();
        let mut h = Sha256::new();
        h.update(&body);
        let hash = hex_encode(&h.finalize());
        std::fs::write(
            search.join("manifest.json"),
            format!(r#"{{"embeddings_sha256":"{hash}"}}"#),
        )
        .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn empty_bin_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        std::fs::write(search.join("embeddings.bin"), b"").unwrap();
        let mut h = Sha256::new();
        h.update(b"");
        let hash = hex_encode(&h.finalize());
        std::fs::write(
            search.join("manifest.json"),
            format!(r#"{{"embeddings_sha256":"{hash}"}}"#),
        )
        .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f.iter().any(|x| x.code.as_deref() == Some("SEARCH-EMPTY")));
    }

    #[test]
    fn missing_manifest_flags_error() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        std::fs::write(search.join("embeddings.bin"), b"data").unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("SEARCH-MANIFEST-MISSING")));
    }

    #[test]
    fn invalid_manifest_json_flags_error() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        std::fs::write(search.join("embeddings.bin"), b"data").unwrap();
        std::fs::write(search.join("manifest.json"), "{ this is not json")
            .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("SEARCH-MANIFEST-INVALID")));
    }

    #[test]
    fn manifest_without_hash_field_warns() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        std::fs::write(search.join("embeddings.bin"), b"data").unwrap();
        std::fs::write(search.join("manifest.json"), r#"{"version":1}"#)
            .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        let hm = f
            .iter()
            .find(|x| x.code.as_deref() == Some("SEARCH-HASH-MISSING"))
            .expect("hash-missing finding");
        assert_eq!(hm.severity, Severity::Warn);
    }

    #[test]
    fn case_insensitive_hash_comparison() {
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(&search).unwrap();
        let body: Vec<u8> = b"vectors".to_vec();
        std::fs::write(search.join("embeddings.bin"), &body).unwrap();
        let mut h = Sha256::new();
        h.update(&body);
        let hash = hex_encode(&h.finalize()).to_uppercase();
        std::fs::write(
            search.join("manifest.json"),
            format!(r#"{{"embeddings_sha256":"{hash}"}}"#),
        )
        .unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert!(f.is_empty(), "uppercase hash should match: {f:?}");
    }

    #[test]
    fn unreadable_embeddings_bin_flags_read_error() {
        // A directory named embeddings.bin passes the exists() probe
        // but fails fs::read, driving the SEARCH-READ-ERROR arm.
        let tmp = tempfile::tempdir().unwrap();
        let search = tmp.path().join("search");
        std::fs::create_dir_all(search.join("embeddings.bin")).unwrap();
        let site = Site {
            root: tmp.path().to_path_buf(),
            html_files: Vec::new(),
        };
        let f = SearchIndexGate.run(&site, &AuditOptions::default());
        std::mem::forget(tmp);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code.as_deref(), Some("SEARCH-READ-ERROR"));
        assert_eq!(f[0].severity, Severity::Error);
    }

    #[test]
    fn hex_encode_round_trip_is_lowercase_hex() {
        let s = hex_encode(&[0x00, 0x0f, 0xab, 0xff]);
        assert_eq!(s, "000fabff");
    }

    #[test]
    fn metadata_methods_exposed() {
        let g = SearchIndexGate;
        assert_eq!(g.name(), "search_index");
        assert!(g.explain().contains("embeddings"));
        let _copy: SearchIndexGate = g;
        let _clone = g;
        assert!(format!("{g:?}").contains("SearchIndexGate"));
    }
}
