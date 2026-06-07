// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::frontmatter`.

use std::fs;

use ssg::frontmatter::{emit_sidecars, read_sidecar, read_sidecar_for_html};
use tempfile::tempdir;

#[test]
fn emit_sidecars_writes_one_meta_json_per_markdown_file() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let sidecar = dir.path().join("sidecars");
    fs::create_dir_all(&content).unwrap();

    fs::write(
        content.join("a.md"),
        "---\ntitle: A\n---\n# A\n\nHello world.",
    )
    .unwrap();
    fs::write(
        content.join("b.md"),
        "---\ntitle: B\n---\n# B\n\nLorem ipsum dolor sit amet.",
    )
    .unwrap();

    let count = emit_sidecars(&content, &sidecar).unwrap();
    assert_eq!(count, 2);
    assert!(sidecar.join("a.meta.json").exists());
    assert!(sidecar.join("b.meta.json").exists());
}

#[test]
fn emit_sidecars_skips_files_without_frontmatter() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let sidecar = dir.path().join("sidecars");
    fs::create_dir_all(&content).unwrap();

    fs::write(content.join("plain.md"), "# No frontmatter").unwrap();
    let count = emit_sidecars(&content, &sidecar).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn read_sidecar_returns_none_for_missing_file() {
    let dir = tempdir().unwrap();
    let result = read_sidecar(&dir.path().join("nope.html")).unwrap();
    assert!(result.is_none());
}

#[test]
fn read_sidecar_returns_word_count_and_reading_time() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let sidecar = dir.path().join("sidecars");
    fs::create_dir_all(&content).unwrap();

    fs::write(
        content.join("post.md"),
        "---\ntitle: Post\n---\n# Heading\n\nBody words here.",
    )
    .unwrap();
    let _ = emit_sidecars(&content, &sidecar).unwrap();

    let meta = read_sidecar(&sidecar.join("post.html"))
        .unwrap()
        .expect("sidecar");
    assert!(meta.contains_key("word_count"));
    assert!(meta.contains_key("reading_time"));
}

#[test]
fn read_sidecar_for_html_resolves_md_mapping() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let sidecar = dir.path().join("sidecars");
    let site = dir.path().join("site");
    fs::create_dir_all(&content).unwrap();
    fs::create_dir_all(&site).unwrap();

    fs::write(
        content.join("post.md"),
        "---\ntitle: Post\n---\n# H\n\nBody.",
    )
    .unwrap();
    let _ = emit_sidecars(&content, &sidecar).unwrap();

    // The function looks up `.html` paths by mapping back to `.md`.
    let html = site.join("post.html");
    let meta = read_sidecar_for_html(&html, &site, &sidecar).unwrap();
    assert!(
        meta.is_some(),
        "should resolve post.html → post.md.meta.json"
    );
}
