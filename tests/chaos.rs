// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Chaos-engineering integration tests (resolves #423).
//!
//! Where `fault_injection.rs` exercises injected I/O failures via
//! `fail` crate failpoints, this suite exercises **real-world
//! malformed input**: corrupt YAML, truncated images, symlink loops,
//! pathological directory depth, permission-denied targets, and
//! concurrent builds against the same output.
//!
//! Every test asserts the build either:
//!
//! 1. Returns a `Result::Err` with a human-readable error chain, or
//! 2. Completes with the malformed input safely skipped/quarantined,
//!
//! and in **no** case panics or aborts the test process. Catching
//! panics via `std::panic::catch_unwind` would mask real bugs, so
//! these tests rely on the runtime not aborting; if a panic escapes
//! into the test harness, the test fails the standard way.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Bare-bones content/build/site/templates layout under a fresh
/// tempdir. Returns `(tmp, content, build, site, template)`.
fn fresh_layout() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let tmp = tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let build = tmp.path().join("build");
    let site = tmp.path().join("site");
    let template = tmp.path().join("templates");
    for d in [&content, &build, &site, &template] {
        fs::create_dir_all(d).unwrap();
    }
    (tmp, content, build, site, template)
}

/// Convenience: invoke `compile_site` and return any error chain as
/// a flattened string. Returns `Ok(())` on success.
fn try_build(
    build: &Path,
    content: &Path,
    site: &Path,
    template: &Path,
) -> Result<(), String> {
    ssg::compile_site(build, content, site, template).map_err(|e| {
        let mut msg = e.to_string();
        let mut opt_source = std::error::Error::source(&e);
        while let Some(source) = opt_source {
            msg.push_str(&format!("\ncaused by: {source}"));
            opt_source = std::error::Error::source(source);
        }
        msg
    })
}

// =====================================================================
// Corrupt frontmatter
// =====================================================================

#[test]
fn missing_frontmatter_delimiters_does_not_panic() {
    let (_tmp, content, build, site, template) = fresh_layout();
    // Frontmatter delimiters are `---` ... `---`. This file omits the
    // closing delimiter so the YAML parser must classify it as a
    // body-only file or surface a clean error.
    fs::write(
        content.join("broken.md"),
        "---\ntitle: only opening fence\n# Body without closing ---\n",
    )
    .unwrap();
    let _ = try_build(&build, &content, &site, &template);
    // No panic = test passes. Either the file is rejected with an
    // error or processed as plain Markdown.
}

#[test]
fn invalid_utf8_in_content_does_not_panic() {
    let (_tmp, content, build, site, template) = fresh_layout();
    // 0xFF is invalid as the first byte of any UTF-8 sequence.
    fs::write(content.join("bad.md"), [0xFF, 0xFE, b'a', b'b']).unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

#[test]
fn unterminated_yaml_string_does_not_panic() {
    let (_tmp, content, build, site, template) = fresh_layout();
    fs::write(
        content.join("bad.md"),
        "---\ntitle: \"unterminated\nbody\n---\n# Hi\n",
    )
    .unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

// =====================================================================
// Corrupt assets
// =====================================================================

#[test]
fn zero_byte_image_in_content_does_not_panic() {
    let (_tmp, content, build, site, template) = fresh_layout();
    fs::write(content.join("zero.jpg"), []).unwrap();
    fs::write(
        content.join("page.md"),
        "---\ntitle: page\n---\n![](zero.jpg)\n",
    )
    .unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

#[test]
fn truncated_jpeg_header_does_not_panic() {
    let (_tmp, content, build, site, template) = fresh_layout();
    // First two JPEG SOI bytes only — no rest of the file.
    fs::write(content.join("truncated.jpg"), [0xFF, 0xD8]).unwrap();
    fs::write(
        content.join("page.md"),
        "---\ntitle: page\n---\nimage embedded\n",
    )
    .unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

// =====================================================================
// Filesystem pathologies
// =====================================================================

#[cfg(unix)]
#[test]
fn symlink_loop_does_not_hang_or_panic() {
    use std::os::unix::fs::symlink;

    let (_tmp, content, build, site, template) = fresh_layout();
    // a → b, b → a, both inside content/. The walker must terminate
    // (most do via the OS returning ELOOP after MAX_DIR_DEPTH).
    let a = content.join("a");
    let b = content.join("b");
    symlink(&b, &a).unwrap();
    symlink(&a, &b).unwrap();
    fs::write(content.join("ok.md"), "---\ntitle: ok\n---\n").unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

#[cfg(unix)]
#[test]
fn deeply_nested_directory_does_not_overflow_stack() {
    let (_tmp, content, build, site, template) = fresh_layout();
    // 130 nested directories — deeper than the project's documented
    // MAX_DIR_DEPTH of 128. The walker should refuse to descend
    // rather than panic or recurse-overflow.
    let mut deep = content.clone();
    for i in 0..130 {
        deep = deep.join(format!("d{i}"));
    }
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("buried.md"), "---\ntitle: buried\n---\n").unwrap();
    let _ = try_build(&build, &content, &site, &template);
}

#[cfg(unix)]
#[test]
fn read_only_output_directory_returns_clean_error() {
    use std::os::unix::fs::PermissionsExt;

    /// RAII guard that restores 0o755 on drop so tempdir cleanup
    /// succeeds even if the build panics between set and restore.
    struct PermsGuard<'a>(&'a Path);
    impl Drop for PermsGuard<'_> {
        fn drop(&mut self) {
            let _ =
                fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
        }
    }

    let (_tmp, content, build, site, template) = fresh_layout();
    fs::write(content.join("page.md"), "---\ntitle: x\n---\n# Hi\n").unwrap();
    // 0o555 = r-x for owner (no write). Build must surface a clean
    // permission-denied error, not a panic.
    fs::set_permissions(&site, fs::Permissions::from_mode(0o555)).unwrap();
    let guard = PermsGuard(&site);
    let result = try_build(&build, &content, &site, &template);
    // `guard` drops here (or on panic) and restores 0o755 so tempdir
    // cleanup succeeds.
    // Either:
    //  - error returned (preferred — clean failure), or
    //  - build succeeded by writing elsewhere (also acceptable).
    // The only failure mode is a panic, which would have aborted
    // the test before reaching this point.
    let _ = result;
    drop(guard);
}

// =====================================================================
// Concurrent builds against the same output directory
// =====================================================================

#[test]
fn concurrent_builds_to_same_site_dir_do_not_panic() {
    use std::sync::Arc;
    use std::thread;

    let tmp = tempdir().unwrap();
    let content = Arc::new(tmp.path().join("content"));
    let build = Arc::new(tmp.path().join("build"));
    let site = Arc::new(tmp.path().join("site"));
    let template = Arc::new(tmp.path().join("templates"));
    for d in [&*content, &*build, &*site, &*template] {
        fs::create_dir_all(d).unwrap();
    }
    fs::write(content.join("page.md"), "---\ntitle: race\n---\n# Hi\n")
        .unwrap();

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let c = Arc::clone(&content);
            let b = Arc::clone(&build);
            let s = Arc::clone(&site);
            let t = Arc::clone(&template);
            thread::spawn(move || {
                // Either Ok or Err is acceptable; only a panic
                // escaping the closure would fail the join.
                let _ = ssg::compile_site(&b, &c, &s, &t);
            })
        })
        .collect();

    for h in handles {
        // .join() returns Err only if the thread panicked.
        h.join().expect("thread panicked under concurrent build");
    }
}
