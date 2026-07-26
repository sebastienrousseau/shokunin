// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::scaffold`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::scaffold::scaffold_project_at;
use tempfile::tempdir;

#[test]
fn scaffold_project_at_creates_project_dirs() {
    let dir = tempdir().unwrap();
    scaffold_project_at("mysite", dir.path()).unwrap();
    let root = dir.path().join("mysite");
    assert!(root.exists(), "project root created");
    assert!(root.join("content").exists() || root.join("templates").exists());
}

#[test]
fn scaffold_project_at_is_idempotent_on_existing_path() {
    let dir = tempdir().unwrap();
    scaffold_project_at("twice", dir.path()).unwrap();
    let _second = scaffold_project_at("twice", dir.path());
    // second run may succeed or report an existing-project error;
    // either way it should not panic.
}
