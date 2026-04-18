// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON-LD structured data injection plugin.

use super::helpers::{
    extract_date_from_html, extract_description, extract_first_content_image,
    extract_html_lang, extract_meta_author, extract_meta_date, extract_title,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::path::Path;

/// Configuration for the JSON-LD structured data plugin.
#[derive(Debug, Clone)]
pub struct JsonLdConfig {
    /// Base URL of the site (for absolute URLs in JSON-LD).
    pub base_url: String,
    /// Organization name for Organization schema.
    pub org_name: String,
    /// Whether to generate `BreadcrumbList` for every page.
    pub breadcrumbs: bool,
}

/// Injects JSON-LD structured data into HTML files.
///
/// Auto-detects schema.org types from page metadata:
/// - Pages with `<article>` → `Article`
/// - All other pages → `WebPage`
/// - `BreadcrumbList` derived from URL path (opt-in)
///
/// Idempotent: skips files that already contain `application/ld+json`.
#[derive(Debug, Clone)]
pub struct JsonLdPlugin {
    pub(crate) config: JsonLdConfig,
}

impl JsonLdPlugin {
    /// Creates a new `JsonLdPlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: JsonLdConfig) -> Self {
        Self { config }
    }

    /// Creates a `JsonLdPlugin` from site config values.
    #[must_use]
    pub fn from_site(base_url: &str, site_name: &str) -> Self {
        Self {
            config: JsonLdConfig {
                base_url: base_url.to_string(),
                org_name: site_name.to_string(),
                breadcrumbs: true,
            },
        }
    }
}

/// Builds an Article JSON-LD object from page metadata.
fn build_article_jsonld(
    title: &str,
    description: &str,
    page_url: &str,
    org_name: &str,
    author_name: &str,
    image_url: &str,
    date_published: Option<&String>,
    date_modified: Option<&String>,
    lang: &str,
) -> serde_json::Value {
    let mut article = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": title,
        "description": description,
        "url": page_url,
        "inLanguage": if lang.is_empty() { "en" } else { lang },
        "mainEntityOfPage": {
            "@type": "WebPage",
            "@id": page_url
        },
        "publisher": {
            "@type": "Organization",
            "name": org_name
        }
    });

    if !author_name.is_empty() {
        article["author"] = serde_json::json!({
            "@type": "Person",
            "name": author_name
        });
    }

    if !image_url.is_empty() {
        article["image"] = serde_json::json!({
            "@type": "ImageObject",
            "url": image_url
        });
    }

    if let Some(dp) = date_published {
        article["datePublished"] = serde_json::json!(dp);
    }
    if let Some(dm) = date_modified {
        article["dateModified"] = serde_json::json!(dm);
    } else if let Some(dp) = date_published {
        article["dateModified"] = serde_json::json!(dp);
    }

    article
}

/// Builds a `WebPage` JSON-LD object from page metadata.
fn build_webpage_jsonld(
    title: &str,
    description: &str,
    page_url: &str,
    author_name: &str,
    image_url: &str,
    date_published: Option<&String>,
    lang: &str,
) -> serde_json::Value {
    let mut webpage = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": title,
        "description": description,
        "url": page_url,
        "inLanguage": if lang.is_empty() { "en" } else { lang }
    });

    if !author_name.is_empty() {
        webpage["author"] = serde_json::json!({
            "@type": "Person",
            "name": author_name
        });
    }

    if !image_url.is_empty() {
        webpage["image"] = serde_json::json!({
            "@type": "ImageObject",
            "url": image_url
        });
    }

    if let Some(dp) = date_published {
        webpage["datePublished"] = serde_json::json!(dp);
    }

    webpage
}

/// Builds a `BreadcrumbList` JSON-LD object from the URL path, if applicable.
fn build_breadcrumb_jsonld(
    base: &str,
    rel_path: &str,
) -> Option<serde_json::Value> {
    let parts: Vec<&str> = rel_path
        .trim_matches('/')
        .split('/')
        .filter(|p| !p.is_empty() && *p != "index.html")
        .collect();

    if parts.is_empty() {
        return None;
    }

    let mut items = vec![serde_json::json!({
        "@type": "ListItem",
        "position": 1,
        "name": "Home",
        "item": format!("{}/", base)
    })];

    let mut accumulated = String::new();
    for (i, part) in parts.iter().enumerate() {
        accumulated = format!("{accumulated}/{part}");
        let name = part.trim_end_matches(".html").replace('-', " ");
        items.push(serde_json::json!({
            "@type": "ListItem",
            "position": i + 2,
            "name": name,
            "item": format!("{}{}", base, accumulated)
        }));
    }

    Some(serde_json::json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": items
    }))
}

/// Builds all JSON-LD scripts for a single page.
fn build_jsonld_scripts(
    html: &str,
    base: &str,
    rel_path: &str,
    org_name: &str,
    breadcrumbs: bool,
) -> Vec<serde_json::Value> {
    let title = extract_title(html);
    let description = extract_description(html, 160);
    let page_url = format!("{base}/{rel_path}");
    let author_name = extract_meta_author(html);
    let image_url = extract_first_content_image(html);
    let date_published = extract_date_from_html(html, "datePublished")
        .or_else(|| extract_meta_date(html));
    let date_modified = extract_date_from_html(html, "dateModified");
    let lang = extract_html_lang(html);

    let mut scripts = Vec::new();

    if html.contains("<article") {
        scripts.push(build_article_jsonld(
            &title,
            &description,
            &page_url,
            org_name,
            &author_name,
            &image_url,
            date_published.as_ref(),
            date_modified.as_ref(),
            &lang,
        ));
    } else {
        scripts.push(build_webpage_jsonld(
            &title,
            &description,
            &page_url,
            &author_name,
            &image_url,
            date_published.as_ref(),
            &lang,
        ));
    }

    if breadcrumbs {
        if let Some(breadcrumb) = build_breadcrumb_jsonld(base, rel_path) {
            scripts.push(breadcrumb);
        }
    }

    scripts
}

impl Plugin for JsonLdPlugin {
    fn name(&self) -> &'static str {
        "json-ld"
    }

    fn has_transform(&self) -> bool {
        true
    }

    fn transform_html(
        &self,
        html: &str,
        path: &Path,
        ctx: &PluginContext,
    ) -> Result<String> {
        if html.contains("application/ld+json") {
            return Ok(html.to_string());
        }

        let Some(head_pos) = html.find("</head>") else {
            return Ok(html.to_string());
        };

        let base = self.config.base_url.trim_end_matches('/');
        let site_dir = &ctx.site_dir;

        let rel_path = path
            .strip_prefix(site_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let scripts = build_jsonld_scripts(
            html,
            base,
            &rel_path,
            &self.config.org_name,
            self.config.breadcrumbs,
        );

        let mut injection = String::new();
        for script in &scripts {
            let json = serde_json::to_string(script)?;
            injection.push_str(&format!(
                "<script type=\"application/ld+json\">{json}</script>\n"
            ));
        }

        let result =
            format!("{}{}{}", &html[..head_pos], injection, &html[head_pos..]);
        Ok(result)
    }

    fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn ctx(site: &Path) -> PluginContext {
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site,
            Path::new("templates"),
        )
    }

    fn cfg() -> JsonLdConfig {
        JsonLdConfig {
            base_url: "https://example.com".to_string(),
            org_name: "Example Org".to_string(),
            breadcrumbs: true,
        }
    }

    #[test]
    fn name_is_stable() {
        let p = JsonLdPlugin::new(cfg());
        assert_eq!(p.name(), "json-ld");
    }

    #[test]
    fn from_site_constructs_with_breadcrumbs_enabled() {
        let p = JsonLdPlugin::from_site("https://x.example", "X");
        assert_eq!(p.config.base_url, "https://x.example");
        assert_eq!(p.config.org_name, "X");
        assert!(p.config.breadcrumbs);
    }

    // ── build_article_jsonld ───────────────────────────────────

    #[test]
    fn article_includes_author_when_provided() {
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "Jane",
            "",
            None,
            None,
            "en",
        );
        assert_eq!(v["author"]["name"], "Jane");
        assert_eq!(v["author"]["@type"], "Person");
    }

    #[test]
    fn article_omits_author_when_empty() {
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "",
            "",
            None,
            None,
            "en",
        );
        assert!(v.get("author").is_none());
    }

    #[test]
    fn article_includes_image_when_url_present() {
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "",
            "https://x/img.png",
            None,
            None,
            "en",
        );
        assert_eq!(v["image"]["@type"], "ImageObject");
        assert_eq!(v["image"]["url"], "https://x/img.png");
    }

    #[test]
    fn article_uses_date_published_for_date_modified_fallback() {
        let dp = "2025-01-01".to_string();
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "",
            "",
            Some(&dp),
            None,
            "en",
        );
        assert_eq!(v["datePublished"], "2025-01-01");
        assert_eq!(
            v["dateModified"], "2025-01-01",
            "missing dateModified should fall back to datePublished"
        );
    }

    #[test]
    fn article_keeps_distinct_date_modified() {
        let dp = "2025-01-01".to_string();
        let dm = "2025-06-15".to_string();
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "",
            "",
            Some(&dp),
            Some(&dm),
            "en",
        );
        assert_eq!(v["datePublished"], "2025-01-01");
        assert_eq!(v["dateModified"], "2025-06-15");
    }

    #[test]
    fn article_defaults_lang_to_en_when_empty() {
        let v = build_article_jsonld(
            "T",
            "D",
            "https://x/p",
            "Org",
            "",
            "",
            None,
            None,
            "",
        );
        assert_eq!(v["inLanguage"], "en");
    }

    // ── build_webpage_jsonld ───────────────────────────────────

    #[test]
    fn webpage_includes_author_image_date_when_present() {
        let dp = "2025-01-01".to_string();
        let v = build_webpage_jsonld(
            "T",
            "D",
            "https://x/p",
            "Jane",
            "https://x/i.png",
            Some(&dp),
            "fr",
        );
        assert_eq!(v["@type"], "WebPage");
        assert_eq!(v["author"]["name"], "Jane");
        assert_eq!(v["image"]["url"], "https://x/i.png");
        assert_eq!(v["datePublished"], "2025-01-01");
        assert_eq!(v["inLanguage"], "fr");
    }

    #[test]
    fn webpage_omits_optional_fields_when_empty() {
        let v = build_webpage_jsonld("T", "D", "https://x/p", "", "", None, "");
        assert!(v.get("author").is_none());
        assert!(v.get("image").is_none());
        assert!(v.get("datePublished").is_none());
        assert_eq!(v["inLanguage"], "en");
    }

    // ── build_breadcrumb_jsonld ────────────────────────────────

    #[test]
    fn breadcrumb_returns_none_for_root_path() {
        // Just `index.html` (or empty path) → no breadcrumb chain.
        assert!(build_breadcrumb_jsonld("https://x", "/").is_none());
        assert!(build_breadcrumb_jsonld("https://x", "index.html").is_none());
    }

    #[test]
    fn breadcrumb_builds_chain_for_nested_path() {
        let v = build_breadcrumb_jsonld("https://x", "blog/my-post/index.html")
            .expect("should produce breadcrumb for nested path");
        assert_eq!(v["@type"], "BreadcrumbList");
        let items = v["itemListElement"].as_array().unwrap();
        assert_eq!(items.len(), 3); // Home + blog + my-post
        assert_eq!(items[0]["name"], "Home");
        assert_eq!(items[1]["name"], "blog");
        assert_eq!(items[2]["name"], "my post"); // hyphens → spaces
    }

    #[test]
    fn breadcrumb_handles_html_extension_in_part_name() {
        let v = build_breadcrumb_jsonld("https://x", "page.html").unwrap();
        let items = v["itemListElement"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1]["name"], "page");
    }

    // ── build_jsonld_scripts ───────────────────────────────────

    #[test]
    fn build_scripts_picks_article_when_article_tag_present() {
        let html = r#"<html><head><title>Post</title></head>
            <body><article>content</article></body></html>"#;
        let scripts =
            build_jsonld_scripts(html, "https://x", "p/", "Org", false);
        assert_eq!(scripts[0]["@type"], "Article");
    }

    #[test]
    fn build_scripts_picks_webpage_when_no_article_tag() {
        let html = "<html><head><title>P</title></head><body>x</body></html>";
        let scripts =
            build_jsonld_scripts(html, "https://x", "p/", "Org", false);
        assert_eq!(scripts[0]["@type"], "WebPage");
    }

    #[test]
    fn build_scripts_includes_breadcrumb_when_enabled() {
        let html = "<html><head><title>P</title></head><body>x</body></html>";
        let scripts =
            build_jsonld_scripts(html, "https://x", "blog/post/", "Org", true);
        assert!(
            scripts.iter().any(|s| s["@type"] == "BreadcrumbList"),
            "breadcrumb should be present when enabled and path nested"
        );
    }

    #[test]
    fn build_scripts_skips_breadcrumb_when_disabled() {
        let html = "<html><head><title>P</title></head><body>x</body></html>";
        let scripts =
            build_jsonld_scripts(html, "https://x", "blog/post/", "Org", false);
        assert!(!scripts.iter().any(|s| s["@type"] == "BreadcrumbList"));
    }

    // ── after_compile end-to-end ───────────────────────────────

    #[test]
    fn after_compile_no_op_when_site_missing() {
        let dir = tempdir().unwrap();
        let nope = dir.path().join("nope");
        JsonLdPlugin::new(cfg()).after_compile(&ctx(&nope)).unwrap();
    }

    #[test]
    fn transform_html_injects_jsonld() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());
        let html = "<html><head><title>X</title></head><body>x</body></html>";
        let page_path = dir.path().join("index.html");
        let after = JsonLdPlugin::new(cfg())
            .transform_html(html, &page_path, &c)
            .unwrap();
        assert!(after.contains("application/ld+json"));
        assert!(after.contains("\"@type\":\"WebPage\""));
    }

    #[test]
    fn transform_html_skips_existing_jsonld() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());
        let html = r#"<html><head><script type="application/ld+json">{"@type":"X"}</script><title>X</title></head></html>"#;
        let page_path = dir.path().join("p.html");
        let after = JsonLdPlugin::new(cfg())
            .transform_html(html, &page_path, &c)
            .unwrap();
        // Only one JSON-LD block — no duplicate injected.
        assert_eq!(after.matches("application/ld+json").count(), 1);
        assert!(after.contains(r#"{"@type":"X"}"#));
    }

    #[test]
    fn transform_html_skips_without_head_tag() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());
        let raw = "<!doctype html><html><body>only</body></html>";
        let page_path = dir.path().join("frag.html");
        let after = JsonLdPlugin::new(cfg())
            .transform_html(raw, &page_path, &c)
            .unwrap();
        assert_eq!(after, raw);
    }
}
