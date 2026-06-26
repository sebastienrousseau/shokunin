// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Content fingerprinting for incremental builds.
//!
//! This module provides `BuildCache`, which tracks SHA-256-style
//! fingerprints of content files so that only files modified since the
//! last build need to be re-processed.
//!
//! # Overview
//!
//! 1. On startup, call `BuildCache::load` to read the previous
//!    fingerprint map from `.ssg-cache.json`.
//! 2. Call `BuildCache::changed_files` with the content directory to
//!    obtain the list of files whose contents have changed (or are new).
//! 3. After a successful build, call `BuildCache::update` to record
//!    the current fingerprints, then `BuildCache::save` to persist
//!    them to disk.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use ssg::cache::BuildCache;
//!
//! let cache_path = Path::new(".ssg-cache.json");
//! let content_dir = Path::new("content");
//!
//! let mut cache = BuildCache::load(cache_path).unwrap();
//! let changed = cache.changed_files(content_dir).unwrap();
//!
//! // … build only `changed` files …
//!
//! cache.update(content_dir).unwrap();
//! cache.save().unwrap();
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Default name for the on-disk cache file.
const DEFAULT_CACHE_FILE: &str = ".ssg-cache.json";

/// Persisted fingerprint map used for incremental builds.
///
/// Each entry maps a file path (relative to the content directory) to a
/// hex-encoded hash of that file's contents. Comparing the stored hash
/// against the current hash tells us whether the file has changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    /// Path to the cache file on disk.
    #[serde(skip)]
    cache_path: PathBuf,

    /// Map from relative file paths to their content fingerprints.
    fingerprints: HashMap<PathBuf, String>,
}

impl BuildCache {
    // -----------------------------------------------------------------
    // Construction / persistence
    // -----------------------------------------------------------------

    /// Load a previously saved cache from `cache_path`.
    ///
    /// If the file does not exist a fresh, empty cache is returned.
    /// Any other I/O or parse error is propagated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let cache_path = dir.path().join(".ssg-cache.json");
    /// // Loading a missing path returns a fresh, empty cache.
    /// let cache = BuildCache::load(&cache_path).unwrap();
    /// assert!(cache.is_empty());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or
    /// contains invalid JSON.
    pub fn load(cache_path: &Path) -> Result<Self> {
        if !cache_path.exists() {
            return Ok(Self {
                cache_path: cache_path.to_path_buf(),
                fingerprints: HashMap::new(),
            });
        }

        fail_point!("cache::read", |_| {
            anyhow::bail!("injected: cache::read")
        });
        let data = fs::read_to_string(cache_path).with_context(|| {
            format!("failed to read cache file: {}", cache_path.display())
        })?;

        fail_point!("cache::parse", |_| {
            anyhow::bail!("injected: cache::parse")
        });
        let mut cache: Self =
            serde_json::from_str(&data).with_context(|| {
                format!("failed to parse cache file: {}", cache_path.display())
            })?;

        cache.cache_path = cache_path.to_path_buf();
        Ok(cache)
    }

    /// Create a new empty cache that will be written to `cache_path`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use std::path::Path;
    ///
    /// let cache = BuildCache::new(Path::new(".ssg-cache.json"));
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[must_use]
    pub fn new(cache_path: &Path) -> Self {
        Self {
            cache_path: cache_path.to_path_buf(),
            fingerprints: HashMap::new(),
        }
    }

    /// Persist the current fingerprint map to the cache file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let cache = BuildCache::new(&dir.path().join("cache.json"));
    /// cache.save().unwrap();
    /// assert!(dir.path().join("cache.json").exists());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("failed to serialize cache")?;
        fail_point!("cache::write", |_| {
            anyhow::bail!("injected: cache::write")
        });
        fs::write(&self.cache_path, json).with_context(|| {
            format!("failed to write cache file: {}", self.cache_path.display())
        })?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Fingerprinting helpers
    // -----------------------------------------------------------------

    /// Compute a deterministic hex fingerprint of the given file.
    ///
    /// Uses streaming I/O via `stream::stream_hash` — reads in 8 KB
    /// chunks so memory usage is constant regardless of file size.
    fn fingerprint(path: &Path) -> Result<String> {
        crate::stream::stream_hash(path)
    }

    /// Recursively collect all files under `dir`, returning paths
    /// relative to `dir`.
    fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if !dir.exists() {
            return Ok(files);
        }
        Self::walk(dir, dir, &mut files)?;
        files.sort();
        Ok(files)
    }

    /// Recursive directory walker.
    fn walk(base: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let entries = fs::read_dir(current).with_context(|| {
            format!("cannot read directory: {}", current.display())
        })?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk(base, &path, out)?;
            } else {
                let rel = path
                    .strip_prefix(base)
                    .with_context(|| "strip_prefix failed")?;
                out.push(rel.to_path_buf());
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // Public query / mutation API
    // -----------------------------------------------------------------

    /// Return the set of files in `content_dir` that have changed since
    /// the fingerprints were last recorded, plus any newly added files.
    ///
    /// Deleted files (present in cache but absent on disk) are *not*
    /// included in the returned list, but they will be removed from the
    /// internal map on the next [`update`](Self::update) call.
    ///
    /// The returned paths are **absolute**.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use tempfile::tempdir;
    /// use std::fs;
    ///
    /// let dir = tempdir().unwrap();
    /// let content = dir.path().join("content");
    /// fs::create_dir(&content).unwrap();
    /// fs::write(content.join("a.md"), "hello").unwrap();
    /// let cache = BuildCache::new(&dir.path().join("cache.json"));
    /// // Every file is "changed" against an empty cache.
    /// let changed = cache.changed_files(&content).unwrap();
    /// assert_eq!(changed.len(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if `content_dir` cannot be read or individual
    /// files cannot be hashed.
    pub fn changed_files(&self, content_dir: &Path) -> Result<Vec<PathBuf>> {
        let files = Self::collect_files(content_dir)?;
        let mut changed = Vec::new();

        for rel in &files {
            let abs = content_dir.join(rel);
            let hash = Self::fingerprint(&abs)?;

            match self.fingerprints.get(rel) {
                Some(cached) if *cached == hash => {
                    // unchanged -- skip
                }
                _ => {
                    changed.push(abs);
                }
            }
        }

        Ok(changed)
    }

    /// Re-scan `content_dir` and replace the entire fingerprint map
    /// with fresh hashes.
    ///
    /// Call this after a successful build so the next invocation of
    /// [`changed_files`](Self::changed_files) reflects the new state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use tempfile::tempdir;
    /// use std::fs;
    ///
    /// let dir = tempdir().unwrap();
    /// let content = dir.path().join("content");
    /// fs::create_dir(&content).unwrap();
    /// fs::write(content.join("a.md"), "hi").unwrap();
    /// let mut cache = BuildCache::new(&dir.path().join("cache.json"));
    /// cache.update(&content).unwrap();
    /// assert_eq!(cache.len(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be read.
    pub fn update(&mut self, content_dir: &Path) -> Result<()> {
        let files = Self::collect_files(content_dir)?;
        let mut map = HashMap::with_capacity(files.len());

        for rel in files {
            let abs = content_dir.join(&rel);
            let hash = Self::fingerprint(&abs)?;
            let _prev = map.insert(rel, hash);
        }

        self.fingerprints = map;
        Ok(())
    }

    /// Return the number of entries currently in the fingerprint map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use std::path::Path;
    ///
    /// let cache = BuildCache::new(Path::new("cache.json"));
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    /// Return `true` if the fingerprint map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    /// use std::path::Path;
    ///
    /// let cache = BuildCache::new(Path::new("cache.json"));
    /// assert!(cache.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    /// Return the path to the default cache file relative to the
    /// project root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::cache::BuildCache;
    ///
    /// assert_eq!(BuildCache::default_path(), ".ssg-cache.json");
    /// ```
    #[must_use]
    pub const fn default_path() -> &'static str {
        DEFAULT_CACHE_FILE
    }
}

// =====================================================================
// Tests
// =====================================================================
#[cfg(test)]
#[allow(unused_results, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: create a temp dir with a few content files and return
    /// `(tmp_dir, content_dir, cache_path)`.
    fn setup() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().ok().unwrap();
        let content = tmp.path().join("content");
        fs::create_dir_all(&content).ok();
        let cache_path = tmp.path().join(".ssg-cache.json");
        (tmp, content, cache_path)
    }

    fn write_file(dir: &Path, name: &str, contents: &str) {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&p, contents).ok();
    }

    // 1. Loading a missing cache yields an empty map.
    #[test]
    fn load_missing_cache() {
        let tmp = TempDir::new().ok().unwrap();
        let cache_path = tmp.path().join("nonexistent.json");
        let cache = BuildCache::load(&cache_path).ok().unwrap();
        assert!(cache.is_empty());
    }

    // 2. Loading a valid cache round-trips correctly.
    #[test]
    fn load_valid_cache() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "hello");

        let mut cache = BuildCache::load(&cache_path).ok().unwrap();
        cache.update(&content).ok();
        cache.save().ok();

        let loaded = BuildCache::load(&cache_path).ok().unwrap();
        assert_eq!(loaded.len(), 1);
    }

    // 3. Detect changed files.
    #[test]
    fn detect_changes() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "v1");

        let mut cache = BuildCache::load(&cache_path).ok().unwrap();
        cache.update(&content).ok();
        cache.save().ok();

        // Modify the file.
        write_file(&content, "a.md", "v2");

        let cache2 = BuildCache::load(&cache_path).ok().unwrap();
        let changed = cache2.changed_files(&content).ok().unwrap();
        assert_eq!(changed.len(), 1);
        assert!(changed[0].ends_with("a.md"));
    }

    // 4. No changes detected when content is identical.
    #[test]
    fn detect_no_changes() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "same");

        let mut cache = BuildCache::load(&cache_path).ok().unwrap();
        cache.update(&content).ok();
        cache.save().ok();

        let cache2 = BuildCache::load(&cache_path).ok().unwrap();
        let changed = cache2.changed_files(&content).ok().unwrap();
        assert!(changed.is_empty());
    }

    // 5. New files appear as changed.
    #[test]
    fn new_files_are_changed() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "hello");

        let mut cache = BuildCache::load(&cache_path).ok().unwrap();
        cache.update(&content).ok();
        cache.save().ok();

        // Add a new file.
        write_file(&content, "b.md", "world");

        let cache2 = BuildCache::load(&cache_path).ok().unwrap();
        let changed = cache2.changed_files(&content).ok().unwrap();
        assert_eq!(changed.len(), 1);
        assert!(changed[0].ends_with("b.md"));
    }

    // 6. Deleted files are pruned from the map on update.
    #[test]
    fn deleted_files_pruned() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "keep");
        write_file(&content, "b.md", "delete-me");

        let mut cache = BuildCache::load(&cache_path).ok().unwrap();
        cache.update(&content).ok();
        assert_eq!(cache.len(), 2);

        // Delete one file.
        fs::remove_file(content.join("b.md")).ok();

        cache.update(&content).ok();
        assert_eq!(cache.len(), 1);
    }

    // 7. Save / load round-trip preserves all entries.
    #[test]
    fn save_load_roundtrip() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "x.md", "data1");
        write_file(&content, "sub/y.md", "data2");

        let mut cache = BuildCache::new(&cache_path);
        cache.update(&content).ok();
        cache.save().ok();

        let loaded = BuildCache::load(&cache_path).ok().unwrap();
        assert_eq!(loaded.len(), 2);
    }

    // 8. Empty content directory yields no changed files.
    #[test]
    fn empty_directory() {
        let (_tmp, content, cache_path) = setup();
        let cache = BuildCache::load(&cache_path).ok().unwrap();
        let changed = cache.changed_files(&content).ok().unwrap();
        assert!(changed.is_empty());
    }

    // 9. Non-existent content directory yields no changed files.
    #[test]
    fn nonexistent_directory() {
        let tmp = TempDir::new().ok().unwrap();
        let cache_path = tmp.path().join(".ssg-cache.json");
        let cache = BuildCache::load(&cache_path).ok().unwrap();
        let changed =
            cache.changed_files(&tmp.path().join("nope")).ok().unwrap();
        assert!(changed.is_empty());
    }

    // 10. Fingerprint is deterministic for the same content.
    #[test]
    fn fingerprint_deterministic() {
        let tmp = TempDir::new().ok().unwrap();
        let path = tmp.path().join("test.txt");
        fs::write(&path, "deterministic").ok();

        let h1 = BuildCache::fingerprint(&path).ok().unwrap();
        let h2 = BuildCache::fingerprint(&path).ok().unwrap();
        assert_eq!(h1, h2);
    }

    // 11. Different content produces different fingerprints.
    #[test]
    fn fingerprint_varies_with_content() {
        let tmp = TempDir::new().ok().unwrap();
        let p1 = tmp.path().join("a.txt");
        let p2 = tmp.path().join("b.txt");
        fs::write(&p1, "alpha").ok();
        fs::write(&p2, "beta").ok();

        let h1 = BuildCache::fingerprint(&p1).ok().unwrap();
        let h2 = BuildCache::fingerprint(&p2).ok().unwrap();
        assert_ne!(h1, h2);
    }

    // 12. Subdirectory files are tracked correctly.
    #[test]
    fn subdirectory_tracking() {
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "posts/2024/hello.md", "hi");
        write_file(&content, "pages/about.md", "about");

        let mut cache = BuildCache::new(&cache_path);
        cache.update(&content).ok();
        assert_eq!(cache.len(), 2);

        // Modify nested file.
        write_file(&content, "posts/2024/hello.md", "updated");
        let changed = cache.changed_files(&content).ok().unwrap();
        assert_eq!(changed.len(), 1);
    }

    // 13. Corrupted JSON in cache file returns an error.
    #[test]
    fn build_cache_load_corrupted_json() {
        // Arrange
        let tmp = TempDir::new().ok().unwrap();
        let cache_path = tmp.path().join(".ssg-cache.json");
        fs::write(&cache_path, "{ not valid json !!!").ok();

        // Act
        let result = BuildCache::load(&cache_path);

        // Assert — malformed JSON must produce an error
        assert!(result.is_err(), "corrupted JSON should fail to load");
    }

    // 14. Empty directory produces no changes.
    #[test]
    fn build_cache_empty_directory() {
        // Arrange
        let (_tmp, content, cache_path) = setup();
        let mut cache = BuildCache::new(&cache_path);
        cache.update(&content).ok();

        // Act
        let changed = cache.changed_files(&content).ok().unwrap();

        // Assert
        assert!(changed.is_empty(), "empty directory should have no changes");
        assert_eq!(cache.len(), 0);
    }

    // 15. File present in cache but deleted from disk is detected on update.
    #[test]
    fn build_cache_file_removed_detected() {
        // Arrange
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "keep");
        write_file(&content, "b.md", "remove-me");

        let mut cache = BuildCache::new(&cache_path);
        cache.update(&content).ok();
        assert_eq!(cache.len(), 2);

        // Act — delete one file, then update the cache
        fs::remove_file(content.join("b.md")).ok();
        cache.update(&content).ok();

        // Assert — removed file is no longer in the fingerprint map
        assert_eq!(cache.len(), 1, "deleted file should be pruned from cache");
    }

    // 17. default_path() returns the compile-time constant.
    #[test]
    fn default_path_returns_compile_time_constant() {
        // Covers the const fn at lines 250-252. The function is a
        // trivial static-string accessor but it's part of the
        // public API so we exercise it explicitly.
        assert_eq!(BuildCache::default_path(), DEFAULT_CACHE_FILE);
        assert!(!BuildCache::default_path().is_empty());
    }

    // 18. walk() propagates read_dir errors via with_context.
    #[test]
    fn walk_errors_on_nonexistent_directory() {
        // Covers the with_context format! closure at lines 156-158.
        // We call the walker directly with a path that doesn't
        // exist — fs::read_dir returns Err, the closure fires, and
        // the format! inside it evaluates (closing lines 157-158).
        let tmp = TempDir::new().ok().unwrap();
        let missing = tmp.path().join("does-not-exist");
        let mut out = Vec::new();
        let result = BuildCache::walk(tmp.path(), &missing, &mut out);
        assert!(result.is_err(), "walk should Err on missing dir");
        let msg = format!("{:?}", result.unwrap_err());
        assert!(
            msg.contains("cannot read directory"),
            "error should contain with_context message: {msg}"
        );
    }

    // ── load/save error-path closures ───────────────────────────────

    // 19. load() — read_to_string failure triggers the with_context closure.
    // We make `cache_path` a directory so File::open succeeds existence
    // check but read_to_string fails ("Is a directory").
    #[test]
    fn load_read_failure_fires_with_context_closure() {
        let tmp = TempDir::new().ok().unwrap();
        let cache_path = tmp.path().join("cache-as-dir");
        fs::create_dir_all(&cache_path).ok();
        let err = BuildCache::load(&cache_path).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("failed to read cache file"),
            "error chain should contain load read context: {msg}"
        );
    }

    // 20. load() — parse failure triggers the parse with_context closure.
    // Distinct from `build_cache_load_corrupted_json` — that test only
    // asserts is_err; here we assert the closure-generated message
    // (lines 96-98) made it into the error chain.
    #[test]
    fn load_parse_failure_message_contains_path() {
        let tmp = TempDir::new().ok().unwrap();
        let cache_path = tmp.path().join("bad.json");
        fs::write(&cache_path, b"{ this is not json").ok();
        let err = BuildCache::load(&cache_path).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("failed to parse cache file"),
            "error chain should contain parse context: {msg}"
        );
        assert!(
            msg.contains("bad.json"),
            "error chain should contain file path: {msg}"
        );
    }

    // 21. save() — write failure triggers the write with_context closure.
    // We point cache_path at a directory we never create, then create
    // an intermediate file (not dir) at the parent so fs::write fails
    // (parent is a file, not a directory).
    #[test]
    fn save_write_failure_fires_with_context_closure() {
        let tmp = TempDir::new().ok().unwrap();
        let parent_as_file = tmp.path().join("not-a-dir");
        fs::write(&parent_as_file, b"i am a file").ok();
        let cache_path = parent_as_file.join("cache.json");
        let cache = BuildCache::new(&cache_path);
        let err = cache.save().unwrap_err();
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("failed to write cache file"),
            "error chain should contain save write context: {msg}"
        );
    }

    // 16. Unchanged files do not appear in the changed list.
    #[test]
    fn build_cache_unchanged_files_not_reported() {
        // Arrange
        let (_tmp, content, cache_path) = setup();
        write_file(&content, "a.md", "stable");
        write_file(&content, "b.md", "also stable");

        let mut cache = BuildCache::new(&cache_path);
        cache.update(&content).ok();
        cache.save().ok();

        // Act — reload without modifying any files
        let cache2 = BuildCache::load(&cache_path).ok().unwrap();
        let changed = cache2.changed_files(&content).ok().unwrap();

        // Assert — nothing should be reported as changed
        assert!(
            changed.is_empty(),
            "unchanged files must not be in changed list"
        );
    }
}
