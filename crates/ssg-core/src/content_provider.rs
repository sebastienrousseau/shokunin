// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `ContentProvider` — abstract I/O for the renderer.
//!
//! The build-time pipeline reads markdown + templates from the local
//! filesystem (`std::fs`). The Edge / WASM renderer needs the same
//! sources but from Cloudflare KV, Vercel Edge Config, an in-memory
//! cache, or anywhere else.
//!
//! This trait is the seam: every renderer code path that needs to
//! resolve a source dependency goes through `ContentProvider`.
//! Build-time uses [`FsContentProvider`]; runtime adapters
//! (Cloudflare Workers, Vercel Edge) supply their own implementations.
//!
//! ## Why in `ssg-core`?
//!
//! `ssg-core` is the WASM-compatible crate. The trait must compile to
//! `wasm32-unknown-unknown` so the same Rust renderer code can be
//! consumed by both the native build binary and the Edge WASM renderer.
//!
//! ## Determinism
//!
//! Implementations MUST be deterministic for the lifetime of a single
//! render request. If the same key is fetched twice in one render,
//! both calls must return the same bytes. Adapters that wrap a
//! mutable backing store (KV, Edge Config) should snapshot at the
//! start of a render request.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Outcome of a `ContentProvider` lookup.
///
/// Kept distinct from `Result<Option<…>>` because adapters frequently
/// want to distinguish a hard error (KV unreachable) from a benign
/// miss (key not in store).
///
/// # Examples
///
/// ```
/// use ssg_core::ProviderError;
///
/// let err = ProviderError::NotFound { key: "foo.md".into() };
/// assert!(err.to_string().contains("not found"));
/// assert!(err.to_string().contains("foo.md"));
/// ```
#[derive(Debug)]
pub enum ProviderError {
    /// Key was not present in the underlying store.
    NotFound {
        /// The key that was requested.
        key: String,
    },
    /// Backend I/O failure (network, disk, decode).
    Backend {
        /// Human-readable detail, suitable for logs.
        detail: String,
    },
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { key } => {
                write!(f, "ContentProvider: key not found: {key}")
            }
            Self::Backend { detail } => {
                write!(f, "ContentProvider: backend error: {detail}")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

/// Specialised `Result` for [`ContentProvider`] lookups.
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Abstract content store consumed by the renderer.
///
/// Keys are stable, URL-safe path-shaped strings (`content/posts/foo.md`,
/// `templates/post.html`). Adapters MAY mangle keys internally (KV
/// namespace prefixing, slash-to-underscore, etc.) but MUST present the
/// canonical key surface to the renderer.
///
/// ## Object safety
///
/// The trait is intentionally object-safe so renderer code can hold a
/// `&dyn ContentProvider` without monomorphising every site that uses
/// a different adapter.
pub trait ContentProvider {
    /// Fetches the raw bytes for `key`, or returns an error.
    ///
    /// Implementations should be cheap to call — the renderer may
    /// fetch the same key multiple times in a single request and
    /// expects in-process memoisation upstream.
    ///
    /// # Errors
    /// - [`ProviderError::NotFound`] if `key` is not present.
    /// - [`ProviderError::Backend`] for any other failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::{ContentProvider, MemoryContentProvider};
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// mem.insert("page.md", b"# Hello".to_vec());
    /// let bytes = mem.fetch("page.md").unwrap();
    /// assert_eq!(bytes, b"# Hello");
    /// ```
    fn fetch(&self, key: &str) -> ProviderResult<Vec<u8>>;

    /// Convenience: fetches `key` and decodes as UTF-8.
    ///
    /// Default impl wraps [`Self::fetch`] + `String::from_utf8`.
    /// Adapters that store text natively (KV strings, Edge Config
    /// JSON values) can override for a zero-copy path.
    ///
    /// # Errors
    /// - Any error returned by [`Self::fetch`].
    /// - [`ProviderError::Backend`] if the bytes are not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::{ContentProvider, MemoryContentProvider};
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// mem.insert("a.md", b"hello".to_vec());
    /// assert_eq!(mem.fetch_string("a.md").unwrap(), "hello");
    /// ```
    fn fetch_string(&self, key: &str) -> ProviderResult<String> {
        let bytes = self.fetch(key)?;
        String::from_utf8(bytes).map_err(|e| ProviderError::Backend {
            detail: format!("invalid utf-8 in {key}: {e}"),
        })
    }

    /// Reports whether `key` exists without materialising the bytes.
    ///
    /// Default impl delegates to [`Self::fetch`] and discards the
    /// payload. Adapters with a cheaper HEAD-style probe (CDN cache,
    /// KV metadata) SHOULD override.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::{ContentProvider, MemoryContentProvider};
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// mem.insert("k", b"v".to_vec());
    /// assert!(mem.contains("k"));
    /// assert!(!mem.contains("missing"));
    /// ```
    fn contains(&self, key: &str) -> bool {
        self.fetch(key).is_ok()
    }
}

// ---------------------------------------------------------------------------
// FsContentProvider — std::fs-backed (build time)
// ---------------------------------------------------------------------------

/// Filesystem-backed `ContentProvider` for the build-time pipeline.
///
/// Resolves keys relative to a configured root directory. This is the
/// default adapter used by `ssg build` and is intentionally a thin
/// wrapper around `std::fs::read` so the existing batch pipeline keeps
/// its byte-identical behaviour (AC9).
///
/// # Examples
///
/// ```
/// use ssg_core::{ContentProvider, FsContentProvider};
///
/// let dir = tempfile::tempdir().unwrap();
/// std::fs::write(dir.path().join("a.md"), b"# A").unwrap();
/// let fs = FsContentProvider::new(dir.path());
/// assert_eq!(fs.fetch("a.md").unwrap(), b"# A");
/// ```
#[derive(Debug, Clone)]
pub struct FsContentProvider {
    root: PathBuf,
}

impl FsContentProvider {
    /// Constructs an `FsContentProvider` rooted at `root`.
    ///
    /// `root` is typically the site directory — every fetched key is
    /// resolved as `root.join(key)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::FsContentProvider;
    ///
    /// let dir = tempfile::tempdir().unwrap();
    /// let fs = FsContentProvider::new(dir.path());
    /// assert_eq!(fs.root(), dir.path());
    /// ```
    #[must_use]
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Returns the configured root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::FsContentProvider;
    /// use std::path::Path;
    ///
    /// let fs = FsContentProvider::new("/tmp/site");
    /// assert_eq!(fs.root(), Path::new("/tmp/site"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a key against the configured root.
    ///
    /// Rejects keys containing `..` segments to prevent escape from
    /// the root directory — adapters MUST NOT trust untrusted keys at
    /// the Edge and the same caution applies at build time.
    fn resolve(&self, key: &str) -> ProviderResult<PathBuf> {
        if key.split('/').any(|seg| seg == "..") {
            return Err(ProviderError::Backend {
                detail: format!("rejected traversal key: {key}"),
            });
        }
        Ok(self.root.join(key))
    }
}

impl ContentProvider for FsContentProvider {
    fn fetch(&self, key: &str) -> ProviderResult<Vec<u8>> {
        let path = self.resolve(key)?;
        match std::fs::read(&path) {
            Ok(bytes) => Ok(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ProviderError::NotFound { key: key.into() })
            }
            Err(e) => Err(ProviderError::Backend {
                detail: format!("read {}: {e}", path.display()),
            }),
        }
    }

    fn contains(&self, key: &str) -> bool {
        self.resolve(key).is_ok_and(|p| p.exists())
    }
}

// ---------------------------------------------------------------------------
// MemoryContentProvider — in-process map (tests + WASM bootstrap)
// ---------------------------------------------------------------------------

/// In-memory `ContentProvider` backed by a key→bytes map.
///
/// Suited to unit tests (no tempdir setup) and to the WASM Edge
/// runtime where the JS host pre-loads a small set of source files
/// before calling `render_page_isr`.
///
/// # Examples
///
/// ```
/// use ssg_core::{ContentProvider, MemoryContentProvider};
///
/// let mut mem = MemoryContentProvider::new();
/// mem.insert("a", b"1".to_vec());
/// assert!(mem.contains("a"));
/// assert_eq!(mem.fetch("a").unwrap(), b"1");
/// ```
#[derive(Debug, Clone, Default)]
pub struct MemoryContentProvider {
    map: BTreeMap<String, Vec<u8>>,
}

impl MemoryContentProvider {
    /// Constructs an empty `MemoryContentProvider`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::MemoryContentProvider;
    ///
    /// let mem = MemoryContentProvider::new();
    /// assert!(mem.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a key/value pair, returning the previous value (if any).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::MemoryContentProvider;
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// assert!(mem.insert("k", b"v1".to_vec()).is_none());
    /// let prev = mem.insert("k", b"v2".to_vec());
    /// assert_eq!(prev.as_deref(), Some(&b"v1"[..]));
    /// ```
    pub fn insert<K: Into<String>, V: Into<Vec<u8>>>(
        &mut self,
        key: K,
        value: V,
    ) -> Option<Vec<u8>> {
        self.map.insert(key.into(), value.into())
    }

    /// Returns the number of keys currently stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::MemoryContentProvider;
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// assert_eq!(mem.len(), 0);
    /// mem.insert("a", b"x".to_vec());
    /// mem.insert("b", b"y".to_vec());
    /// assert_eq!(mem.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Reports whether the provider holds no entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_core::MemoryContentProvider;
    ///
    /// let mut mem = MemoryContentProvider::new();
    /// assert!(mem.is_empty());
    /// mem.insert("k", b"v".to_vec());
    /// assert!(!mem.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl ContentProvider for MemoryContentProvider {
    fn fetch(&self, key: &str) -> ProviderResult<Vec<u8>> {
        self.map
            .get(key)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound { key: key.into() })
    }

    fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn memory_provider_round_trip() {
        let mut mem = MemoryContentProvider::new();
        assert!(mem.is_empty());
        let _ = mem.insert("a.md", b"hello".to_vec());
        assert_eq!(mem.len(), 1);
        assert!(!mem.is_empty());
        assert!(mem.contains("a.md"));
        assert!(!mem.contains("missing"));

        let bytes = mem.fetch("a.md").unwrap();
        assert_eq!(bytes, b"hello");
        let text = mem.fetch_string("a.md").unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn memory_provider_not_found_is_distinct() {
        let mem = MemoryContentProvider::new();
        match mem.fetch("nope") {
            Err(ProviderError::NotFound { key }) => assert_eq!(key, "nope"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn memory_provider_invalid_utf8_is_backend_error() {
        let mut mem = MemoryContentProvider::new();
        let _ = mem.insert("bad", vec![0xffu8, 0xfe, 0xfd]);
        match mem.fetch_string("bad") {
            Err(ProviderError::Backend { detail }) => {
                assert!(detail.contains("invalid utf-8"));
            }
            other => panic!("expected Backend, got {other:?}"),
        }
    }

    #[test]
    fn provider_error_display() {
        let nf = ProviderError::NotFound { key: "a".into() };
        let be = ProviderError::Backend {
            detail: "boom".into(),
        };
        assert!(format!("{nf}").contains("not found"));
        assert!(format!("{be}").contains("backend"));
    }

    #[test]
    fn fs_provider_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hello.md");
        std::fs::write(&path, b"# Hello").unwrap();

        let fs = FsContentProvider::new(dir.path());
        assert_eq!(fs.root(), dir.path());
        let bytes = fs.fetch("hello.md").unwrap();
        assert_eq!(bytes, b"# Hello");
        assert!(fs.contains("hello.md"));
        assert!(!fs.contains("absent.md"));
    }

    #[test]
    fn fs_provider_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsContentProvider::new(dir.path());
        match fs.fetch("../etc/passwd") {
            Err(ProviderError::Backend { detail }) => {
                assert!(detail.contains("traversal"));
            }
            other => panic!("expected traversal rejection, got {other:?}"),
        }
        assert!(!fs.contains("../etc/passwd"));
    }

    #[test]
    fn fs_provider_missing_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsContentProvider::new(dir.path());
        match fs.fetch("nope.md") {
            Err(ProviderError::NotFound { key }) => assert_eq!(key, "nope.md"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn provider_error_debug() {
        let nf = ProviderError::NotFound { key: "k".into() };
        let s = format!("{nf:?}");
        assert!(s.contains("NotFound"));
    }

    #[test]
    fn fs_provider_root_accessor() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsContentProvider::new(dir.path());
        assert_eq!(fs.root(), dir.path());
    }

    #[test]
    fn fs_provider_clone() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsContentProvider::new(dir.path());
        let cloned = fs.clone();
        assert_eq!(cloned.root(), fs.root());
    }

    #[test]
    fn fs_provider_fetch_string_decodes_utf8() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "héllo").unwrap();
        let fs = FsContentProvider::new(dir.path());
        assert_eq!(fs.fetch_string("a.md").unwrap(), "héllo");
    }

    #[test]
    fn fs_provider_fetch_string_rejects_invalid_utf8() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.md"), [0xffu8, 0xfe, 0xfd])
            .unwrap();
        let fs = FsContentProvider::new(dir.path());
        match fs.fetch_string("bad.md") {
            Err(ProviderError::Backend { detail }) => {
                assert!(detail.contains("invalid utf-8"));
            }
            other => panic!("expected Backend, got {other:?}"),
        }
    }

    #[test]
    fn fs_provider_contains_when_present() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "x").unwrap();
        let fs = FsContentProvider::new(dir.path());
        assert!(fs.contains("a.md"));
    }

    #[test]
    fn fs_provider_nested_traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let fs = FsContentProvider::new(dir.path());
        match fs.fetch("a/../../b") {
            Err(ProviderError::Backend { detail }) => {
                assert!(detail.contains("traversal"));
            }
            other => panic!("expected traversal rejection, got {other:?}"),
        }
    }

    #[test]
    fn memory_provider_insert_returns_previous_value() {
        let mut mem = MemoryContentProvider::new();
        assert!(mem.insert("k", b"v1".to_vec()).is_none());
        let prev = mem.insert("k", b"v2".to_vec());
        assert_eq!(prev.as_deref(), Some(&b"v1"[..]));
        assert_eq!(mem.fetch("k").unwrap(), b"v2");
    }

    #[test]
    fn memory_provider_default_equivalent_to_new() {
        let a = MemoryContentProvider::default();
        let b = MemoryContentProvider::new();
        assert_eq!(a.len(), b.len());
        assert!(a.is_empty());
    }

    #[test]
    fn provider_error_display_messages() {
        let nf = ProviderError::NotFound { key: "x".into() };
        assert_eq!(format!("{nf}"), "ContentProvider: key not found: x");
        let be = ProviderError::Backend { detail: "y".into() };
        assert_eq!(format!("{be}"), "ContentProvider: backend error: y");
    }

    #[test]
    fn provider_error_is_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(ProviderError::NotFound { key: "k".into() });
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn memory_provider_contains_via_trait_object() {
        let mut mem = MemoryContentProvider::new();
        let _ = mem.insert("a", b"1".to_vec());
        let provider: &dyn ContentProvider = &mem;
        assert!(provider.contains("a"));
        assert!(!provider.contains("missing"));
    }
}
