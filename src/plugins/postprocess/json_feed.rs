// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON Feed 1.1 plugin.
//!
//! Emits a `feed.json` file at the site root conforming to the
//! [JSON Feed 1.1 spec](https://jsonfeed.org/version/1.1).
//!
//! Runs alongside `RssAggregatePlugin` and `AtomFeedPlugin` in
//! `after_compile`, reading the same `.meta.json` sidecars (with the
//! same `build_dir/.meta` and `rss.xml` fallbacks).

use super::helpers::{parse_rfc2822_lenient, read_meta_sidecars};
use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use crate::util::head_dom::inject_before_head_close;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// JSON Feed version URL — the required top-level `version` field.
const JSON_FEED_VERSION: &str = "https://jsonfeed.org/version/1.1";

/// Maximum number of items emitted per feed (matches RSS/Atom).
const MAX_ITEMS: usize = 50;

/// Generates a JSON Feed 1.1 `feed.json` from `.meta.json` sidecars.
///
/// Runs in `after_compile`, alongside `RssAggregatePlugin` and
/// `AtomFeedPlugin`. Mirrors the sidecar discovery logic of
/// `AtomFeedPlugin` (`site_dir` → `build_dir/.meta` → `rss.xml` fallback)
/// so the three feeds stay in sync.
#[derive(Debug, Clone, Copy)]
pub struct JsonFeedPlugin;

impl Plugin for JsonFeedPlugin {
    fn name(&self) -> &'static str {
        "json-feed"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        let mut meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();

        if meta_entries.is_empty() {
            let meta_dir = ctx.build_dir.join(".meta");
            if meta_dir.exists() {
                meta_entries =
                    read_meta_sidecars(&meta_dir).unwrap_or_default();
            }
        }

        if meta_entries.is_empty() {
            meta_entries = super::atom::extract_entries_from_rss(&ctx.site_dir);
        }

        let base_url = ctx
            .config
            .as_ref()
            .map(|c| c.base_url.trim_end_matches('/').to_string())
            .unwrap_or_default();

        let site_name = ctx
            .config
            .as_ref()
            .map(|c| c.site_name.clone())
            .unwrap_or_default();
        let feed_title = if site_name.is_empty() {
            "Untitled".to_string()
        } else {
            site_name
        };

        let default_locale = extract_default_locale(ctx);
        let known_locales = extract_known_locales(ctx);

        let mut items = collect_items(&meta_entries, &base_url, &known_locales);
        items.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
        items.truncate(MAX_ITEMS);

        if items.is_empty() {
            return Ok(());
        }

        let feed_url = if base_url.is_empty() {
            "feed.json".to_string()
        } else {
            format!("{base_url}/feed.json")
        };
        let home_page_url = if base_url.is_empty() {
            "/".to_string()
        } else {
            format!("{base_url}/")
        };

        let feed_json = build_feed_json(
            &feed_title,
            &home_page_url,
            &feed_url,
            &default_locale,
            &items,
        );

        let feed_path = ctx.site_dir.join("feed.json");
        let serialized = serde_json::to_string_pretty(&feed_json)
            .unwrap_or_else(|_| feed_json.to_string());
        fs::write(&feed_path, serialized).with_path(&feed_path)?;

        inject_json_feed_link(&ctx.site_dir, &feed_url)?;

        log::info!(
            "[json-feed] Generated feed.json with {} items",
            items.len()
        );
        Ok(())
    }
}

/// A single JSON Feed item ready for serialisation.
pub(super) struct JsonFeedItem {
    pub sort_key: String,
    pub id: String,
    pub url: String,
    pub title: String,
    pub content_html: String,
    pub date_published: String,
    pub date_modified: String,
    pub author: String,
    pub tags: Vec<String>,
    pub language: Option<String>,
}

impl JsonFeedItem {
    /// Convert to a `serde_json::Value` for the items array.
    pub(super) fn to_json(&self) -> Value {
        let mut obj = Map::new();
        let _ = obj.insert("id".into(), Value::String(self.id.clone()));
        let _ = obj.insert("url".into(), Value::String(self.url.clone()));
        let _ = obj.insert("title".into(), Value::String(self.title.clone()));
        let _ = obj.insert(
            "content_html".into(),
            Value::String(self.content_html.clone()),
        );
        let _ = obj.insert(
            "date_published".into(),
            Value::String(self.date_published.clone()),
        );
        let _ = obj.insert(
            "date_modified".into(),
            Value::String(self.date_modified.clone()),
        );

        // authors[] — JSON Feed 1.1 uses an array.
        let author_name = if self.author.is_empty() {
            "Unknown".to_string()
        } else {
            self.author.clone()
        };
        let _ = obj.insert(
            "authors".into(),
            Value::Array(vec![json!({ "name": author_name })]),
        );

        // tags[] — always present (may be empty).
        let _ = obj.insert(
            "tags".into(),
            Value::Array(
                self.tags.iter().map(|t| Value::String(t.clone())).collect(),
            ),
        );

        if let Some(ref lang) = self.language {
            let _ = obj.insert("language".into(), Value::String(lang.clone()));
        }

        Value::Object(obj)
    }
}

/// Builds the top-level feed `Value`.
pub(super) fn build_feed_json(
    title: &str,
    home_page_url: &str,
    feed_url: &str,
    language: &str,
    items: &[JsonFeedItem],
) -> Value {
    let items_json: Vec<Value> =
        items.iter().map(JsonFeedItem::to_json).collect();

    let mut feed = Map::new();
    let _ = feed.insert(
        "version".into(),
        Value::String(JSON_FEED_VERSION.to_string()),
    );
    let _ = feed.insert("title".into(), Value::String(title.to_string()));
    let _ = feed.insert(
        "home_page_url".into(),
        Value::String(home_page_url.to_string()),
    );
    let _ = feed.insert("feed_url".into(), Value::String(feed_url.to_string()));
    if !language.is_empty() {
        let _ =
            feed.insert("language".into(), Value::String(language.to_string()));
    }
    let _ = feed.insert("items".into(), Value::Array(items_json));
    Value::Object(feed)
}

/// Collects JSON Feed items from metadata sidecars.
pub(super) fn collect_items(
    meta_entries: &[(String, HashMap<String, String>)],
    base_url: &str,
    known_locales: &[String],
) -> Vec<JsonFeedItem> {
    meta_entries
        .iter()
        .filter_map(|(rel_path, meta)| {
            build_item(rel_path, meta, base_url, known_locales)
        })
        .collect()
}

/// Builds a single `JsonFeedItem` from metadata, or `None` if invalid.
pub(super) fn build_item(
    rel_path: &str,
    meta: &HashMap<String, String>,
    base_url: &str,
    known_locales: &[String],
) -> Option<JsonFeedItem> {
    if rel_path.is_empty() {
        return None;
    }
    let title = meta.get("title").cloned().unwrap_or_default();
    if title.is_empty() {
        return None;
    }

    let description = meta.get("description").cloned().unwrap_or_default();
    let pub_date = meta.get("item_pub_date").cloned().unwrap_or_default();
    let modified_date = meta
        .get("last_build_date")
        .or_else(|| meta.get("date_modified"))
        .cloned()
        .unwrap_or_else(|| pub_date.clone());
    let author = meta.get("author").cloned().unwrap_or_default();

    let url = if base_url.is_empty() {
        format!("{rel_path}/")
    } else {
        format!("{base_url}/{rel_path}/")
    };

    let date_published = parse_rfc2822_lenient(&pub_date)
        .map_or_else(|| pub_date.clone(), |dt| dt.to_rfc3339());
    let date_modified = parse_rfc2822_lenient(&modified_date)
        .map_or_else(|| modified_date.clone(), |dt| dt.to_rfc3339());

    // Tags: prefer "tags" (comma-separated), fall back to "category".
    let mut tags: Vec<String> = meta
        .get("tags")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if tags.is_empty() {
        if let Some(cat) = meta.get("category") {
            let trimmed = cat.trim();
            if !trimmed.is_empty() {
                tags.push(trimmed.to_string());
            }
        }
    }

    // Per-item language: explicit `language`/`locale` meta, else
    // derived from path prefix matching a known locale (e.g. "fr/...").
    let language = meta
        .get("language")
        .or_else(|| meta.get("locale"))
        .cloned()
        .or_else(|| detect_locale_from_path(rel_path, known_locales));

    Some(JsonFeedItem {
        sort_key: date_published.clone(),
        id: url.clone(),
        url,
        title,
        content_html: description,
        date_published,
        date_modified,
        author,
        tags,
        language,
    })
}

/// Resolve the top-level feed language: prefer `i18n.default_locale`,
/// then `config.language`, then `"en"`.
pub(super) fn extract_default_locale(ctx: &PluginContext) -> String {
    if let Some(cfg) = ctx.config.as_ref() {
        if let Some(ref i18n) = cfg.i18n {
            if !i18n.default_locale.is_empty() {
                return i18n.default_locale.clone();
            }
        }
        if !cfg.language.is_empty() {
            return cfg.language.clone();
        }
    }
    "en".to_string()
}

/// Get the list of configured locales (for per-item language detection).
pub(super) fn extract_known_locales(ctx: &PluginContext) -> Vec<String> {
    ctx.config
        .as_ref()
        .and_then(|c| c.i18n.as_ref())
        .map(|i| i.locales.clone())
        .unwrap_or_default()
}

/// If the first path segment matches a known locale code, return it.
fn detect_locale_from_path(
    rel_path: &str,
    known_locales: &[String],
) -> Option<String> {
    let first = rel_path.split('/').next()?;
    if known_locales.iter().any(|l| l == first) {
        Some(first.to_string())
    } else {
        None
    }
}

/// Inject `<link rel="alternate" type="application/feed+json">` into
/// every HTML page under `site_dir` that doesn't already have one.
pub(super) fn inject_json_feed_link(
    site_dir: &Path,
    feed_url: &str,
) -> Result<(), SsgError> {
    let html_files = crate::walk::walk_files(site_dir, "html")
        .map_err(|e| SsgError::io(e, site_dir))?;
    for path in &html_files {
        let html = fs::read_to_string(path).with_path(path)?;

        if html.contains("application/feed+json") {
            continue;
        }
        let link_tag = format!(
            "  <link rel=\"alternate\" type=\"application/feed+json\" title=\"JSON Feed\" href=\"{feed_url}\"/>\n"
        );
        let modified = inject_before_head_close(&html, &link_tag);
        if modified != html {
            fs::write(path, &modified).with_path(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use anyhow::Result;
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

    fn make_ctx(site_dir: &Path) -> PluginContext {
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
            cdn_prefix: None,
            image: crate::cmd::ImageConfig::default(),
            edge_headers: crate::cmd::EdgeHeadersConfig::default(),
        };
        PluginContext::with_config(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
            config,
        )
    }

    #[test]
    fn test_json_feed_top_level_fields() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Hello World".to_string());
        let _ = meta.insert(
            "description".to_string(),
            "<p>A test post</p>".to_string(),
        );
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Alice".to_string());
        let _ = meta.insert("tags".to_string(), "rust, web".to_string());
        write_meta_sidecar(tmp.path(), "hello", &meta);

        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;

        let feed_path = tmp.path().join("feed.json");
        assert!(feed_path.exists(), "feed.json should be created");

        let raw = fs::read_to_string(&feed_path)?;
        let value: Value = serde_json::from_str(&raw)?;
        assert_eq!(value["version"], JSON_FEED_VERSION);
        assert_eq!(value["title"], "Test Site");
        assert_eq!(value["home_page_url"], "https://example.com/");
        assert_eq!(value["feed_url"], "https://example.com/feed.json");
        assert_eq!(value["language"], "en");
        assert!(value["items"].is_array());
        assert_eq!(value["items"].as_array().unwrap().len(), 1);
        Ok(())
    }

    #[test]
    fn test_json_feed_item_required_fields() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Item Test".to_string());
        let _ =
            meta.insert("description".to_string(), "<p>body</p>".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Bob".to_string());
        let _ = meta.insert("tags".to_string(), "alpha".to_string());
        write_meta_sidecar(tmp.path(), "item-test", &meta);

        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;

        let value: Value = serde_json::from_str(&fs::read_to_string(
            tmp.path().join("feed.json"),
        )?)?;
        let item = &value["items"][0];

        assert!(item["id"].is_string());
        assert!(item["url"].is_string());
        assert_eq!(item["title"], "Item Test");
        assert_eq!(item["content_html"], "<p>body</p>");
        assert_eq!(item["date_published"], "2026-04-11T06:06:06+00:00");
        assert_eq!(item["date_modified"], "2026-04-11T06:06:06+00:00");

        let authors = item["authors"].as_array().unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0]["name"], "Bob");

        let tags = item["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "alpha");
        Ok(())
    }

    #[test]
    fn test_json_feed_injects_link_into_html() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Link Test".to_string());
        let _ = meta.insert("description".to_string(), "x".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        write_meta_sidecar(tmp.path(), "linktest", &meta);

        let html_path = tmp.path().join("index.html");
        fs::write(
            &html_path,
            "<html><head><title>T</title></head><body></body></html>",
        )?;

        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;

        let html = fs::read_to_string(&html_path)?;
        assert!(
            html.contains("application/feed+json"),
            "missing feed+json link tag in: {html}"
        );
        assert!(html.contains("href=\"https://example.com/feed.json\""));
        Ok(())
    }

    #[test]
    fn test_json_feed_empty_site_dir() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;
        assert!(!tmp.path().join("feed.json").exists());
        Ok(())
    }

    #[test]
    fn test_json_feed_sorts_descending_and_truncates() -> Result<()> {
        let tmp = tempdir()?;
        for i in 0..60 {
            let mut meta = HashMap::new();
            let _ = meta.insert("title".to_string(), format!("P{i}"));
            let _ = meta.insert("description".to_string(), format!("body {i}"));
            let _ = meta.insert(
                "item_pub_date".to_string(),
                format!(
                    "Thu, {:02} Apr 2026 {:02}:00:00 +0000",
                    (i % 28) + 1,
                    i % 24
                ),
            );
            write_meta_sidecar(tmp.path(), &format!("post-{i:03}"), &meta);
        }

        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;
        let value: Value = serde_json::from_str(&fs::read_to_string(
            tmp.path().join("feed.json"),
        )?)?;
        let items = value["items"].as_array().unwrap();
        assert_eq!(items.len(), MAX_ITEMS);
        // sorted descending => first sort_key >= last
        let first = items[0]["date_published"].as_str().unwrap();
        let last = items[items.len() - 1]["date_published"].as_str().unwrap();
        assert!(first >= last);
        Ok(())
    }

    #[test]
    fn test_json_feed_empty_author_shows_unknown() -> Result<()> {
        let tmp = tempdir()?;
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        let _ = meta.insert("description".to_string(), "b".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        write_meta_sidecar(tmp.path(), "noauth", &meta);

        let ctx = make_ctx(tmp.path());
        JsonFeedPlugin.after_compile(&ctx)?;
        let value: Value = serde_json::from_str(&fs::read_to_string(
            tmp.path().join("feed.json"),
        )?)?;
        assert_eq!(value["items"][0]["authors"][0]["name"], "Unknown");
        Ok(())
    }

    #[test]
    fn test_json_feed_locale_detection_from_path() {
        let known = vec!["en".to_string(), "fr".to_string()];
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Bonjour".to_string());
        let _ = meta.insert("description".to_string(), "x".to_string());
        let item =
            build_item("fr/bonjour", &meta, "https://example.com", &known)
                .unwrap();
        assert_eq!(item.language, Some("fr".to_string()));
    }

    #[test]
    fn test_json_feed_explicit_locale_overrides_path() {
        let known = vec!["en".to_string(), "fr".to_string()];
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        let _ = meta.insert("description".to_string(), "x".to_string());
        let _ = meta.insert("language".to_string(), "de".to_string());
        let item = build_item("fr/post", &meta, "https://example.com", &known)
            .unwrap();
        assert_eq!(item.language, Some("de".to_string()));
    }

    #[test]
    fn test_json_feed_skips_empty_title() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), String::new());
        assert!(build_item("post", &meta, "https://example.com", &[]).is_none());
    }

    #[test]
    fn test_json_feed_skips_empty_path() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        assert!(build_item("", &meta, "https://example.com", &[]).is_none());
    }

    #[test]
    fn test_json_feed_id_matches_url() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        let _ = meta.insert("description".to_string(), "x".to_string());
        let item = build_item("p", &meta, "https://example.com", &[]).unwrap();
        assert_eq!(item.id, item.url);
        assert_eq!(item.url, "https://example.com/p/");
    }

    #[test]
    fn test_json_feed_plugin_name() {
        assert_eq!(JsonFeedPlugin.name(), "json-feed");
    }

    #[test]
    fn test_json_feed_plugin_registers() {
        use crate::plugin::PluginManager;
        let mut pm = PluginManager::new();
        pm.register(JsonFeedPlugin);
        assert!(pm.names().contains(&"json-feed"));
    }

    #[test]
    fn test_json_feed_idempotent_link_injection() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("page.html");
        fs::write(
            &html_path,
            "<html><head><title>T</title></head><body></body></html>",
        )?;
        inject_json_feed_link(tmp.path(), "https://example.com/feed.json")?;
        let first = fs::read_to_string(&html_path)?;
        inject_json_feed_link(tmp.path(), "https://example.com/feed.json")?;
        let second = fs::read_to_string(&html_path)?;
        assert_eq!(first, second);
        assert_eq!(
            second.matches("application/feed+json").count(),
            1,
            "should inject exactly one link tag"
        );
        Ok(())
    }

    #[test]
    fn test_json_feed_link_skips_files_without_head() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("frag.html");
        fs::write(&html_path, "<div>no head</div>")?;
        inject_json_feed_link(tmp.path(), "https://example.com/feed.json")?;
        let result = fs::read_to_string(&html_path)?;
        assert!(!result.contains("application/feed+json"));
        Ok(())
    }

    #[test]
    fn test_extract_default_locale_prefers_i18n() {
        use crate::i18n::I18nConfig;
        let tmp = tempdir().unwrap();
        let config = crate::cmd::SsgConfig {
            base_url: "https://example.com".to_string(),
            site_name: "S".to_string(),
            site_title: "S".to_string(),
            site_description: String::new(),
            language: "en".to_string(),
            content_dir: std::path::PathBuf::from("c"),
            output_dir: std::path::PathBuf::from("b"),
            template_dir: std::path::PathBuf::from("t"),
            serve_dir: None,
            i18n: Some(I18nConfig {
                default_locale: "fr".to_string(),
                locales: vec!["en".into(), "fr".into()],
                url_prefix: crate::i18n::UrlPrefixStrategy::SubPath,
            }),
            cdn_prefix: None,
            image: crate::cmd::ImageConfig::default(),
            edge_headers: crate::cmd::EdgeHeadersConfig::default(),
        };
        let ctx = PluginContext::with_config(
            Path::new("c"),
            Path::new("b"),
            tmp.path(),
            Path::new("t"),
            config,
        );
        assert_eq!(extract_default_locale(&ctx), "fr");
        assert_eq!(extract_known_locales(&ctx), vec!["en", "fr"]);
    }

    #[test]
    fn test_extract_default_locale_falls_back_to_language() {
        let tmp = tempdir().unwrap();
        let ctx = make_ctx(tmp.path());
        assert_eq!(extract_default_locale(&ctx), "en");
        assert!(extract_known_locales(&ctx).is_empty());
    }

    #[test]
    fn test_extract_default_locale_no_config_defaults_en() {
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            Path::new("s"),
            Path::new("t"),
        );
        assert_eq!(extract_default_locale(&ctx), "en");
    }

    #[test]
    fn test_build_feed_json_omits_empty_language() {
        let items: Vec<JsonFeedItem> = vec![JsonFeedItem {
            sort_key: "2026".into(),
            id: "https://x/".into(),
            url: "https://x/".into(),
            title: "T".into(),
            content_html: "h".into(),
            date_published: "2026".into(),
            date_modified: "2026".into(),
            author: "A".into(),
            tags: vec![],
            language: None,
        }];
        let v = build_feed_json(
            "Title",
            "https://x/",
            "https://x/feed.json",
            "",
            &items,
        );
        assert!(v.get("language").is_none());
        assert_eq!(v["version"], JSON_FEED_VERSION);
        assert_eq!(v["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_tags_fallback_to_category() {
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        let _ = meta.insert("category".to_string(), "Tech".to_string());
        let item = build_item("p", &meta, "https://example.com", &[]).unwrap();
        assert_eq!(item.tags, vec!["Tech"]);
    }

    #[test]
    fn test_json_feed_no_base_url() -> Result<()> {
        let tmp = tempdir()?;
        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "T".to_string());
        let _ = meta.insert("description".to_string(), "x".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        write_meta_sidecar(tmp.path(), "p", &meta);

        let config = crate::cmd::SsgConfig {
            base_url: String::new(),
            site_name: "S".to_string(),
            site_title: "S".to_string(),
            site_description: String::new(),
            language: "en".to_string(),
            content_dir: std::path::PathBuf::from("c"),
            output_dir: std::path::PathBuf::from("b"),
            template_dir: std::path::PathBuf::from("t"),
            serve_dir: None,
            i18n: None,
            cdn_prefix: None,
            image: crate::cmd::ImageConfig::default(),
            edge_headers: crate::cmd::EdgeHeadersConfig::default(),
        };
        let ctx = PluginContext::with_config(
            Path::new("c"),
            Path::new("b"),
            tmp.path(),
            Path::new("t"),
            config,
        );
        JsonFeedPlugin.after_compile(&ctx)?;

        let value: Value = serde_json::from_str(&fs::read_to_string(
            tmp.path().join("feed.json"),
        )?)?;
        assert_eq!(value["feed_url"], "feed.json");
        assert_eq!(value["home_page_url"], "/");
        assert_eq!(value["items"][0]["url"], "p/");
        Ok(())
    }
}
