// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RSS aggregate plugin.

use super::helpers::{
    extract_xml_value, parse_rfc2822_lenient, read_meta_sidecars, xml_escape,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Aggregates per-page RSS items into the root `rss.xml` feed.
#[derive(Debug, Clone, Copy)]
pub struct RssAggregatePlugin;

/// Builds a list of `(sort_key, xml_item)` pairs from metadata entries.
fn collect_articles(
    meta_entries: &[(String, std::collections::HashMap<String, String>)],
    base_url: &str,
) -> Vec<(String, String)> {
    let mut articles: Vec<(String, String)> = Vec::new();
    for (rel_path, meta) in meta_entries {
        if rel_path.is_empty() {
            continue;
        }

        let title = meta.get("title").cloned().unwrap_or_default();
        let description = meta.get("description").cloned().unwrap_or_default();
        let pub_date = meta.get("item_pub_date").cloned().unwrap_or_default();
        let author = meta.get("author").cloned().unwrap_or_default();
        let banner = meta.get("banner").or_else(|| meta.get("image")).cloned();
        let category = meta.get("category").cloned();
        let tags = meta.get("tags").cloned();

        if title.is_empty() {
            continue;
        }

        let link = if base_url.is_empty() {
            format!("{rel_path}/")
        } else {
            format!("{base_url}/{rel_path}/")
        };

        let sort_key = parse_rfc2822_lenient(&pub_date)
            .map_or_else(|| pub_date.clone(), |dt| dt.to_rfc3339());

        let escaped_desc = xml_escape(&description);

        // Build optional elements
        let mut extras = String::new();

        // Enclosure for banner/image (P2 fix)
        if let Some(ref img) = banner {
            let img_url = if img.starts_with("http") {
                img.clone()
            } else if !base_url.is_empty() {
                format!("{base_url}/{}", img.trim_start_matches('/'))
            } else {
                img.clone()
            };
            let mime = if img_url.ends_with(".webp") {
                "image/webp"
            } else if img_url.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            extras.push_str(&format!(
                "\n      <enclosure url=\"{img_url}\" type=\"{mime}\" length=\"0\"/>"
            ));
        }

        // Category elements (P2 fix)
        if let Some(ref cat) = category {
            extras.push_str(&format!(
                "\n      <category>{}</category>",
                xml_escape(cat)
            ));
        }
        if let Some(ref t) = tags {
            for tag in t.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    extras.push_str(&format!(
                        "\n      <category>{}</category>",
                        xml_escape(tag)
                    ));
                }
            }
        }

        let item = format!(
            r#"    <item>
      <title>{title}</title>
      <link>{link}</link>
      <description>{escaped_desc}</description>
      <guid isPermaLink="true">{link}</guid>
      <pubDate>{pub_date}</pubDate>
      <author>{author}</author>{extras}
    </item>"#
        );

        articles.push((sort_key, item));
    }
    articles
}

/// Formats the final RSS XML channel document.
fn build_rss_channel(
    channel_title: &str,
    channel_link: &str,
    channel_desc: &str,
    base_url: &str,
    language: &str,
    last_build_date: &str,
    copyright: &str,
    items_xml: &str,
) -> String {
    let mut channel_extras = String::new();
    if !language.is_empty() {
        channel_extras
            .push_str(&format!("\n    <language>{language}</language>"));
    }
    if !last_build_date.is_empty() {
        channel_extras.push_str(&format!(
            "\n    <lastBuildDate>{last_build_date}</lastBuildDate>"
        ));
    }
    if !copyright.is_empty() {
        channel_extras.push_str(&format!(
            "\n    <copyright>{}</copyright>",
            xml_escape(copyright)
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>{channel_title}</title>
    <link>{channel_link}</link>
    <description>{channel_desc}</description>
    <atom:link href="{base_url}/rss.xml" rel="self" type="application/rss+xml"/>{channel_extras}
{items_xml}
  </channel>
</rss>
"#
    )
}

impl Plugin for RssAggregatePlugin {
    fn name(&self) -> &'static str {
        "rss-aggregate"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let rss_path = ctx.site_dir.join("rss.xml");
        if !rss_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&rss_path)
            .with_context(|| format!("cannot read {}", rss_path.display()))?;

        if content.matches("<item>").count() > 1 {
            return Ok(());
        }

        let meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();

        let base_url = ctx
            .config
            .as_ref()
            .map(|c| c.base_url.trim_end_matches('/').to_string())
            .unwrap_or_default();

        let language = extract_language(ctx);
        let copyright = extract_copyright(&meta_entries);

        let mut articles = collect_articles(&meta_entries, &base_url);
        articles.sort_by(|a, b| b.0.cmp(&a.0));
        articles.truncate(50);

        if articles.is_empty() {
            return Ok(());
        }

        let last_build_date = extract_last_build_date(&articles);

        let items_xml: String = articles
            .iter()
            .map(|(_, xml)| xml.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let channel_title = extract_xml_value(&content, "title")
            .unwrap_or_else(|| "Untitled".to_string());
        let channel_link = extract_xml_value(&content, "link")
            .unwrap_or_else(|| base_url.clone());
        let channel_desc =
            extract_xml_value(&content, "description").unwrap_or_default();

        let rebuilt = build_rss_channel(
            &channel_title,
            &channel_link,
            &channel_desc,
            &base_url,
            &language,
            &last_build_date,
            &copyright,
            &items_xml,
        );

        fs::write(&rss_path, rebuilt)
            .with_context(|| format!("cannot write {}", rss_path.display()))?;

        log::info!(
            "[rss-aggregate] Rebuilt rss.xml with {} article items",
            articles.len()
        );
        Ok(())
    }
}

/// Extracts the language setting from the plugin context.
fn extract_language(ctx: &PluginContext) -> String {
    ctx.config
        .as_ref()
        .and_then(|c| {
            if c.site_name.is_empty() {
                None
            } else {
                Some("en".to_string())
            }
        })
        .unwrap_or_else(|| "en".to_string())
}

/// Extracts the copyright string from meta entries.
fn extract_copyright(
    meta_entries: &[(String, std::collections::HashMap<String, String>)],
) -> String {
    meta_entries
        .iter()
        .find_map(|(_, m)| m.get("copyright").cloned())
        .unwrap_or_default()
}

/// Extracts the last build date from the most recent article.
fn extract_last_build_date(articles: &[(String, String)]) -> String {
    articles
        .first()
        .and_then(|(_, xml)| {
            xml.find("<pubDate>").and_then(|s| {
                let after = &xml[s + 9..];
                after.find("</pubDate>").map(|e| after[..e].to_string())
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
    fn test_rss_aggregate_single_item_trigger() -> Result<()> {
        let tmp = tempdir()?;
        let rss_path = tmp.path().join("rss.xml");
        fs::write(
            &rss_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>My Site</title>
    <link>https://example.com</link>
    <description>A test site</description>
    <item>
      <title>Feed itself</title>
      <link>https://example.com/rss.xml</link>
    </item>
  </channel>
</rss>"#,
        )?;

        let ctx = test_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_rss_aggregate_with_full_metadata() -> Result<()> {
        let tmp = tempdir()?;

        let rss_path = tmp.path().join("rss.xml");
        fs::write(
            &rss_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Blog</title>
    <link>https://example.com</link>
    <description>A test blog</description>
    <item>
      <title>Placeholder</title>
    </item>
  </channel>
</rss>"#,
        )?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Article One".to_string());
        let _ = meta.insert(
            "description".to_string(),
            "First article desc".to_string(),
        );
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Alice".to_string());
        let _ = meta
            .insert("banner".to_string(), "/images/banner.webp".to_string());
        let _ = meta.insert("category".to_string(), "Technology".to_string());
        let _ = meta.insert("tags".to_string(), "rust, web".to_string());
        let _ = meta.insert(
            "copyright".to_string(),
            "Copyright 2026 Alice".to_string(),
        );
        write_meta_sidecar(tmp.path(), "article-one", &meta);

        let ctx = make_atom_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&rss_path)?;

        assert!(
            result.contains(
                "<enclosure url=\"https://example.com/images/banner.webp\""
            ),
            "Should have enclosure with base_url prefix: {result}"
        );
        assert!(
            result.contains("type=\"image/webp\""),
            "Should detect webp MIME type: {result}"
        );
        assert!(
            result.contains("<category>Technology</category>"),
            "Should have category element: {result}"
        );
        assert!(
            result.contains("<category>rust</category>"),
            "Should have tag category 'rust': {result}"
        );
        assert!(
            result.contains("<category>web</category>"),
            "Should have tag category 'web': {result}"
        );
        assert!(
            result.contains("<language>en</language>"),
            "Should have language element: {result}"
        );
        assert!(
            result.contains("<lastBuildDate>"),
            "Should have lastBuildDate: {result}"
        );
        assert!(
            result.contains("<copyright>Copyright 2026 Alice</copyright>"),
            "Should have copyright: {result}"
        );

        Ok(())
    }

    #[test]
    fn test_rss_aggregate_banner_with_image_field() -> Result<()> {
        let tmp = tempdir()?;

        let rss_path = tmp.path().join("rss.xml");
        fs::write(
            &rss_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>T</title><link>https://example.com</link><description>D</description><item><title>X</title></item></channel></rss>"#,
        )?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Image Test".to_string());
        let _ =
            meta.insert("description".to_string(), "Testing image".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Mon, 01 Sep 2025 12:00:00 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Bob".to_string());
        let _ = meta.insert(
            "image".to_string(),
            "https://cdn.example.com/photo.png".to_string(),
        );
        write_meta_sidecar(tmp.path(), "img-test", &meta);

        let ctx = make_atom_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&rss_path)?;
        assert!(
            result.contains("url=\"https://cdn.example.com/photo.png\""),
            "Should use absolute image URL as-is: {result}"
        );
        assert!(
            result.contains("type=\"image/png\""),
            "Should detect png MIME type: {result}"
        );
        Ok(())
    }

    #[test]
    fn test_rss_aggregate_jpeg_mime() -> Result<()> {
        let tmp = tempdir()?;

        let rss_path = tmp.path().join("rss.xml");
        fs::write(
            &rss_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>T</title><link>https://example.com</link><description>D</description><item><title>X</title></item></channel></rss>"#,
        )?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "JPEG Test".to_string());
        let _ = meta.insert("description".to_string(), "desc".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Mon, 01 Sep 2025 12:00:00 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Carol".to_string());
        let _ = meta.insert("banner".to_string(), "/img/photo.jpg".to_string());
        write_meta_sidecar(tmp.path(), "jpeg-test", &meta);

        let ctx = make_atom_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&rss_path)?;
        assert!(
            result.contains("type=\"image/jpeg\""),
            "Should default to image/jpeg for .jpg: {result}"
        );
        Ok(())
    }

    #[test]
    fn test_rss_aggregate_skips_multi_item() -> Result<()> {
        let tmp = tempdir()?;

        let rss_path = tmp.path().join("rss.xml");
        let original = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>T</title><link>x</link><description>D</description>
<item><title>A</title></item>
<item><title>B</title></item>
</channel></rss>"#;
        fs::write(&rss_path, original)?;

        let ctx = test_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&rss_path)?;
        assert_eq!(result, original, "Should not modify feed with >1 items");
        Ok(())
    }

    #[test]
    fn test_collect_articles_empty_entries() {
        let articles = collect_articles(&[], "https://example.com");
        assert!(
            articles.is_empty(),
            "no meta entries should produce no articles"
        );
    }

    #[test]
    fn test_collect_articles_skips_empty_title() {
        let mut meta = HashMap::new();
        let _ =
            meta.insert("description".to_string(), "no title here".to_string());
        let entries = vec![("page".to_string(), meta)];
        let articles = collect_articles(&entries, "https://example.com");
        assert!(
            articles.is_empty(),
            "entries without title should be skipped"
        );
    }

    #[test]
    fn test_collect_articles_skips_empty_path() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Has Title".to_string());
        let entries = vec![(String::new(), meta)];
        let articles = collect_articles(&entries, "https://example.com");
        assert!(
            articles.is_empty(),
            "entries with empty path should be skipped"
        );
    }

    #[test]
    fn test_collect_articles_multiple_entries_sorted() {
        let mut meta1 = HashMap::new();
        let _ = meta1.insert("title".to_string(), "Older".to_string());
        let _ = meta1.insert("description".to_string(), "old".to_string());
        let _ = meta1.insert(
            "item_pub_date".to_string(),
            "Mon, 01 Jan 2024 00:00:00 +0000".to_string(),
        );
        let _ = meta1.insert("author".to_string(), "A".to_string());

        let mut meta2 = HashMap::new();
        let _ = meta2.insert("title".to_string(), "Newer".to_string());
        let _ = meta2.insert("description".to_string(), "new".to_string());
        let _ = meta2.insert(
            "item_pub_date".to_string(),
            "Wed, 01 Jan 2025 00:00:00 +0000".to_string(),
        );
        let _ = meta2.insert("author".to_string(), "B".to_string());

        let entries = vec![
            ("old-post".to_string(), meta1),
            ("new-post".to_string(), meta2),
        ];
        let mut articles = collect_articles(&entries, "https://example.com");
        assert_eq!(articles.len(), 2);

        // Sort descending like the plugin does
        articles.sort_by(|a, b| b.0.cmp(&a.0));
        assert!(
            articles[0].1.contains("<title>Newer</title>"),
            "newest article should sort first"
        );
    }

    #[test]
    fn test_collect_articles_xml_escapes_description() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Escape Test".to_string());
        let _ = meta.insert(
            "description".to_string(),
            "Use <b>bold</b> & \"quotes\"".to_string(),
        );
        let _ = meta.insert("author".to_string(), "X".to_string());
        let entries = vec![("esc".to_string(), meta)];
        let articles = collect_articles(&entries, "");
        assert_eq!(articles.len(), 1);
        let xml = &articles[0].1;
        assert!(
            xml.contains("&lt;b&gt;bold&lt;/b&gt;"),
            "angle brackets should be escaped: {xml}"
        );
        assert!(xml.contains("&amp;"), "ampersands should be escaped: {xml}");
    }

    #[test]
    fn test_build_rss_channel_minimal() {
        let result = build_rss_channel(
            "Title",
            "https://x.example",
            "Desc",
            "https://x.example",
            "",
            "",
            "",
            "",
        );
        assert!(result.contains("<title>Title</title>"));
        assert!(result.contains("<link>https://x.example</link>"));
        assert!(result.contains("<description>Desc</description>"));
        assert!(
            !result.contains("<language>"),
            "no language when empty string supplied"
        );
        assert!(
            !result.contains("<lastBuildDate>"),
            "no lastBuildDate when empty string supplied"
        );
    }

    #[test]
    fn test_build_rss_channel_with_all_extras() {
        let result = build_rss_channel(
            "T",
            "L",
            "D",
            "https://x.example",
            "en",
            "Mon, 01 Jan 2024 00:00:00 +0000",
            "Copyright 2024 X",
            "<item><title>A</title></item>",
        );
        assert!(result.contains("<language>en</language>"));
        assert!(result.contains(
            "<lastBuildDate>Mon, 01 Jan 2024 00:00:00 +0000</lastBuildDate>"
        ));
        assert!(result.contains("<copyright>Copyright 2024 X</copyright>"));
        assert!(result.contains("<item><title>A</title></item>"));
    }

    #[test]
    fn test_extract_last_build_date_from_articles() {
        let articles = vec![
            ("2025".to_string(), "<item><pubDate>Mon, 01 Sep 2025 12:00:00 +0000</pubDate></item>".to_string()),
            ("2024".to_string(), "<item><pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate></item>".to_string()),
        ];
        let date = extract_last_build_date(&articles);
        assert_eq!(date, "Mon, 01 Sep 2025 12:00:00 +0000");
    }

    #[test]
    fn test_extract_last_build_date_empty() {
        let articles: Vec<(String, String)> = vec![];
        let date = extract_last_build_date(&articles);
        assert!(date.is_empty());
    }

    #[test]
    fn test_rss_no_file_is_noop() -> Result<()> {
        let tmp = tempdir()?;
        // No rss.xml exists
        let ctx = test_ctx(tmp.path());
        RssAggregatePlugin.after_compile(&ctx)?;
        assert!(!tmp.path().join("rss.xml").exists());
        Ok(())
    }
}
