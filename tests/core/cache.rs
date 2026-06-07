// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::cache::BuildCache`.
//!
//! These tests drive the public API surface (the same surface a downstream
//! crate would import) end-to-end: load, detect-changes, update, persist.

use std::fs;
use std::path::Path;

use ssg::cache::BuildCache;
use tempfile::tempdir;

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create_dir_all");
    }
    fs::write(path, body).expect("write");
}

#[test]
fn load_missing_cache_yields_empty_state() {
    let dir = tempdir().unwrap();
    let cache = BuildCache::load(&dir.path().join(".ssg-cache.json"))
        .expect("load");
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn new_file_is_reported_as_changed() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    write_file(&content.join("post.md"), "# Hello");

    let cache = BuildCache::load(&dir.path().join(".ssg-cache.json"))
        .expect("load");
    let changed = cache.changed_files(&content).expect("changed_files");

    assert_eq!(changed.len(), 1, "single new file should appear once");
    assert!(changed[0].ends_with("post.md"));
}

#[test]
fn unchanged_file_is_not_reported_after_update() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join(".ssg-cache.json");
    let content = dir.path().join("content");
    write_file(&content.join("post.md"), "# Hello");

    let mut cache = BuildCache::load(&cache_path).expect("load");
    cache.update(&content).expect("update");
    cache.save().expect("save");

    let after = BuildCache::load(&cache_path).expect("reload");
    let changed = after.changed_files(&content).expect("changed_files");
    assert!(changed.is_empty(), "no file should be reported as changed");
}

#[test]
fn modified_file_is_detected_on_next_scan() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join(".ssg-cache.json");
    let content = dir.path().join("content");
    let post = content.join("post.md");
    write_file(&post, "# Original");

    let mut cache = BuildCache::load(&cache_path).expect("load");
    cache.update(&content).expect("first update");
    cache.save().expect("first save");

    // Mutate the file body so its fingerprint differs.
    fs::write(&post, "# Edited").expect("rewrite");

    let after = BuildCache::load(&cache_path).expect("reload");
    let changed = after.changed_files(&content).expect("changed_files");
    assert_eq!(changed.len(), 1);
    assert!(changed[0].ends_with("post.md"));
}

#[test]
fn save_roundtrip_preserves_entry_count() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join(".ssg-cache.json");
    let content = dir.path().join("content");
    for i in 0..5 {
        write_file(&content.join(format!("post-{i}.md")), "# Page");
    }

    let mut cache = BuildCache::load(&cache_path).expect("load");
    cache.update(&content).expect("update");
    cache.save().expect("save");
    assert_eq!(cache.len(), 5);

    let reloaded = BuildCache::load(&cache_path).expect("reload");
    assert_eq!(reloaded.len(), 5);
    assert!(!reloaded.is_empty());
}
