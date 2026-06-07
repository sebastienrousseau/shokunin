// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::stream`.

use std::fs;

use ssg::stream::{stream_copy, stream_hash};
use tempfile::tempdir;

#[test]
fn stream_copy_copies_full_file_byte_for_byte() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.bin");
    let dst = dir.path().join("dst.bin");
    let payload = vec![0xABu8; 8192];
    fs::write(&src, &payload).unwrap();
    let copied = stream_copy(&src, &dst).unwrap();
    assert_eq!(copied as usize, payload.len());
    assert_eq!(fs::read(&dst).unwrap(), payload);
}

#[test]
fn stream_hash_returns_a_non_empty_hex_string() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("h.txt");
    fs::write(&path, "hello world").unwrap();
    let h = stream_hash(&path).unwrap();
    assert!(!h.is_empty());
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

