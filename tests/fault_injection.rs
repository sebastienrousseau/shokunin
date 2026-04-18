// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(missing_docs)]
#![cfg(feature = "test-fault-injection")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Fault-injection integration tests.
//!
//! These tests use the [`fail`](https://docs.rs/fail) crate to
//! activate failpoints sprinkled in front of `fs::write` /
//! `fs::create_dir_all` call sites in the library, and assert that
//! every error path is correctly propagated as an `anyhow::Error`
//! with the right context.
//!
//! Failpoints are **process-global state**, so this entire test
//! suite lives in its own integration test binary (separate from
//! the lib test binary). That isolation is what lets the regular
//! lib tests in `src/scaffold.rs` continue to run with the
//! `test-fault-injection` feature enabled — they live in a
//! different process and never see the activated failpoints.
//!
//! Run with:
//!
//! ```sh
//! cargo test --features test-fault-injection --test fault_injection
//! ```
//!
//! Each test serializes its activate → run → deactivate sequence
//! via [`serial_test::serial`] so concurrent tests in this binary
//! don't fight over the same global failpoint state. The teardown
//! is performed in a `Drop` guard so a panicking assertion still
//! cleans up.

use serial_test::serial;
use ssg::cache::BuildCache;
use ssg::cmd::SsgConfig;
use ssg::plugin::{Plugin, PluginContext};
use ssg::plugins::MinifyPlugin;
use ssg::scaffold::scaffold_project_at;
use std::fs;
use tempfile::tempdir;

/// RAII guard that disables a failpoint on drop.
struct FailGuard<'a>(&'a str);

impl Drop for FailGuard<'_> {
    fn drop(&mut self) {
        let _ = fail::cfg(self.0, "off");
    }
}

/// Activates `name`, runs `scaffold_project_at` against a fresh
/// tempdir, deactivates the failpoint, and returns the resulting
/// error so the caller can assert against its message.
fn run_scaffold_with_failpoint(name: &str) -> anyhow::Error {
    let _guard = FailGuard(name);
    fail::cfg(name, "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    scaffold_project_at("fault-test-site", dir.path())
        .expect_err("scaffold should fail when failpoint is active")
}

#[test]
#[serial]
fn scaffold_fault_create_dir_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::create-dir");
    assert!(format!("{err:?}").contains("scaffold::create-dir"));
}

#[test]
#[serial]
fn scaffold_fault_write_config_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-config");
    assert!(format!("{err:?}").contains("scaffold::write-config"));
}

#[test]
#[serial]
fn scaffold_fault_write_index_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-index");
    assert!(format!("{err:?}").contains("scaffold::write-index"));
}

#[test]
#[serial]
fn scaffold_fault_write_about_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-about");
    assert!(format!("{err:?}").contains("scaffold::write-about"));
}

#[test]
#[serial]
fn scaffold_fault_write_post_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-post");
    assert!(format!("{err:?}").contains("scaffold::write-post"));
}

#[test]
#[serial]
fn scaffold_fault_write_base_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-base");
    assert!(format!("{err:?}").contains("scaffold::write-base"));
}

#[test]
#[serial]
fn scaffold_fault_write_page_tpl_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-page-tpl");
    assert!(format!("{err:?}").contains("scaffold::write-page-tpl"));
}

#[test]
#[serial]
fn scaffold_fault_write_post_tpl_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-post-tpl");
    assert!(format!("{err:?}").contains("scaffold::write-post-tpl"));
}

#[test]
#[serial]
fn scaffold_fault_write_index_tpl_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-index-tpl");
    assert!(format!("{err:?}").contains("scaffold::write-index-tpl"));
}

#[test]
#[serial]
fn scaffold_fault_write_css_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-css");
    assert!(format!("{err:?}").contains("scaffold::write-css"));
}

#[test]
#[serial]
fn scaffold_fault_write_nav_returns_err() {
    let err = run_scaffold_with_failpoint("scaffold::write-nav");
    assert!(format!("{err:?}").contains("scaffold::write-nav"));
}

// =====================================================================
// cmd::validate_path_safety
// =====================================================================

#[test]
#[serial]
fn cmd_fault_symlink_metadata_returns_err() {
    // Activate the failpoint that sits in front of fs::symlink_metadata
    // inside validate_path_safety. We need an existing directory so
    // the `path.exists()` branch is taken.
    let _guard = FailGuard("cmd::symlink-metadata");
    fail::cfg("cmd::symlink-metadata", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let mut config = SsgConfig::default();
    config.content_dir = dir.path().to_path_buf();
    config.output_dir = dir.path().to_path_buf();
    config.template_dir = dir.path().to_path_buf();

    let err = config.validate().expect_err("validate should fail");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("injected: cmd::symlink-metadata"),
        "expected injected error, got: {msg}"
    );
}

// =====================================================================
// cache::BuildCache load + save
// =====================================================================

#[test]
#[serial]
fn cache_fault_read_returns_err() {
    let _guard = FailGuard("cache::read");
    fail::cfg("cache::read", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("cache.json");
    fs::write(&cache_path, "{}").expect("seed cache file");

    let err = BuildCache::load(&cache_path)
        .expect_err("load should fail when cache::read failpoint is active");
    assert!(format!("{err:?}").contains("injected: cache::read"));
}

#[test]
#[serial]
fn cache_fault_parse_returns_err() {
    let _guard = FailGuard("cache::parse");
    fail::cfg("cache::parse", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("cache.json");
    fs::write(&cache_path, r#"{"fingerprints":{}}"#).expect("seed");

    let err = BuildCache::load(&cache_path)
        .expect_err("load should fail when cache::parse failpoint is active");
    assert!(format!("{err:?}").contains("injected: cache::parse"));
}

#[test]
#[serial]
fn cache_fault_write_returns_err() {
    let _guard = FailGuard("cache::write");
    fail::cfg("cache::write", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let cache_path = dir.path().join("cache.json");
    let cache = BuildCache::new(&cache_path);

    let err = cache
        .save()
        .expect_err("save should fail when cache::write failpoint is active");
    assert!(format!("{err:?}").contains("injected: cache::write"));
}

// =====================================================================
// plugins::MinifyPlugin read + write
// =====================================================================

#[test]
#[serial]
fn plugins_fault_minify_read_returns_err() {
    let _guard = FailGuard("plugins::minify-read");
    fail::cfg("plugins::minify-read", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let site = dir.path().to_path_buf();
    fs::write(site.join("index.html"), "<p>x</p>").expect("seed html");

    let ctx = PluginContext::new(&site, &site, &site, &site);
    let err = MinifyPlugin
        .after_compile(&ctx)
        .expect_err("after_compile should fail when read failpoint is active");
    assert!(format!("{err:?}").contains("injected: plugins::minify-read"));
}

#[test]
#[serial]
fn plugins_fault_minify_write_returns_err() {
    let _guard = FailGuard("plugins::minify-write");
    fail::cfg("plugins::minify-write", "return").expect("activate failpoint");

    let dir = tempdir().expect("tempdir");
    let site = dir.path().to_path_buf();
    fs::write(site.join("index.html"), "<p>x</p>").expect("seed html");

    let ctx = PluginContext::new(&site, &site, &site, &site);
    let err = MinifyPlugin
        .after_compile(&ctx)
        .expect_err("after_compile should fail when write failpoint is active");
    assert!(format!("{err:?}").contains("injected: plugins::minify-write"));
}
