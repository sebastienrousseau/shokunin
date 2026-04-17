// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Internationalisation (i18n) routing primitives
//!
//! Provides hreflang link injection, per-locale sitemap generation,
//! and a language switcher HTML helper.
//!
//! ## Overview
//!
//! The `I18nPlugin` scans the site output directory for locale-prefixed
//! subdirectories (e.g. `/en/`, `/fr/`) and:
//!
//! 1. Injects `<link rel="alternate" hreflang="…">` tags into every HTML
//!    page that exists in multiple locales.
//! 2. Adds an `x-default` alternate pointing to the default locale.
//! 3. Generates per-locale sitemaps (`sitemap-en.xml`, `sitemap-fr.xml`, …)
//!    with `xhtml:link` alternates.
//!
//! The injection is **idempotent** — pages that already contain hreflang
//! links are skipped.

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

// ── Configuration ────────────────────────────────────────────────────

/// Strategy for constructing locale-specific URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum UrlPrefixStrategy {
    /// Locale appears as a path prefix: `https://example.com/fr/about`
    #[default]
    SubPath,
    /// Locale appears as a subdomain: `https://fr.example.com/about`
    SubDomain,
}

/// Parsed `[i18n]` configuration section.
///
/// # Example (TOML)
///
/// ```toml
/// [i18n]
/// default_locale = "en"
/// locales = ["en", "fr", "de"]
/// url_prefix = "sub_path"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    /// The default / fallback locale (used for `x-default`).
    pub default_locale: String,
    /// All supported locales.
    pub locales: Vec<String>,
    /// How locale URLs are constructed.
    #[serde(default)]
    pub url_prefix: UrlPrefixStrategy,
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            default_locale: "en".to_string(),
            locales: vec!["en".to_string()],
            url_prefix: UrlPrefixStrategy::default(),
        }
    }
}

// ── Plugin ───────────────────────────────────────────────────────────

/// I18n plugin that injects hreflang links and generates per-locale sitemaps.
#[derive(Debug)]
pub struct I18nPlugin {
    config: I18nConfig,
}

impl I18nPlugin {
    /// Creates a new `I18nPlugin` with the given i18n configuration.
    #[must_use]
    pub const fn new(config: I18nConfig) -> Self {
        Self { config }
    }
}

impl Plugin for I18nPlugin {
    fn name(&self) -> &'static str {
        "i18n"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Only operate when more than one locale is configured.
        if self.config.locales.len() < 2 {
            return Ok(());
        }

        // Detect which locale directories actually exist on disk.
        let present_locales =
            detect_locale_dirs(&ctx.site_dir, &self.config.locales);
        if present_locales.len() < 2 {
            return Ok(());
        }

        // Collect the set of relative page paths per locale.
        let pages = collect_locale_pages(&ctx.site_dir, &present_locales)?;

        // Determine the base URL (needed for sitemaps).
        let base_url = ctx.config.as_ref().map_or_else(
            || "https://example.com".to_string(),
            |c| c.base_url.clone(),
        );

        // Inject hreflang into each HTML page.
        inject_hreflang_all(
            &ctx.site_dir,
            &pages,
            &present_locales,
            &self.config.default_locale,
            &base_url,
            &self.config.url_prefix,
        )?;

        // Generate per-locale sitemaps.
        generate_locale_sitemaps(
            &ctx.site_dir,
            &pages,
            &present_locales,
            &self.config.default_locale,
            &base_url,
            &self.config.url_prefix,
        )?;

        // Generate locale redirect index.html at site root.
        crate::server::generate_locale_redirect(
            &ctx.site_dir,
            &present_locales,
            &self.config.default_locale,
        )?;

        Ok(())
    }
}

// ── Locale detection ─────────────────────────────────────────────────

/// Returns the subset of `locales` that have a matching directory inside
/// `site_dir`.
fn detect_locale_dirs(site_dir: &Path, locales: &[String]) -> Vec<String> {
    locales
        .iter()
        .filter(|l| site_dir.join(l).is_dir())
        .cloned()
        .collect()
}

// ── Page collection ──────────────────────────────────────────────────

/// For each relative page path (e.g. `about/index.html`), records which
/// locales provide that page.
///
/// Returns `path -> set-of-locales`.
fn collect_locale_pages(
    site_dir: &Path,
    locales: &[String],
) -> Result<HashMap<String, HashSet<String>>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();

    for locale in locales {
        let locale_dir = site_dir.join(locale);
        if !locale_dir.is_dir() {
            continue;
        }
        collect_html_files_recursive(
            &locale_dir,
            &locale_dir,
            locale,
            &mut map,
        )?;
    }

    Ok(map)
}

/// Recursively walk `current` under `root`, recording relative HTML paths.
fn collect_html_files_recursive(
    root: &Path,
    current: &Path,
    locale: &str,
    map: &mut HashMap<String, HashSet<String>>,
) -> Result<()> {
    let entries = fs::read_dir(current).with_context(|| {
        format!("Failed to read directory {}", current.display())
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_html_files_recursive(root, &path, locale, map)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let _ = map.entry(rel).or_default().insert(locale.to_string());
        }
    }

    Ok(())
}

// ── Hreflang injection ───────────────────────────────────────────────

/// Sentinel substring used for idempotency checks.
const HREFLANG_MARKER: &str = "rel=\"alternate\" hreflang=";

/// Inject hreflang `<link>` tags into every HTML page that exists in at
/// least two locales.
fn inject_hreflang_all(
    site_dir: &Path,
    pages: &HashMap<String, HashSet<String>>,
    locales: &[String],
    default_locale: &str,
    base_url: &str,
    strategy: &UrlPrefixStrategy,
) -> Result<()> {
    let base = base_url.trim_end_matches('/');
    let mut count = 0usize;

    for (rel_path, page_locales) in pages {
        // Only inject when the page exists in more than one locale.
        if page_locales.len() < 2 {
            continue;
        }

        for locale in locales {
            if !page_locales.contains(locale) {
                continue;
            }

            let file = site_dir.join(locale).join(rel_path);
            if !file.exists() {
                continue;
            }

            let html = fs::read_to_string(&file).with_context(|| {
                format!("Failed to read {}", file.display())
            })?;

            // Idempotency: skip if already injected.
            if html.contains(HREFLANG_MARKER) {
                continue;
            }

            let links = build_hreflang_links(
                rel_path,
                page_locales,
                default_locale,
                base,
                strategy,
            );

            let html = if let Some(injected) =
                inject_before_head_close(&html, &links)
            {
                injected
            } else {
                html
            };

            // Also inject visible language switcher at the marker
            let html = inject_lang_switcher(
                &html,
                locale,
                rel_path,
                &page_locales.iter().cloned().collect::<Vec<_>>(),
                base,
                strategy,
            );

            fs::write(&file, html).with_context(|| {
                format!("Failed to write {}", file.display())
            })?;
            count += 1;
        }
    }

    if count > 0 {
        println!(
            "[i18n] Injected hreflang + lang switcher into {count} HTML pages"
        );
    }

    Ok(())
}

/// Replaces the `<!-- ssg:lang-switcher -->` marker with a full language
/// switcher listing every available locale. Called by the i18n plugin
/// only when multiple locales are present on disk.
fn inject_lang_switcher(
    html: &str,
    current_locale: &str,
    rel_path: &str,
    locales: &[String],
    base_url: &str,
    strategy: &UrlPrefixStrategy,
) -> String {
    if !html.contains(LANG_SWITCHER_MARKER) {
        return html.to_string();
    }
    let mut sorted = locales.to_vec();
    sorted.sort();
    let switcher = generate_lang_switcher_html(
        &sorted,
        current_locale,
        rel_path,
        base_url,
        strategy,
    );
    html.replace(LANG_SWITCHER_MARKER, &switcher)
}

/// Marker comment embedded in templates where the language switcher
/// should be injected. Kept invisible in single-locale sites.
const LANG_SWITCHER_MARKER: &str = "<!-- ssg:lang-switcher -->";

/// Build the hreflang `<link>` block for a single page.
fn build_hreflang_links(
    rel_path: &str,
    page_locales: &HashSet<String>,
    default_locale: &str,
    base: &str,
    strategy: &UrlPrefixStrategy,
) -> String {
    let mut links = String::new();

    let mut sorted: Vec<&String> = page_locales.iter().collect();
    sorted.sort();

    for locale in &sorted {
        let href = build_url(base, locale, rel_path, strategy);
        links.push_str(&format!(
            "    <link rel=\"alternate\" hreflang=\"{locale}\" href=\"{href}\" />\n"
        ));
    }

    // x-default points to the default locale.
    let default_href = build_url(base, default_locale, rel_path, strategy);
    links.push_str(&format!(
        "    <link rel=\"alternate\" hreflang=\"x-default\" href=\"{default_href}\" />\n"
    ));

    links
}

/// Construct a full URL for a given locale + relative path.
fn build_url(
    base: &str,
    locale: &str,
    rel_path: &str,
    strategy: &UrlPrefixStrategy,
) -> String {
    match strategy {
        UrlPrefixStrategy::SubPath => {
            format!("{base}/{locale}/{rel_path}")
        }
        UrlPrefixStrategy::SubDomain => {
            // Replace scheme://host with scheme://locale.host
            if let Some(idx) = base.find("://") {
                let (scheme, rest) = base.split_at(idx + 3);
                format!("{scheme}{locale}.{rest}/{rel_path}")
            } else {
                // Fallback: treat as sub-path.
                format!("{base}/{locale}/{rel_path}")
            }
        }
    }
}

/// Insert `links` just before the first `</head>` tag, if present.
fn inject_before_head_close(html: &str, links: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let pos = lower.find("</head>")?;
    let mut result = String::with_capacity(html.len() + links.len());
    result.push_str(&html[..pos]);
    result.push_str(links);
    result.push_str(&html[pos..]);
    Some(result)
}

// ── Per-locale sitemaps ──────────────────────────────────────────────

/// Generate `sitemap-{locale}.xml` for every present locale.
fn generate_locale_sitemaps(
    site_dir: &Path,
    pages: &HashMap<String, HashSet<String>>,
    locales: &[String],
    default_locale: &str,
    base_url: &str,
    strategy: &UrlPrefixStrategy,
) -> Result<()> {
    let base = base_url.trim_end_matches('/');

    for locale in locales {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"\n\
                     xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
        );

        let mut paths: Vec<&String> = pages
            .iter()
            .filter(|(_, locs)| locs.contains(locale))
            .map(|(p, _)| p)
            .collect();
        paths.sort();

        for rel_path in &paths {
            let loc = build_url(base, locale, rel_path, strategy);
            xml.push_str("  <url>\n");
            xml.push_str(&format!("    <loc>{loc}</loc>\n"));

            // xhtml:link alternates for all locales that share this page.
            if let Some(page_locales) = pages.get(*rel_path) {
                let mut alts: Vec<&String> = page_locales.iter().collect();
                alts.sort();
                for alt_locale in &alts {
                    let alt_href =
                        build_url(base, alt_locale, rel_path, strategy);
                    xml.push_str(&format!(
                        "    <xhtml:link rel=\"alternate\" hreflang=\"{alt_locale}\" href=\"{alt_href}\" />\n"
                    ));
                }
                // x-default
                let default_href =
                    build_url(base, default_locale, rel_path, strategy);
                xml.push_str(&format!(
                    "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{default_href}\" />\n"
                ));
            }

            xml.push_str("  </url>\n");
        }

        xml.push_str("</urlset>\n");

        let sitemap_path = site_dir.join(format!("sitemap-{locale}.xml"));
        fs::write(&sitemap_path, &xml).with_context(|| {
            format!("Failed to write {}", sitemap_path.display())
        })?;
    }

    println!("[i18n] Generated {} locale sitemaps", locales.len());
    Ok(())
}

// ── Accept-Language parsing ─────────────────────────────────────────

/// Parses an Accept-Language header value into a sorted list of locale
/// preferences (highest quality first).
///
/// Example: "fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5"
/// Returns: `["fr-CH", "fr", "en", "de", "*"]`
#[must_use]
pub fn parse_accept_language(header: &str) -> Vec<String> {
    if header.trim().is_empty() {
        return Vec::new();
    }

    let mut entries: Vec<(String, f64)> = header
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let mut segments = part.splitn(2, ';');
            let locale = segments.next()?.trim().to_string();
            if locale.is_empty() {
                return None;
            }
            let quality = segments
                .next()
                .and_then(|q| {
                    let q = q.trim();
                    q.strip_prefix("q=")
                        .and_then(|v| v.trim().parse::<f64>().ok())
                })
                .unwrap_or(1.0);
            Some((locale, quality))
        })
        .collect();

    // Sort by quality descending; stable sort preserves order for equal quality.
    entries.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    entries.into_iter().map(|(locale, _)| locale).collect()
}

/// Given a list of preferred locales (from Accept-Language) and a list
/// of available locales (directories on disk), returns the best match.
///
/// Matching rules:
/// 1. Exact match (e.g., "fr-CH" matches "fr-CH")
/// 2. Prefix match (e.g., "fr-CH" matches "fr")
/// 3. Default locale fallback
#[must_use]
pub fn negotiate_locale(
    preferred: &[String],
    available: &[String],
    default_locale: &str,
) -> String {
    let available_lower: Vec<String> =
        available.iter().map(|l| l.to_lowercase()).collect();

    for pref in preferred {
        // Skip wildcard
        if pref == "*" {
            continue;
        }
        let pref_lower = pref.to_lowercase();

        // Exact match
        if let Some(idx) = available_lower.iter().position(|a| *a == pref_lower)
        {
            return available[idx].clone();
        }

        // Prefix match: preferred "fr-CH" matches available "fr"
        let prefix = pref_lower.split('-').next().unwrap_or(&pref_lower);
        if let Some(idx) = available_lower.iter().position(|a| *a == prefix) {
            return available[idx].clone();
        }
    }

    default_locale.to_string()
}

// ── Language switcher helper ─────────────────────────────────────────

/// Generates an HTML snippet for a language switcher navigation.
///
/// This is a pure function that can be called from any plugin or template
/// helper to produce a `<nav>` block with links to all locale variants
/// of the current page.
///
/// # Arguments
///
/// * `locales` — All available locales.
/// * `current_locale` — The locale of the page being rendered.
/// * `current_path` — The relative path of the page (e.g. `about/index.html`).
/// * `base_url` — The site base URL.
/// * `strategy` — How locale URLs are constructed.
///
/// # Example
///
/// ```rust
/// use ssg::i18n::{generate_lang_switcher_html, UrlPrefixStrategy};
///
/// let html = generate_lang_switcher_html(
///     &["en".into(), "fr".into(), "de".into()],
///     "en",
///     "about/index.html",
///     "https://example.com",
///     &UrlPrefixStrategy::SubPath,
/// );
/// assert!(html.contains("lang=\"fr\""));
/// ```
#[must_use]
pub fn generate_lang_switcher_html(
    locales: &[String],
    current_locale: &str,
    current_path: &str,
    base_url: &str,
    strategy: &UrlPrefixStrategy,
) -> String {
    let base = base_url.trim_end_matches('/');
    let mut html = String::from(
        "<nav class=\"lang-switcher\" aria-label=\"Language\">\n  <ul>\n",
    );

    for locale in locales {
        let href = build_url(base, locale, current_path, strategy);
        let aria = if locale == current_locale {
            " aria-current=\"page\""
        } else {
            ""
        };
        html.push_str(&format!(
            "    <li><a href=\"{href}\" lang=\"{locale}\" hreflang=\"{locale}\"{aria}>{locale}</a></li>\n"
        ));
    }

    html.push_str("  </ul>\n</nav>\n");
    html
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use std::path::Path;
    use tempfile::tempdir;

    fn make_ctx(site_dir: &Path) -> PluginContext {
        let config = crate::cmd::SsgConfig::builder()
            .site_name("test".to_string())
            .base_url("https://example.com".to_string())
            .build()
            .expect("test config");
        PluginContext::with_config(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
            config,
        )
    }

    /// Helper: create an HTML file with a `</head>` tag.
    fn write_html(dir: &Path, rel: &str, body: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        let html = format!(
            "<!DOCTYPE html><html><head><title>Test</title></head><body>{body}</body></html>"
        );
        fs::write(&path, html).expect("write html");
    }

    // ── detect_locale_dirs ───────────────────────────────────────

    #[test]
    fn detect_finds_existing_locale_dirs() {
        let tmp = tempdir().unwrap();
        fs::create_dir(tmp.path().join("en")).unwrap();
        fs::create_dir(tmp.path().join("fr")).unwrap();

        let found = detect_locale_dirs(
            tmp.path(),
            &["en".into(), "fr".into(), "de".into()],
        );
        assert_eq!(found, vec!["en", "fr"]);
    }

    #[test]
    fn detect_returns_empty_when_none_exist() {
        let tmp = tempdir().unwrap();
        let found = detect_locale_dirs(tmp.path(), &["en".into(), "fr".into()]);
        assert!(found.is_empty());
    }

    // ── hreflang injection ───────────────────────────────────────

    #[test]
    fn injects_hreflang_into_shared_pages() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "Hello");
        write_html(site, "fr/index.html", "Bonjour");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        let plugin = I18nPlugin::new(config);
        plugin.after_compile(&ctx).unwrap();

        // Both files should contain hreflang links.
        let en = fs::read_to_string(site.join("en/index.html")).unwrap();
        let fr = fs::read_to_string(site.join("fr/index.html")).unwrap();

        assert!(en.contains(HREFLANG_MARKER), "en missing hreflang");
        assert!(fr.contains(HREFLANG_MARKER), "fr missing hreflang");

        // Check x-default points to en.
        assert!(
            en.contains("hreflang=\"x-default\""),
            "en missing x-default"
        );
        assert!(
            en.contains("https://example.com/en/index.html"),
            "en x-default wrong href"
        );
    }

    #[test]
    fn skips_pages_existing_in_only_one_locale() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "Hello");
        write_html(site, "en/about.html", "About");
        // fr only has index
        write_html(site, "fr/index.html", "Bonjour");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        // about.html only exists in en — should NOT have hreflang.
        let about = fs::read_to_string(site.join("en/about.html")).unwrap();
        assert!(
            !about.contains(HREFLANG_MARKER),
            "about.html should not have hreflang"
        );
    }

    #[test]
    fn idempotent_injection() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "Hello");
        write_html(site, "fr/index.html", "Bonjour");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        let plugin = I18nPlugin::new(config);

        // Run twice.
        plugin.after_compile(&ctx).unwrap();
        plugin.after_compile(&ctx).unwrap();

        let en = fs::read_to_string(site.join("en/index.html")).unwrap();
        let count = en.matches(HREFLANG_MARKER).count();
        // en + fr + x-default = 3 links, and only one run should inject.
        assert_eq!(count, 3, "expected 3 hreflang links, got {count}");
    }

    // ── x-default ────────────────────────────────────────────────

    #[test]
    fn x_default_points_to_default_locale() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/page.html", "EN");
        write_html(site, "fr/page.html", "FR");
        write_html(site, "de/page.html", "DE");

        let config = I18nConfig {
            default_locale: "fr".into(),
            locales: vec!["en".into(), "fr".into(), "de".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en = fs::read_to_string(site.join("en/page.html")).unwrap();
        // x-default should point to fr (the configured default).
        assert!(
            en.contains("hreflang=\"x-default\" href=\"https://example.com/fr/page.html\""),
            "x-default should point to fr"
        );
    }

    // ── multi-locale detection ───────────────────────────────────

    #[test]
    fn three_locale_injection() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "EN");
        write_html(site, "fr/index.html", "FR");
        write_html(site, "de/index.html", "DE");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into(), "de".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en = fs::read_to_string(site.join("en/index.html")).unwrap();
        // Should have de, en, fr + x-default = 4 links.
        let count = en.matches(HREFLANG_MARKER).count();
        assert_eq!(
            count, 4,
            "expected 4 hreflang links for 3 locales + x-default"
        );
    }

    // ── sitemap generation ───────────────────────────────────────

    #[test]
    fn generates_per_locale_sitemaps() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "EN");
        write_html(site, "fr/index.html", "FR");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en_sm = site.join("sitemap-en.xml");
        let fr_sm = site.join("sitemap-fr.xml");
        assert!(en_sm.exists(), "sitemap-en.xml should exist");
        assert!(fr_sm.exists(), "sitemap-fr.xml should exist");

        let en_content = fs::read_to_string(&en_sm).unwrap();
        assert!(
            en_content.contains("<loc>https://example.com/en/index.html</loc>")
        );
        assert!(en_content.contains("xhtml:link"));
        assert!(en_content.contains("hreflang=\"x-default\""));
    }

    // ── SubDomain strategy ───────────────────────────────────────

    #[test]
    fn subdomain_strategy_builds_correct_urls() {
        let url = build_url(
            "https://example.com",
            "fr",
            "about/index.html",
            &UrlPrefixStrategy::SubDomain,
        );
        assert_eq!(url, "https://fr.example.com/about/index.html");
    }

    #[test]
    fn subpath_strategy_builds_correct_urls() {
        let url = build_url(
            "https://example.com",
            "fr",
            "about/index.html",
            &UrlPrefixStrategy::SubPath,
        );
        assert_eq!(url, "https://example.com/fr/about/index.html");
    }

    // ── Language switcher ────────────────────────────────────────

    #[test]
    fn lang_switcher_html() {
        let html = generate_lang_switcher_html(
            &["en".into(), "fr".into()],
            "en",
            "about/index.html",
            "https://example.com",
            &UrlPrefixStrategy::SubPath,
        );
        assert!(html.contains("lang=\"en\""));
        assert!(html.contains("lang=\"fr\""));
        assert!(html.contains("aria-current=\"page\""));
        assert!(html.contains("class=\"lang-switcher\""));
    }

    // ── inject_before_head_close ─────────────────────────────────

    #[test]
    fn inject_before_head_close_works() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let result = inject_before_head_close(html, "INJECTED\n").unwrap();
        assert!(result.contains("INJECTED\n</head>"));
    }

    #[test]
    fn inject_before_head_close_returns_none_without_head() {
        let html = "<html><body>no head</body></html>";
        assert!(inject_before_head_close(html, "X").is_none());
    }

    // ── Plugin basics ────────────────────────────────────────────

    #[test]
    fn plugin_name() {
        let p = I18nPlugin::new(I18nConfig::default());
        assert_eq!(p.name(), "i18n");
    }

    #[test]
    fn plugin_skips_nonexistent_site_dir() {
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            Path::new("/does/not/exist"),
            Path::new("t"),
        );
        let p = I18nPlugin::new(I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        });
        assert!(p.after_compile(&ctx).is_ok());
    }

    #[test]
    fn plugin_skips_single_locale() {
        let tmp = tempdir().unwrap();
        let ctx = make_ctx(tmp.path());
        let p = I18nPlugin::new(I18nConfig::default());
        // Default has only "en" — should be a no-op.
        assert!(p.after_compile(&ctx).is_ok());
    }

    // ── I18nConfig defaults ──────────────────────────────────────

    #[test]
    fn default_config() {
        let cfg = I18nConfig::default();
        assert_eq!(cfg.default_locale, "en");
        assert_eq!(cfg.locales, vec!["en"]);
        assert_eq!(cfg.url_prefix, UrlPrefixStrategy::SubPath);
    }

    // ── Nested page paths ────────────────────────────────────────

    // ── Language switcher edge cases ────────────────────────────────

    #[test]
    fn lang_switcher_empty_locales() {
        let html = generate_lang_switcher_html(
            &[],
            "en",
            "index.html",
            "https://example.com",
            &UrlPrefixStrategy::SubPath,
        );
        assert!(html.contains("<nav"));
        assert!(html.contains("</nav>"));
        // No <li> items
        assert!(!html.contains("<li>"));
    }

    #[test]
    fn lang_switcher_single_locale() {
        let html = generate_lang_switcher_html(
            &["en".into()],
            "en",
            "index.html",
            "https://example.com",
            &UrlPrefixStrategy::SubPath,
        );
        assert!(html.contains("aria-current=\"page\""));
        // Only one <li>
        assert_eq!(html.matches("<li>").count(), 1);
    }

    #[test]
    fn lang_switcher_subdomain_strategy() {
        let html = generate_lang_switcher_html(
            &["en".into(), "fr".into()],
            "fr",
            "about/index.html",
            "https://example.com",
            &UrlPrefixStrategy::SubDomain,
        );
        assert!(html.contains("https://en.example.com/about/index.html"));
        assert!(html.contains("https://fr.example.com/about/index.html"));
    }

    // ── Per-locale sitemap with xhtml:link alternates ────────────

    #[test]
    fn sitemap_contains_xhtml_link_alternates() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "EN");
        write_html(site, "fr/index.html", "FR");
        write_html(site, "de/index.html", "DE");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into(), "de".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en_sm = fs::read_to_string(site.join("sitemap-en.xml")).unwrap();
        // Should contain xhtml:link alternates for all 3 locales + x-default
        assert!(en_sm.contains("hreflang=\"en\""));
        assert!(en_sm.contains("hreflang=\"fr\""));
        assert!(en_sm.contains("hreflang=\"de\""));
        assert!(en_sm.contains("hreflang=\"x-default\""));
        // x-default should point to en (default locale)
        assert!(en_sm.contains(
            "hreflang=\"x-default\" href=\"https://example.com/en/index.html\""
        ));
    }

    // ── I18nPlugin with actual locale directories ───────────────

    #[test]
    fn plugin_with_locale_dirs_but_no_shared_pages_skips_injection() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        // en has page A, fr has page B — no overlap
        write_html(site, "en/about.html", "EN About");
        write_html(site, "fr/contact.html", "FR Contact");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        // No hreflang should be injected since no pages are shared
        let en = fs::read_to_string(site.join("en/about.html")).unwrap();
        let fr = fs::read_to_string(site.join("fr/contact.html")).unwrap();
        assert!(!en.contains(HREFLANG_MARKER));
        assert!(!fr.contains(HREFLANG_MARKER));
    }

    #[test]
    fn plugin_skips_when_only_one_locale_dir_exists() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        // Only en directory exists, fr is configured but missing
        write_html(site, "en/index.html", "EN");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en = fs::read_to_string(site.join("en/index.html")).unwrap();
        assert!(!en.contains(HREFLANG_MARKER));
    }

    // ── build_url subdomain fallback ────────────────────────────

    #[test]
    fn subdomain_strategy_fallback_without_scheme() {
        // When base has no "://" it falls back to sub-path style
        let url = build_url(
            "example.com",
            "fr",
            "page.html",
            &UrlPrefixStrategy::SubDomain,
        );
        assert_eq!(url, "example.com/fr/page.html");
    }

    #[test]
    fn nested_pages_get_hreflang() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/docs/guide.html", "EN Guide");
        write_html(site, "fr/docs/guide.html", "FR Guide");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let en = fs::read_to_string(site.join("en/docs/guide.html")).unwrap();
        assert!(en.contains(HREFLANG_MARKER));
        assert!(en.contains("https://example.com/en/docs/guide.html"));
        assert!(en.contains("https://example.com/fr/docs/guide.html"));
    }

    // ── parse_accept_language ───────────────────────────────────

    #[test]
    fn parse_accept_language_basic() {
        let result = parse_accept_language("en, fr, de");
        assert_eq!(result, vec!["en", "fr", "de"]);
    }

    #[test]
    fn parse_accept_language_with_quality() {
        let result = parse_accept_language(
            "fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5",
        );
        assert_eq!(result, vec!["fr-CH", "fr", "en", "de", "*"]);
    }

    #[test]
    fn parse_accept_language_with_whitespace() {
        let result = parse_accept_language("  en , fr ; q=0.8 , de ; q=0.5 ");
        assert_eq!(result, vec!["en", "fr", "de"]);
    }

    #[test]
    fn parse_accept_language_empty() {
        let result = parse_accept_language("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_accept_language_single() {
        let result = parse_accept_language("en");
        assert_eq!(result, vec!["en"]);
    }

    #[test]
    fn parse_accept_language_wildcard_only() {
        let result = parse_accept_language("*");
        assert_eq!(result, vec!["*"]);
    }

    // ── negotiate_locale ────────────────────────────────────────

    #[test]
    fn negotiate_exact_match() {
        let preferred = vec!["fr".into()];
        let available = vec!["en".into(), "fr".into(), "de".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "en"), "fr");
    }

    #[test]
    fn negotiate_prefix_match() {
        let preferred = vec!["fr-CH".into()];
        let available = vec!["en".into(), "fr".into(), "de".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "en"), "fr");
    }

    #[test]
    fn negotiate_default_fallback() {
        let preferred = vec!["ja".into()];
        let available = vec!["en".into(), "fr".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "en"), "en");
    }

    #[test]
    fn negotiate_case_insensitive() {
        let preferred = vec!["FR".into()];
        let available = vec!["en".into(), "fr".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "en"), "fr");
    }

    #[test]
    fn negotiate_wildcard_ignored() {
        let preferred = vec!["*".into()];
        let available = vec!["en".into(), "fr".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "en"), "en");
    }

    #[test]
    fn negotiate_no_match_returns_default() {
        let preferred: Vec<String> = vec![];
        let available = vec!["en".into(), "fr".into()];
        assert_eq!(negotiate_locale(&preferred, &available, "fr"), "fr");
    }

    // ── generate_locale_redirect ────────────────────────────────

    #[test]
    fn locale_redirect_contains_all_locales() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();
        fs::create_dir_all(site).unwrap();

        let locales = vec!["en".into(), "fr".into(), "de".into()];
        crate::server::generate_locale_redirect(site, &locales, "en").unwrap();

        let content = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(content.contains("\"en\""), "missing en locale");
        assert!(content.contains("\"fr\""), "missing fr locale");
        assert!(content.contains("\"de\""), "missing de locale");
    }

    #[test]
    fn locale_redirect_noscript_fallback() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();
        fs::create_dir_all(site).unwrap();

        crate::server::generate_locale_redirect(
            site,
            &["en".into(), "fr".into()],
            "en",
        )
        .unwrap();

        let content = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(content.contains("<noscript>"), "missing noscript tag");
        assert!(
            content.contains("url=/en/"),
            "noscript should redirect to default locale"
        );
    }

    #[test]
    fn locale_redirect_preserves_existing_non_redirect_index() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();
        fs::create_dir_all(site).unwrap();

        // Write a custom index.html first
        fs::write(site.join("index.html"), "<html>Custom</html>").unwrap();

        crate::server::generate_locale_redirect(
            site,
            &["en".into(), "fr".into()],
            "en",
        )
        .unwrap();

        let content = fs::read_to_string(site.join("index.html")).unwrap();
        assert_eq!(content, "<html>Custom</html>");
    }

    #[test]
    fn after_compile_generates_locale_redirect() {
        let tmp = tempdir().unwrap();
        let site = tmp.path();

        write_html(site, "en/index.html", "EN");
        write_html(site, "fr/index.html", "FR");

        let config = I18nConfig {
            default_locale: "en".into(),
            locales: vec!["en".into(), "fr".into()],
            url_prefix: UrlPrefixStrategy::SubPath,
        };

        let ctx = make_ctx(site);
        I18nPlugin::new(config).after_compile(&ctx).unwrap();

        let index = site.join("index.html");
        assert!(index.exists(), "root index.html should be generated");
        let content = fs::read_to_string(&index).unwrap();
        assert!(content.contains("ssg-locale-redirect"));
        assert!(content.contains("\"en\""));
        assert!(content.contains("\"fr\""));
    }
}
