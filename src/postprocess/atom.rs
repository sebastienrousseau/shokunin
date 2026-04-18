// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Atom 1.0 feed plugin.

use super::helpers::{parse_rfc2822_lenient, read_meta_sidecars, xml_escape};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generates an Atom 1.0 `atom.xml` feed from `.meta.json` sidecars.
///
/// Runs after `RssAggregatePlugin` in `after_compile`. Reads the same
/// sidecar files, sorts entries by date descending, and limits to 50.
#[derive(Debug, Clone, Copy)]
pub struct AtomFeedPlugin;

impl Plugin for AtomFeedPlugin {
    fn name(&self) -> &'static str {
        "atom-feed"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let mut meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();

        // Fall back to build_dir/.meta for sidecars emitted by
        // TemplatePlugin::before_compile (not present in site_dir
        // when staticdatagen doesn't copy them).
        if meta_entries.is_empty() {
            let meta_dir = ctx.build_dir.join(".meta");
            if meta_dir.exists() {
                meta_entries =
                    read_meta_sidecars(&meta_dir).unwrap_or_default();
            }
        }

        // Last resort: extract entries from an existing rss.xml
        // (staticdatagen generates rss.xml natively even without sidecars).
        if meta_entries.is_empty() {
            meta_entries = extract_entries_from_rss(&ctx.site_dir);
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

        let mut articles = collect_atom_entries(&meta_entries, &base_url);
        articles.sort_by(|a, b| b.0.cmp(&a.0));
        articles.truncate(50);

        if articles.is_empty() {
            return Ok(());
        }

        let feed_xml = build_atom_feed(&feed_title, &base_url, &articles);

        let atom_path = ctx.site_dir.join("atom.xml");
        fs::write(&atom_path, &feed_xml)
            .with_context(|| format!("cannot write {}", atom_path.display()))?;

        let atom_self_link = if base_url.is_empty() {
            "atom.xml".to_string()
        } else {
            format!("{base_url}/atom.xml")
        };
        inject_atom_link(&ctx.site_dir, &atom_self_link)?;

        log::info!(
            "[atom-feed] Generated atom.xml with {} entries",
            articles.len()
        );
        Ok(())
    }
}

/// Collects Atom entries from metadata sidecars.
fn collect_atom_entries(
    meta_entries: &[(String, std::collections::HashMap<String, String>)],
    base_url: &str,
) -> Vec<(String, AtomEntry)> {
    let mut articles = Vec::new();
    for (rel_path, meta) in meta_entries {
        if let Some(entry) = build_atom_entry(rel_path, meta, base_url) {
            articles.push(entry);
        }
    }
    articles
}

/// Builds a single Atom entry from metadata, or `None` if data is insufficient.
fn build_atom_entry(
    rel_path: &str,
    meta: &std::collections::HashMap<String, String>,
    base_url: &str,
) -> Option<(String, AtomEntry)> {
    if rel_path.is_empty() {
        return None;
    }

    let title = meta.get("title").cloned().unwrap_or_default();
    if title.is_empty() {
        return None;
    }

    let description = meta.get("description").cloned().unwrap_or_default();
    let pub_date = meta.get("item_pub_date").cloned().unwrap_or_default();
    let author = meta.get("author").cloned().unwrap_or_default();

    let link = if base_url.is_empty() {
        format!("{rel_path}/")
    } else {
        format!("{base_url}/{rel_path}/")
    };

    let rfc3339 = parse_rfc2822_lenient(&pub_date)
        .map_or_else(|| pub_date.clone(), |dt| dt.to_rfc3339());

    Some((
        rfc3339.clone(),
        AtomEntry {
            title,
            link: link.clone(),
            id: link,
            updated: rfc3339.clone(),
            published: rfc3339,
            summary: description,
            author,
        },
    ))
}

/// Builds the complete Atom XML feed from entries.
fn build_atom_feed(
    feed_title: &str,
    base_url: &str,
    articles: &[(String, AtomEntry)],
) -> String {
    let feed_updated = &articles[0].0;
    let entries_xml: String = articles
        .iter()
        .map(|(_, entry)| entry.to_xml())
        .collect::<Vec<_>>()
        .join("\n");

    let atom_self_link = if base_url.is_empty() {
        "atom.xml".to_string()
    } else {
        format!("{base_url}/atom.xml")
    };

    let feed_id = if base_url.is_empty() {
        "/".to_string()
    } else {
        base_url.to_string()
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>{feed_title}</title>
  <link href="{atom_self_link}" rel="self" type="application/atom+xml"/>
  <link href="{base_url}"/>
  <id>{feed_id}</id>
  <updated>{feed_updated}</updated>
{entries_xml}
</feed>
"#,
        feed_title = xml_escape(feed_title),
    )
}

/// A single Atom entry's data.
pub(super) struct AtomEntry {
    pub title: String,
    pub link: String,
    pub id: String,
    pub updated: String,
    pub published: String,
    pub summary: String,
    pub author: String,
}

impl AtomEntry {
    pub(super) fn to_xml(&self) -> String {
        let author_name = if self.author.is_empty() {
            "Unknown".to_string()
        } else {
            xml_escape(&self.author)
        };
        format!(
            r#"  <entry>
    <title>{title}</title>
    <link href="{link}"/>
    <id>{id}</id>
    <updated>{updated}</updated>
    <published>{published}</published>
    <summary>{summary}</summary>
    <author><name>{author}</name></author>
  </entry>"#,
            title = xml_escape(&self.title),
            link = xml_escape(&self.link),
            id = xml_escape(&self.id),
            updated = xml_escape(&self.updated),
            published = xml_escape(&self.published),
            summary = xml_escape(&self.summary),
            author = author_name,
        )
    }
}

/// Extracts entry metadata from an existing `rss.xml` when no sidecars
/// are available. Returns entries in the same format as `read_meta_sidecars`.
fn extract_entries_from_rss(
    site_dir: &Path,
) -> Vec<(String, std::collections::HashMap<String, String>)> {
    let rss_path = site_dir.join("rss.xml");
    let Ok(rss_content) = fs::read_to_string(&rss_path) else {
        return Vec::new();
    };

    let mut entries = Vec::new();

    // Simple XML parsing: extract <item>…</item> blocks
    let mut search_from = 0;
    while let Some(item_start) = rss_content[search_from..].find("<item>") {
        let abs_start = search_from + item_start;
        let Some(item_end) = rss_content[abs_start..].find("</item>") else {
            break;
        };
        let item = &rss_content[abs_start..abs_start + item_end + 7];

        let mut meta = std::collections::HashMap::new();
        if let Some(title) = extract_xml_tag(item, "title") {
            let _ = meta.insert("title".to_string(), title);
        }
        if let Some(desc) = extract_xml_tag(item, "description") {
            let _ = meta.insert("description".to_string(), desc);
        }
        if let Some(date) = extract_xml_tag(item, "pubDate") {
            let _ = meta.insert("item_pub_date".to_string(), date);
        }
        if let Some(author) = extract_xml_tag(item, "author") {
            let _ = meta.insert("author".to_string(), author);
        }

        // Derive relative path from <link>
        let rel_path = extract_xml_tag(item, "link")
            .map(|link| {
                link.trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .unwrap_or_default();

        if !rel_path.is_empty() && meta.contains_key("title") {
            entries.push((rel_path, meta));
        }

        search_from = abs_start + item_end + 7;
    }

    entries
}

/// Extracts text content from a simple XML tag.
///
/// Handles both `<tag>content</tag>` and `<tag attr="...">content</tag>`.
/// Strips CDATA wrappers and decodes common XML entities.
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    // Match both <tag> and <tag attr="...">
    let open_plain = format!("<{tag}>");
    let open_attr = format!("<{tag} ");
    let close = format!("</{tag}>");

    let (start, content_start) = if let Some(pos) = xml.find(&open_plain) {
        (pos, pos + open_plain.len())
    } else if let Some(pos) = xml.find(&open_attr) {
        let gt = xml[pos..].find('>')?;
        (pos, pos + gt + 1)
    } else {
        return None;
    };

    let _ = start; // used for finding the tag
    let end = xml[content_start..].find(&close)? + content_start;
    let content = xml[content_start..end].trim();

    // Strip CDATA wrapper
    let content = content
        .strip_prefix("<![CDATA[")
        .and_then(|s| s.strip_suffix("]]>"))
        .unwrap_or(content);

    // Decode common XML entities
    let decoded = content
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");

    let decoded = decoded.trim();
    if decoded.is_empty() {
        None
    } else {
        Some(xml_escape(decoded))
    }
}

/// Inject `<link rel="alternate" type="application/atom+xml">` into
/// HTML files that don't already have one.
pub(super) fn inject_atom_link(site_dir: &Path, atom_url: &str) -> Result<()> {
    let html_files = crate::walk::walk_files(site_dir, "html")?;
    for path in &html_files {
        let html = fs::read_to_string(path)
            .with_context(|| format!("cannot read {}", path.display()))?;

        if html.contains("application/atom+xml") {
            continue;
        }

        // Insert before </head>
        if let Some(pos) = html.find("</head>") {
            let link_tag = format!(
                "  <link rel=\"alternate\" type=\"application/atom+xml\" title=\"Atom Feed\" href=\"{atom_url}\"/>\n"
            );
            let modified =
                format!("{}{}{}", &html[..pos], link_tag, &html[pos..]);
            fs::write(path, &modified)
                .with_context(|| format!("cannot write {}", path.display()))?;
        }
    }
    Ok(())
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

    #[test]
    fn test_atom_feed_valid_namespace_and_elements() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Hello World".to_string());
        let _ =
            meta.insert("description".to_string(), "A test post".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Alice".to_string());
        write_meta_sidecar(tmp.path(), "hello", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let atom_path = tmp.path().join("atom.xml");
        assert!(atom_path.exists(), "atom.xml should be created");

        let content = fs::read_to_string(&atom_path)?;
        assert!(
            content.contains("xmlns=\"http://www.w3.org/2005/Atom\""),
            "Missing Atom namespace"
        );
        assert!(content.contains("<feed"), "Missing <feed> element");
        assert!(content.contains("<title>"), "Missing <title>");
        assert!(content.contains("rel=\"self\""), "Missing self link");
        assert!(content.contains("<id>"), "Missing <id>");
        assert!(content.contains("<updated>"), "Missing <updated>");
        assert!(content.contains("<entry>"), "Missing <entry>");
        assert!(content.contains("<author>"), "Missing <author>");
        assert!(
            content.contains("<name>Alice</name>"),
            "Missing author name"
        );
        assert!(content.contains("<summary>"), "Missing <summary>");
        assert!(content.contains("<published>"), "Missing <published>");
        Ok(())
    }

    #[test]
    fn test_atom_feed_entry_count_matches() -> Result<()> {
        let tmp = tempdir()?;

        for i in 0..5 {
            let mut meta = HashMap::new();
            let _ = meta.insert("title".to_string(), format!("Post {i}"));
            let _ = meta.insert("description".to_string(), format!("Desc {i}"));
            let _ = meta.insert(
                "item_pub_date".to_string(),
                format!("Thu, {:02} Apr 2026 06:06:06 +0000", 10 + i),
            );
            let _ = meta.insert("author".to_string(), "Bob".to_string());
            write_meta_sidecar(tmp.path(), &format!("post-{i}"), &meta);
        }

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("atom.xml"))?;
        let entry_count = content.matches("<entry>").count();
        assert_eq!(entry_count, 5, "Expected 5 entries, got {entry_count}");
        Ok(())
    }

    #[test]
    fn test_atom_feed_dates_are_rfc3339() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Date Test".to_string());
        let _ =
            meta.insert("description".to_string(), "Testing dates".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Charlie".to_string());
        write_meta_sidecar(tmp.path(), "datepost", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("atom.xml"))?;
        assert!(
            content.contains("2026-04-11T06:06:06+00:00"),
            "Expected RFC 3339 date in atom.xml, got:\n{content}"
        );
        Ok(())
    }

    #[test]
    fn test_atom_feed_idempotent() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Idempotent".to_string());
        let _ = meta.insert("description".to_string(), "Test".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Dave".to_string());
        write_meta_sidecar(tmp.path(), "idem", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;
        let first = fs::read_to_string(tmp.path().join("atom.xml"))?;

        AtomFeedPlugin.after_compile(&ctx)?;
        let second = fs::read_to_string(tmp.path().join("atom.xml"))?;

        assert_eq!(first, second, "Atom feed should be idempotent");
        Ok(())
    }

    #[test]
    fn test_atom_feed_injects_link_into_html() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "Link Test".to_string());
        let _ = meta.insert("description".to_string(), "Test".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "Eve".to_string());
        write_meta_sidecar(tmp.path(), "linktest", &meta);

        let html_path = tmp.path().join("index.html");
        fs::write(
            &html_path,
            "<html><head><title>Test</title></head><body></body></html>",
        )?;

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let html = fs::read_to_string(&html_path)?;
        assert!(
            html.contains("application/atom+xml"),
            "HTML should have atom link tag"
        );
        Ok(())
    }

    #[test]
    fn test_atom_plugin_registers() {
        use crate::plugin::PluginManager;
        let mut pm = PluginManager::new();
        pm.register(AtomFeedPlugin);
        assert_eq!(pm.len(), 1);
        assert_eq!(pm.names(), vec!["atom-feed"]);
    }

    #[test]
    fn test_atom_feed_sorts_descending() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta_old = HashMap::new();
        let _ = meta_old.insert("title".to_string(), "Old Post".to_string());
        let _ = meta_old.insert("description".to_string(), "old".to_string());
        let _ = meta_old.insert(
            "item_pub_date".to_string(),
            "Mon, 01 Jan 2025 00:00:00 +0000".to_string(),
        );
        let _ = meta_old.insert("author".to_string(), "Alice".to_string());
        write_meta_sidecar(tmp.path(), "old-post", &meta_old);

        let mut meta_new = HashMap::new();
        let _ = meta_new.insert("title".to_string(), "New Post".to_string());
        let _ = meta_new.insert("description".to_string(), "new".to_string());
        let _ = meta_new.insert(
            "item_pub_date".to_string(),
            "Fri, 11 Apr 2026 12:00:00 +0000".to_string(),
        );
        let _ = meta_new.insert("author".to_string(), "Bob".to_string());
        write_meta_sidecar(tmp.path(), "new-post", &meta_new);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("atom.xml"))?;
        let first_entry_pos = content.find("<entry>").unwrap();
        let new_title_pos = content.find("New Post").unwrap();
        let old_title_pos = content.find("Old Post").unwrap();
        assert!(
            new_title_pos < old_title_pos,
            "Newer post should come first"
        );
        assert!(
            new_title_pos > first_entry_pos,
            "Title should be inside an entry"
        );
        Ok(())
    }

    #[test]
    fn test_atom_feed_empty_author_shows_unknown() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), "No Author".to_string());
        let _ = meta.insert("description".to_string(), "test".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        write_meta_sidecar(tmp.path(), "no-author", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("atom.xml"))?;
        assert!(
            content.contains("<name>Unknown</name>"),
            "Empty author should show 'Unknown': {content}"
        );
        Ok(())
    }

    #[test]
    fn test_atom_feed_skips_empty_title() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta.insert("title".to_string(), String::new());
        let _ = meta.insert("description".to_string(), "test".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        write_meta_sidecar(tmp.path(), "no-title", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let atom_path = tmp.path().join("atom.xml");
        assert!(
            !atom_path.exists(),
            "Should not create atom.xml when all entries have empty titles"
        );
        Ok(())
    }

    #[test]
    fn test_atom_feed_xml_escapes_content() -> Result<()> {
        let tmp = tempdir()?;

        let mut meta = HashMap::new();
        let _ = meta
            .insert("title".to_string(), "Tom & Jerry <friends>".to_string());
        let _ = meta
            .insert("description".to_string(), "A \"great\" show".to_string());
        let _ = meta.insert(
            "item_pub_date".to_string(),
            "Thu, 11 Apr 2026 06:06:06 +0000".to_string(),
        );
        let _ = meta.insert("author".to_string(), "O'Brien".to_string());
        write_meta_sidecar(tmp.path(), "escape-test", &meta);

        let ctx = make_atom_ctx(tmp.path());
        AtomFeedPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("atom.xml"))?;
        assert!(content.contains("Tom &amp; Jerry"), "& should be escaped");
        assert!(
            content.contains("&lt;friends&gt;"),
            "< and > should be escaped"
        );
        assert!(
            content.contains("&quot;great&quot;"),
            "quotes should be escaped"
        );
        assert!(
            content.contains("O&apos;Brien"),
            "apostrophe should be escaped"
        );
        Ok(())
    }

    // -----------------------------------------------------------------
    // AtomEntry::to_xml direct test
    // -----------------------------------------------------------------

    #[test]
    fn test_atom_entry_to_xml() {
        let entry = AtomEntry {
            title: "Test Post".to_string(),
            link: "https://example.com/test/".to_string(),
            id: "https://example.com/test/".to_string(),
            updated: "2026-04-11T06:06:06+00:00".to_string(),
            published: "2026-04-11T06:06:06+00:00".to_string(),
            summary: "A test summary".to_string(),
            author: "Alice".to_string(),
        };
        let xml = entry.to_xml();
        assert!(xml.contains("<entry>"));
        assert!(xml.contains("</entry>"));
        assert!(xml.contains("<title>Test Post</title>"));
        assert!(xml.contains("href=\"https://example.com/test/\""));
        assert!(xml.contains("<name>Alice</name>"));
        assert!(xml.contains("<summary>A test summary</summary>"));
    }

    #[test]
    fn test_atom_entry_empty_author() {
        let entry = AtomEntry {
            title: "No Author".to_string(),
            link: "https://example.com/".to_string(),
            id: "https://example.com/".to_string(),
            updated: "2026-01-01T00:00:00+00:00".to_string(),
            published: "2026-01-01T00:00:00+00:00".to_string(),
            summary: String::new(),
            author: String::new(),
        };
        let xml = entry.to_xml();
        assert!(
            xml.contains("<name>Unknown</name>"),
            "Empty author should show 'Unknown'"
        );
    }

    // -----------------------------------------------------------------
    // inject_atom_link
    // -----------------------------------------------------------------

    #[test]
    fn test_inject_atom_link_adds_tag() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("page.html");
        fs::write(
            &html_path,
            "<html><head><title>Test</title></head><body></body></html>",
        )?;

        inject_atom_link(tmp.path(), "https://example.com/atom.xml")?;

        let result = fs::read_to_string(&html_path)?;
        assert!(
            result.contains("application/atom+xml"),
            "Should inject atom link: {result}"
        );
        assert!(
            result.contains("href=\"https://example.com/atom.xml\""),
            "Should have correct href: {result}"
        );
        Ok(())
    }

    #[test]
    fn test_inject_atom_link_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("page.html");
        fs::write(
            &html_path,
            "<html><head><title>Test</title></head><body></body></html>",
        )?;

        inject_atom_link(tmp.path(), "https://example.com/atom.xml")?;
        let first = fs::read_to_string(&html_path)?;

        inject_atom_link(tmp.path(), "https://example.com/atom.xml")?;
        let second = fs::read_to_string(&html_path)?;

        assert_eq!(first, second, "inject_atom_link should be idempotent");
        assert_eq!(
            second.matches("application/atom+xml").count(),
            1,
            "Should have exactly one atom link"
        );
        Ok(())
    }

    #[test]
    fn test_inject_atom_link_no_head() -> Result<()> {
        let tmp = tempdir()?;
        let html_path = tmp.path().join("nohead.html");
        fs::write(&html_path, "<html><body>No head</body></html>")?;

        inject_atom_link(tmp.path(), "https://example.com/atom.xml")?;

        let result = fs::read_to_string(&html_path)?;
        assert!(
            !result.contains("application/atom+xml"),
            "Should not inject when there is no </head>"
        );
        Ok(())
    }
}
