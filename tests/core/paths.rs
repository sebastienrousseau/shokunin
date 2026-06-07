// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::{Paths, PathsBuilder}` (top-level facade).

use std::path::PathBuf;

use ssg::{create_directories, Paths};
use tempfile::tempdir;

#[test]
fn builder_with_explicit_dirs_round_trips() {
    let dir = tempdir().unwrap();
    let p = Paths::builder()
        .site(dir.path().join("public"))
        .content(dir.path().join("content"))
        .build_dir(dir.path().join("build"))
        .template(dir.path().join("templates"))
        .build()
        .expect("paths");
    assert!(p.site.ends_with("public"));
    assert!(p.content.ends_with("content"));
    assert!(p.build.ends_with("build"));
    assert!(p.template.ends_with("templates"));
}

#[test]
fn builder_relative_to_anchors_all_four_dirs() {
    let dir = tempdir().unwrap();
    let p = Paths::builder()
        .relative_to(dir.path())
        .build()
        .expect("paths");
    assert!(p.site.starts_with(dir.path()));
    assert!(p.content.starts_with(dir.path()));
    assert!(p.build.starts_with(dir.path()));
    assert!(p.template.starts_with(dir.path()));
}

#[test]
fn builder_default_falls_back_to_relative_defaults() {
    let p = Paths::builder().build().expect("paths");
    assert_eq!(p.site, PathBuf::from("public"));
    assert_eq!(p.content, PathBuf::from("content"));
    assert_eq!(p.build, PathBuf::from("build"));
    assert_eq!(p.template, PathBuf::from("templates"));
}

#[test]
fn create_directories_creates_all_four_dirs() {
    let dir = tempdir().unwrap();
    let p = Paths {
        site: dir.path().join("public"),
        content: dir.path().join("content"),
        build: dir.path().join("build"),
        template: dir.path().join("templates"),
    };
    create_directories(&p).expect("create");
    assert!(p.site.is_dir());
    assert!(p.content.is_dir());
    assert!(p.build.is_dir());
    assert!(p.template.is_dir());
}

// Note: `create_directories`'s PathTraversal branches at lib.rs:374-394
// are functionally unreachable in normal use because `fs::create_dir_all`
// runs BEFORE `is_safe_path`, and `is_safe_path` only flags `..` for
// non-existent paths. The branches will be addressed in P3.2 via either
// a refactor or a `#[coverage(off)]` annotation.
