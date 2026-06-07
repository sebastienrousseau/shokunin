// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::collections`.

use std::fs;

use serde::Deserialize;
use ssg::collections::{get_collection, get_entry, Entry};
use tempfile::tempdir;

#[derive(Debug, Deserialize, PartialEq)]
struct Post {
    title: String,
}

#[test]
fn get_collection_loads_all_markdown_entries() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "---\ntitle: A\n---\n# A").unwrap();
    fs::write(dir.path().join("b.md"), "---\ntitle: B\n---\n# B").unwrap();
    let entries: Vec<Entry<Post>> = get_collection(dir.path()).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn get_collection_empty_dir_yields_empty_vec() {
    let dir = tempdir().unwrap();
    let entries: Vec<Entry<Post>> = get_collection(dir.path()).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn get_entry_finds_existing_post() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("hello.md"), "---\ntitle: H\n---\n# H").unwrap();
    let entry: Option<Entry<Post>> = get_entry(dir.path(), "hello").unwrap();
    assert_eq!(entry.unwrap().data.title, "H");
}

#[test]
fn get_entry_returns_none_for_missing_slug() {
    let dir = tempdir().unwrap();
    let entry: Option<Entry<Post>> = get_entry(dir.path(), "missing").unwrap();
    assert!(entry.is_none());
}
