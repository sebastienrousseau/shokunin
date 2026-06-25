// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "templates")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Regression tests for issue #536 — YAML data files loaded into
//! template globals are parsed with a real YAML parser, not
//! `serde_json::from_str`.

use std::fs;
use std::sync::{Mutex, OnceLock};

use ssg::template_engine::TemplateEngine;
use tempfile::tempdir;

/// Guards the global `log::Log` install + the `log::warn!` interception
/// so the malformed-YAML test can observe the warning without racing
/// other tests that also touch the logger.
fn log_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn yaml_data_file_indent_list_is_parsed() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();

    fs::write(
        data.join("nav.yml"),
        "links:\n  - home\n  - about\n  - blog\n",
    )
    .unwrap();

    let result = TemplateEngine::load_data_files(&content);
    let nav = result.get("nav").expect("nav.yml should be parsed");
    let links = nav
        .get("links")
        .and_then(|v| v.as_array())
        .expect("links should be a YAML sequence");
    assert_eq!(links.len(), 3);
    assert_eq!(links[0].as_str(), Some("home"));
    assert_eq!(links[2].as_str(), Some("blog"));
}

#[test]
fn yaml_data_file_flow_list_is_parsed() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();

    fs::write(data.join("tags.yaml"), "names: [rust, wasm, ssg]\n").unwrap();

    let result = TemplateEngine::load_data_files(&content);
    let tags = result.get("tags").expect("tags.yaml should be parsed");
    let names = tags
        .get("names")
        .and_then(|v| v.as_array())
        .expect("names should be a YAML flow sequence");
    assert_eq!(names.len(), 3);
    assert_eq!(names[1].as_str(), Some("wasm"));
}

#[test]
fn yaml_data_file_nested_map_is_parsed() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();

    fs::write(
        data.join("site.yml"),
        "author:\n  name: Jane\n  email: jane@example.com\n",
    )
    .unwrap();

    let result = TemplateEngine::load_data_files(&content);
    let site = result.get("site").expect("site.yml should be parsed");
    let author = site
        .get("author")
        .and_then(|v| v.as_object())
        .expect("author should be a YAML mapping");
    assert_eq!(author.get("name").and_then(|v| v.as_str()), Some("Jane"));
    assert_eq!(
        author.get("email").and_then(|v| v.as_str()),
        Some("jane@example.com")
    );
}

/// Captures every `log::warn!` message into a shared buffer so the
/// malformed-YAML test can assert a warning was emitted (#536 AC).
struct WarnCapture {
    buf: Mutex<Vec<String>>,
}

static CAPTURE: OnceLock<&'static WarnCapture> = OnceLock::new();

impl log::Log for WarnCapture {
    fn enabled(&self, m: &log::Metadata<'_>) -> bool {
        m.level() <= log::Level::Warn
    }
    fn log(&self, record: &log::Record<'_>) {
        if record.level() == log::Level::Warn {
            self.buf.lock().unwrap().push(format!("{}", record.args()));
        }
    }
    fn flush(&self) {}
}

fn install_capture() -> &'static WarnCapture {
    CAPTURE.get_or_init(|| {
        let leaked: &'static WarnCapture = Box::leak(Box::new(WarnCapture {
            buf: Mutex::new(Vec::new()),
        }));
        // First-installer wins; if another test already installed a
        // logger this returns Err and we silently fall back to checking
        // that the file was *not* inserted into the map.
        let _ = log::set_logger(leaked);
        log::set_max_level(log::LevelFilter::Warn);
        leaked
    })
}

#[test]
fn yaml_data_file_malformed_logs_warning_and_is_skipped() {
    let _guard = log_lock().lock().unwrap();
    let cap = install_capture();
    cap.buf.lock().unwrap().clear();

    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    fs::create_dir_all(&content).unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();

    // Intentionally malformed: unbalanced flow-list opener.
    fs::write(data.join("broken.yml"), "items: [unclosed\n").unwrap();
    fs::write(data.join("good.yaml"), "x: 1\n").unwrap();

    let result = TemplateEngine::load_data_files(&content);

    // Good file still loads, broken one is dropped.
    assert!(result.contains_key("good"));
    assert!(!result.contains_key("broken"));

    // Warning recorded when our capture is the installed logger.
    // If another test installed a logger first, we accept the
    // skip-on-failure behaviour as sufficient evidence.
    if log::logger().enabled(
        &log::Metadata::builder()
            .level(log::Level::Warn)
            .target("ssg")
            .build(),
    ) {
        let logged = cap.buf.lock().unwrap();
        let saw_warn = logged
            .iter()
            .any(|m| m.contains("broken.yml") && m.contains("parse"));
        assert!(
            saw_warn,
            "expected a parse warning for broken.yml, got: {logged:?}"
        );
    }
}
