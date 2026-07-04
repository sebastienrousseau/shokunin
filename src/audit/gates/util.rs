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

/// Reads a single attribute value out of a tag.
///
/// Accepts double, single, or unquoted attribute values, plus
/// valueless (boolean) attributes — HTML minifiers collapse `alt=""`
/// to a bare `alt`, which returns `Some(String::new())`. Match is
/// case-insensitive on the attribute name, case-preserving on the
/// value, and tokenises real attribute boundaries so `data-href=`
/// never satisfies a lookup for `href`.
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
/// // Minified valueless attribute (`alt=""` collapsed to `alt`).
/// assert_eq!(hreflang_attr("<img alt src=a.png>", "alt"), Some(String::new()));
/// ```
pub fn hreflang_attr(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut i = usize::from(bytes.first() == Some(&b'<'));
    // Skip the tag name itself.
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>'
    {
        i += 1;
    }
    while i < bytes.len() {
        // Skip whitespace and self-closing slashes between attributes.
        while i < bytes.len()
            && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/')
        {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'>' {
            return None;
        }
        // Attribute name.
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let attr_name = &tag[name_start..i];
        // Optional `= value` (whitespace around `=` tolerated).
        let mut j = i;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'=' {
            // Valueless attribute (e.g. minified bare `alt`).
            if attr_name.eq_ignore_ascii_case(name) {
                return Some(String::new());
            }
            continue;
        }
        // Value start (skip `=` and any whitespace after it).
        let mut k = j + 1;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        let (value_start, value_end, resume) =
            if k < bytes.len() && (bytes[k] == b'"' || bytes[k] == b'\'') {
                let quote = bytes[k] as char;
                let vstart = k + 1;
                // Unterminated quote: bail — the tag is malformed.
                let close = vstart + tag[vstart..].find(quote)?;
                (vstart, close, close + 1)
            } else {
                let mut vend = k;
                while vend < bytes.len()
                    && !bytes[vend].is_ascii_whitespace()
                    && bytes[vend] != b'>'
                {
                    vend += 1;
                }
                (k, vend, vend)
            };
        if attr_name.eq_ignore_ascii_case(name) {
            return Some(tag[value_start..value_end].to_string());
        }
        i = resume;
    }
    None
}

/// Returns a copy of `html` with the *contents* of every `<script>` and
/// `<style>` element blanked to spaces (byte offsets are preserved;
/// newlines survive so line-based diagnostics stay stable).
///
/// Tag-scanning gates use this so markup embedded in JS string literals
/// — e.g. a client-side search overlay building `<a href="…">` result
/// rows — is never mistaken for document links.
///
/// # Examples
///
/// ```
/// use ssg::audit::gates::strip_script_and_style;
/// let html = r#"<script>var s='<a href="/x">';</script><a href="/y">y</a>"#;
/// let stripped = strip_script_and_style(html);
/// assert!(!stripped.contains("/x"));
/// assert!(stripped.contains("/y"));
/// ```
#[must_use]
pub fn strip_script_and_style(html: &str) -> String {
    let mut out = html.as_bytes().to_vec();
    let lower = html.to_ascii_lowercase();
    for (open, close) in &[("<script", "</script"), ("<style", "</style")] {
        let mut cursor = 0;
        while let Some(rel) = lower[cursor..].find(open) {
            let abs = cursor + rel;
            let content_start = find_tag_end(&lower, abs);
            let content_end = lower[content_start..]
                .find(close)
                .map_or(lower.len(), |e| content_start + e);
            for b in &mut out[content_start..content_end] {
                if !b.is_ascii_whitespace() {
                    *b = b' ';
                }
            }
            cursor = content_end.max(abs + open.len());
        }
    }
    // Replaced regions start/end on char boundaries (str::find results)
    // and are fully space-filled, so the buffer stays valid UTF-8; the
    // lossy conversion is a no-op safety net.
    String::from_utf8_lossy(&out).into_owned()
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

    #[test]
    fn hreflang_attr_valueless_attribute_returns_empty_value() {
        // Minifiers collapse `alt=""` to a bare `alt` token.
        let tag = "<img alt height=33 role=presentation width=100>";
        assert_eq!(hreflang_attr(tag, "alt"), Some(String::new()));
        assert_eq!(hreflang_attr(tag, "height"), Some("33".to_string()));
        assert_eq!(hreflang_attr(tag, "width"), Some("100".to_string()));
    }

    #[test]
    fn hreflang_attr_rejects_substring_attribute_names() {
        // `data-href` must not satisfy a lookup for `href`, and
        // `hreflang` must not satisfy a lookup for `lang`.
        let tag = r#"<a data-href="/wrong" hreflang="en-GB">"#;
        assert_eq!(hreflang_attr(tag, "href"), None);
        assert_eq!(hreflang_attr(tag, "lang"), None);
        assert_eq!(hreflang_attr(tag, "hreflang"), Some("en-GB".to_string()));
    }

    #[test]
    fn hreflang_attr_unquoted_mixed_case_value_preserved() {
        // Minified output: unquoted, original casing kept in value.
        let tag = "<meta http-equiv=Content-Security-Policy content=x>";
        assert_eq!(
            hreflang_attr(tag, "HTTP-EQUIV"),
            Some("Content-Security-Policy".to_string())
        );
    }

    #[test]
    fn hreflang_attr_tolerates_whitespace_around_equals() {
        let tag = r#"<a href = "/spaced">"#;
        assert_eq!(hreflang_attr(tag, "href"), Some("/spaced".to_string()));
    }

    #[test]
    fn hreflang_attr_ignores_tag_name_prefix() {
        // The tag name itself must never be parsed as an attribute.
        assert_eq!(hreflang_attr("<content x=1>", "content"), None);
    }

    #[test]
    fn hreflang_attr_unterminated_tag_after_last_attr_returns_none() {
        // The scan consumes the trailing unquoted value and runs off
        // the (gt-less) end of the tag without a match.
        assert_eq!(hreflang_attr("<a foo=bar", "href"), None);
        assert_eq!(hreflang_attr(r#"<a foo="bar""#, "href"), None);
    }

    #[test]
    fn strip_script_and_style_blanks_embedded_markup() {
        let html = "<script>var h='<a href=\"/ghost\">x</a>';</script>\
                    <style>a[href=\"/styled\"]{}</style>\
                    <a href=\"/real\">r</a>";
        let s = strip_script_and_style(html);
        assert_eq!(s.len(), html.len(), "offsets must be preserved");
        assert!(!s.contains("/ghost"));
        assert!(!s.contains("/styled"));
        assert!(s.contains("/real"));
    }

    #[test]
    fn strip_script_and_style_preserves_newlines() {
        let html = "<script>\nline1\nline2\n</script>\n<p>after</p>";
        let s = strip_script_and_style(html);
        assert_eq!(
            s.matches('\n').count(),
            html.matches('\n').count(),
            "newlines inside stripped regions must survive"
        );
        assert!(s.contains("<p>after</p>"));
    }

    #[test]
    fn strip_script_and_style_unclosed_script_blanks_to_eof() {
        let html = "<p>keep</p><script>var x='<a href=\"/js\">';";
        let s = strip_script_and_style(html);
        assert!(s.contains("keep"));
        assert!(!s.contains("/js"));
    }

    #[test]
    fn strip_script_and_style_handles_multibyte_content() {
        let html = "<script>var s='héllo — “quoted”';</script><p>café</p>";
        let s = strip_script_and_style(html);
        assert!(s.contains("café"));
        assert!(!s.contains("héllo"));
    }
}
