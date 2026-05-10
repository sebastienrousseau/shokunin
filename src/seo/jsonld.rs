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

// =====================================================================
// JSON-LD validation (resolves #467)
// =====================================================================

/// A single validation failure against a JSON-LD block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonLdValidationError {
    /// The schema.org `@type` of the block (or "Unknown" if absent).
    pub schema_type: String,
    /// Required field that was missing or had the wrong shape.
    pub field: String,
    /// Human-readable reason.
    pub reason: String,
}

impl std::fmt::Display for JsonLdValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] missing/invalid `{}` — {}",
            self.schema_type, self.field, self.reason
        )
    }
}

/// Walks an HTML string, extracts every `<script type="application/ld+json">`
/// block, parses it as JSON, and validates required fields per
/// schema.org `@type`.
///
/// Supported types (with their required-field guards):
///
/// - **`Article`** — `headline`, `datePublished`, `author`, `image`
/// - **`WebPage`** — `name`, `url`, `inLanguage`
/// - **`BreadcrumbList`** — `itemListElement` (non-empty array)
/// - **`FAQPage`** — `mainEntity` (non-empty array of `Question`)
/// - **`LocalBusiness`** — `name`, `address`
/// - **`Organization`** — `name`, `url`
///
/// Returns the empty vector if every block parses and passes its
/// required-field check. Unknown `@type` values are treated as
/// pass-through (no required fields enforced) so user-extended
/// schemas don't trigger false negatives.
#[must_use]
pub fn validate_jsonld(html: &str) -> Vec<JsonLdValidationError> {
    let mut errors = Vec::new();

    for block in extract_jsonld_blocks(html) {
        match serde_json::from_str::<serde_json::Value>(&block) {
            Ok(value) => validate_one(&value, &mut errors),
            Err(parse_err) => {
                errors.push(JsonLdValidationError {
                    schema_type: "Unparseable".to_string(),
                    field: "(payload)".to_string(),
                    reason: format!("invalid JSON: {parse_err}"),
                });
            }
        }
    }

    errors
}

/// Returns the inner JSON of every `<script type="application/ld+json">`
/// block. Tolerant of attribute order and whitespace.
///
/// Resolves audit items #4 + #5:
/// - `type` is parsed as a discrete attribute value rather than
///   substring-matched, so `type="application/ld+json/extra"` no
///   longer falsely qualifies.
/// - The `</script>` close finder is JSON-string-aware: a literal
///   `</script>` *inside* a JSON string value (e.g.
///   `"description": "code: </script>"`) is correctly skipped over.
///   The HTML5 spec actually forbids `</script>` inside script
///   bodies even in strings — most authors escape as `<\/script>`
///   — but our extractor handles either form gracefully.
fn extract_jsonld_blocks(html: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let lower = html.to_lowercase();
    let mut cursor = 0;

    while let Some(rel_open) = lower[cursor..].find("<script") {
        let abs_open = cursor + rel_open;
        // Use find_tag_end equivalent: advance past `>` while
        // skipping any `>` characters that appear inside quoted
        // attribute values. Without this, `<script type="text/x>y">`
        // would close prematurely at the inner `>`.
        let tag_end = find_html_tag_end(&lower, abs_open);
        let tag = &lower[abs_open..tag_end];
        cursor = tag_end;

        if !is_jsonld_script_tag(tag) {
            continue;
        }

        let Some(close) = find_script_close_skipping_strings(&html[cursor..])
        else {
            break;
        };
        // Use the original-case slice for the JSON payload —
        // schema.org values are case-sensitive.
        blocks.push(html[cursor..cursor + close].trim().to_string());
        cursor += close + "</script>".len();
    }

    blocks
}

/// Returns `true` if the `<script ...>` tag declares
/// `type="application/ld+json"` exactly (any quoting; no
/// substring match).
fn is_jsonld_script_tag(tag: &str) -> bool {
    extract_attr(tag, "type")
        .is_some_and(|v| v.eq_ignore_ascii_case("application/ld+json"))
}

/// Extracts the value of an HTML attribute from an open-tag string.
/// Tolerant of quoting and whitespace. Returns `None` if the
/// attribute is absent or has no value.
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let needle = format!("{}=", name.to_lowercase());
    let idx = lower.find(&needle)?;
    // Make sure the match starts at a token boundary (preceding
    // char is whitespace or `<` or the very start of `tag`).
    let pre = lower.as_bytes().get(idx.wrapping_sub(1));
    let boundary_ok = idx == 0
        || matches!(pre, Some(b) if b.is_ascii_whitespace() || *b == b'<');
    if !boundary_ok {
        return None;
    }
    let rest = &tag[idx + needle.len()..];
    let trimmed = rest.trim_start();
    if let Some(s) = trimmed.strip_prefix('"') {
        s.find('"').map(|e| s[..e].to_string())
    } else if let Some(s) = trimmed.strip_prefix('\'') {
        s.find('\'').map(|e| s[..e].to_string())
    } else {
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(trimmed.len());
        Some(trimmed[..end].to_string())
    }
}

/// Returns the byte offset of `</script>` in `body` while ignoring
/// occurrences that appear *inside* a JSON string literal.
///
/// The walker tracks two pieces of state: whether we're currently
/// inside a `"..."` string, and whether the previous byte was the
/// JSON escape character `\`. Scanning is done in bytes (UTF-8 is
/// not relevant for the ASCII-only delimiters we care about).
fn find_script_close_skipping_strings(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let needle = b"</script>";
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        if in_string {
            if escape {
                escape = false;
            } else if bytes[i] == b'\\' {
                escape = true;
            } else if bytes[i] == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        // Case-insensitive check for `</script>`.
        if i + needle.len() <= bytes.len()
            && bytes[i..i + needle.len()].eq_ignore_ascii_case(needle)
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Like `accessibility::find_tag_end` — returns the index just past
/// the `>` that closes the open tag at `tag_start`, while skipping
/// `>` characters that occur inside quoted attribute values.
fn find_html_tag_end(html: &str, tag_start: usize) -> usize {
    let bytes = html.as_bytes();
    let mut i = tag_start;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return i + 1,
                _ => {}
            },
        }
        i += 1;
    }
    bytes.len()
}

/// Validates a single parsed JSON-LD value (object or array).
fn validate_one(
    value: &serde_json::Value,
    errors: &mut Vec<JsonLdValidationError>,
) {
    // schema.org allows top-level @graph arrays; descend into them.
    if let Some(graph) = value.get("@graph").and_then(|v| v.as_array()) {
        for entry in graph {
            validate_one(entry, errors);
        }
        return;
    }

    // Array at top level — validate each entry.
    if let Some(array) = value.as_array() {
        for entry in array {
            validate_one(entry, errors);
        }
        return;
    }

    let schema_type = value
        .get("@type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let required: &[&str] = match schema_type.as_str() {
        "Article" | "NewsArticle" | "BlogPosting" => {
            &["headline", "datePublished", "author", "image"]
        }
        "WebPage" => &["name", "url", "inLanguage"],
        "BreadcrumbList" => &["itemListElement"],
        "FAQPage" => &["mainEntity"],
        "LocalBusiness" | "Restaurant" | "Store" => &["name", "address"],
        "Organization" => &["name", "url"],
        // Unknown types: don't enforce required fields. Users may ship
        // custom @types that are still valid schema.org extensions.
        _ => return,
    };

    for field in required {
        match value.get(*field) {
            None => errors.push(JsonLdValidationError {
                schema_type: schema_type.clone(),
                field: (*field).to_string(),
                reason: "field absent".to_string(),
            }),
            Some(serde_json::Value::Null) => {
                errors.push(JsonLdValidationError {
                    schema_type: schema_type.clone(),
                    field: (*field).to_string(),
                    reason: "field is null".to_string(),
                });
            }
            Some(serde_json::Value::String(s)) if s.trim().is_empty() => {
                errors.push(JsonLdValidationError {
                    schema_type: schema_type.clone(),
                    field: (*field).to_string(),
                    reason: "field is empty string".to_string(),
                });
            }
            Some(serde_json::Value::Array(a)) if a.is_empty() => {
                errors.push(JsonLdValidationError {
                    schema_type: schema_type.clone(),
                    field: (*field).to_string(),
                    reason: "array is empty".to_string(),
                });
            }
            _ => {}
        }
    }

    // BreadcrumbList: itemListElement entries should each be ListItem
    // with a `position` and `name`. Catch the most common regression.
    if schema_type == "BreadcrumbList" {
        if let Some(items) =
            value.get("itemListElement").and_then(|v| v.as_array())
        {
            for (idx, item) in items.iter().enumerate() {
                if item.get("position").is_none() {
                    errors.push(JsonLdValidationError {
                        schema_type: schema_type.clone(),
                        field: format!("itemListElement[{idx}].position"),
                        reason: "ListItem missing position".to_string(),
                    });
                }
                if item.get("name").is_none() && item.get("item").is_none() {
                    errors.push(JsonLdValidationError {
                        schema_type: schema_type.clone(),
                        field: format!("itemListElement[{idx}].name|item"),
                        reason: "ListItem missing name and item".to_string(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

    // ── JSON-LD validation (issue #467) ────────────────────────────

    #[test]
    fn validate_extracts_block() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"WebPage",
             "name":"Hi","url":"https://x.test/","inLanguage":"en"}
            </script></head><body></body></html>"#;
        assert!(validate_jsonld(html).is_empty());
    }

    #[test]
    fn validate_flags_missing_required_field_on_article() {
        let html = r#"<script type="application/ld+json">
            {"@context":"https://schema.org","@type":"Article",
             "headline":"H","datePublished":"2026-05-10","author":"A"}
        </script>"#;
        let errs = validate_jsonld(html);
        assert!(
            errs.iter()
                .any(|e| e.schema_type == "Article" && e.field == "image"),
            "expected Article.image violation, got {errs:?}"
        );
    }

    #[test]
    fn validate_flags_empty_breadcrumb_list() {
        let html = r#"<script type="application/ld+json">
            {"@context":"https://schema.org","@type":"BreadcrumbList",
             "itemListElement":[]}
        </script>"#;
        let errs = validate_jsonld(html);
        assert!(
            errs.iter().any(|e| e.field == "itemListElement"),
            "expected itemListElement empty-array error, got {errs:?}"
        );
    }

    #[test]
    fn validate_breadcrumb_listitem_missing_position() {
        let html = r#"<script type="application/ld+json">
            {"@type":"BreadcrumbList",
             "itemListElement":[{"name":"Home","item":"https://x/"}]}
        </script>"#;
        let errs = validate_jsonld(html);
        assert!(
            errs.iter()
                .any(|e| e.field == "itemListElement[0].position"),
            "expected position-missing error, got {errs:?}"
        );
    }

    #[test]
    fn validate_unparseable_json() {
        let html = r#"<script type="application/ld+json">{not json}</script>"#;
        let errs = validate_jsonld(html);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].schema_type, "Unparseable");
    }

    #[test]
    fn validate_descends_into_graph() {
        let html = r#"<script type="application/ld+json">
            {"@context":"https://schema.org","@graph":[
                {"@type":"WebPage","name":"H"}
            ]}
        </script>"#;
        let errs = validate_jsonld(html);
        // WebPage missing url and inLanguage
        assert!(errs
            .iter()
            .any(|e| e.schema_type == "WebPage" && e.field == "url"));
        assert!(errs
            .iter()
            .any(|e| e.schema_type == "WebPage" && e.field == "inLanguage"));
    }

    #[test]
    fn validate_unknown_type_passes_through() {
        let html = r#"<script type="application/ld+json">
            {"@type":"CustomThing","foo":"bar"}
        </script>"#;
        assert!(validate_jsonld(html).is_empty());
    }

    #[test]
    fn validate_handles_multiple_blocks() {
        let html = r#"
            <script type="application/ld+json">{"@type":"Organization","name":"O","url":"https://o/"}</script>
            <script type="application/ld+json">{"@type":"Article","headline":"H"}</script>
        "#;
        let errs = validate_jsonld(html);
        // Org passes; Article missing 3 of 4 required.
        assert_eq!(
            errs.iter()
                .filter(|e| e.schema_type == "Organization")
                .count(),
            0
        );
        assert!(
            errs.iter().filter(|e| e.schema_type == "Article").count() >= 3
        );
    }

    // ── Strict type-attribute parsing (audit fix item #4) ──────────

    #[test]
    fn validate_skips_extra_qualified_type() {
        // `application/ld+json/extra` must NOT be treated as JSON-LD.
        // Pre-fix: `tag.contains("application/ld+json")` falsely
        // matched this.
        let html = r#"<script type="application/ld+json/extra">
            {"@type":"Article"}
        </script>"#;
        assert!(
            validate_jsonld(html).is_empty(),
            "non-JSON-LD type must not be validated"
        );
    }

    #[test]
    fn validate_recognises_type_with_single_quotes() {
        let html = r#"<script type='application/ld+json'>
            {"@type":"Organization","name":"O","url":"https://o/"}
        </script>"#;
        assert!(validate_jsonld(html).is_empty());
    }

    #[test]
    fn validate_recognises_type_after_other_attrs() {
        let html = r#"<script id="ld1" type="application/ld+json">
            {"@type":"Organization","name":"O","url":"https://o/"}
        </script>"#;
        assert!(validate_jsonld(html).is_empty());
    }

    // ── String-literal-aware </script> finder (audit fix item #5) ──

    #[test]
    fn validate_handles_close_script_inside_json_string() {
        // The old extractor truncated at the first `</script>` inside
        // a string value, producing parse-failure noise. The fixed
        // extractor only honours `</script>` outside JSON strings.
        let html = r#"<script type="application/ld+json">
            {"@type":"Article",
             "headline":"H","datePublished":"2026-01-01",
             "author":"A","image":"https://x/i.png",
             "description":"this contains a </script> inside the string and is still valid JSON"}
        </script>"#;
        let errs = validate_jsonld(html);
        // Article has all 4 required fields. The pre-fix bug would
        // have produced an Unparseable error because the extractor
        // would close at the inner `</script>`, leaving truncated
        // JSON.
        assert!(
            errs.iter().all(|e| e.schema_type != "Unparseable"),
            "no parse errors expected, got {errs:?}"
        );
    }

    #[test]
    fn extract_attr_returns_none_when_attribute_absent() {
        assert_eq!(extract_attr("<script src=x>", "type"), None);
    }

    #[test]
    fn extract_attr_handles_double_quoted_value() {
        assert_eq!(
            extract_attr(r#"<script type="application/ld+json">"#, "type"),
            Some("application/ld+json".to_string())
        );
    }

    #[test]
    fn extract_attr_rejects_substring_match_in_other_attribute() {
        // `data-mytype="foo"` must NOT match a `type=` query.
        assert_eq!(extract_attr(r#"<script data-mytype="foo">"#, "type"), None);
    }
}
