// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::process`.

use ssg::process::ensure_directory;
use tempfile::tempdir;

#[test]
fn ensure_directory_creates_missing_dir() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("nested/child");
    ensure_directory(&target, "child directory").unwrap();
    assert!(target.is_dir());
}

#[test]
fn ensure_directory_is_idempotent_on_existing_dir() {
    let dir = tempdir().unwrap();
    ensure_directory(dir.path(), "existing").unwrap();
    ensure_directory(dir.path(), "existing").unwrap();
}
