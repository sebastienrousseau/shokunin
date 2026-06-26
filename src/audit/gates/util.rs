// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Internal helpers shared across audit gates.
//!
//! Provides a quote-aware HTML tag end-detector (so SVG data URLs and
//! `?…&…` query strings inside `src` / `href` attributes do not
//! truncate tags) plus a small attribute extractor that mirrors
//! `accessibility::extract_attr_value`'s behaviour.

/// Returns the absolute end index (one past the closing `>`) of the
/// HTML tag that starts at `tag_start`. Skips `>` characters inside
/// quoted attribute values (single and double quotes).
///
/// # Examples
///
/// ```
/// use ssg::audit::gates::find_tag_end;
/// let html = "<br>x";
/// assert_eq!(find_tag_end(html, 0), 4);
/// ```
#[allow(dead_code)]
pub fn find_tag_end(html: &str, tag_start: usize) -> usize {
    let bytes = html.as_bytes();
    let mut i = tag_start;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return i + 1,
                _ => {}
            },
        }
        i += 1;
    }
    bytes.len()
}

/// Reads a single attribute value out of a tag. Accepts double, single,
/// or unquoted attribute values. Match is case-insensitive on the
/// attribute name, case-preserving on the value.
///
/// The slightly odd name (`hreflang_attr`) is historical — this used
/// to live in the hreflang gate; promoted to a shared helper when the
/// CSP gate needed the same parser.
///
/// # Examples
///
/// ```
/// use ssg::audit::gates::hreflang_attr;
/// let tag = r#"<link rel="alternate" hreflang="en" href="/en/">"#;
/// assert_eq!(hreflang_attr(tag, "hreflang"), Some("en".to_string()));
/// assert_eq!(hreflang_attr(tag, "href"), Some("/en/".to_string()));
/// ```
pub fn hreflang_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pat = format!("{name}=");
    let start = lower.find(&pat)?;
    let after = &tag[start + pat.len()..];
    let trimmed = after.trim_start();
    if let Some(rest) = trimmed.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    if let Some(rest) = trimmed.strip_prefix('\'') {
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }
    let end = trimmed
        .find(|c: char| c.is_whitespace() || c == '>')
        .unwrap_or(trimmed.len());
    Some(trimmed[..end].to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn find_tag_end_handles_quoted_gt() {
        let h = "<img src=\"data:>foo\" alt=\"x\">rest";
        let end = find_tag_end(h, 0);
        assert_eq!(&h[..end], "<img src=\"data:>foo\" alt=\"x\">");
    }

    #[test]
    fn hreflang_attr_handles_quotes() {
        let tag = r#"<link rel='alternate' hreflang="en" href=/en/>"#;
        assert_eq!(hreflang_attr(tag, "hreflang"), Some("en".to_string()));
        assert_eq!(hreflang_attr(tag, "href"), Some("/en/".to_string()));
    }

    #[test]
    fn hreflang_attr_missing_returns_none() {
        assert_eq!(hreflang_attr("<a>", "href"), None);
    }

    #[test]
    fn find_tag_end_returns_len_when_unterminated() {
        let h = "<img src=\"x\" alt=\"y\"";
        assert_eq!(find_tag_end(h, 0), h.len());
    }

    #[test]
    fn find_tag_end_skips_gt_inside_single_quotes() {
        let h = "<a href='data:>foo'>rest";
        let end = find_tag_end(h, 0);
        assert_eq!(&h[..end], "<a href='data:>foo'>");
    }

    #[test]
    fn find_tag_end_simple_unquoted() {
        let h = "<br>x";
        assert_eq!(find_tag_end(h, 0), 4);
    }

    #[test]
    fn hreflang_attr_handles_single_quotes() {
        let tag = r#"<a href='single-quoted'>"#;
        assert_eq!(
            hreflang_attr(tag, "href"),
            Some("single-quoted".to_string())
        );
    }

    #[test]
    fn hreflang_attr_unquoted_value_terminated_by_space() {
        let tag = "<a href=/path other=1>";
        assert_eq!(hreflang_attr(tag, "href"), Some("/path".to_string()));
    }

    #[test]
    fn hreflang_attr_unquoted_value_terminated_by_gt() {
        let tag = "<a href=/end>";
        assert_eq!(hreflang_attr(tag, "href"), Some("/end".to_string()));
    }

    #[test]
    fn hreflang_attr_case_insensitive_match() {
        let tag = r#"<A HREF="upper">"#;
        assert_eq!(hreflang_attr(tag, "href"), Some("upper".to_string()));
    }

    #[test]
    fn hreflang_attr_unterminated_double_quote_returns_none() {
        let tag = r#"<a href="never-closed"#;
        assert_eq!(hreflang_attr(tag, "href"), None);
    }

    #[test]
    fn hreflang_attr_unterminated_single_quote_returns_none() {
        let tag = r#"<a href='never-closed"#;
        assert_eq!(hreflang_attr(tag, "href"), None);
    }
}
