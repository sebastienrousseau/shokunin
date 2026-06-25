// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Parser-driven helpers for HTML `<head>` manipulation.
//!
//! Replaces the previous `str::find` / `str::replace` patterns scattered
//! across the SEO, `og_image`, `llm`, `ai`, `i18n`, `highlight`, `sbom`,
//! `atom`, `json_feed`, and `jsonld` plugins (issues #538, #539, #540).
//!
//! Three operations are exposed:
//!
//! - [`inject_before_head_close`] — append a payload immediately before
//!   the real `</head>` end-tag, never inside a `<pre>` / `<code>` /
//!   comment / `<script>` template that happens to contain the literal
//!   `</head>` string.
//! - [`extract_head_meta`] — single `lol_html` walk that pulls the
//!   document `<title>`, the `<html lang>` attribute, and the existing
//!   `<link rel="canonical">` href in one pass.
//! - [`remove_canonical_links`] — strip every
//!   `<link rel~="canonical">` in `<head>` without disturbing
//!   non-canonical `<link>` elements or matching literals embedded in
//!   `<pre>` blocks.
//!
//! All three guard against the standard `str::find` failure modes:
//! comments containing the literal markup, `<pre>` / `<code>` samples
//! that quote the same tag, attribute order and quoting variants, and
//! multiple matches in body content.

use crate::util::html_rewriter::rewrite_html;
use lol_html::html_content::ContentType;
use lol_html::{element, end_tag, text};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Metadata extracted from the document `<head>` in a single parser
/// walk.
///
/// All fields default to the empty string when the corresponding markup
/// is missing — matching the historical `str::find`-based helpers in
/// `src/plugins/seo/helpers.rs` so call sites can swap in without
/// branch-equivalence regressions.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HeadMeta {
    /// Plain-text content of the first `<title>` element, with inner
    /// tags stripped and whitespace collapsed. Empty if no `<title>`.
    pub title: String,
    /// Value of `<html lang="…">`. Empty if absent.
    pub lang: String,
    /// `href` of the first `<link rel="canonical">`. Empty if absent.
    pub canonical: String,
}

/// Inserts `payload` immediately before the real `</head>` end-tag.
///
/// Returns the input unchanged when no `<head>` element exists or when
/// the underlying `lol_html` rewrite fails (allocation exhaustion is the
/// only documented failure mode). `payload` is treated as raw HTML so
/// callers retain control over tag construction; callers that need
/// escaping must do so themselves.
///
/// Idempotency is the **caller's** responsibility — this helper inserts
/// unconditionally and will append a second payload on a second call.
#[must_use]
pub fn inject_before_head_close(html: &str, payload: &str) -> String {
    if payload.is_empty() {
        return html.to_string();
    }

    let payload_owned = payload.to_string();
    let injected = Rc::new(Cell::new(false));
    let injected_cb = Rc::clone(&injected);

    let handler = element!("head", move |el| {
        let pl = payload_owned.clone();
        let cb = Rc::clone(&injected_cb);
        let _ = el.on_end_tag(end_tag!(move |end| {
            end.before(&pl, ContentType::Html);
            cb.set(true);
            Ok(())
        }));
        Ok(())
    });

    let out =
        rewrite_html(html, vec![handler]).unwrap_or_else(|_| html.to_string());

    if injected.get() {
        out
    } else {
        html.to_string()
    }
}

/// Extracts `{title, lang, canonical}` from `html` in a single
/// `lol_html` pass.
///
/// - `title` is the text content of the first `<title>` element with
///   inner tags stripped (matching the historical
///   `extract_title` behaviour). Whitespace is collapsed and trimmed;
///   returns the empty string when the element is missing or its text
///   content is whitespace-only.
/// - `lang` is the `lang` attribute on the root `<html>` element.
///   `<html lang="…">` inside a `<pre>` block is ignored because
///   `lol_html` only matches the real element.
/// - `canonical` is the `href` attribute of the first
///   `<link rel~="canonical">` in `<head>`. Selector matches the
///   space-separated token set so `rel="canonical other-token"` is
///   detected, while quoting style is irrelevant (the parser normalises
///   it).
#[must_use]
pub fn extract_head_meta(html: &str) -> HeadMeta {
    let title_buf: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let title_done: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let lang: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let canonical: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let title_buf_text = Rc::clone(&title_buf);
    let title_done_text = Rc::clone(&title_done);
    let title_text_handler = text!("title", move |t| {
        if !title_done_text.get() {
            title_buf_text.borrow_mut().push_str(t.as_str());
            if t.last_in_text_node() {
                title_done_text.set(true);
            }
        }
        Ok(())
    });

    let lang_cb = Rc::clone(&lang);
    let html_handler = element!("html", move |el| {
        if let Some(value) = el.get_attribute("lang") {
            *lang_cb.borrow_mut() = value;
        }
        Ok(())
    });

    let canonical_cb = Rc::clone(&canonical);
    let canonical_handler = element!("link[rel~=\"canonical\" i]", move |el| {
        if canonical_cb.borrow().is_empty() {
            if let Some(href) = el.get_attribute("href") {
                *canonical_cb.borrow_mut() = href;
            }
        }
        Ok(())
    });

    let _ = rewrite_html(
        html,
        vec![title_text_handler, html_handler, canonical_handler],
    );

    let raw_title = title_buf.borrow().clone();
    let title = collapse_ws(strip_tags(raw_title.trim()).trim());
    let lang_val = lang.borrow().clone();
    let canonical_val = canonical.borrow().clone();

    HeadMeta {
        title,
        lang: lang_val,
        canonical: canonical_val,
    }
}

/// Removes every `<link rel~="canonical">` element from the document.
///
/// Uses the `link[rel~="canonical" i]` selector so any attribute order
/// or quoting is handled by the parser, including `rel="canonical
/// other-token"` (the space-separated token set match). Literal
/// `<link rel="canonical">` text embedded inside a `<pre>` / `<code>`
/// block or an HTML comment is left untouched because `lol_html` only
/// dispatches on real elements.
#[must_use]
pub fn remove_canonical_links(html: &str) -> String {
    let handler = element!("link[rel~=\"canonical\" i]", |el| {
        el.remove();
        Ok(())
    });
    rewrite_html(html, vec![handler]).unwrap_or_else(|_| html.to_string())
}

/// Replaces every existing `<link rel~="canonical">` with a single
/// `payload` injected just before `</head>`, in one parser pass.
///
/// Equivalent to `inject_before_head_close(&remove_canonical_links(html),
/// payload)` but executed in a single `lol_html` walk so that the
/// surrounding whitespace residue from the removal is consumed at the
/// same time as the new link is inserted — keeping the canonical
/// plugin's `transform_html` byte-stable across repeated invocations
/// (idempotency requirement).
#[must_use]
pub fn replace_canonical_link(html: &str, payload: &str) -> String {
    let payload_owned = payload.to_string();
    let injected = Rc::new(Cell::new(false));
    let injected_cb = Rc::clone(&injected);

    let canonical_handler = element!("link[rel~=\"canonical\" i]", |el| {
        el.remove();
        Ok(())
    });

    let head_handler = element!("head", move |el| {
        let pl = payload_owned.clone();
        let cb = Rc::clone(&injected_cb);
        let _ = el.on_end_tag(end_tag!(move |end| {
            end.before(&pl, ContentType::Html);
            cb.set(true);
            Ok(())
        }));
        Ok(())
    });

    let out = rewrite_html(html, vec![canonical_handler, head_handler])
        .unwrap_or_else(|_| html.to_string());

    if injected.get() {
        out
    } else {
        html.to_string()
    }
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ── inject_before_head_close ────────────────────────────────────

    #[test]
    fn inject_at_real_head_close() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let out = inject_before_head_close(html, "<meta name=\"x\">");
        assert!(out.contains("<meta name=\"x\"></head>"));
        assert_eq!(out.matches("<meta name=\"x\">").count(), 1);
    }

    #[test]
    fn inject_skips_pre_block_literal() {
        let html = "<html><head><title>T</title></head>\
                    <body><pre>&lt;/head&gt;</pre></body></html>";
        let out = inject_before_head_close(html, "<meta name=\"x\">");
        assert_eq!(out.matches("<meta name=\"x\">").count(), 1);
        assert!(out.contains("<pre>&lt;/head&gt;</pre>"));
    }

    #[test]
    fn inject_skips_comment_literal() {
        let html =
            "<html><head><title>T</title></head><body><!-- </head> --></body></html>";
        let out = inject_before_head_close(html, "<meta name=\"x\">");
        assert_eq!(out.matches("<meta name=\"x\">").count(), 1);
        assert!(out.contains("<!-- </head> -->"));
    }

    #[test]
    fn inject_returns_input_when_no_head() {
        let html = "<html><body>no head</body></html>";
        let out = inject_before_head_close(html, "<meta>");
        assert_eq!(out, html);
    }

    #[test]
    fn inject_empty_payload_returns_input() {
        let html = "<html><head></head></html>";
        let out = inject_before_head_close(html, "");
        assert_eq!(out, html);
    }

    // ── extract_head_meta ───────────────────────────────────────────

    #[test]
    fn extract_title_from_real_title_not_comment() {
        let html = "<html><head><!-- <title>Old</title> --><title>Real</title></head></html>";
        let meta = extract_head_meta(html);
        assert_eq!(meta.title, "Real");
    }

    #[test]
    fn extract_lang_from_html_not_pre() {
        let html = "<html lang=\"en-GB\"><head></head>\
                    <body><pre>&lt;html lang=\"fr\"&gt;</pre></body></html>";
        let meta = extract_head_meta(html);
        assert_eq!(meta.lang, "en-GB");
    }

    #[test]
    fn extract_canonical_returns_href() {
        let html = r#"<html><head><link rel="canonical" href="https://x"></head></html>"#;
        let meta = extract_head_meta(html);
        assert_eq!(meta.canonical, "https://x");
    }

    #[test]
    fn extract_returns_defaults_when_absent() {
        let html = "<html><head></head><body></body></html>";
        let meta = extract_head_meta(html);
        assert!(meta.title.is_empty());
        assert!(meta.lang.is_empty());
        assert!(meta.canonical.is_empty());
    }

    #[test]
    fn extract_collapses_title_whitespace() {
        let html = "<html><head><title>  Hello   World  </title></head></html>";
        let meta = extract_head_meta(html);
        assert_eq!(meta.title, "Hello World");
    }

    // ── remove_canonical_links ──────────────────────────────────────

    #[test]
    fn remove_double_quoted_canonical() {
        let html = r#"<head><link rel="canonical" href="/old"><title>x</title></head>"#;
        let out = remove_canonical_links(html);
        assert!(!out.contains("rel=\"canonical\""));
        assert!(out.contains("<title>x</title>"));
    }

    #[test]
    fn remove_single_quoted_canonical() {
        let html = "<head><link rel='canonical' href='/old'></head>";
        let out = remove_canonical_links(html);
        assert!(!out.contains("canonical"));
    }

    #[test]
    fn remove_unquoted_canonical() {
        let html = "<head><link rel=canonical href=/old></head>";
        let out = remove_canonical_links(html);
        assert!(!out.contains("canonical"));
    }

    #[test]
    fn remove_keeps_non_canonical_link() {
        let html = r#"<head><link rel="stylesheet" href="/x.css"></head>"#;
        let out = remove_canonical_links(html);
        assert_eq!(out, html);
    }

    #[test]
    fn remove_multiple_canonicals() {
        let html = r#"<head><link rel="canonical" href="/a"><link rel="canonical" href="/b"></head>"#;
        let out = remove_canonical_links(html);
        assert!(!out.contains("canonical"));
    }

    #[test]
    fn remove_leaves_pre_literal_untouched() {
        let html = "<html><head></head>\
                    <body><pre>&lt;link rel=\"canonical\"&gt;</pre></body></html>";
        let out = remove_canonical_links(html);
        assert!(out.contains("<pre>&lt;link rel=\"canonical\"&gt;</pre>"));
    }
}
