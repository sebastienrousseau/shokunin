// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HTML fix plugin.

use super::helpers::rfc2822_to_iso8601;
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Repairs HTML output:
/// - Fix 7: Upgrades JSON-LD `@context` from `http://schema.org/` to
///   `https://schema.org`.
/// - Fix 9: Repairs broken `.class=` image syntax where `<p` is
///   injected into `<img>` tags.
#[derive(Debug, Clone, Copy)]
pub struct HtmlFixPlugin;

impl Plugin for HtmlFixPlugin {
    fn name(&self) -> &'static str {
        "html-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let all_html = crate::walk::walk_files(&ctx.site_dir, "html")?;
        let cache = ctx.cache.as_ref();
        let html_files: Vec<_> = all_html
            .into_iter()
            .filter(|p| cache.is_none_or(|c| c.has_changed(p)))
            .collect();
        let fixed = std::sync::atomic::AtomicUsize::new(0);

        html_files.iter().try_for_each(|path| -> Result<()> {
            let html = fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;

            let modified = apply_html_fixes(&html);

            if modified != html {
                fs::write(path, &modified).with_context(|| {
                    format!("cannot write {}", path.display())
                })?;
                let _ =
                    fixed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Ok(())
        })?;

        let count = fixed.load(std::sync::atomic::Ordering::Relaxed);
        if count > 0 {
            log::info!("[html-fix] Repaired {count} HTML file(s)");
        }
        Ok(())
    }
}

/// Applies all HTML fixes to a single page and returns the modified content.
fn apply_html_fixes(html: &str) -> String {
    let mut modified = html.to_string();

    if needs_schema_context_fix(&modified) {
        modified = modified
            .replace("\"http://schema.org/\"", "\"https://schema.org\"")
            .replace("\"http://schema.org\"", "\"https://schema.org\"");
    }

    if modified.contains("application/ld+json") {
        modified = fix_jsonld_dates(&modified);
    }

    if modified.contains("<p src=") {
        modified = fix_broken_img_tags(&modified);
    }

    if needs_class_syntax_fix(&modified) {
        modified = fix_literal_class_syntax(&modified);
    }

    if needs_mobile_web_app_capable_meta(&modified) {
        modified = inject_mobile_web_app_capable_meta(&modified);
    }

    if has_empty_preload(&modified) {
        modified = remove_empty_preload_links(&modified);
    }

    modified
}

/// Returns `true` if the HTML contains `http://schema.org` context that needs upgrading.
fn needs_schema_context_fix(html: &str) -> bool {
    html.contains("\"http://schema.org/\"")
        || html.contains("\"http://schema.org\"")
}

/// Returns `true` if the HTML contains literal `.class=` syntax to fix.
fn needs_class_syntax_fix(html: &str) -> bool {
    html.contains(".class=&quot;") || html.contains(".class=\"")
}

/// Returns `true` if the HTML appears to contain a `<link rel="preload">`
/// tag whose `href` is empty or absent. Chrome logs
/// "<link rel=preload> has an invalid href value" for these. The check
/// is intentionally cheap; `remove_empty_preload_links` does the precise
/// per-tag work only if this returns `true`.
fn has_empty_preload(html: &str) -> bool {
    // The cheapest signal of "preload + no real href" is `href` followed
    // immediately by space or `>` (bare attribute) anywhere in the same
    // document, *and* a preload link somewhere too. False positives just
    // trigger the precise rewriter, which is idempotent.
    let has_preload = html.contains("rel=preload")
        || html.contains("rel=\"preload\"")
        || html.contains("rel='preload'");
    let has_empty_href = html.contains("href=\"\"")
        || html.contains("href=''")
        || html.contains(" href ")
        || html.contains(" href>")
        || html.contains(" href/>");
    has_preload && has_empty_href
}

/// Removes any `<link>` tag that declares `rel="preload"` and has an empty
/// or missing `href`. Idempotent.
pub(super) fn remove_empty_preload_links(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while cursor < html.len() {
        // Find the next `<link` (case-insensitive) starting at cursor.
        let Some(rel_offset) =
            html[cursor..].to_ascii_lowercase().find("<link")
        else {
            out.push_str(&html[cursor..]);
            break;
        };
        let tag_start = cursor + rel_offset;
        out.push_str(&html[cursor..tag_start]);

        // Walk forward to the closing `>`, respecting quoted attribute values.
        let bytes = html.as_bytes();
        let mut j = tag_start;
        let mut quote: Option<u8> = None;
        while j < bytes.len() {
            let b = bytes[j];
            match quote {
                Some(q) if b == q => quote = None,
                Some(_) => {}
                None => match b {
                    b'"' | b'\'' => quote = Some(b),
                    b'>' => break,
                    _ => {}
                },
            }
            j += 1;
        }
        let tag_end = (j + 1).min(html.len());
        let tag = &html[tag_start..tag_end];
        let lower = tag.to_ascii_lowercase();
        let is_preload = lower.contains("rel=\"preload\"")
            || lower.contains("rel='preload'")
            || lower.contains("rel=preload");
        let has_real_href = href_is_present_and_non_empty(&lower);
        // Drop only empty-href preload tags; keep everything else.
        if !is_preload || has_real_href {
            out.push_str(tag);
        }
        cursor = tag_end;
    }
    out
}

/// Returns `true` if a (lowercased) tag string has a `href` attribute that
/// is present and non-empty. Tolerates double, single, and unquoted forms.
fn href_is_present_and_non_empty(lower_tag: &str) -> bool {
    if lower_tag.contains("href=\"\"") || lower_tag.contains("href=''") {
        return false;
    }
    let Some(idx) = lower_tag.find("href") else {
        return false;
    };
    // Must be followed by `=`, possibly with surrounding whitespace.
    let after = lower_tag[idx + 4..].trim_start();
    let Some(rest) = after.strip_prefix('=') else {
        return false;
    };
    let rest = rest.trim_start();
    match rest.chars().next() {
        None | Some('>') => false,
        Some('"') => rest.len() > 1 && !rest.starts_with("\"\""),
        Some('\'') => rest.len() > 1 && !rest.starts_with("''"),
        Some(c) if c.is_whitespace() => false,
        Some(_) => true,
    }
}

/// Returns `true` if the HTML emits the legacy
/// `apple-mobile-web-app-capable` meta but lacks the modern
/// `mobile-web-app-capable` meta that Chrome now requires. Tolerates
/// quoted, single-quoted, or unquoted attribute values (post-minify HTML
/// often drops quotes around short values like `yes`).
fn needs_mobile_web_app_capable_meta(html: &str) -> bool {
    let has_legacy = html.contains("apple-mobile-web-app-capable");
    let has_modern = find_modern_mobile_web_app_capable(html).is_some();
    has_legacy && !has_modern
}

/// Returns the byte offset of a `name=...mobile-web-app-capable...` meta
/// attribute that is **not** the apple variant, or `None` if none found.
fn find_modern_mobile_web_app_capable(html: &str) -> Option<usize> {
    // Search for the bare attribute name in any of the three quoting
    // styles, then verify it isn't preceded by `apple-` (which would make
    // it the legacy variant).
    let needles = [
        "name=\"mobile-web-app-capable\"",
        "name='mobile-web-app-capable'",
        "name=mobile-web-app-capable",
    ];
    for n in &needles {
        if let Some(pos) = html.find(n) {
            return Some(pos);
        }
    }
    None
}

/// Injects `<meta name="mobile-web-app-capable" content="yes">` immediately
/// after the legacy Apple variant so installed-PWA support works in Chrome
/// without console deprecation warnings. Handles minified HTML where the
/// `name=` attribute may be unquoted and may appear after `content=`.
pub(super) fn inject_mobile_web_app_capable_meta(html: &str) -> String {
    let modern = "<meta name=\"mobile-web-app-capable\" content=\"yes\">";
    // Find the apple-variant attribute name. Tolerate quoted/unquoted forms.
    let candidates = [
        "name=\"apple-mobile-web-app-capable\"",
        "name='apple-mobile-web-app-capable'",
        "name=apple-mobile-web-app-capable",
    ];
    let name_pos = candidates.iter().find_map(|n| html.find(n));
    let Some(name_pos) = name_pos else {
        return html.to_string();
    };
    // Walk forward to the next `>` that closes this <meta> tag.
    let after = &html[name_pos..];
    let Some(rel_close) = after.find('>') else {
        return html.to_string();
    };
    let insert_at = name_pos + rel_close + 1;
    format!("{}{modern}{}", &html[..insert_at], &html[insert_at..])
}

/// Fix JSON-LD date fields from RFC 2822 to ISO 8601.
pub(super) fn fix_jsonld_dates(html: &str) -> String {
    let mut result = html.to_string();

    // Match "datePublished":"..." and "dateModified":"..." patterns
    for field in &["datePublished", "dateModified"] {
        let pattern = format!("\"{field}\":\"");
        let mut search_from = 0;
        while let Some(start) = result[search_from..].find(&pattern) {
            let abs_start = search_from + start + pattern.len();
            if let Some(end) = result[abs_start..].find('"') {
                let date_str = &result[abs_start..abs_start + end];
                // Only convert if it looks like RFC 2822 (starts with
                // a day abbreviation like "Mon," "Tue,", etc.)
                if date_str.len() > 5
                    && date_str.as_bytes()[3] == b','
                    && date_str.as_bytes()[0].is_ascii_alphabetic()
                {
                    let iso = rfc2822_to_iso8601(date_str);
                    if iso != date_str {
                        result = format!(
                            "{}{}{}",
                            &result[..abs_start],
                            iso,
                            &result[abs_start + end..]
                        );
                    }
                }
                search_from = abs_start + 1;
            } else {
                break;
            }
        }
    }

    result
}

/// Repair broken `<img ... <p src="...">` patterns by reconstructing
/// valid `<img>` tags.
pub(super) fn fix_broken_img_tags(html: &str) -> String {
    let mut result = html.to_string();
    // Pattern: <img ... <p src="URL">
    // Replace with: <img ... src="URL" />
    while let Some(p_pos) = result.find("<p src=") {
        // Look backwards for the <img tag start
        let before = &result[..p_pos];
        if let Some(img_start) = before.rfind("<img") {
            // Extract the src value from <p src="...">
            let after_p = &result[p_pos..]; // includes "<p src="
            if let Some(quote_start) = after_p.find("src=\"") {
                let val_start = quote_start + 5; // skip src="
                let remaining = &after_p[val_start..];
                if let Some(quote_end) = remaining.find('"') {
                    let src_value = remaining[..quote_end].to_string();
                    // Find the closing > of this broken tag
                    let close_offset = remaining[quote_end..]
                        .find('>')
                        .map_or(result.len(), |i| {
                            p_pos + val_start + quote_end + i + 1
                        });

                    // Extract existing attributes from the img tag portion
                    let img_attrs = result[img_start + 4..p_pos].trim();
                    let img_attrs_clean =
                        img_attrs.trim_end_matches(|c: char| {
                            c.is_whitespace() || c == '<'
                        });

                    let new_img = format!(
                        "<img {img_attrs_clean} src=\"{src_value}\" />"
                    );
                    result = format!(
                        "{}{}{}",
                        &result[..img_start],
                        new_img,
                        &result[close_offset..]
                    );
                    continue;
                }
            }
        }
        // If we can't parse, skip to avoid infinite loop
        break;
    }
    result
}

/// Remove literal `.class=&quot;...&quot;` or `.class="..."` from HTML
/// and apply them as actual class attributes.
pub(super) fn fix_literal_class_syntax(html: &str) -> String {
    let mut result = html.to_string();

    // Handle .class=&quot;...&quot; (HTML-encoded quotes)
    result = fix_class_syntax_variant(&result, ".class=&quot;", "&quot;");
    // Handle .class="..." (literal quotes)
    result = fix_class_syntax_variant(&result, ".class=\"", "\"");

    result
}

/// Handles one variant of the `.class=` syntax fix.
fn fix_class_syntax_variant(
    html: &str,
    open_pattern: &str,
    close_pattern: &str,
) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find(open_pattern) {
        let after = &result[start + open_pattern.len()..];
        if let Some(end) = after.find(close_pattern) {
            let class_value = after[..end].to_string();
            let remove_end =
                start + open_pattern.len() + end + close_pattern.len();
            result = format!("{}{}", &result[..start], &result[remove_end..]);
            inject_class_attr(&mut result, start, &class_value);
        } else {
            break;
        }
    }
    result
}

/// Injects a class attribute into the nearest preceding tag if it doesn't already have one.
fn inject_class_attr(html: &mut String, pos: usize, class_value: &str) {
    if let Some(tag_end) = html[..pos].rfind('>') {
        if let Some(tag_start) = html[..tag_end].rfind('<') {
            let tag = &html[tag_start..tag_end];
            if !tag.contains("class=") {
                let insert_pos = tag_end;
                *html = format!(
                    "{} class=\"{}\"{}",
                    &html[..insert_pos],
                    class_value,
                    &html[insert_pos..]
                );
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx(site_dir: &Path) -> PluginContext {
        crate::test_support::init_logger();
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
        )
    }

    #[test]
    fn test_html_fix_upgrades_jsonld_context() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("index.html");
        fs::write(
            &html_path,
            r#"<html><head>
<script type="application/ld+json">
{"@context":"http://schema.org/","@type":"WebPage"}
</script>
</head><body></body></html>"#,
        )?;

        let ctx = test_ctx(tmp.path());
        HtmlFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&html_path)?;
        assert!(result.contains("\"https://schema.org\""));
        assert!(!result.contains("\"http://schema.org/\""));
        Ok(())
    }

    #[test]
    fn test_html_fix_converts_jsonld_dates() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("article.html");
        fs::write(
            &html_path,
            r#"<html><head>
<script type="application/ld+json">
{"@context":"https://schema.org","@type":"Article","datePublished":"Thu, 11 Apr 2026 06:06:06 +0000","dateModified":"Mon, 01 Sep 2025 06:06:06 +0000"}
</script>
</head><body></body></html>"#,
        )?;

        let ctx = test_ctx(tmp.path());
        HtmlFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&html_path)?;
        assert!(
            result.contains("\"datePublished\":\"2026-04-11"),
            "Expected ISO date, got: {result}"
        );
        assert!(
            result.contains("\"dateModified\":\"2025-09-01"),
            "Expected ISO date, got: {result}"
        );
        assert!(!result.contains("Thu, 11 Apr"));
        Ok(())
    }

    #[test]
    fn test_fix_broken_img_tags() {
        let input =
            r#"<img alt="test" class="w-25" title="test" <p src="image.jpg">"#;
        let result = fix_broken_img_tags(input);
        assert!(result.contains("src=\"image.jpg\""));
        assert!(!result.contains("<p src="));
    }

    #[test]
    fn test_fix_literal_class_syntax() {
        let input = r#"<img alt="test" src="img.jpg">.class=&quot;w-25 float-start&quot;"#;
        let result = fix_literal_class_syntax(input);
        assert!(!result.contains(".class=&quot;"));
    }

    // -----------------------------------------------------------------
    // fix_jsonld_dates
    // -----------------------------------------------------------------

    #[test]
    fn test_fix_jsonld_dates_iso_passthrough() {
        let input =
            r#"{"datePublished":"2026-04-11","dateModified":"2025-09-01"}"#;
        let result = fix_jsonld_dates(input);
        assert_eq!(result, input, "ISO dates should pass through unchanged");
    }

    #[test]
    fn test_fix_jsonld_dates_converts_rfc2822() {
        let input = r#"{"datePublished":"Thu, 11 Apr 2026 06:06:06 +0000"}"#;
        let result = fix_jsonld_dates(input);
        assert!(
            result.contains("\"datePublished\":\"2026-04-11T06:06:06+00:00\""),
            "Should convert RFC 2822 to ISO 8601, got: {result}"
        );
    }

    #[test]
    fn test_fix_jsonld_dates_both_fields() {
        let input = r#"{"datePublished":"Mon, 01 Sep 2025 12:00:00 +0000","dateModified":"Tue, 02 Sep 2025 14:30:00 +0000"}"#;
        let result = fix_jsonld_dates(input);
        assert!(result.contains("2025-09-01T12:00:00+00:00"));
        assert!(result.contains("2025-09-02T14:30:00+00:00"));
    }

    // -----------------------------------------------------------------
    // fix_broken_img_tags
    // -----------------------------------------------------------------

    #[test]
    fn test_fix_broken_img_tags_multiple() {
        let input =
            r#"<img alt="a" <p src="one.jpg"><img alt="b" <p src="two.jpg">"#;
        let result = fix_broken_img_tags(input);
        assert!(result.contains("src=\"one.jpg\""), "first img: {result}");
        assert!(result.contains("src=\"two.jpg\""), "second img: {result}");
        assert!(
            !result.contains("<p src="),
            "no broken tags remain: {result}"
        );
    }

    #[test]
    fn test_fix_broken_img_tags_none() {
        let input = r#"<img alt="ok" src="good.jpg" />"#;
        let result = fix_broken_img_tags(input);
        assert_eq!(
            result, input,
            "No broken tags should leave input unchanged"
        );
    }

    // -----------------------------------------------------------------
    // fix_literal_class_syntax
    // -----------------------------------------------------------------

    #[test]
    fn test_fix_literal_class_syntax_html_encoded() {
        let input =
            r#"<img src="img.jpg">.class=&quot;w-25 float-start&quot; rest"#;
        let result = fix_literal_class_syntax(input);
        assert!(
            !result.contains(".class=&quot;"),
            "should remove .class=&quot;"
        );
        assert!(
            result.contains("class=\"w-25 float-start\""),
            "should inject class attr, got: {result}"
        );
    }

    #[test]
    fn test_fix_literal_class_syntax_literal_quotes() {
        let input = r#"<img src="img.jpg">.class="my-class" rest"#;
        let result = fix_literal_class_syntax(input);
        assert!(
            !result.contains(".class=\""),
            "should remove .class=\", got: {result}"
        );
        assert!(
            result.contains("class=\"my-class\""),
            "should inject class attr, got: {result}"
        );
    }

    #[test]
    fn test_fix_literal_class_syntax_no_class() {
        let input = r#"<img src="img.jpg"> some text"#;
        let result = fix_literal_class_syntax(input);
        assert_eq!(result, input, "No .class= should leave input unchanged");
    }

    // -----------------------------------------------------------------
    // inject_mobile_web_app_capable_meta
    // -----------------------------------------------------------------

    #[test]
    fn test_inject_mobile_web_app_capable_meta_added() {
        let input = r#"<head><meta name="apple-mobile-web-app-capable" content="yes"></head>"#;
        let result = inject_mobile_web_app_capable_meta(input);
        assert!(
            result.contains(
                r#"<meta name="mobile-web-app-capable" content="yes">"#
            ),
            "modern meta should be injected, got: {result}"
        );
        assert!(
            result.contains(
                r#"<meta name="apple-mobile-web-app-capable" content="yes">"#
            ),
            "legacy meta must remain for backwards compatibility"
        );
    }

    // -----------------------------------------------------------------
    // remove_empty_preload_links
    // -----------------------------------------------------------------

    #[test]
    fn test_remove_empty_preload_drops_bare_href() {
        let input = r#"<head><link as=image fetchpriority=high href rel=preload type=image/webp><title>x</title></head>"#;
        let result = remove_empty_preload_links(input);
        assert!(
            !result.contains("rel=preload"),
            "empty preload should be removed, got: {result}"
        );
        assert!(result.contains("<title>x</title>"), "rest preserved");
    }

    #[test]
    fn test_remove_empty_preload_drops_quoted_empty_href() {
        let input = r#"<link rel="preload" href="" as="image">"#;
        let result = remove_empty_preload_links(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_empty_preload_keeps_valid_preload() {
        let input = r#"<link rel="preload" href="/banner.webp" as="image">"#;
        let result = remove_empty_preload_links(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_remove_empty_preload_preserves_utf8() {
        let input = r#"<title>日本語</title><link rel=preload href as=image><p>テスト</p>"#;
        let result = remove_empty_preload_links(input);
        assert!(result.contains("日本語"));
        assert!(result.contains("テスト"));
        assert!(!result.contains("rel=preload"));
    }

    #[test]
    fn test_apply_html_fixes_idempotent_on_modern_meta() {
        let input = r#"<head><meta name="apple-mobile-web-app-capable" content="yes"><meta name="mobile-web-app-capable" content="yes"></head>"#;
        let result = apply_html_fixes(input);
        // Should not double-inject when modern meta already exists.
        let count = result.matches("name=\"mobile-web-app-capable\"").count();
        assert_eq!(count, 1, "no duplicate injection, got: {result}");
    }
}
