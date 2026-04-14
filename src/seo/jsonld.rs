// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON-LD structured data injection plugin.

use super::helpers::{
    collect_html_files_recursive, extract_date_from_html, extract_description,
    extract_first_content_image, extract_html_lang, extract_meta_author,
    extract_meta_date, extract_title,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use rayon::prelude::*;
use std::fs;

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

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files_recursive(&ctx.site_dir)?;
        let injected = std::sync::atomic::AtomicUsize::new(0);
        let base = self.config.base_url.trim_end_matches('/');
        let site_dir = &ctx.site_dir;

        html_files.par_iter().try_for_each(|path| -> Result<()> {
            let html = fs::read_to_string(path)?;

            if html.contains("application/ld+json") {
                return Ok(());
            }

            let Some(head_pos) = html.find("</head>") else {
                return Ok(());
            };

            let rel_path = path
                .strip_prefix(site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let scripts = build_jsonld_scripts(
                &html,
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

            let result = format!(
                "{}{}{}",
                &html[..head_pos],
                injection,
                &html[head_pos..]
            );
            fs::write(path, result)?;
            let _ = injected.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })?;

        let count = injected.load(std::sync::atomic::Ordering::Relaxed);
        if count > 0 {
            log::info!(
                "[json-ld] Injected structured data into {count} page(s)"
            );
        }
        Ok(())
    }
}
