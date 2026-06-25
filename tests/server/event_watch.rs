// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::event_watch` — exercises the real
//! [`notify`] backend against a temp directory so the OS-event path,
//! not just the pure-logic debouncer, is covered.
//!
//! These tests are intentionally tolerant of wall-clock timing: they
//! assert behaviour (a batch is delivered, paths are deduplicated,
//! debounce collapses N writes into 1 batch) but allow generous
//! windows so they don't flake on slow CI runners.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use ssg::event_watch::{
    debounce_paths, event_should_propagate, ChangeBatch, EventWatcher,
    RecvOutcome, DEFAULT_DEBOUNCE, MAX_BATCH_PATHS,
};
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Pure-logic suite — no filesystem, no notify backend, deterministic.
// ---------------------------------------------------------------------------

#[test]
fn debounce_paths_empty_input() {
    assert!(debounce_paths(&[], Duration::from_millis(100)).is_empty());
}

#[test]
fn debounce_paths_collapses_burst_into_one_batch() {
    // AC6: 4 saves within 80 ms with a 100 ms window => 1 batch.
    let t0 = Instant::now();
    let events = vec![
        (PathBuf::from("style.css"), t0),
        (PathBuf::from("style.css"), t0 + Duration::from_millis(20)),
        (PathBuf::from("style.css"), t0 + Duration::from_millis(40)),
        (PathBuf::from("style.css"), t0 + Duration::from_millis(80)),
    ];
    let batches = debounce_paths(&events, Duration::from_millis(100));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].paths, vec![PathBuf::from("style.css")]);
}

#[test]
fn debounce_paths_splits_at_window_boundary() {
    let t0 = Instant::now();
    let events = vec![
        (PathBuf::from("a"), t0),
        (PathBuf::from("b"), t0 + Duration::from_millis(150)),
    ];
    let out = debounce_paths(&events, Duration::from_millis(100));
    assert_eq!(out.len(), 2);
}

#[test]
fn debounce_paths_dedupes_same_path_in_window() {
    let t0 = Instant::now();
    let events = vec![
        (PathBuf::from("x"), t0),
        (PathBuf::from("x"), t0 + Duration::from_millis(10)),
        (PathBuf::from("x"), t0 + Duration::from_millis(20)),
    ];
    let out = debounce_paths(&events, Duration::from_millis(100));
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].paths.len(), 1);
}

#[test]
fn change_batch_is_empty_for_empty_paths() {
    let b = ChangeBatch { paths: vec![] };
    assert!(b.is_empty());
}

#[test]
fn change_batch_len_matches_path_count() {
    let b = ChangeBatch {
        paths: vec![PathBuf::from("a"), PathBuf::from("b")],
    };
    assert_eq!(b.len(), 2);
}

#[test]
fn event_should_propagate_filters_access_events() {
    use notify::event::{AccessKind, CreateKind};
    use notify::EventKind;
    assert!(event_should_propagate(&EventKind::Create(CreateKind::File)));
    assert!(!event_should_propagate(&EventKind::Access(AccessKind::Any)));
}

#[test]
fn default_debounce_constant_is_one_hundred_millis() {
    assert_eq!(DEFAULT_DEBOUNCE, Duration::from_millis(100));
}

#[test]
fn max_batch_paths_is_reasonable_bound() {
    // Sanity: cap should be much larger than a typical project but
    // small enough that we don't allocate gigabytes of paths.
    assert!(MAX_BATCH_PATHS >= 1_000);
    assert!(MAX_BATCH_PATHS <= 1_000_000);
}

// ---------------------------------------------------------------------------
// Live notify-backed suite — touches the filesystem, may flake on very
// slow runners but uses generous timeouts.
// ---------------------------------------------------------------------------

/// Helper: spin a watcher on `dir` and wait up to `timeout` for the
/// first non-empty batch.
fn wait_for_batch(
    watcher: &EventWatcher,
    timeout: Duration,
) -> Option<ChangeBatch> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match watcher.recv_timeout(Duration::from_millis(200)) {
            RecvOutcome::Batch(b) if !b.is_empty() => return Some(b),
            RecvOutcome::Batch(_) | RecvOutcome::Timeout => continue,
            RecvOutcome::Closed => return None,
        }
    }
    None
}

#[test]
fn ac1_event_watcher_observes_a_modify() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "hello").unwrap();

    let watcher =
        EventWatcher::with_debounce(dir.path(), Duration::from_millis(50))
            .unwrap();

    // Give notify a beat to subscribe before the write.
    thread::sleep(Duration::from_millis(100));

    fs::write(dir.path().join("a.md"), "world").unwrap();

    let batch = wait_for_batch(&watcher, Duration::from_secs(5))
        .expect("watcher should deliver one batch");
    assert!(!batch.is_empty(), "batch must carry at least one path");
    let names: Vec<String> = batch
        .paths
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    assert!(
        names.iter().any(|n| n == "a.md"),
        "batch must contain a.md, got {names:?}"
    );
}

#[test]
fn ac6_burst_writes_coalesce_into_one_batch() {
    // 4 writes inside the debounce window => the watcher delivers a
    // single batch containing a.md exactly once.
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "0").unwrap();

    let watcher =
        EventWatcher::with_debounce(dir.path(), Duration::from_millis(200))
            .unwrap();
    thread::sleep(Duration::from_millis(100));

    for i in 0..4 {
        fs::write(dir.path().join("a.md"), i.to_string()).unwrap();
        thread::sleep(Duration::from_millis(30));
    }

    let batch = wait_for_batch(&watcher, Duration::from_secs(5))
        .expect("first batch should arrive");
    // After the first batch we shouldn't get a second one within
    // the debounce window — drain briefly to be sure.
    let extra = watcher.recv_timeout(Duration::from_millis(300));
    let extra_count = match extra {
        RecvOutcome::Batch(b) if !b.is_empty() => 1,
        _ => 0,
    };

    assert!(!batch.is_empty(), "first batch must carry the writes");
    assert_eq!(
        extra_count, 0,
        "burst writes should collapse into one batch, not many"
    );
}

#[test]
fn watcher_dropped_does_not_leak_thread() {
    // Smoke: build a watcher, drop it, ensure we can build another in
    // the same process without resource exhaustion.
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("x.md"), "1").unwrap();

    for _ in 0..3 {
        let watcher = EventWatcher::new(dir.path()).unwrap();
        drop(watcher);
    }
}

#[test]
fn ac7_watcher_rejects_missing_directory() {
    // Missing directory => Err from notify.watch(); no panic.
    let res = EventWatcher::new(std::path::Path::new(
        "/definitely/does/not/exist/for/ssg/test",
    ));
    assert!(res.is_err());
}

#[test]
fn watcher_default_debounce_is_one_hundred_millis() {
    let dir = tempdir().unwrap();
    let watcher = EventWatcher::new(dir.path()).unwrap();
    assert_eq!(watcher.debounce(), DEFAULT_DEBOUNCE);
}

#[test]
fn watcher_debug_format_includes_struct_name() {
    let dir = tempdir().unwrap();
    let watcher = EventWatcher::new(dir.path()).unwrap();
    let d = format!("{watcher:?}");
    assert!(d.contains("EventWatcher"));
}
