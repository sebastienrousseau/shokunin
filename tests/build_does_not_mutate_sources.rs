// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression guard for issue #543 — `preprocess_content` in-place writer.
//!
//! Before v0.0.44 the build path called a `preprocess_content` helper that
//! rewrote every `*.md` file under `content/` in place, appending a
//! `<!--frontmatter-processed-->` sentinel so subsequent runs could
//! short-circuit. That helper:
//!
//! - dirtied users' git working trees on every build,
//! - left source files partially transformed if the build crashed mid-pass,
//! - polluted commit history and `git blame` with sentinel comments, and
//! - silently changed files that editors with auto-reload might already
//!   have open.
//!
//! This test snapshots a SHA-256 of every file in a synthesised `content/`
//! tree, runs the build, snapshots again, and asserts byte-for-byte
//! equality. It also runs the build a second time and re-checks, covering
//! the idempotency requirement (AC5 in the issue).
//!
//! The test does NOT depend on `staticdatagen` succeeding — for the
//! purpose of "did we write to a source file?" it doesn't matter whether
//! the compiler errors out further down the pipeline. We assert only on
//! the immutability of the inputs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{arg, Command};
use sha2::{Digest, Sha256};
use ssg::process;
use tempfile::TempDir;

/// Walks `root` recursively and returns a `path → SHA-256-hex` map for
/// every regular file underneath it. Sorted by `BTreeMap` so the
/// comparison output (if a test fails) is stable and diffable.
fn hash_tree(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_dir() {
            walk(root, &path, out);
        } else if ft.is_file() {
            let bytes = fs::read(&path).expect("read source file");
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let digest = hasher.finalize();
            // sha2 0.11 returns a `GenericArray` which does not impl
            // `LowerHex` directly, so hex-encode by hand. 32 bytes →
            // 64-char lower-case hex string.
            let mut hex = String::with_capacity(digest.len() * 2);
            for byte in digest {
                hex.push_str(&format!("{byte:02x}"));
            }
            let rel = path
                .strip_prefix(root)
                .expect("path is under root")
                .to_path_buf();
            let _ = out.insert(rel, hex);
        }
    }
}

/// Builds a small but representative content tree at `dir` covering the
/// shapes the old `preprocess_content` walker used to touch:
/// - a frontmatter-bearing `.md` file (the primary target of the writer),
/// - a `.md` with no frontmatter (control),
/// - a non-markdown file (must never be touched),
/// - a nested subdirectory (the old writer was non-recursive, but the new
///   regression test asserts that no file at any depth is mutated).
fn seed_content(dir: &Path) {
    fs::create_dir_all(dir).unwrap();

    fs::write(
        dir.join("index.md"),
        "---\ntitle: Home\ndate: 2026-06-25\n---\n# Welcome\n\nHello world.\n",
    )
    .unwrap();

    fs::write(
        dir.join("plain.md"),
        "# Plain page\n\nNo frontmatter on this one.\n",
    )
    .unwrap();

    fs::write(
        dir.join("notes.txt"),
        "not markdown — must not be touched\n",
    )
    .unwrap();

    let nested = dir.join("posts");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("2026-06-25-first.md"),
        "---\ntitle: First Post\n---\nBody.\n",
    )
    .unwrap();
}

/// Constructs the `clap::ArgMatches` shape that `process::args` consumes.
fn make_matches(
    content: &Path,
    output: &Path,
    site: &Path,
    template: &Path,
) -> clap::ArgMatches {
    Command::new("ssg")
        .arg(arg!(--"content" <CONTENT> "Content directory"))
        .arg(arg!(--"output"  <OUTPUT>  "Output directory"))
        .arg(arg!(--"new"     <NEW>     "Site directory"))
        .arg(arg!(--"template" <TEMPLATE> "Template directory"))
        .get_matches_from(vec![
            "ssg",
            "--content",
            content.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--new",
            site.to_str().unwrap(),
            "--template",
            template.to_str().unwrap(),
        ])
}

#[test]
fn build_leaves_source_tree_unchanged() {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("output");
    let site = tmp.path().join("site");
    let template = tmp.path().join("template");

    seed_content(&content);

    // Capture the pre-build hash set. This is the ground truth: any
    // post-build divergence means the build path mutated a source file.
    let before = hash_tree(&content);
    assert!(
        !before.is_empty(),
        "fixture seeding produced an empty content tree — test is meaningless"
    );

    // Run the build. We deliberately ignore the result: staticdatagen
    // may or may not succeed depending on the surrounding template /
    // site directories, but that is orthogonal to the question this
    // test answers ("did anything write to content/?").
    let matches = make_matches(&content, &output, &site, &template);
    let _ = process::args(&matches);

    let after = hash_tree(&content);
    assert_eq!(
        before, after,
        "build mutated one or more source files under content/ — \
         this is the regression guarded by issue #543"
    );

    // Specifically guard against the historical sentinel ever
    // reappearing in any source file.
    for rel in after.keys() {
        let abs = content.join(rel);
        if let Ok(text) = fs::read_to_string(&abs) {
            assert!(
                !text.contains("<!--frontmatter-processed-->"),
                "source file {} contains the v0.0.43 sentinel comment — \
                 the destructive `preprocess_content` writer has returned",
                rel.display()
            );
        }
    }
}

#[test]
fn repeat_build_is_idempotent_for_sources() {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("output");
    let site = tmp.path().join("site");
    let template = tmp.path().join("template");

    seed_content(&content);
    let before = hash_tree(&content);

    let matches = make_matches(&content, &output, &site, &template);
    let _ = process::args(&matches);
    let after_first = hash_tree(&content);
    assert_eq!(
        before, after_first,
        "first build mutated source tree (issue #543 regression)"
    );

    // Second back-to-back build — the symptom of the old writer was
    // that run #2 would either re-append the sentinel or short-circuit
    // based on its presence. Either way, run #1 → run #2 was the
    // observable state change. Assert it cannot happen.
    let matches = make_matches(&content, &output, &site, &template);
    let _ = process::args(&matches);
    let after_second = hash_tree(&content);
    assert_eq!(
        after_first, after_second,
        "second build mutated source tree — builds are not idempotent \
         with respect to content/ (issue #543 regression)"
    );
}
