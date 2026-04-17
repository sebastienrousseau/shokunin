#![forbid(unsafe_code)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ssg-wasm — WebAssembly bindings for SSG
//!
//! Exposes `ssg-core` functions via `wasm-bindgen` for use in browsers,
//! Cloudflare Workers, Deno Deploy, and other WASM runtimes.
//!
//! ## Usage (JavaScript)
//!
//! ```javascript
//! import init, { compile_markdown, compile_page } from './ssg_wasm.js';
//!
//! await init();
//!
//! const html = compile_markdown("# Hello\n\nWorld");
//! console.log(html); // <h1>Hello</h1>\n<p>World</p>
//!
//! const result = compile_page("---\ntitle: Test\n---\n# Body");
//! console.log(result); // { frontmatter: { title: "Test" }, html: "<h1>Body</h1>" }
//! ```

use wasm_bindgen::prelude::*;

/// Compile Markdown to HTML.
///
/// Supports GitHub Flavored Markdown: tables, strikethrough, task lists.
#[wasm_bindgen]
pub fn compile_markdown(input: &str) -> String {
    ssg_core::compile_markdown(input)
}

/// Parse frontmatter and compile a complete page.
///
/// Returns a JSON object: `{ "frontmatter": {...}, "html": "..." }`
#[wasm_bindgen]
pub fn compile_page(input: &str) -> Result<JsValue, JsValue> {
    let (frontmatter, html) = ssg_core::compile_page(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = serde_json::json!({
        "frontmatter": frontmatter,
        "html": html,
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Strip HTML tags from a string.
#[wasm_bindgen]
pub fn strip_html(input: &str) -> String {
    ssg_core::strip_html_tags(input)
}
