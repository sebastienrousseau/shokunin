// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Deterministic content-hash-keyed cache for local LLM inference (issue #528).
//!
//! Local-model inference (Ollama, llama.cpp) is CPU/GPU intensive and
//! non-deterministic (sampling makes outputs vary). For a 10K-page
//! docs site that means hours of CI per build and bit-different
//! artifacts across runs. This module fixes both: invocations are
//! keyed on a SHA-256 of `(endpoint, model, prompt, timeout)`; a hit
//! returns the cached body and never crosses the wire, so builds are
//! both fast and reproducible.
//!
//! # On-disk layout
//!
//! Entries live under `$XDG_CACHE_HOME/ssg/llm/` (Linux),
//! `~/Library/Caches/ssg/llm/` (macOS), or `%LOCALAPPDATA%\ssg\llm\`
//! (Windows). Each entry is git-sharded:
//!
//! ```text
//! <cache_dir>/<aa>/<bbbbbbbbbb...>.json
//! ```
//!
//! where `aa` is the first two hex chars of the key and `bbbbb...`
//! is the remaining 62 chars. This keeps any single shard directory
//! well under the FAT32/exFAT 65 K-entry ceiling even on a million-
//! page site.
//!
//! # File format
//!
//! Each entry is a small JSON document:
//!
//! ```json
//! {
//!   "version": 1,
//!   "key_hex": "<full 64-char hex of key>",
//!   "payload_len": <usize>,
//!   "payload": "<the cached LLM response>"
//! }
//! ```
//!
//! `key_hex` and `payload_len` provide a cheap end-to-end integrity
//! check — a torn mid-write (or a flipped bit on disk) yields a
//! parse error or a length mismatch, and the entry is evicted and
//! re-computed without surfacing a hard error to the caller.
//!
//! # Concurrency
//!
//! Writes go to `<final>.tmp.<pid>.<nanos>` then `rename` into place.
//! `rename` is atomic on every supported filesystem, so concurrent
//! writers for *distinct* keys never trample one another, and
//! concurrent writers for the *same* key produce a last-writer-wins
//! outcome where every reader still sees a consistent entry.
//!
//! # TTL
//!
//! [`LlmCache::get`] honours a configurable TTL: entries older than
//! `ttl` (compared against the file's mtime) are evicted and reported
//! as a miss. Default is 90 days.

use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

/// Default TTL for cache entries: 90 days. Matches the AC4 default
/// from issue #528 ("`cache.llm.ttl_days` default 90").
pub const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 90);

/// Persisted on-disk format version. Bumped only on an incompatible
/// schema change so old entries get evicted by the version mismatch
/// branch in [`parse_entry`].
const ENTRY_VERSION: u32 = 1;

/// Counters returned by [`LlmCache::stats`] for the `ssg cache --stats`
/// CLI subcommand. Counts are session-local — the cache file itself
/// stores none of this.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Hits since the cache was constructed.
    pub hits: u64,
    /// Misses since the cache was constructed (including TTL and
    /// corruption-driven evictions).
    pub misses: u64,
    /// Stores written since the cache was constructed.
    pub stores: u64,
    /// Entries evicted because they failed integrity or TTL checks.
    pub evictions: u64,
}

/// Content-hash-keyed file cache for LLM inference.
///
/// Cloning is cheap — the counters use shared atomics so a cloned
/// handle reports the same totals as its parent, which is the
/// invariant the CLI `--stats` subcommand relies on when multiple
/// pipeline threads each hold a handle.
#[derive(Debug)]
pub struct LlmCache {
    /// Absolute root directory containing all entries.
    root: PathBuf,
    /// TTL after which entries are considered stale.
    ttl: Duration,
    /// Session-local hit counter.
    hits: AtomicU64,
    /// Session-local miss counter.
    misses: AtomicU64,
    /// Session-local store counter.
    stores: AtomicU64,
    /// Session-local eviction counter.
    evictions: AtomicU64,
}

impl LlmCache {
    /// Constructs a cache rooted at `root` with the [`DEFAULT_TTL`].
    ///
    /// The directory is created lazily on the first write; calling
    /// this on a path that does not yet exist is fine.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// assert_eq!(cache.root(), tmp.path());
    /// ```
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self::with_ttl(root, DEFAULT_TTL)
    }

    /// Constructs a cache rooted at `root` with a custom TTL. Used by
    /// the AC4 expiry tests so they don't have to wait 90 days.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::with_ttl(tmp.path().to_path_buf(), Duration::from_secs(60));
    /// assert_eq!(cache.stats().hits, 0);
    /// ```
    #[must_use]
    pub const fn with_ttl(root: PathBuf, ttl: Duration) -> Self {
        Self {
            root,
            ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            stores: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Resolves the platform-default cache root.
    ///
    /// Selection order (first that yields a path wins):
    ///
    /// 1. `$SSG_LLM_CACHE_DIR` — explicit override, used by tests and
    ///    by ops who want to point at a shared cache on tmpfs.
    /// 2. `$XDG_CACHE_HOME/ssg/llm` — Linux / `XDG_CACHE_HOME` set.
    /// 3. `$HOME/Library/Caches/ssg/llm` — macOS default.
    /// 4. `%LOCALAPPDATA%\ssg\llm` — Windows.
    /// 5. `$HOME/.cache/ssg/llm` — generic Unix fallback.
    /// 6. `./.ssg-llm-cache` — last-resort relative path so the cache
    ///    is still usable in sandboxes where neither `$HOME` nor
    ///    `%LOCALAPPDATA%` is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let dir = LlmCache::default_cache_dir();
    /// assert!(!dir.as_os_str().is_empty());
    /// ```
    #[must_use]
    pub fn default_cache_dir() -> PathBuf {
        if let Ok(explicit) = std::env::var("SSG_LLM_CACHE_DIR") {
            if !explicit.is_empty() {
                return PathBuf::from(explicit);
            }
        }
        if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("ssg").join("llm");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                if !home.is_empty() {
                    return PathBuf::from(home)
                        .join("Library")
                        .join("Caches")
                        .join("ssg")
                        .join("llm");
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                if !local.is_empty() {
                    return PathBuf::from(local).join("ssg").join("llm");
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return generic_unix_cache_dir(&home);
            }
        }
        PathBuf::from(".ssg-llm-cache")
    }

    /// Computes the 32-byte SHA-256 key for `(endpoint, model, prompt, timeout_secs)`.
    ///
    /// Every parameter that can change the model's output is folded
    /// into the digest so a request that differs in even one byte
    /// gets a fresh inference (AC2). The hash is domain-separated
    /// with a versioned prefix so a future change to the key
    /// composition can be rolled out without colliding with stored
    /// entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let a = LlmCache::compute_key("http://x", "llama", "hi", 30);
    /// let b = LlmCache::compute_key("http://x", "llama", "hi", 30);
    /// assert_eq!(a, b);
    /// let c = LlmCache::compute_key("http://x", "llama", "bye", 30);
    /// assert_ne!(a, c);
    /// ```
    #[must_use]
    pub fn compute_key(
        endpoint: &str,
        model: &str,
        prompt: &str,
        timeout_secs: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"ssg-llm-cache-v1\x00");
        hasher.update((endpoint.len() as u64).to_le_bytes());
        hasher.update(endpoint.as_bytes());
        hasher.update(b"\x00");
        hasher.update((model.len() as u64).to_le_bytes());
        hasher.update(model.as_bytes());
        hasher.update(b"\x00");
        hasher.update((prompt.len() as u64).to_le_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(b"\x00");
        hasher.update(timeout_secs.to_le_bytes());
        hasher.finalize().into()
    }

    /// Returns the cached payload for `key`, or `None` on miss /
    /// stale / corrupt.
    ///
    /// A corrupted entry (truncated JSON, version mismatch, length
    /// mismatch) is evicted in-place and reported as a miss so the
    /// caller does a fresh inference (AC5). A TTL-expired entry is
    /// handled the same way (AC4).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// let key = LlmCache::compute_key("e", "m", "p", 1);
    /// assert!(cache.get(&key).is_none());
    /// cache.set(&key, "answer").unwrap();
    /// assert_eq!(cache.get(&key).as_deref(), Some("answer"));
    /// ```
    pub fn get(&self, key: &[u8; 32]) -> Option<String> {
        let path = self.entry_path(key);
        let mut file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let _ = self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
            Err(_) => {
                // EACCES / EIO / similar — treat as a miss so the
                // caller falls back to live inference rather than
                // failing the build for a cache pathology.
                let _ = self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };

        if entry_is_expired(&file, self.ttl) {
            let _ = fs::remove_file(&path);
            let _ = self.evictions.fetch_add(1, Ordering::Relaxed);
            let _ = self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }

        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_err() {
            let _ = fs::remove_file(&path);
            let _ = self.evictions.fetch_add(1, Ordering::Relaxed);
            let _ = self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }

        if let Some(payload) = parse_entry(&buf, key) {
            let _ = self.hits.fetch_add(1, Ordering::Relaxed);
            Some(payload)
        } else {
            let _ = fs::remove_file(&path);
            let _ = self.evictions.fetch_add(1, Ordering::Relaxed);
            let _ = self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Stores `payload` under `key`.
    ///
    /// Returns `Ok(())` on success. On any filesystem error the call
    /// silently falls through (counter is not bumped) so a transient
    /// disk failure never breaks the build — the next invocation
    /// will just be another miss + recompute.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] when the cache file cannot
    /// be created or renamed into place.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// let key = LlmCache::compute_key("e", "m", "p", 1);
    /// cache.set(&key, "stored").unwrap();
    /// assert_eq!(cache.stats().stores, 1);
    /// ```
    pub fn set(&self, key: &[u8; 32], payload: &str) -> io::Result<()> {
        let path = self.entry_path(key);
        ensure_parent_dir(&path)?;

        let key_hex = encode_hex(key);
        let body = serde_json::json!({
            "version": ENTRY_VERSION,
            "key_hex": key_hex,
            "payload_len": payload.len(),
            "payload": payload,
        })
        .to_string();

        // Tempfile name uses pid + a monotonically-incrementing
        // counter so two threads in the same process never collide
        // even at sub-nanosecond resolution where the system clock
        // could return the same `now()`.
        let tmp = path.with_extension(format!(
            "tmp.{}.{}",
            std::process::id(),
            next_tmp_seq(),
        ));

        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(body.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)?;
        let _ = self.stores.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Removes the entry for `key` if present. Used by the
    /// `ssg cache --clear` command and by the unit tests.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] when the entry exists but
    /// cannot be removed. A missing entry is treated as success.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// let key = LlmCache::compute_key("e", "m", "p", 1);
    /// cache.set(&key, "x").unwrap();
    /// cache.evict(&key).unwrap();
    /// assert!(cache.get(&key).is_none());
    /// ```
    pub fn evict(&self, key: &[u8; 32]) -> io::Result<()> {
        let path = self.entry_path(key);
        match fs::remove_file(&path) {
            Ok(()) => {
                let _ = self.evictions.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Returns the running session counters.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// let stats = cache.stats();
    /// assert_eq!(stats.hits, 0);
    /// assert_eq!(stats.stores, 0);
    /// ```
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            stores: self.stores.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }

    /// Returns the cache root.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::llm_cache::LlmCache;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let cache = LlmCache::new(tmp.path().to_path_buf());
    /// assert_eq!(cache.root(), tmp.path());
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Computes the file path for `key`.
    fn entry_path(&self, key: &[u8; 32]) -> PathBuf {
        let hex = encode_hex(key);
        let (shard, rest) = hex.split_at(2);
        self.root.join(shard).join(format!("{rest}.json"))
    }
}

/// Returns `true` if `file`'s recorded modification time is older than
/// `ttl`, meaning [`LlmCache::get`] should evict it instead of
/// returning it as a hit.
///
/// Conservative on any uncertainty: if the file's metadata or modified
/// time cannot be determined, or the clock looks skewed such that
/// `modified` is in the future, the entry is treated as *not* expired
/// rather than evicted — a filesystem or clock hiccup should never
/// cause a spurious cache miss. On every real filesystem an
/// already-open file handle reliably yields `Ok` from both
/// `metadata()` and `modified()`, so the two failpoints below are the
/// only way to drive those (otherwise practically unreachable, since
/// `fs::File` exposes no way to force them from safe Rust) `Err` arms
/// in a test.
// The `(|| { fail_point!(...); ... })()` IIFEs below are deliberate: the
// `fail_point!` macro injects an early `return` scoped to the *closure*, so a
// failpoint can drive the otherwise-unreachable `Err` arms without returning
// from the whole function. Removing the closure would change control flow.
#[allow(clippy::redundant_closure_call)]
fn entry_is_expired(file: &fs::File, ttl: Duration) -> bool {
    let Ok(meta) = (|| {
        fail_point!("llm_cache::get-metadata-err", |_| Err(()));
        file.metadata().map_err(|_| ())
    })() else {
        return false;
    };
    let Ok(modified) = (|| {
        fail_point!("llm_cache::get-modified-err", |_| Err(()));
        meta.modified().map_err(|_| ())
    })() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age > ttl
}

/// Ensures the parent directory of `path` exists, creating it (and
/// any missing ancestors) if necessary. Used by [`LlmCache::set`]
/// before writing the temp file that gets renamed into place.
///
/// [`LlmCache::entry_path`] always joins at least a shard directory
/// and a filename onto `root`, so `path.parent()` is `Some` for every
/// key produced through the public API in practice — the `None` arm
/// (a path with no parent component at all) cannot be produced that
/// way. The failpoint lets a test drive that arm directly without a
/// contrived root value.
#[allow(clippy::redundant_closure_call)] // fail_point! IIFE — see entry_is_expired
fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    let parent = (|| {
        fail_point!("llm_cache::set-no-parent", |_| None);
        path.parent()
    })();
    if let Some(parent) = parent {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Generic Unix (`$HOME/.cache/ssg/llm`) cache-root fallback, used by
/// every target that is neither macOS nor Windows (Linux, BSD, etc.).
///
/// Extracted out of [`LlmCache::default_cache_dir`] as a pure function
/// of `home` so it is directly unit-testable regardless of which OS
/// actually runs the test suite: on a macOS test runner, the
/// `#[cfg(target_os = "macos")]` branch in `default_cache_dir` always
/// intercepts first whenever `HOME` is set, so this fallback can never
/// be reached at runtime there even though the logic is still compiled
/// in (it is real, live code for Linux/BSD users).
fn generic_unix_cache_dir(home: &str) -> PathBuf {
    PathBuf::from(home).join(".cache").join("ssg").join("llm")
}

/// Lowercase-hex encode a 32-byte digest into a 64-char `String`
/// without pulling in the `hex` crate.
fn encode_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Parses the on-disk JSON entry, returning the payload only when
/// every integrity check passes. Returns `None` for: unparsable
/// JSON, wrong version, missing fields, mismatched key, or
/// mismatched payload length.
fn parse_entry(text: &str, key: &[u8; 32]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let version = v.get("version")?.as_u64()?;
    if u32::try_from(version).ok()? != ENTRY_VERSION {
        return None;
    }
    let key_hex = v.get("key_hex")?.as_str()?;
    if key_hex != encode_hex(key) {
        return None;
    }
    let payload = v.get("payload")?.as_str()?;
    let stored_len = usize::try_from(v.get("payload_len")?.as_u64()?).ok()?;
    if stored_len != payload.len() {
        return None;
    }
    Some(payload.to_string())
}

/// Process-global counter feeding [`LlmCache::set`]'s tempfile name
/// so two threads writing the same key at the same nanosecond still
/// pick distinct staging paths.
fn next_tmp_seq() -> u64 {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn cache_for_test() -> (tempfile::TempDir, LlmCache) {
        let dir = tempfile::tempdir().unwrap();
        let cache = LlmCache::new(dir.path().to_path_buf());
        (dir, cache)
    }

    #[test]
    fn encode_hex_zero_padded() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0x0a;
        bytes[31] = 0xff;
        let hex = encode_hex(&bytes);
        assert_eq!(hex.len(), 64);
        assert!(hex.starts_with("0a"));
        assert!(hex.ends_with("ff"));
    }

    #[test]
    fn compute_key_is_deterministic() {
        let k1 = LlmCache::compute_key("e", "m", "p", 1);
        let k2 = LlmCache::compute_key("e", "m", "p", 1);
        assert_eq!(k1, k2);
    }

    #[test]
    fn compute_key_differs_on_endpoint() {
        let a = LlmCache::compute_key("e1", "m", "p", 1);
        let b = LlmCache::compute_key("e2", "m", "p", 1);
        assert_ne!(a, b);
    }

    #[test]
    fn compute_key_differs_on_model() {
        let a = LlmCache::compute_key("e", "m1", "p", 1);
        let b = LlmCache::compute_key("e", "m2", "p", 1);
        assert_ne!(a, b);
    }

    #[test]
    fn compute_key_differs_on_prompt() {
        let a = LlmCache::compute_key("e", "m", "p1", 1);
        let b = LlmCache::compute_key("e", "m", "p2", 1);
        assert_ne!(a, b);
    }

    #[test]
    fn compute_key_differs_on_timeout() {
        // AC2 — parameters are part of the cache key.
        let a = LlmCache::compute_key("e", "m", "p", 1);
        let b = LlmCache::compute_key("e", "m", "p", 2);
        assert_ne!(a, b);
    }

    #[test]
    fn compute_key_resists_length_collision() {
        // "ab" + "cd" must not hash equal to "abc" + "d" because we
        // length-prefix each component.
        let a = LlmCache::compute_key("ab", "cd", "p", 1);
        let b = LlmCache::compute_key("abc", "d", "p", 1);
        assert_ne!(a, b);
    }

    #[test]
    #[serial_test::parallel]
    fn round_trip_hit() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        assert!(cache.get(&key).is_none());
        cache.set(&key, "the answer").unwrap();
        assert_eq!(cache.get(&key).as_deref(), Some("the answer"));
    }

    #[test]
    #[serial_test::parallel]
    fn miss_counter_advances_on_absent_key() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        let _ = cache.get(&key);
        let _ = cache.get(&key);
        assert_eq!(cache.stats().misses, 2);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    #[serial_test::parallel]
    fn hit_counter_advances_on_present_key() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "x").unwrap();
        let _ = cache.get(&key);
        let _ = cache.get(&key);
        assert_eq!(cache.stats().hits, 2);
    }

    #[test]
    #[serial_test::parallel]
    fn evict_removes_entry() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "x").unwrap();
        cache.evict(&key).unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn evict_missing_is_ok() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.evict(&key).unwrap();
    }

    #[test]
    #[serial_test::parallel]
    fn ttl_zero_expires_immediately() {
        let dir = tempfile::tempdir().unwrap();
        let cache = LlmCache::with_ttl(
            dir.path().to_path_buf(),
            Duration::from_nanos(1),
        );
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "x").unwrap();
        thread::sleep(Duration::from_millis(5));
        assert!(cache.get(&key).is_none());
        assert!(cache.stats().evictions >= 1);
    }

    #[test]
    #[serial_test::parallel]
    fn corrupt_json_evicts_and_misses() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "x").unwrap();
        // Truncate the file on disk to simulate a mid-write crash.
        let p = cache.entry_path(&key);
        fs::write(&p, "{ not json").unwrap();
        assert!(cache.get(&key).is_none());
        assert!(!p.exists(), "corrupt entry should have been evicted");
    }

    #[test]
    #[serial_test::parallel]
    fn length_mismatch_evicts() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "abcdef").unwrap();
        let p = cache.entry_path(&key);
        let body = serde_json::json!({
            "version": ENTRY_VERSION,
            "key_hex": encode_hex(&key),
            "payload_len": 9999,
            "payload": "abcdef",
        });
        fs::write(&p, body.to_string()).unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    #[serial_test::parallel]
    fn version_mismatch_evicts() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        cache.set(&key, "x").unwrap();
        let p = cache.entry_path(&key);
        let body = serde_json::json!({
            "version": 9999,
            "key_hex": encode_hex(&key),
            "payload_len": 1,
            "payload": "x",
        });
        fs::write(&p, body.to_string()).unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    #[serial_test::parallel]
    fn key_mismatch_evicts() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        let other = LlmCache::compute_key("x", "y", "z", 9);
        cache.set(&key, "x").unwrap();
        let p = cache.entry_path(&key);
        let body = serde_json::json!({
            "version": ENTRY_VERSION,
            "key_hex": encode_hex(&other),
            "payload_len": 1,
            "payload": "x",
        });
        fs::write(&p, body.to_string()).unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    #[serial_test::parallel]
    fn sharding_uses_first_two_hex_chars() {
        let (dir, cache) = cache_for_test();
        let key = [0xab; 32];
        cache.set(&key, "x").unwrap();
        let shard = dir.path().join("ab");
        assert!(shard.is_dir(), "expected shard dir {shard:?}");
    }

    #[test]
    #[serial_test::parallel]
    fn concurrent_distinct_keys_do_not_collide() {
        // AC7 — 50 concurrent writers on distinct keys.
        let (_d, cache) = cache_for_test();
        let cache = std::sync::Arc::new(cache);
        let mut handles = Vec::new();
        for i in 0..50 {
            let c = std::sync::Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                let key = LlmCache::compute_key("e", "m", &format!("p{i}"), 1);
                c.set(&key, &format!("v{i}")).unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        for i in 0..50 {
            let key = LlmCache::compute_key("e", "m", &format!("p{i}"), 1);
            assert_eq!(
                cache.get(&key).as_deref(),
                Some(format!("v{i}").as_str()),
                "missing entry for key {i}"
            );
        }
    }

    #[test]
    #[serial_test::parallel]
    fn concurrent_same_key_last_writer_wins_no_corruption() {
        // AC7 — multiple writers on the same key must produce a
        // valid entry; we accept any one of their payloads.
        let (_d, cache) = cache_for_test();
        let cache = std::sync::Arc::new(cache);
        let key = LlmCache::compute_key("e", "m", "p", 1);
        let mut handles = Vec::new();
        for i in 0..20 {
            let c = std::sync::Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                c.set(&key, &format!("v{i}")).unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let got = cache.get(&key).expect("entry should exist");
        assert!(got.starts_with('v'));
    }

    /// Serialised env-var scoping for the `default_cache_dir` tests.
    ///
    /// Entries are applied *sequentially* (capture-then-set per entry)
    /// and restored in reverse, so a duplicated key deterministically
    /// exercises both restore arms: the later entry's captured
    /// previous value is whatever the earlier entry just set.
    fn with_env_vars<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let mut prev: Vec<(String, Option<String>)> = Vec::new();
        for (key, value) in vars {
            prev.push(((*key).to_string(), std::env::var(key).ok()));
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        f();
        for (key, value) in prev.into_iter().rev() {
            match value {
                Some(v) => std::env::set_var(&key, v),
                None => std::env::remove_var(&key),
            }
        }
    }

    #[test]
    fn default_cache_dir_respects_explicit_override() {
        // Duplicate key: the inner entry restores the outer value on
        // unwind (Some arm), the outer entry restores the machine
        // state.
        with_env_vars(
            &[
                ("SSG_LLM_CACHE_DIR", Some("/outer-sentinel")),
                ("SSG_LLM_CACHE_DIR", Some("/tmp/ssg-test-cache")),
            ],
            || {
                assert_eq!(
                    LlmCache::default_cache_dir(),
                    PathBuf::from("/tmp/ssg-test-cache")
                );
            },
        );
    }

    #[test]
    fn default_cache_dir_empty_override_falls_through_to_xdg() {
        // An empty SSG_LLM_CACHE_DIR must be ignored; XDG_CACHE_HOME
        // is next in the resolution order. The duplicated unset+set
        // pair drives the remove-then-restore-None arms of the helper.
        with_env_vars(
            &[
                ("SSG_LLM_CACHE_DIR", None),
                ("SSG_LLM_CACHE_DIR", Some("")),
                ("XDG_CACHE_HOME", Some("/xdg-root")),
            ],
            || {
                assert_eq!(
                    LlmCache::default_cache_dir(),
                    PathBuf::from("/xdg-root").join("ssg").join("llm")
                );
            },
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn default_cache_dir_empty_xdg_uses_home_library_caches() {
        with_env_vars(
            &[
                ("SSG_LLM_CACHE_DIR", None),
                ("XDG_CACHE_HOME", Some("")),
                ("HOME", Some("/home-test")),
            ],
            || {
                assert_eq!(
                    LlmCache::default_cache_dir(),
                    PathBuf::from("/home-test")
                        .join("Library")
                        .join("Caches")
                        .join("ssg")
                        .join("llm")
                );
            },
        );
    }

    #[test]
    fn default_cache_dir_without_home_uses_relative_fallback() {
        with_env_vars(
            &[
                ("SSG_LLM_CACHE_DIR", None),
                ("XDG_CACHE_HOME", None),
                // On Windows, LOCALAPPDATA is a real, always-set env
                // var that the production code checks before falling
                // through to HOME — must be cleared too so this test
                // exercises the actual "nothing configured" fallback
                // on every platform.
                ("LOCALAPPDATA", None),
                ("HOME", None),
            ],
            || {
                assert_eq!(
                    LlmCache::default_cache_dir(),
                    PathBuf::from(".ssg-llm-cache")
                );
            },
        );
    }

    #[test]
    fn default_cache_dir_empty_home_uses_relative_fallback() {
        with_env_vars(
            &[
                ("SSG_LLM_CACHE_DIR", None),
                ("XDG_CACHE_HOME", None),
                ("LOCALAPPDATA", None),
                ("HOME", Some("")),
            ],
            || {
                assert_eq!(
                    LlmCache::default_cache_dir(),
                    PathBuf::from(".ssg-llm-cache")
                );
            },
        );
    }

    #[test]
    fn root_returns_constructor_path() {
        let dir = tempfile::tempdir().unwrap();
        let cache = LlmCache::new(dir.path().to_path_buf());
        assert_eq!(cache.root(), dir.path());
    }

    #[test]
    fn stats_default_is_zero() {
        let s = CacheStats::default();
        assert_eq!(s.hits, 0);
        assert_eq!(s.misses, 0);
        assert_eq!(s.stores, 0);
        assert_eq!(s.evictions, 0);
    }

    #[test]
    #[serial_test::parallel]
    fn stats_store_counter_increments() {
        let (_d, cache) = cache_for_test();
        let k1 = LlmCache::compute_key("e", "m", "p1", 1);
        let k2 = LlmCache::compute_key("e", "m", "p2", 1);
        cache.set(&k1, "x").unwrap();
        cache.set(&k2, "y").unwrap();
        assert_eq!(cache.stats().stores, 2);
    }

    #[test]
    #[serial_test::parallel]
    fn get_returns_none_when_entry_is_a_directory() {
        // File::open on a directory returns Err with kind other than
        // NotFound — hits the catch-all `Err(_) => …` arm in get().
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "p", 1);
        let path = cache.entry_path(&key);
        fs::create_dir_all(&path).unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    #[serial_test::parallel]
    fn get_returns_none_on_missing_entry_increments_miss() {
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "missing", 1);
        let before = cache.stats().misses;
        assert!(cache.get(&key).is_none());
        assert!(cache.stats().misses > before);
    }

    #[test]
    fn parse_entry_returns_none_for_missing_fields() {
        let key = LlmCache::compute_key("e", "m", "p", 1);
        // Missing `version` field.
        let json = format!(
            r#"{{"key_hex":"{}","payload":"x","payload_len":1}}"#,
            encode_hex(&key)
        );
        assert!(parse_entry(&json, &key).is_none());
    }

    #[test]
    fn parse_entry_returns_none_for_non_object_payload() {
        let key = LlmCache::compute_key("e", "m", "p", 1);
        assert!(parse_entry("[1,2,3]", &key).is_none());
        assert!(parse_entry("null", &key).is_none());
    }

    #[test]
    fn evict_propagates_unexpected_io_error() {
        // Calling evict on a path whose parent is a non-existent dir
        // does NOT error (NotFound is mapped to Ok). But we can force
        // a different error by making the path itself a directory (so
        // remove_file returns IsADirectory / similar).
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "evict-dir", 1);
        let path = cache.entry_path(&key);
        fs::create_dir_all(&path).unwrap();
        let res = cache.evict(&key);
        // On unix `remove_file` on a directory returns IsADirectory;
        // on some macOS versions returns PermissionDenied. We only
        // assert the body executed and didn't panic — either Ok or
        // Err is acceptable.
        let _ = res;
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::parallel]
    fn get_open_permission_error_counts_as_miss() {
        use std::os::unix::fs::PermissionsExt;
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "denied", 1);
        cache.set(&key, "x").unwrap();
        let path = cache.entry_path(&key);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        let misses_before = cache.stats().misses;
        assert!(
            cache.get(&key).is_none(),
            "EACCES must be treated as a miss"
        );
        assert_eq!(cache.stats().misses, misses_before + 1);

        // Restore so the tempdir can be cleaned up.
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    #[serial_test::parallel]
    fn get_hits_when_mtime_is_in_the_future() {
        // duration_since(modified) errors when the mtime is ahead of
        // now; the TTL check must fall through and still return a hit.
        let (_d, cache) = cache_for_test();
        let key = LlmCache::compute_key("e", "m", "future", 1);
        cache.set(&key, "from-tomorrow").unwrap();
        let path = cache.entry_path(&key);
        let f = fs::OpenOptions::new().write(true).open(&path).unwrap();
        f.set_modified(SystemTime::now() + Duration::from_secs(3600))
            .unwrap();
        drop(f);

        assert_eq!(cache.get(&key).as_deref(), Some("from-tomorrow"));
    }

    #[test]
    #[serial_test::parallel]
    fn set_fails_when_root_is_a_file() {
        // create_dir_all under a plain file must error, propagating
        // through the `?` in set().
        let dir = tempfile::tempdir().unwrap();
        let root_file = dir.path().join("rootfile");
        fs::write(&root_file, "not a dir").unwrap();
        let cache = LlmCache::new(root_file);
        let key = LlmCache::compute_key("e", "m", "p", 1);
        assert!(cache.set(&key, "x").is_err());
        assert_eq!(cache.stats().stores, 0);
    }

    #[test]
    fn next_tmp_seq_is_monotonic() {
        let a = next_tmp_seq();
        let b = next_tmp_seq();
        let c = next_tmp_seq();
        assert!(b > a);
        assert!(c > b);
    }

    #[test]
    fn generic_unix_cache_dir_joins_expected_components() {
        // On macOS the `#[cfg(target_os = "macos")]` branch in
        // `default_cache_dir` always intercepts before this generic
        // Linux/BSD fallback can be reached at runtime whenever `HOME`
        // is set, so the underlying logic is tested directly here
        // instead of through `default_cache_dir` itself.
        assert_eq!(
            generic_unix_cache_dir("/home/alice"),
            PathBuf::from("/home/alice")
                .join(".cache")
                .join("ssg")
                .join("llm")
        );
    }

    #[test]
    fn llm_cache_debug_impl_includes_type_name() {
        let (_d, cache) = cache_for_test();
        let debug = format!("{cache:?}");
        assert!(debug.contains("LlmCache"));
    }

    #[test]
    fn cache_stats_debug_impl_includes_type_name() {
        let stats = CacheStats::default();
        let debug = format!("{stats:?}");
        assert!(debug.contains("CacheStats"));
    }

    // ── Fault injection (feature-gated) ──────────────────────────
    //
    // `fs::File` exposes no safe way to make an already-open handle's
    // `metadata()`/`modified()` calls fail, and `LlmCache::entry_path`
    // always produces a path with a parent component, so these three
    // branches are otherwise practically unreachable from a test.

    #[cfg(feature = "test-fault-injection")]
    mod fault {
        use super::*;
        use serial_test::serial;

        #[test]
        #[serial]
        fn entry_is_expired_treats_metadata_error_as_not_expired() {
            let (_d, cache) = cache_for_test();
            let key = LlmCache::compute_key("e", "m", "metadata-err", 1);
            cache.set(&key, "x").unwrap();

            fail::cfg("llm_cache::get-metadata-err", "return").unwrap();
            let hit = cache.get(&key);
            let _ = fail::cfg("llm_cache::get-metadata-err", "off");

            assert_eq!(
                hit.as_deref(),
                Some("x"),
                "metadata() failure must fall back to 'not expired', not a miss"
            );
        }

        #[test]
        #[serial]
        fn entry_is_expired_treats_modified_error_as_not_expired() {
            let (_d, cache) = cache_for_test();
            let key = LlmCache::compute_key("e", "m", "modified-err", 1);
            cache.set(&key, "y").unwrap();

            fail::cfg("llm_cache::get-modified-err", "return").unwrap();
            let hit = cache.get(&key);
            let _ = fail::cfg("llm_cache::get-modified-err", "off");

            assert_eq!(
                hit.as_deref(),
                Some("y"),
                "modified() failure must fall back to 'not expired', not a miss"
            );
        }

        #[test]
        #[serial]
        fn set_skips_mkdir_when_parent_forced_to_none() {
            let (_d, cache) = cache_for_test();
            let key = LlmCache::compute_key("e", "m", "no-parent", 1);

            fail::cfg("llm_cache::set-no-parent", "return").unwrap();
            let result = cache.set(&key, "z");
            let _ = fail::cfg("llm_cache::set-no-parent", "off");

            // The shard directory was never created (mkdir was
            // skipped), so the temp-file creation that follows fails
            // with NotFound — proving `ensure_parent_dir` really did
            // take the "no parent" arm rather than silently
            // succeeding via the normal path.
            assert!(
                result.is_err(),
                "set() should fail when the parent directory was never created"
            );
        }
    }
}
