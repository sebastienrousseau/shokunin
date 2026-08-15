// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Release version-coherence gate.
//!
//! Every publishable crate in the workspace must carry the same
//! version as the root `ssg` crate, and every internal dependency
//! pin must name that same version.
//!
//! ## What this catches
//!
//! Version drift between the root crate and the workspace members it
//! depends on. `release.yml` publishes `ssg-core`, `ssg-rpc-macro`,
//! `ssg-rpc`, `ssg-search` and `ssg-a11y` to crates.io before `ssg`
//! itself, because crates.io verifies path dependencies against the
//! registry. If a member's version was not bumped along with the
//! root, the first `cargo publish` of the release aborts with
//! `error: crate <name>@<version> already exists on crates.io index`
//! and nothing downstream of it publishes.
//!
//! That is not hypothetical. Between v0.0.47 and v0.0.50 the members
//! stayed pinned at `0.0.47` while the root advanced to `0.0.50`, so
//! the v0.0.48 release failed at the very first publish, `ssg` was
//! skipped, and crates.io fell two versions behind the README's own
//! install instruction.
//!
//! The failure surfaced only after a tag had been pushed — the point
//! at which it is most expensive, because a tag that produced no
//! release has to be deleted and re-cut. This test moves the same
//! signal to every `cargo test` run.
//!
//! ## Methodology
//!
//! Parse the root and member manifests directly. No TOML crate: the
//! fields involved are single-line `key = "value"` entries, and
//! `tests/docs_accuracy.rs` sets the precedent for keeping this kind
//! of gate dependency-free.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// Members that `release.yml` publishes to crates.io, in the
/// dependency order it uses. `ssg-wasm` is absent deliberately: it
/// carries `publish = false` and is not a dependency of `ssg`.
const PUBLISHED_MEMBERS: &[&str] = &[
    "ssg-core",
    "ssg-rpc-macro",
    "ssg-rpc",
    "ssg-search",
    "ssg-a11y",
];

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn manifest_of(member: &str) -> PathBuf {
    workspace().join("crates").join(member).join("Cargo.toml")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Returns the `version = "…"` value from a manifest's `[package]`
/// table, ignoring `version` keys that appear in dependency tables.
fn package_version(path: &Path) -> String {
    let toml = read(path);
    let mut in_package = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("version = \"") {
                if let Some(v) = rest.split('"').next() {
                    return v.to_string();
                }
            }
        }
    }
    panic!("no [package] version in {}", path.display());
}

/// Returns every `version = "…"` pin attached to an internal
/// `ssg-*` path dependency in `path`, as `(crate_name, pinned)`.
///
/// Matches the single-line inline-table form the workspace uses
/// throughout, e.g.
/// `ssg-core = { path = "crates/ssg-core", version = "0.0.50" }`.
fn internal_pins(path: &Path) -> Vec<(String, String)> {
    let toml = read(path);
    let mut pins = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some((name, rest)) = trimmed.split_once(" = ") else {
            continue;
        };
        if !name.starts_with("ssg-") || !rest.contains("path = ") {
            continue;
        }
        if let Some(after) = rest.split("version = \"").nth(1) {
            if let Some(v) = after.split('"').next() {
                pins.push((name.to_string(), v.to_string()));
            }
        }
    }
    pins
}

#[test]
fn published_members_match_the_root_version() {
    let root = package_version(&workspace().join("Cargo.toml"));
    let mut drift = Vec::new();

    for member in PUBLISHED_MEMBERS {
        let got = package_version(&manifest_of(member));
        if got != root {
            drift.push(format!(
                "  crates/{member}/Cargo.toml: {got}  (want {root})"
            ));
        }
    }

    assert!(
        drift.is_empty(),
        "workspace members have drifted from the root version {root}.\n\
         `cargo publish` will abort with \"already exists on crates.io \
         index\" on the first of these and the release will publish \
         nothing:\n{}\n\nBump each to {root}, refresh Cargo.lock, and \
         re-run.",
        drift.join("\n")
    );
}

#[test]
fn ssg_wasm_matches_the_root_version_even_though_it_is_unpublished() {
    // `publish = false` keeps it off crates.io, but leaving it behind
    // makes `cargo metadata` output and release notes inconsistent,
    // and it is the member most likely to be forgotten precisely
    // because the publish step never touches it.
    let root = package_version(&workspace().join("Cargo.toml"));
    let got = package_version(&manifest_of("ssg-wasm"));
    assert_eq!(
        got, root,
        "crates/ssg-wasm/Cargo.toml is at {got}, root is at {root}"
    );
}

#[test]
fn internal_dependency_pins_match_the_root_version() {
    let root = package_version(&workspace().join("Cargo.toml"));
    let mut stale = Vec::new();

    let mut manifests = vec![workspace().join("Cargo.toml")];
    manifests.extend(PUBLISHED_MEMBERS.iter().map(|m| manifest_of(m)));
    manifests.push(manifest_of("ssg-wasm"));

    for manifest in &manifests {
        for (dep, pinned) in internal_pins(manifest) {
            if pinned != root {
                stale.push(format!(
                    "  {}: {dep} pinned at {pinned}  (want {root})",
                    manifest
                        .strip_prefix(workspace())
                        .unwrap_or(manifest)
                        .display()
                ));
            }
        }
    }

    assert!(
        stale.is_empty(),
        "internal dependency pins have drifted from the root version \
         {root}.\ncrates.io resolves these against the registry, so a \
         stale pin either publishes against an old sub-crate or fails \
         outright:\n{}",
        stale.join("\n")
    );
}

#[test]
fn published_members_list_matches_the_release_workflow() {
    // The list above only protects what `release.yml` actually
    // publishes. If a member is added to the workflow loop without
    // being added here, it silently loses its version gate.
    let workflow = read(&workspace().join(".github/workflows/release.yml"));
    let loop_line = workflow
        .lines()
        .find(|l| l.trim_start().starts_with("for c in ssg-"))
        .expect("release.yml no longer has the `for c in ssg-…` publish loop");

    let in_workflow: Vec<&str> = loop_line
        .trim()
        .trim_start_matches("for c in ")
        .trim_end_matches("; do")
        .split_whitespace()
        .collect();

    assert_eq!(
        in_workflow, PUBLISHED_MEMBERS,
        "release.yml publishes a different set of crates than this \
         test guards.\nrelease.yml: {in_workflow:?}\nthis test:  \
         {PUBLISHED_MEMBERS:?}"
    );
}
