// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! News sitemap fix plugin.

use super::helpers::{read_meta_sidecars, rfc2822_to_iso8601, xml_escape};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Repairs news-sitemap.xml by populating entries from front-matter
/// metadata instead of using placeholder values.
#[derive(Debug, Clone, Copy)]
pub struct NewsSitemapFixPlugin;

impl Plugin for NewsSitemapFixPlugin {
    fn name(&self) -> &'static str {
        "news-sitemap-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let path = ctx.site_dir.join("news-sitemap.xml");
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;

        // If no placeholder issues, skip
        if !content.contains("Unnamed Publication")
            && !content.contains("Untitled Article")
            && !content.contains("<loc></loc>")
        {
            return Ok(());
        }

        let meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();

        // Get base_url from config
        let base_url = ctx
            .config
            .as_ref()
            .map(|c| c.base_url.trim_end_matches('/').to_string())
            .unwrap_or_default();

        // Build news entries from metadata
        let news_entries: Vec<String> = meta_entries
            .iter()
            .filter_map(|(rel_path, meta)| {
                build_news_entry(rel_path, meta, &base_url)
            })
            .collect();

        if news_entries.is_empty() {
            return Ok(());
        }

        // Rebuild the news sitemap
        let rebuilt = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9">
{}
</urlset>
"#,
            news_entries.join("\n")
        );

        fs::write(&path, rebuilt)
            .with_context(|| format!("cannot write {}", path.display()))?;

        log::info!(
            "[news-sitemap-fix] Rebuilt news-sitemap.xml with {} entries",
            news_entries.len()
        );
        Ok(())
    }
}

/// Builds a single `<url>` entry for the news sitemap from metadata.
fn build_news_entry(
    rel_path: &str,
    meta: &std::collections::HashMap<String, String>,
    base_url: &str,
) -> Option<String> {
    let title = meta.get("title").cloned().unwrap_or_default();
    let name = meta
        .get("author")
        .or_else(|| meta.get("name"))
        .cloned()
        .unwrap_or_default();
    let language = meta
        .get("language")
        .cloned()
        .unwrap_or_else(|| "en".to_string());

    if title.is_empty() || rel_path.is_empty() {
        return None;
    }

    let pub_date = meta
        .get("item_pub_date")
        .map(|d| rfc2822_to_iso8601(d))
        .unwrap_or_default();

    let loc = if base_url.is_empty() {
        format!("{rel_path}/index.html")
    } else {
        format!("{base_url}/{rel_path}/index.html")
    };

    let keywords = meta
        .get("keywords")
        .or_else(|| meta.get("tags"))
        .cloned()
        .unwrap_or_default();
    let extras = if keywords.is_empty() {
        String::new()
    } else {
        format!(
            "\n    <news:keywords>{}</news:keywords>",
            xml_escape(&keywords)
        )
    };

    Some(format!(
        r"<url>
  <loc>{loc}</loc>
  <news:news>
    <news:publication>
      <news:name>{name}</news:name>
      <news:language>{language}</news:language>
    </news:publication>
    <news:publication_date>{pub_date}</news:publication_date>
    <news:title>{title}</news:title>{extras}
  </news:news>
</url>"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_meta_sidecar(
        dir: &Path,
        slug: &str,
        meta: &HashMap<String, String>,
    ) {
        let page_dir = dir.join(slug);
        fs::create_dir_all(&page_dir).expect("create page dir");
        let meta_path = page_dir.join("page.meta.json");
        let json = serde_json::to_string(meta).expect("serialize meta");
        fs::write(&meta_path, json).expect("write meta");
    }

    fn make_atom_ctx(site_dir: &Path) -> PluginContext {
        crate::test_support::init_logger();
        let config = crate::cmd::SsgConfig {
            base_url: "https://example.com".to_string(),
            site_name: "Test Site".to_string(),
            site_title: "Test Site".to_string(),
            site_description: "A test site".to_string(),
            language: "en".to_string(),
            content_dir: std::path::PathBuf::from("content"),
            output_dir: std::path::PathBuf::from("build"),
            template_dir: std::path::PathBuf::from("templates"),
            serve_dir: None,
            i18n: None,
        };
        PluginContext::with_config(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
            config,
        )
    }

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
    fn test_news_sitemap_with_keywords() -> Result<()> {
        let tmp = tempdir()?;

        let news_path = tmp.path().join("news-sitemap.xml");
        fs::write(
            &news_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9">
<url>
  <loc></loc>
  <news:news>
    <news:publication>
      <news:name>Unnamed Publication</news:name>
      <news:language>en</news:language>
    </news:publication>
    <news:title>Untitled Article</news:title>
  </news:news>
</url>
</urlset>"#,
        )?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Breaking News".to_string());
        let _ = meta.insert("author".to_string(), "Reporter".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert(
            "keywords".to_string(),
            "rust, programming, web".to_string(),
        );
        let _ = meta.insert("language".to_string(), "fr".to_string());
        write_meta_sidecar(tmp.path(), "breaking", &meta);

        let ctx = make_atom_ctx(tmp.path());
        NewsSitemapFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&news_path)?;
        assert!(
            result.contains(
                "<news:keywords>rust, programming, web</news:keywords>"
            ),
            "Should inject keywords: {result}"
        );
        assert!(
            result.contains("<news:name>Reporter</news:name>"),
            "Should use author name: {result}"
        );
        assert!(
            result.contains("<news:language>fr</news:language>"),
            "Should use custom language: {result}"
        );
        assert!(
            !result.contains("Unnamed Publication"),
            "Should not have placeholder: {result}"
        );
        assert!(
            !result.contains("Untitled Article"),
            "Should not have placeholder: {result}"
        );
        Ok(())
    }

    #[test]
    fn test_news_sitemap_with_tags_fallback() -> Result<()> {
        let tmp = tempdir()?;

        let news_path = tmp.path().join("news-sitemap.xml");
        fs::write(
            &news_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:news="http://www.google.com/schemas/sitemap-news/0.9">
<url>
  <loc></loc>
  <news:news>
    <news:title>Untitled Article</news:title>
  </news:news>
</url>
</urlset>"#,
        )?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Tagged Post".to_string());
        let _ = meta.insert("author".to_string(), "Writer".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Mon, 01 Sep 2025 12:00:00 +0000".to_string(),
        );
        let _ = meta.insert("tags".to_string(), "tech, science".to_string());
        write_meta_sidecar(tmp.path(), "tagged", &meta);

        let ctx = make_atom_ctx(tmp.path());
        NewsSitemapFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&news_path)?;
        assert!(
            result.contains("<news:keywords>tech, science</news:keywords>"),
            "Should fall back to tags for keywords: {result}"
        );
        Ok(())
    }

    #[test]
    fn test_news_sitemap_skips_when_no_placeholders() -> Result<()> {
        let tmp = tempdir()?;

        let news_path = tmp.path().join("news-sitemap.xml");
        let original = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<url>
  <loc>https://example.com/good</loc>
  <news:news>
    <news:title>Good Article</news:title>
  </news:news>
</url>
</urlset>"#;
        fs::write(&news_path, original)?;

        let ctx = test_ctx(tmp.path());
        NewsSitemapFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&news_path)?;
        assert_eq!(
            result, original,
            "Should not modify well-formed news sitemap"
        );
        Ok(())
    }
}
