#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![allow(unsafe_code)]
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

pub mod isr;
pub mod rpc;

pub use isr::{render_page_isr, render_page_isr_impl};
pub use rpc::{rpc_dispatch, rpc_dispatch_impl, RpcResponse};

use wasm_bindgen::prelude::*;

/// Compile Markdown to HTML.
///
/// Supports GitHub Flavored Markdown: tables, strikethrough, task lists.
///
/// # Examples
///
/// ```
/// let html = ssg_wasm::compile_markdown("# Hi");
/// assert!(html.contains("<h1>Hi</h1>"));
/// ```
#[wasm_bindgen]
pub fn compile_markdown(input: &str) -> String {
    ssg_core::compile_markdown(input)
}

/// Parse frontmatter and compile a complete page.
///
/// Returns a JSON object: `{ "frontmatter": {...}, "html": "..." }`
///
/// # Errors
/// Returns a JS error string if the markdown frontmatter cannot be
/// parsed.
///
/// # Examples
///
/// `compile_page` round-trips through `JsValue` and so cannot be
/// invoked in a native doctest. Use [`ssg_core::compile_page`] for the
/// native API; this entry point is wired up for JavaScript callers:
///
/// ```no_run
/// # use wasm_bindgen::JsValue;
/// let result: Result<JsValue, JsValue> =
///     ssg_wasm::compile_page("---\ntitle: Hi\n---\n# Body");
/// assert!(result.is_ok());
/// ```
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
///
/// # Examples
///
/// ```
/// let text = ssg_wasm::strip_html("<p>Hello <b>world</b></p>");
/// assert_eq!(text, "Hello world");
/// ```
#[wasm_bindgen]
pub fn strip_html(input: &str) -> String {
    ssg_core::strip_html_tags(input)
}
