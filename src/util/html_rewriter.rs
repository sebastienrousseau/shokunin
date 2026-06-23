// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Thin wrapper around Cloudflare's [`lol_html`] streaming HTML rewriter.
//!
//! Replaces the previous fragile `str::find` / `str::rfind` rewriting
//! that lived in the image, search, CSP, and html-fix plugins (issue
//! #525). Compared to the old approach `lol_html`:
//!
//! - **Skips HTML comments.** `<!-- <img …> -->` is byte-identical in
//!   the output rather than being half-rewritten.
//! - **Preserves character entities.** `alt="Café &amp; bar"` round-trips
//!   verbatim — no double-encoding, no entity decoding inside attribute
//!   values.
//! - **Streams.** Peak RSS stays flat regardless of input size; there is
//!   no full-document DOM materialised in memory.
//!
//! ## API
//!
//! The crate-private functions [`rewrite_html`] and
//! [`extract_text_with_filter`] cover every call site inside SSG.
//! Plugins pass closure-based [`ElementContentHandlers`] or write to a
//! `&mut String` aggregator and let the wrapper deal with feeding
//! chunks through `lol_html::HtmlRewriter`.

use crate::error::SsgError;
use lol_html::html_content::TextChunk;
use lol_html::{
    rewrite_str, ElementContentHandlers, RewriteStrSettings, Selector,
};
use std::borrow::Cow;

/// Run `lol_html` over `html` with the supplied
/// `(selector, handlers)` pairs and return the rewritten document.
///
/// Maps any `lol_html` failure (selector parse error, encoder failure,
/// memory-limit overflow) onto [`SsgError::Io`] with a synthetic
/// `<lol_html>` path so callers don't have to deal with `lol_html`'s
/// error types directly.
///
/// # Errors
///
/// Returns [`SsgError::Io`] if `lol_html` rejects any selector or
/// fails while writing output chunks.
pub fn rewrite_html<'h>(
    html: &str,
    handlers: Vec<(Cow<'h, Selector>, ElementContentHandlers<'h>)>,
) -> Result<String, SsgError> {
    let mut settings = RewriteStrSettings::new();
    for h in handlers {
        settings = settings.append_element_content_handler(h);
    }
    rewrite_str(html, settings).map_err(|e| {
        SsgError::io(
            std::io::Error::other(format!("lol_html rewrite failed: {e}")),
            "<lol_html>",
        )
    })
}

/// Extract concatenated text content from every element matching
/// `selector`, separating successive elements with a single space.
///
/// Returns the **decoded** text — `&amp;` becomes `&`, `&lt;` becomes
/// `<`, and so on — matching the on-screen rendering rather than the
/// raw HTML bytes. This is the right behaviour for search-index
/// extraction (issue #525 AC4).
///
/// # Errors
///
/// Returns [`SsgError::Io`] if `lol_html` rejects the selector or
/// fails while parsing the input.
pub fn extract_text_with_filter(
    html: &str,
    selector: &str,
) -> Result<Vec<String>, SsgError> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let parsed: Selector = selector.parse().map_err(|e| {
        SsgError::io(
            std::io::Error::other(format!(
                "invalid selector {selector:?}: {e}"
            )),
            "<lol_html>",
        )
    })?;

    // `Rc<RefCell<...>>` because `lol_html` handlers are owned closures
    // that outlive any local borrow scope.
    let buf: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let scratch: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let scratch_for_text = Rc::clone(&scratch);
    let scratch_for_end = Rc::clone(&scratch);
    let buf_for_end = Rc::clone(&buf);

    let text_handler = move |t: &mut TextChunk<'_>| {
        scratch_for_text.borrow_mut().push_str(t.as_str());
        Ok(())
    };

    let element_handler =
        move |el: &mut lol_html::html_content::Element<'_, '_>| {
            let buf_for_handler = Rc::clone(&buf_for_end);
            let scratch_for_handler = Rc::clone(&scratch_for_end);
            let _ = el.on_end_tag(Box::new(move |_end| {
                let mut s = scratch_for_handler.borrow_mut();
                let decoded = decode_html_entities(s.trim());
                let collapsed = collapse_whitespace(&decoded);
                if !collapsed.is_empty() {
                    buf_for_handler.borrow_mut().push(collapsed);
                }
                s.clear();
                Ok(())
            }));
            Ok(())
        };

    let handlers = vec![
        (
            Cow::Owned(parsed.clone()),
            ElementContentHandlers::default().text(text_handler),
        ),
        (
            Cow::Owned(parsed),
            ElementContentHandlers::default().element(element_handler),
        ),
    ];

    let _ = rewrite_html(html, handlers)?;
    let result = buf.borrow().clone();
    Ok(result)
}

/// Minimal HTML entity decoder for text-extraction call sites.
///
/// Handles the five XML/HTML named references (`&amp;`, `&lt;`, `&gt;`,
/// `&quot;`, `&apos;`), decimal (`&#39;`) and hex (`&#x27;`) numeric
/// character references, plus `&nbsp;`. Unknown references pass through
/// verbatim so the output for non-canonical input is the least
/// surprising.
///
/// The set is deliberately narrow — search-index titles / headings /
/// body text only ever contain these forms in practice, and a full
/// HTML5-spec named-reference table would balloon binary size for no
/// observable benefit.
#[must_use]
pub fn decode_html_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            // Try to match a named or numeric reference up to the next ';'
            if let Some(rel_semi) = s[i..].find(';') {
                let entity = &s[i..=i + rel_semi];
                if let Some(decoded) = decode_one_entity(entity) {
                    out.push_str(&decoded);
                    i += rel_semi + 1;
                    continue;
                }
            }
        }
        // Walk a single UTF-8 scalar; the byte index `i` always sits on
        // a char boundary because we never split inside a multi-byte
        // sequence (we only advance past `&...;` runs which are ASCII).
        // The `next()` always yields Some because the while-loop bound
        // (`i < bytes.len()`) means there is at least one more char.
        let Some(ch) = s[i..].chars().next() else {
            break;
        };
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn decode_one_entity(entity: &str) -> Option<String> {
    // `entity` is `&...;` inclusive.
    let inner = entity.strip_prefix('&')?.strip_suffix(';')?;
    let decoded = match inner {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00A0}',
        n if n.starts_with("#x") || n.starts_with("#X") => {
            let cp = u32::from_str_radix(&n[2..], 16).ok()?;
            char::from_u32(cp)?
        }
        n if n.starts_with('#') => {
            let cp = n[1..].parse::<u32>().ok()?;
            char::from_u32(cp)?
        }
        _ => return None,
    };
    Some(decoded.to_string())
}

/// Collapses runs of ASCII whitespace into a single space and trims.
///
/// Matches the historical `strip_tags` behaviour in
/// `src/plugins/search.rs` so the search index stays byte-identical
/// across the port.
#[must_use]
pub fn collapse_whitespace(s: &str) -> String {
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
    use lol_html::element;

    #[test]
    fn rewrite_html_noop_returns_input() {
        let html = "<p>hello</p>";
        let out = rewrite_html(html, Vec::new()).unwrap();
        assert_eq!(out, html);
    }

    #[test]
    fn rewrite_html_replace_text_via_handler() {
        let html = "<p>hello <span>world</span></p>";
        let out = rewrite_html(
            html,
            vec![element!("span", |el| {
                el.set_inner_content(
                    "rust",
                    lol_html::html_content::ContentType::Text,
                );
                Ok(())
            })],
        )
        .unwrap();
        assert!(out.contains("rust"), "out={out}");
    }

    #[test]
    fn extract_text_with_filter_decodes_entities() {
        let html = "<title>My &amp; Title</title>";
        let texts = extract_text_with_filter(html, "title").unwrap();
        assert_eq!(texts, vec!["My & Title".to_string()]);
    }

    #[test]
    fn extract_text_with_filter_collapses_whitespace() {
        let html = "<h1>  hello   world  </h1>";
        let texts = extract_text_with_filter(html, "h1").unwrap();
        assert_eq!(texts, vec!["hello world".to_string()]);
    }

    #[test]
    fn extract_text_with_filter_skips_empty_elements() {
        let html = "<h2></h2><h2>real</h2>";
        let texts = extract_text_with_filter(html, "h2").unwrap();
        assert_eq!(texts, vec!["real".to_string()]);
    }

    #[test]
    fn extract_text_with_filter_invalid_selector_errors() {
        let err = extract_text_with_filter("<p></p>", ":::").unwrap_err();
        assert!(matches!(err, SsgError::Io { .. }));
    }

    #[test]
    fn collapse_whitespace_basic() {
        assert_eq!(collapse_whitespace("  a  b  "), "a b");
        assert_eq!(collapse_whitespace(""), "");
    }
}
