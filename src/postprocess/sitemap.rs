// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Sitemap fix plugin.

use super::helpers::{
    normalise_url_in_xml_line, read_meta_sidecars, rfc2822_to_iso_date,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

/// Repairs sitemap.xml by removing duplicate XML declarations,
/// normalising double-slash URLs, and updating per-page lastmod dates.
#[derive(Debug, Clone, Copy)]
pub struct SitemapFixPlugin;

impl Plugin for SitemapFixPlugin {
    fn name(&self) -> &'static str {
        "sitemap-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let sitemap_path = ctx.site_dir.join("sitemap.xml");
        if !sitemap_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&sitemap_path).with_context(|| {
            format!("cannot read {}", sitemap_path.display())
        })?;

        let meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();
        let date_map = collect_date_map(&meta_entries);

        let result = strip_duplicate_xml_decls_and_fix_urls(&content);

        // Second pass: update lastmod based on the <loc> in each <url> block
        let updated = update_lastmod_from_loc(&result, &date_map);

        fs::write(&sitemap_path, updated).with_context(|| {
            format!("cannot write {}", sitemap_path.display())
        })?;

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
fn extract_best_date(meta: &HashMap<String, String>) -> Option<String> {
    meta.get("item_pub_date")
        .and_then(|d| rfc2822_to_iso_date(d))
        .or_else(|| {
            meta.get("last_build_date")
                .and_then(|d| rfc2822_to_iso_date(d))
        })
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

        let processed = if line.contains("<loc>")
            || line.contains("<link>")
            || line.contains("<atom:link")
        {
            normalise_url_in_xml_line(line)
        } else {
            line.to_string()
        };

        result.push_str(&processed);
        result.push('\n');
    }

    result
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
        let tmp = tempdir()?;
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
        )?;

        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&sitemap)?;
        assert_eq!(result.matches("<?xml").count(), 1);
        Ok(())
    }

    #[test]
    fn test_sitemap_fix_normalises_double_slashes() -> Result<()> {
        let tmp = tempdir()?;
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
        )?;

        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&sitemap)?;
        assert!(result.contains("https://example.com/index.html"));
        assert!(!result.contains("com//index"));
        Ok(())
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
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        SitemapFixPlugin.after_compile(&ctx)?;
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
}
