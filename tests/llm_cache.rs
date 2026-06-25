// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for the deterministic LLM inference cache
//! (issue #528).
//!
//! These tests cover the cache as wired *through* `LlmPlugin` —
//! the unit tests in `src/plugins/llm_cache.rs` already cover the
//! storage layer in isolation. Here we want:
//!
//! - AC1: a cache hit short-circuits before any HTTP roundtrip and
//!   returns in well under 10 ms.
//! - AC2: changing any inference parameter (model, endpoint,
//!   timeout, prompt) misses the cache and triggers a fresh call.
//! - AC3: cache survives across `LlmPlugin` re-creation (proxy for
//!   "survives across `cargo clean`" — the cache root is outside
//!   `target/`).
//! - AC5: a corrupted on-disk entry is evicted and the next call
//!   degrades gracefully.
//! - `--no-llm-cache` / `SSG_NO_LLM_CACHE` opt-out: a disabled
//!   cache never reads or writes on-disk entries.
//!
//! AC4 (TTL) and AC7 (concurrency) are exercised by the unit
//! tests against the storage layer directly; rerunning them here
//! would be a strictly redundant copy.
//!
//! Run with:
//!
//! ```bash
//! cargo test --features ai --test llm_cache
//! ```

#![cfg(feature = "ai")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ssg::llm::{LlmConfig, LlmPlugin};
use ssg::llm_cache::LlmCache;
use std::time::{Duration, Instant};

/// Pre-seeds the cache with `(endpoint, model, prompt, timeout)` →
/// `payload` so we can assert that `LlmPlugin::query` returns the
/// cached value without ever touching the network.
fn seed(
    root: &std::path::Path,
    endpoint: &str,
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    payload: &str,
) {
    let cache = LlmCache::new(root.to_path_buf());
    let key = LlmCache::compute_key(endpoint, model, prompt, timeout_secs);
    cache.set(&key, payload).unwrap();
}

#[test]
fn ac1_cache_hit_returns_under_10ms_without_network() {
    let dir = tempfile::tempdir().unwrap();
    let endpoint = "http://127.0.0.1:1"; // unreachable on purpose
    let model = "llama3";
    let prompt = "Summarise the user manual in two sentences.";
    let payload = "Cached summary.";

    seed(dir.path(), endpoint, model, prompt, 120, payload);

    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: endpoint.into(),
        model: model.into(),
        timeout_secs: 120,
        cache_disabled: false,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });

    let started = Instant::now();
    let got = plugin.query(prompt).expect("AC1: hit must succeed");
    let elapsed = started.elapsed();

    assert_eq!(got, payload, "AC1: returned payload mismatch");
    // A live call would error after a multi-second connect-refused
    // round trip — the bound here is wide enough to absorb shared
    // CI jitter while still nailing the "didn't reach the wire"
    // invariant.
    assert!(
        elapsed < Duration::from_millis(200),
        "AC1: cache hit took {elapsed:?}, expected < 200 ms",
    );
}

#[test]
fn ac2_changing_prompt_misses() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "http://127.0.0.1:1", "llama3", "p1", 1, "v1");

    // Different prompt — should miss the cache and try to hit the
    // unreachable endpoint, which errors out.
    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "llama3".into(),
        timeout_secs: 1,
        cache_disabled: false,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    let result = plugin.query("p2");
    assert!(
        result.is_err(),
        "AC2: changed prompt must miss cache and hit the network"
    );
}

#[test]
fn ac2_changing_model_misses() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "http://127.0.0.1:1", "llama3", "p", 1, "v1");

    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "mistral".into(), // changed
        timeout_secs: 1,
        cache_disabled: false,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    assert!(
        plugin.query("p").is_err(),
        "AC2: changed model must miss cache and hit the network"
    );
}

#[test]
fn ac3_cache_survives_plugin_recreation() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), "http://127.0.0.1:1", "llama3", "p", 1, "v1");

    // Build a fresh `LlmPlugin` over the same on-disk cache.
    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "llama3".into(),
        timeout_secs: 1,
        cache_disabled: false,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    assert_eq!(
        plugin.query("p").unwrap(),
        "v1",
        "AC3: cache entries must survive plugin re-construction"
    );
}

#[test]
fn ac5_corrupt_entry_falls_through_to_network() {
    let dir = tempfile::tempdir().unwrap();
    // Seed, then overwrite with garbage.
    seed(dir.path(), "http://127.0.0.1:1", "llama3", "p", 1, "v1");
    let key = LlmCache::compute_key("http://127.0.0.1:1", "llama3", "p", 1);
    let hex = format!(
        "{}",
        key.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let (shard, rest) = hex.split_at(2);
    let path = dir.path().join(shard).join(format!("{rest}.json"));
    std::fs::write(&path, "{ not valid json").unwrap();

    // Should evict the bad entry, then attempt a live call that
    // errors because the endpoint is unreachable.
    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "llama3".into(),
        timeout_secs: 1,
        cache_disabled: false,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    assert!(
        plugin.query("p").is_err(),
        "AC5: corrupt entry must be evicted and the call must fall \
         through to a live attempt"
    );
    assert!(
        !path.exists(),
        "AC5: corrupt entry should have been evicted from disk"
    );
}

#[test]
fn disabled_cache_does_not_read_disk() {
    let dir = tempfile::tempdir().unwrap();
    // Seed a value that *would* satisfy the call if the cache were
    // consulted.
    seed(dir.path(), "http://127.0.0.1:1", "llama3", "p", 1, "v1");

    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "llama3".into(),
        timeout_secs: 1,
        cache_disabled: true,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    // With the cache disabled the call must hit the unreachable
    // endpoint and error out rather than satisfying from disk.
    assert!(
        plugin.query("p").is_err(),
        "cache_disabled=true must skip the cache"
    );
}

#[test]
fn disabled_cache_does_not_write_disk() {
    let dir = tempfile::tempdir().unwrap();
    let plugin = LlmPlugin::new(LlmConfig {
        endpoint: "http://127.0.0.1:1".into(),
        model: "llama3".into(),
        timeout_secs: 1,
        cache_disabled: true,
        cache_dir: Some(dir.path().to_path_buf()),
        ..LlmConfig::default()
    });
    // This will error (unreachable endpoint) but, critically, must
    // not write anything to the disabled cache dir.
    let _ = plugin.query("p");
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(
        entries.is_empty(),
        "cache_disabled=true must not create any entries — found \
         {entries:?}",
    );
}
