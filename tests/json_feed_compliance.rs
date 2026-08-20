// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration test: validate JSON Feed 1.1 spec compliance against the
//! `blog` example output.
//!
//! Builds (or reuses) `examples/blog/public/feed.json`, then asserts:
//! - every required top-level field per the JSON Feed 1.1 spec is present
//! - every required per-item field is present
//! - `version` matches `https://jsonfeed.org/version/1.1`
//! - no unexpected top-level or per-item fields appear outside the spec's
//!   `_<extension>` (underscore-prefixed) namespace
//! - the per-page HTML `<head>` injects the matching
//!   `<link rel="alternate" type="application/feed+json">` (AC4)
//!
//! Resolves issue #523, AC6.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::Duration,
};

/// Serialises blog-example builds across tests to avoid duplicate spawns
/// fighting for the same dev-server port.
fn build_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Set of top-level fields allowed by JSON Feed 1.1.
/// <https://jsonfeed.org/version/1.1>
const ALLOWED_TOP_LEVEL_FIELDS: &[&str] = &[
    "version",
    "title",
    "home_page_url",
    "feed_url",
    "description",
    "user_comment",
    "next_url",
    "icon",
    "favicon",
    "authors",
    "language",
    "expired",
    "hubs",
    "items",
];

/// Set of fields allowed on each item in JSON Feed 1.1.
const ALLOWED_ITEM_FIELDS: &[&str] = &[
    "id",
    "url",
    "external_url",
    "title",
    "content_html",
    "content_text",
    "summary",
    "image",
    "banner_image",
    "date_published",
    "date_modified",
    "authors",
    "tags",
    "language",
    "attachments",
];

/// Workspace root (directory containing the top-level `Cargo.toml`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Run `cargo run --quiet --example blog` with a hard timeout, killing
/// it once enough time has elapsed for the build to complete (the
/// example also boots a dev server that would block forever).
fn run_blog_example(timeout: Duration) {
    let mut child = Command::new("cargo")
        .current_dir(workspace_root())
        .args(["run", "--quiet", "--example", "blog"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn cargo for blog example");

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(e) => panic!("error waiting on blog example: {e}"),
        }
    }
}

/// Ensure the blog example output exists by triggering a build if
/// `feed.json` is missing. Serialised across parallel tests so the dev
/// server port (and the build artifacts) don't race.
fn ensure_blog_feed() -> PathBuf {
    let _guard = build_lock().lock().unwrap_or_else(|p| p.into_inner());
    let root = workspace_root();
    let feed = root
        .join("examples")
        .join("blog")
        .join("public")
        .join("feed.json");
    if !feed.exists() {
        run_blog_example(Duration::from_secs(120));
    }
    assert!(
        feed.exists(),
        "blog example did not produce feed.json at {}",
        feed.display()
    );
    feed
}

/// RFC 3339 sanity check: starts with `YYYY-MM-DDTHH:MM:SS`.
const fn looks_like_rfc3339(s: &str) -> bool {
    if s.len() < 19 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
}

#[test]
fn blog_feed_json_is_valid_json_feed_1_1() {
    let feed_path = ensure_blog_feed();
    let raw = fs::read_to_string(&feed_path).expect("read feed.json");
    let value: Value =
        serde_json::from_str(&raw).expect("feed.json should parse as JSON");

    // AC2: required top-level fields
    assert_eq!(
        value["version"], "https://jsonfeed.org/version/1.1",
        "version must be the JSON Feed 1.1 URL"
    );
    assert!(value["title"].is_string(), "title must be a string");
    assert!(
        value["home_page_url"].is_string(),
        "home_page_url must be a string"
    );
    assert!(value["feed_url"].is_string(), "feed_url must be a string");

    let items = value["items"].as_array().expect("items must be an array");
    assert!(!items.is_empty(), "blog example must emit ≥1 items");

    // AC5: top-level language should be the site's default locale
    assert!(value["language"].is_string(), "top-level language required");

    // Top-level unknown fields (outside spec / underscore namespace)
    let obj = value
        .as_object()
        .expect("top-level value must be an object");
    for key in obj.keys() {
        if key.starts_with('_') {
            continue; // extension namespace per spec
        }
        assert!(
            ALLOWED_TOP_LEVEL_FIELDS.contains(&key.as_str()),
            "unexpected top-level field `{key}` (not in JSON Feed 1.1 spec)"
        );
    }
}

#[test]
fn blog_feed_json_items_have_required_fields() {
    let feed_path = ensure_blog_feed();
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&feed_path).expect("read feed.json"),
    )
    .expect("parse feed.json");

    let items = value["items"].as_array().expect("items must be an array");

    for (i, item) in items.iter().enumerate() {
        let obj = item
            .as_object()
            .unwrap_or_else(|| panic!("item {i} must be an object: {item}"));

        // AC3: every required per-item field
        assert!(
            obj.get("id").is_some_and(Value::is_string),
            "item {i} missing string id"
        );
        assert!(
            obj.get("url").is_some_and(Value::is_string),
            "item {i} missing string url"
        );
        assert!(
            obj.get("title").is_some_and(Value::is_string),
            "item {i} missing string title"
        );

        let has_html = obj.get("content_html").is_some_and(Value::is_string);
        let has_text = obj.get("content_text").is_some_and(Value::is_string);
        assert!(
            has_html || has_text,
            "item {i} must have content_html OR content_text"
        );

        let date_pub = obj
            .get("date_published")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("item {i} missing date_published"));
        assert!(
            looks_like_rfc3339(date_pub),
            "item {i} date_published `{date_pub}` is not RFC 3339"
        );

        let date_mod = obj
            .get("date_modified")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("item {i} missing date_modified"));
        assert!(
            looks_like_rfc3339(date_mod),
            "item {i} date_modified `{date_mod}` is not RFC 3339"
        );

        let authors = obj
            .get("authors")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("item {i} missing authors[]"));
        assert!(!authors.is_empty(), "item {i} authors[] must have ≥1 entry");
        for (j, author) in authors.iter().enumerate() {
            assert!(
                author.get("name").is_some_and(Value::is_string),
                "item {i} author {j} missing name"
            );
        }

        assert!(
            obj.get("tags").is_some_and(Value::is_array),
            "item {i} missing tags[]"
        );

        // Reject unexpected per-item fields outside the `_ext` namespace
        for key in obj.keys() {
            if key.starts_with('_') {
                continue;
            }
            assert!(
                ALLOWED_ITEM_FIELDS.contains(&key.as_str()),
                "item {i} has unexpected field `{key}` not in JSON Feed 1.1"
            );
        }
    }
}

#[test]
fn blog_pages_inject_json_feed_alternate_link() {
    let feed_path = ensure_blog_feed();
    let public = feed_path.parent().expect("feed.json must have a parent");
    let index = public.join("index.html");
    assert!(
        index.exists(),
        "index.html must exist at {}",
        index.display()
    );

    let html = fs::read_to_string(&index).expect("read index.html");
    assert!(
        html.contains("application/feed+json"),
        "AC4: index.html must contain <link rel=alternate type=application/feed+json>"
    );
    assert!(
        html.contains("/feed.json"),
        "AC4: index.html alternate link must reference /feed.json"
    );
}
