// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::fs_ops`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::fs;

use ssg::error::SsgError;
use ssg::fs_ops::{
    copy_dir_all, is_safe_path, verify_and_copy_files, verify_file_safety,
};
use tempfile::tempdir;

#[test]
fn is_safe_path_accepts_normal_directory() {
    let dir = tempdir().unwrap();
    assert!(is_safe_path(dir.path()).unwrap());
}

#[test]
fn is_safe_path_returns_false_for_traversal_components() {
    let path = std::path::PathBuf::from("../escape.txt");
    let safe = is_safe_path(&path).unwrap();
    assert!(!safe, "paths with `..` components must be flagged unsafe");
}

#[test]
fn verify_file_safety_rejects_oversize_or_missing_file() {
    let dir = tempdir().unwrap();
    // Missing file → Io variant (not PathTraversal); verify the error type.
    let err = verify_file_safety(&dir.path().join("absent.txt")).unwrap_err();
    assert!(matches!(err, SsgError::Io { .. }));
}

#[test]
fn copy_dir_all_copies_files_recursively() {
    let src_dir = tempdir().unwrap();
    let dst_dir = tempdir().unwrap();
    let src = src_dir.path();
    let dst = dst_dir.path().join("out");

    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("a.txt"), "one").unwrap();
    fs::write(src.join("sub/b.txt"), "two").unwrap();

    copy_dir_all(src, &dst).unwrap();
    assert!(dst.join("a.txt").exists());
    assert!(dst.join("sub/b.txt").exists());
}

#[test]
fn verify_and_copy_files_round_trips_content() {
    let src_dir = tempdir().unwrap();
    let dst_dir = tempdir().unwrap();
    fs::write(src_dir.path().join("note.md"), "body").unwrap();
    verify_and_copy_files(src_dir.path(), dst_dir.path()).unwrap();
    let copied = fs::read_to_string(dst_dir.path().join("note.md")).unwrap();
    assert_eq!(copied, "body");
}
