// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use ssg_wasm::{compile_markdown, compile_page, strip_html};

// ---------------------------------------------------------------------------
// compile_markdown
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn compile_markdown_heading() {
    let html = compile_markdown("# Hello");
    assert!(html.contains("<h1>"), "expected <h1> tag in output: {html}");
}

#[wasm_bindgen_test]
fn compile_markdown_empty() {
    let html = compile_markdown("");
    assert!(
        html.trim().is_empty(),
        "expected empty output for empty input, got: {html}"
    );
}

#[wasm_bindgen_test]
fn compile_markdown_unicode() {
    let html = compile_markdown("# 日本語テスト\n\nこんにちは");
    assert!(html.contains("<h1>"), "expected <h1> tag: {html}");
    assert!(
        html.contains("日本語テスト"),
        "expected Japanese heading text: {html}"
    );
    assert!(
        html.contains("こんにちは"),
        "expected Japanese body text: {html}"
    );
}

#[wasm_bindgen_test]
fn compile_markdown_gfm_table() {
    let md = "| a | b |\n|---|---|\n| 1 | 2 |";
    let html = compile_markdown(md);
    assert!(
        html.contains("<table>"),
        "expected <table> tag for GFM table: {html}"
    );
}

// ---------------------------------------------------------------------------
// compile_page
// ---------------------------------------------------------------------------

/// Helper: unwrap a `Result<JsValue, JsValue>` with a descriptive panic.
fn unwrap_page(result: Result<JsValue, JsValue>) -> JsValue {
    result.unwrap_or_else(|err| {
        let msg = err.as_string().unwrap_or_else(|| format!("{err:?}"));
        panic!("compile_page returned Err: {msg}");
    })
}

/// Serialise a `JsValue` to a JSON string for simple assertion checks.
fn js_to_json(val: &JsValue) -> String {
    js_sys::JSON::stringify(val)
        .expect("JSON.stringify failed")
        .into()
}

#[wasm_bindgen_test]
fn compile_page_yaml() {
    let input = "---\ntitle: Test\n---\n# Body";
    let val = unwrap_page(compile_page(input));
    let json = js_to_json(&val);
    assert!(
        json.contains("title"),
        "expected 'title' key in JSON output: {json}"
    );
}

#[wasm_bindgen_test]
fn compile_page_toml() {
    let input = "+++\ntitle = \"Test\"\n+++\n# Body";
    let val = unwrap_page(compile_page(input));
    let json = js_to_json(&val);
    assert!(
        json.contains("title"),
        "expected 'title' key in TOML frontmatter output: {json}"
    );
}

#[wasm_bindgen_test]
fn compile_page_no_frontmatter() {
    let input = "Just text";
    let val = unwrap_page(compile_page(input));
    // Should succeed — frontmatter is optional.
    let json = js_to_json(&val);
    assert!(
        json.contains("html"),
        "expected 'html' key in output: {json}"
    );
}

#[wasm_bindgen_test]
fn compile_page_malformed() {
    // Must not panic regardless of whether it returns Ok or Err.
    let input = "---\n[invalid yaml\n---\nbody";
    let _result = compile_page(input);
}

// ---------------------------------------------------------------------------
// strip_html
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn strip_html_basic() {
    let out = strip_html("<p>Hello <b>world</b></p>");
    assert_eq!(out, "Hello world");
}

#[wasm_bindgen_test]
fn strip_html_empty() {
    let out = strip_html("");
    assert_eq!(out, "");
}

#[wasm_bindgen_test]
fn strip_html_no_tags() {
    let out = strip_html("plain text");
    assert_eq!(out, "plain text");
}

#[wasm_bindgen_test]
fn strip_html_unicode() {
    let out = strip_html("<div>🦀 Rust</div>");
    assert!(
        out.contains("🦀 Rust"),
        "expected '🦀 Rust' in output: {out}"
    );
}
