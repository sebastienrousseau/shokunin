// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Sitemap fix plugin.

use super::helpers::{normalise_url_in_xml_line, read_meta_sidecars};
use crate::dates::parse_flexible_date;
use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::collections::HashMap;
use std::fs;

/// Repairs and canonicalises the generated `sitemap.xml`.
///
/// Removes duplicate XML declarations, normalises double-slash URLs,
/// rewrites `<loc>` values onto the shared directory-URL convention
/// (`…/foo/index.html` → `…/foo/`, via
/// [`crate::urls::derive_page_url`] — spec A2/B1, plan §2 item 1.2),
/// and updates per-page lastmod dates.
#[derive(Debug, Clone, Copy)]
pub struct SitemapFixPlugin;

impl Plugin for SitemapFixPlugin {
    fn name(&self) -> &'static str {
        "sitemap-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        let sitemap_path = ctx.site_dir.join("sitemap.xml");
        if !sitemap_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&sitemap_path).with_path(&sitemap_path)?;

        let meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();
        let date_map = collect_date_map(&meta_entries);

        let result = strip_duplicate_xml_decls_and_fix_urls(&content);

        // Second pass: update lastmod based on the <loc> in each <url> block
        let updated = update_lastmod_from_loc(&result, &date_map);

        fs::write(&sitemap_path, updated).with_path(&sitemap_path)?;

        log::info!("[sitemap-fix] Repaired sitemap.xml");
        Ok(())
    }
}

/// Collects per-page date strings from meta sidecar entries.
fn collect_date_map(
    meta_entries: &[(String, HashMap<String, String>)],
) -> HashMap<String, String> {
    let mut date_map = HashMap::new();
    for (rel_path, meta) in meta_entries {
        if let Some(date) = extract_best_date(meta) {
            let _ = date_map.insert(rel_path.clone(), date);
        }
    }
    date_map
}

/// Extracts the best available date from a metadata map.
///
/// Issue #586 / plan §2 item 1.4 (spec A4): every field runs through
/// the shared flexible date chain (RFC 2822 → long form → ISO 8601),
/// so front matter like `date: July 1, 2026` now yields a valid
/// `<lastmod>`. An unparseable `date` value still passes through
/// verbatim, preserving the plugin's previous output for that case.
fn extract_best_date(meta: &HashMap<String, String>) -> Option<String> {
    let parse_field = |field: &str| {
        let raw = meta.get(field)?;
        match parse_flexible_date(raw) {
            Ok(dt) => Some(dt.to_iso_date()),
            Err(err) => {
                if !raw.is_empty() {
                    log::warn!("[sitemap-fix] '{field}': {err}");
                }
                None
            }
        }
    };
    parse_field("item_pub_date")
        .or_else(|| parse_field("last_build_date"))
        .or_else(|| parse_field("date"))
        .or_else(|| meta.get("date").cloned())
}

/// Strips duplicate XML declarations and normalises URLs in the sitemap.
fn strip_duplicate_xml_decls_and_fix_urls(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut first_decl = true;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<?xml") {
            if first_decl {
                first_decl = false;
                result.push_str(line);
                result.push('\n');
            }
            continue;
        }

        let processed = if line.contains("<loc>") {
            // `<loc>` values go through the shared page-URL derivation
            // so sitemap, canonical `<link>`, feed `<link>`, and the
            // stager's injected `permalink:` all agree on the
            // directory-URL convention (plan §2 item 1.2: one code
            // path — `urls::derive_page_url`).
            canonicalise_loc_urls(&normalise_url_in_xml_line(line))
        } else if line.contains("<link>") || line.contains("<atom:link") {
            normalise_url_in_xml_line(line)
        } else {
            line.to_string()
        };

        result.push_str(&processed);
        result.push('\n');
    }

    result
}

/// Rewrites every `<loc>…</loc>` URL on `line` onto the canonical
/// directory-URL convention via [`crate::urls::derive_page_url`]
/// (spec A2/B1, plan §2 item 1.2): `…/foo/index.html` collapses to
/// `…/foo/`, the root `…/index.html` to `…/`, and non-index paths
/// pass through unchanged. URLs without a scheme are left untouched.
fn canonicalise_loc_urls(line: &str) -> String {
    let Some(open_idx) = line.find("<loc>") else {
        return line.to_string();
    };
    let val_start = open_idx + "<loc>".len();
    let Some(close_rel) = line[val_start..].find("</loc>") else {
        return line.to_string();
    };
    let url = &line[val_start..val_start + close_rel];
    let canonical = canonicalise_page_url(url);
    format!(
        "{}{}{}",
        &line[..val_start],
        canonical,
        &line[val_start + close_rel..]
    )
}

/// Splits an absolute URL into origin + path and re-derives it through
/// [`crate::urls::derive_page_url`]. Non-absolute values (no scheme)
/// are returned unchanged.
fn canonicalise_page_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let after_scheme = &url[scheme_end + 3..];
    let Some(path_rel) = after_scheme.find('/') else {
        // Bare origin (`https://example.com`) — the root URL.
        return format!("{url}/");
    };
    let origin = &url[..scheme_end + 3 + path_rel];
    let rel_path = &after_scheme[path_rel + 1..];
    crate::urls::derive_page_url(origin, rel_path)
}

/// Update `<lastmod>` values based on the preceding `<loc>` URL in each
/// `<url>` block.
pub(super) fn update_lastmod_from_loc(
    xml: &str,
    date_map: &HashMap<String, String>,
) -> String {
    if date_map.is_empty() {
        return xml.to_string();
    }

    let mut result = String::with_capacity(xml.len());
    let mut current_loc = String::new();

    for line in xml.lines() {
        let trimmed = line.trim();

        // Track current <loc> value
        if trimmed.starts_with("<loc>") {
            if let Some(url) = trimmed
                .strip_prefix("<loc>")
                .and_then(|s| s.strip_suffix("</loc>"))
            {
                current_loc = url.to_string();
            }
        }

        // Replace <lastmod> using per-page date if available
        if trimmed.starts_with("<lastmod>") && trimmed.ends_with("</lastmod>") {
            let mut matched = false;
            for (rel_path, date) in date_map {
                if !rel_path.is_empty() && current_loc.contains(rel_path) {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    result.push_str(&format!(
                        "{indent}<lastmod>{date}</lastmod>\n"
                    ));
                    matched = true;
                    break;
                }
            }
            if !matched {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use anyhow::Result;
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
    fn test_sitemap_fix_removes_duplicate_xml_decls() -> Result<()> {
        let tmp = tempdir().unwrap();
        let sitemap = tmp.path().join("sitemap.xml");
        fs::write(
            &sitemap,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <?xml version="1.0" encoding="UTF-8"?>
<url>
  <loc>https://example.com/page1</loc>
  <lastmod>2025-09-01</lastmod>
</url>
    <?xml version="1.0" encoding="UTF-8"?>
<url>
  <loc>https://example.com/page2</loc>
  <lastmod>2025-09-01</lastmod>
</url>
</urlset>"#,
        )
        .unwrap();

        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx).unwrap();

        let result = fs::read_to_string(&sitemap).unwrap();
        assert_eq!(result.matches("<?xml").count(), 1);
        Ok(())
    }

    #[test]
    fn test_sitemap_fix_normalises_double_slashes() -> Result<()> {
        let tmp = tempdir().unwrap();
        let sitemap = tmp.path().join("sitemap.xml");
        fs::write(
            &sitemap,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<url>
  <loc>https://example.com//index.html</loc>
  <lastmod>2025-09-01</lastmod>
</url>
</urlset>"#,
        )
        .unwrap();

        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx).unwrap();

        let result = fs::read_to_string(&sitemap).unwrap();
        // Double slash normalised AND `<loc>` collapsed onto the
        // shared directory-URL convention (plan §2 item 1.2): the
        // root `index.html` publishes as the bare base URL.
        assert!(result.contains("<loc>https://example.com/</loc>"));
        assert!(!result.contains("com//index"));
        assert!(!result.contains("index.html"));
        Ok(())
    }

    #[test]
    fn test_sitemap_fix_collapses_index_html_locs() -> Result<()> {
        // Plan §2 item 1.2: sitemap `<loc>` must agree with canonical
        // `<link>` and feed `<link>` — all derive through
        // `urls::derive_page_url`, so `…/foo/index.html` publishes as
        // the pretty directory URL `…/foo/`.
        let tmp = tempdir().unwrap();
        let sitemap = tmp.path().join("sitemap.xml");
        fs::write(
            &sitemap,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<url>
  <loc>https://example.com/posts/hello/index.html</loc>
  <lastmod>2025-09-01</lastmod>
</url>
<url>
  <loc>https://example.com/feed.xml</loc>
  <lastmod>2025-09-01</lastmod>
</url>
</urlset>"#,
        )
        .unwrap();

        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx).unwrap();

        let result = fs::read_to_string(&sitemap).unwrap();
        assert!(
            result.contains("<loc>https://example.com/posts/hello/</loc>"),
            "index.html should collapse to the directory URL: {result}"
        );
        // Non-index outputs keep their file name.
        assert!(result.contains("<loc>https://example.com/feed.xml</loc>"));
        Ok(())
    }

    #[test]
    fn canonicalise_loc_urls_handles_edge_shapes() {
        // Bare origin → root URL with trailing slash.
        assert_eq!(
            canonicalise_loc_urls("<loc>https://example.com</loc>"),
            "<loc>https://example.com/</loc>"
        );
        // Schemeless values pass through untouched.
        assert_eq!(
            canonicalise_loc_urls("<loc>relative/index.html</loc>"),
            "<loc>relative/index.html</loc>"
        );
        // Lines without a closing tag pass through untouched.
        assert_eq!(
            canonicalise_loc_urls("<loc>https://example.com/a"),
            "<loc>https://example.com/a"
        );
        // Indentation is preserved.
        assert_eq!(
            canonicalise_loc_urls(
                "  <loc>https://example.com/a/index.html</loc>"
            ),
            "  <loc>https://example.com/a/</loc>"
        );
    }

    #[test]
    fn test_update_lastmod_from_loc_empty_map() {
        let xml = "<url><loc>https://example.com</loc><lastmod>2025-01-01</lastmod></url>";
        let result = update_lastmod_from_loc(xml, &HashMap::new());
        assert_eq!(result, xml);
    }

    #[test]
    fn test_update_lastmod_from_loc_with_match() {
        let xml = "<url>\n<loc>https://example.com/blog/</loc>\n<lastmod>2025-01-01</lastmod>\n</url>";
        let mut map = HashMap::new();
        let _ = map.insert("blog".to_string(), "2026-04-11".to_string());
        let result = update_lastmod_from_loc(xml, &map);
        assert!(
            result.contains("<lastmod>2026-04-11</lastmod>"),
            "Should update lastmod: {result}"
        );
    }

    #[test]
    fn name_is_stable() {
        assert_eq!(SitemapFixPlugin.name(), "sitemap-fix");
    }

    #[test]
    fn after_compile_no_op_when_sitemap_missing() -> Result<()> {
        let tmp = tempdir().unwrap();
        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx).unwrap();
        assert!(!tmp.path().join("sitemap.xml").exists());
        Ok(())
    }

    #[test]
    fn extract_best_date_prefers_item_pub_date() {
        let mut meta = HashMap::new();
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert(
            "last_build_date".to_string(),
            "Mon, 01 Sep 2025 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("date".to_string(), "2024-01-01".to_string());
        let date = extract_best_date(&meta);
        assert!(
            date.as_deref().is_some_and(|d| d.contains("2026-04-11")),
            "should prefer item_pub_date, got: {date:?}"
        );
    }

    #[test]
    fn extract_best_date_falls_back_to_last_build_date() {
        let mut meta = HashMap::new();
        let _ = meta.insert(
            "last_build_date".to_string(),
            "Mon, 01 Sep 2025 06:06:06 +0000".to_string(),
        );
        let date = extract_best_date(&meta);
        assert!(
            date.as_deref().is_some_and(|d| d.contains("2025-09-01")),
            "should use last_build_date when item_pub_date absent: {date:?}"
        );
    }

    #[test]
    fn extract_best_date_falls_back_to_date_field() {
        let mut meta = HashMap::new();
        let _ = meta.insert("date".to_string(), "2024-01-01".to_string());
        let date = extract_best_date(&meta);
        assert_eq!(date.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn extract_best_date_returns_none_when_no_dates() {
        let meta = HashMap::new();
        assert!(extract_best_date(&meta).is_none());
    }

    // -----------------------------------------------------------------
    // Flexible date chain (issue #586 / plan §2 item 1.4, spec A4)
    // -----------------------------------------------------------------

    #[test]
    fn extract_best_date_parses_long_form_date_field() {
        crate::test_support::init_logger();
        let mut meta = HashMap::new();
        let _ = meta.insert("date".to_string(), "July 1, 2026".to_string());
        let date = extract_best_date(&meta);
        assert_eq!(date.as_deref(), Some("2026-07-01"));
    }

    #[test]
    fn extract_best_date_parses_iso_item_pub_date() {
        let mut meta = HashMap::new();
        let _ =
            meta.insert("item_pub_date".to_string(), "2026-07-01".to_string());
        let date = extract_best_date(&meta);
        assert_eq!(date.as_deref(), Some("2026-07-01"));
    }

    #[test]
    fn extract_best_date_unparseable_date_passes_through() {
        crate::test_support::init_logger();
        let mut meta = HashMap::new();
        let _ = meta.insert("date".to_string(), "not-a-date".to_string());
        // Verbatim fallback preserves the plugin's previous output.
        let date = extract_best_date(&meta);
        assert_eq!(date.as_deref(), Some("not-a-date"));
    }

    #[test]
    fn extract_best_date_skips_unparseable_pub_date_for_next_field() {
        crate::test_support::init_logger();
        let mut meta = HashMap::new();
        let _ = meta.insert("item_pub_date".to_string(), "garbage".to_string());
        let _ = meta.insert(
            "last_build_date".to_string(),
            "Mon, 01 Sep 2025 06:06:06 +0000".to_string(),
        );
        let date = extract_best_date(&meta);
        assert_eq!(date.as_deref(), Some("2025-09-01"));
    }

    #[test]
    fn collect_date_map_includes_only_pages_with_dates() {
        let mut m1 = HashMap::new();
        let _ = m1.insert("date".to_string(), "2025-01-01".to_string());
        let mut m2 = HashMap::new();
        let _ = m2.insert("title".to_string(), "no date here".to_string());
        let entries =
            vec![("page-a".to_string(), m1), ("page-b".to_string(), m2)];
        let map = collect_date_map(&entries);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("page-a").unwrap(), "2025-01-01");
    }

    #[test]
    fn strip_duplicate_xml_decls_preserves_first_only() {
        let input = "<?xml version=\"1.0\"?>\n<root>\n<?xml version=\"1.0\"?>\n<x/>\n</root>";
        let out = strip_duplicate_xml_decls_and_fix_urls(input);
        assert_eq!(out.matches("<?xml").count(), 1);
        assert!(out.contains("<x/>"));
    }

    #[test]
    fn update_lastmod_no_match_leaves_line_unchanged() {
        let xml = "<url>\n<loc>https://example.com/other/</loc>\n<lastmod>2025-01-01</lastmod>\n</url>";
        let mut map = HashMap::new();
        let _ = map.insert("blog".to_string(), "2026-04-11".to_string());
        let result = update_lastmod_from_loc(xml, &map);
        assert!(
            result.contains("<lastmod>2025-01-01</lastmod>"),
            "non-matching loc should leave lastmod unchanged: {result}"
        );
    }

    #[test]
    fn update_lastmod_skips_empty_rel_path_match() {
        // Edge case: empty rel_path entries shouldn't match anything.
        let xml = "<url>\n<loc>https://example.com/x/</loc>\n<lastmod>2025-01-01</lastmod>\n</url>";
        let mut map = HashMap::new();
        let _ = map.insert(String::new(), "should-not-match".to_string());
        let result = update_lastmod_from_loc(xml, &map);
        assert!(result.contains("<lastmod>2025-01-01</lastmod>"));
        assert!(!result.contains("should-not-match"));
    }

    // -----------------------------------------------------------------
    // extract_best_date: empty date fields fail parsing silently
    // -----------------------------------------------------------------

    #[test]
    fn test_extract_best_date_empty_field_yields_verbatim_date() {
        crate::test_support::init_logger();
        let mut meta = HashMap::new();
        let _ = meta.insert("item_pub_date".to_string(), String::new());
        // The empty value fails the flexible chain without warning and
        // falls through to the verbatim `date` fallback (also absent).
        assert_eq!(extract_best_date(&meta), None);
    }

    // -----------------------------------------------------------------
    // strip_duplicate_xml_decls_and_fix_urls: <atom:link> lines
    // -----------------------------------------------------------------

    #[test]
    fn test_strip_normalises_atom_link_lines() {
        let content = "<?xml version=\"1.0\"?>\n<atom:link href=\"https://example.com//rss.xml\"/>\n";
        let out = strip_duplicate_xml_decls_and_fix_urls(content);
        assert!(
            out.contains("https://example.com/rss.xml"),
            "double slash in atom:link must be normalised: {out}"
        );
    }

    // -----------------------------------------------------------------
    // canonicalise_loc_urls: defensive early returns
    // -----------------------------------------------------------------

    #[test]
    fn test_canonicalise_loc_urls_without_loc_tag() {
        let line = "  <lastmod>2026-01-01</lastmod>";
        assert_eq!(canonicalise_loc_urls(line), line);
    }

    #[test]
    fn test_canonicalise_loc_urls_with_unclosed_loc() {
        let line = "  <loc>https://example.com/page";
        assert_eq!(canonicalise_loc_urls(line), line);
    }

    // -----------------------------------------------------------------
    // update_lastmod_from_loc: unclosed <loc> keeps previous state
    // -----------------------------------------------------------------

    #[test]
    fn test_update_lastmod_ignores_unclosed_loc_line() {
        let mut date_map = HashMap::new();
        let _ = date_map.insert("page".to_string(), "2026-02-02".to_string());
        let xml = "<url>\n<loc>https://example.com/page\n<lastmod>2020-01-01</lastmod>\n</url>\n";
        let out = update_lastmod_from_loc(xml, &date_map);
        assert!(
            out.contains("<lastmod>2020-01-01</lastmod>"),
            "unclosed <loc> must not update current_loc: {out}"
        );
    }

    // -----------------------------------------------------------------
    // Error paths
    // -----------------------------------------------------------------

    #[test]
    fn test_after_compile_errors_on_invalid_utf8_sitemap() {
        let tmp = tempdir().unwrap();
        let sitemap_path = tmp.path().join("sitemap.xml");
        fs::write(&sitemap_path, [0xFF, 0xFE, 0xFD]).unwrap();
        let ctx = test_ctx(tmp.path());
        let err = SitemapFixPlugin.after_compile(&ctx).unwrap_err();
        assert!(format!("{err}").contains("sitemap.xml"));
    }

    #[test]
    #[cfg(unix)]
    fn test_after_compile_write_failure_on_readonly_sitemap() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        let sitemap_path = tmp.path().join("sitemap.xml");
        fs::write(
            &sitemap_path,
            "<?xml version=\"1.0\"?>\n<urlset></urlset>\n",
        )
        .unwrap();
        fs::set_permissions(&sitemap_path, fs::Permissions::from_mode(0o444))
            .unwrap();

        let ctx = test_ctx(tmp.path());
        let result = SitemapFixPlugin.after_compile(&ctx);
        let _ = fs::set_permissions(
            &sitemap_path,
            fs::Permissions::from_mode(0o644),
        );
        let err = result.unwrap_err();
        assert!(format!("{err}").contains("sitemap.xml"));
    }
}
