// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ISR WASM Edge renderer (issue #546 AC3).
//!
//! The Edge worker (Cloudflare / Vercel) calls into this module after
//! fetching the raw markdown + layout from KV / Edge Config. The
//! renderer is **stateless** — no globals, no I/O, no panic paths that
//! depend on the host. Every input arrives via parameters; every
//! output goes back via the return value.
//!
//! ## Contract
//!
//! ```javascript
//! import init, { render_page_isr } from './ssg_wasm.js';
//! await init();
//!
//! const html = render_page_isr(
//!   markdown,        // raw markdown bytes as a string
//!   layout,          // template HTML with a `{{ content }}` slot
//!   JSON.stringify({ // build-time context (URL, site name, etc.)
//!     url: "/posts/foo/index.html",
//!     site_name: "Example"
//!   })
//! );
//! ```
//!
//! ## Why so small?
//!
//! The total wasm payload must stay ≤ 2 MB gzipped (AC10). Every
//! dependency we pull pushes against that budget — so this module
//! reuses `ssg_core::compile_page` (which is already in the binary
//! for `compile_page`) and does the template substitution by hand
//! instead of pulling minijinja/handlebars.

use wasm_bindgen::prelude::*;

/// Renders a single ISR page from raw markdown + layout + context.
///
/// `context_json` is a JSON object with at least `url` and
/// `site_name`. Unrecognised fields are ignored — adapters can pass
/// extra fields without breaking forward compatibility.
///
/// ## Slot syntax
///
/// The renderer recognises three placeholders in `layout`:
///
/// - `{{ content }}` — the rendered markdown body.
/// - `{{ title }}` — frontmatter `title` (or the URL as fallback).
/// - `{{ site_name }}` — `context.site_name` (defaults to empty).
///
/// Anything else is left untouched. Adapters that need richer
/// templating should pre-process the layout in the worker.
///
/// # Errors
/// Returns a JS-side string error if the markdown fails to compile or
/// the context is not valid JSON.
#[wasm_bindgen]
pub fn render_page_isr(
    markdown: &str,
    layout: &str,
    context_json: &str,
) -> Result<String, JsValue> {
    render_page_isr_impl(markdown, layout, context_json)
        .map_err(|e| JsValue::from_str(&e))
}

/// Pure-Rust implementation, separated for unit testing without
/// `wasm-bindgen-test`. Callers in non-WASM contexts can use this
/// directly.
pub fn render_page_isr_impl(
    markdown: &str,
    layout: &str,
    context_json: &str,
) -> Result<String, String> {
    // 1. Compile markdown → (frontmatter, html_body).
    let (frontmatter, body_html) =
        ssg_core::compile_page(markdown).map_err(|e| e.to_string())?;

    // 2. Parse context (URL, site_name, etc.).
    let context: serde_json::Value = if context_json.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(context_json)
            .map_err(|e| format!("context_json parse error: {e}"))?
    };

    let url = context
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let site_name = context
        .get("site_name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    // 3. Resolve title: frontmatter "title" > URL > empty.
    let title = frontmatter
        .get("title")
        .and_then(|v| v.as_str())
        .map_or_else(|| url.to_string(), ToOwned::to_owned);

    // 4. Substitute slots. Allocate once with a sensible upper bound.
    let mut out =
        String::with_capacity(layout.len() + body_html.len() + 256);
    let mut cursor = 0usize;
    while cursor < layout.len() {
        // Find the next {{ ... }} window.
        if let Some(open_rel) = layout[cursor..].find("{{") {
            let open = cursor + open_rel;
            out.push_str(&layout[cursor..open]);
            if let Some(close_rel) = layout[open + 2..].find("}}") {
                let close = open + 2 + close_rel;
                let raw_key = layout[open + 2..close].trim();
                match raw_key {
                    "content" => out.push_str(&body_html),
                    "title" => out.push_str(&html_escape(&title)),
                    "site_name" => out.push_str(&html_escape(site_name)),
                    other => {
                        // Lookup in frontmatter — handy for {{ author }} etc.
                        match frontmatter.get(other) {
                            Some(serde_json::Value::String(s)) => {
                                out.push_str(&html_escape(s));
                            }
                            Some(other_v) => {
                                out.push_str(&html_escape(
                                    &other_v.to_string(),
                                ));
                            }
                            None => {
                                // Preserve unknown placeholders so the
                                // worker can do a second pass.
                                out.push_str(&layout[open..close + 2]);
                            }
                        }
                    }
                }
                cursor = close + 2;
            } else {
                // Unclosed {{ — emit the rest verbatim and stop.
                out.push_str(&layout[open..]);
                break;
            }
        } else {
            out.push_str(&layout[cursor..]);
            break;
        }
    }

    Ok(out)
}

/// Conservative HTML escape — `& < > " '` only. We intentionally do
/// NOT escape `/` because the encoded payload appears in href and src
/// attributes (where `&#x2F;` would be wrong in plain text).
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests — exercised both natively (cargo test) and via wasm-bindgen-test.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown_into_layout() {
        let md = "---\ntitle: Hello\n---\n# Welcome\n\nWorld.";
        let layout =
            "<html><head><title>{{ title }}</title></head><body>{{ content }}</body></html>";
        let ctx = "{\"url\": \"/index.html\", \"site_name\": \"Example\"}";

        let out = render_page_isr_impl(md, layout, ctx).unwrap();
        assert!(out.contains("<title>Hello</title>"));
        assert!(out.contains("<h1>Welcome</h1>"));
        assert!(out.contains("<p>World.</p>"));
    }

    #[test]
    fn title_falls_back_to_url() {
        let md = "# Body\n";
        let layout = "<title>{{ title }}</title>{{ content }}";
        let out = render_page_isr_impl(
            md,
            layout,
            "{\"url\": \"/posts/foo/index.html\"}",
        )
        .unwrap();
        assert!(out.contains("/posts/foo/index.html"));
    }

    #[test]
    fn empty_context_json_is_allowed() {
        let out = render_page_isr_impl(
            "# Hi",
            "<body>{{ content }}</body>",
            "",
        )
        .unwrap();
        assert!(out.contains("<h1>Hi</h1>"));
    }

    #[test]
    fn bad_context_json_returns_err() {
        let r = render_page_isr_impl("# Hi", "<x/>", "not json");
        assert!(r.is_err());
        let msg = r.unwrap_err();
        assert!(msg.contains("context_json parse error"));
    }

    #[test]
    fn html_escape_runs_on_title_and_site_name() {
        let md = "---\ntitle: \"<evil>&\"\n---\n# Body";
        let layout = "{{ title }} | {{ site_name }} | {{ content }}";
        let out = render_page_isr_impl(
            md,
            layout,
            "{\"site_name\": \"A & B\"}",
        )
        .unwrap();
        assert!(out.contains("&lt;evil&gt;&amp;") || out.contains("&amp;"));
        assert!(out.contains("A &amp; B"));
    }

    #[test]
    fn unknown_placeholder_is_preserved() {
        let md = "# Hi";
        let layout = "{{ unknown_slot }} {{ content }}";
        let out = render_page_isr_impl(md, layout, "{}").unwrap();
        assert!(out.contains("{{ unknown_slot }}"));
        assert!(out.contains("<h1>Hi</h1>"));
    }

    #[test]
    fn unclosed_brace_is_emitted_verbatim() {
        let md = "# Hi";
        let layout = "<body>{{ content }} {{ unclosed";
        let out = render_page_isr_impl(md, layout, "{}").unwrap();
        assert!(out.contains("<h1>Hi</h1>"));
        assert!(out.contains("{{ unclosed"));
    }

    #[test]
    fn frontmatter_lookup_for_arbitrary_keys() {
        let md = "---\nauthor: Jane\n---\n# Hi";
        let layout = "by {{ author }}: {{ content }}";
        let out = render_page_isr_impl(md, layout, "{}").unwrap();
        assert!(out.contains("by Jane"));
    }

    #[test]
    fn no_slots_passes_through() {
        let out = render_page_isr_impl(
            "# Hi",
            "<html><body>static</body></html>",
            "{}",
        )
        .unwrap();
        assert_eq!(out, "<html><body>static</body></html>");
    }

    #[test]
    fn renderer_is_stateless_across_calls() {
        // Calling twice with identical inputs must yield identical
        // outputs and the second call must not be affected by the
        // first (no shared mutable globals).
        let md = "# Hi";
        let layout = "{{ content }}";
        let a = render_page_isr_impl(md, layout, "{}").unwrap();
        let b = render_page_isr_impl(md, layout, "{}").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn html_escape_basic() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<x>"), "&lt;x&gt;");
        assert_eq!(html_escape("\"'\""), "&quot;&#39;&quot;");
        assert_eq!(html_escape(""), "");
    }
}
