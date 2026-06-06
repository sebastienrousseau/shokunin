// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SEO meta tag injection plugin.

use super::helpers::{
    escape_attr, extract_canonical, extract_description, extract_existing_meta,
    extract_first_content_image, extract_html_lang, extract_title,
    has_meta_tag,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::path::Path;

/// Injects missing SEO meta tags into HTML files.
///
/// After compilation, this plugin scans all HTML files in the site
/// directory and adds any missing meta tags for description, Open Graph
/// (title, description, type), and Twitter Card.
///
/// The plugin is idempotent — it checks for existing tags before
/// injecting and will not duplicate them.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::SeoPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(SeoPlugin);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SeoPlugin;

impl Plugin for SeoPlugin {
    fn name(&self) -> &'static str {
        "seo"
    }

    fn has_transform(&self) -> bool {
        true
    }

    fn transform_html(
        &self,
        html: &str,
        _path: &Path,
        _ctx: &PluginContext,
    ) -> Result<String> {
        inject_seo_tags_html(html)
    }

    fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}

/// Builds Open Graph meta tags that are missing from the HTML.
fn build_og_tags(
    html: &str,
    title: &str,
    description: &str,
    canonical: &str,
    og_type: &str,
) -> Vec<String> {
    let mut tags = Vec::new();

    if !has_meta_tag(html, "og:title") && !title.is_empty() {
        tags.push(format!(
            "<meta property=\"og:title\" content=\"{}\">",
            escape_attr(title)
        ));
    }

    if !has_meta_tag(html, "og:description") && !description.is_empty() {
        tags.push(format!(
            "<meta property=\"og:description\" content=\"{}\">",
            escape_attr(description)
        ));
    }

    if !has_meta_tag(html, "og:type") {
        tags.push(format!("<meta property=\"og:type\" content=\"{og_type}\">"));
    }

    if !has_meta_tag(html, "og:url") && !canonical.is_empty() {
        tags.push(format!(
            "<meta property=\"og:url\" content=\"{}\">",
            escape_attr(canonical)
        ));
    }

    // OG image: extract from existing meta or first <img> in content
    if !has_meta_tag(html, "og:image") {
        let image = extract_existing_meta(html, "twitter:image");
        let image = if image.is_empty() {
            extract_first_content_image(html)
        } else {
            image
        };
        if !image.is_empty() {
            tags.push(format!(
                "<meta property=\"og:image\" content=\"{}\">",
                escape_attr(&image)
            ));
            // Social platforms render cards faster with explicit dimensions
            if !has_meta_tag(html, "og:image:width") {
                tags.push(
                    "<meta property=\"og:image:width\" content=\"1200\">"
                        .to_string(),
                );
                tags.push(
                    "<meta property=\"og:image:height\" content=\"630\">"
                        .to_string(),
                );
            }
        }
    }

    // OG locale
    if !has_meta_tag(html, "og:locale") {
        let lang = extract_html_lang(html);
        if !lang.is_empty() {
            let locale = lang.replace('-', "_");
            tags.push(format!(
                "<meta property=\"og:locale\" content=\"{}\">",
                escape_attr(&locale)
            ));
        }
    }

    tags
}

/// Builds Twitter Card meta tags that are missing from the HTML.
fn build_twitter_tags(
    html: &str,
    title: &str,
    description: &str,
    twitter_card: &str,
) -> Vec<String> {
    let mut tags = Vec::new();

    if !has_meta_tag(html, "twitter:card") {
        tags.push(format!(
            "<meta name=\"twitter:card\" content=\"{twitter_card}\">"
        ));
    }

    if !has_meta_tag(html, "twitter:title") && !title.is_empty() {
        tags.push(format!(
            "<meta name=\"twitter:title\" content=\"{}\">",
            escape_attr(title)
        ));
    }

    if !has_meta_tag(html, "twitter:description") && !description.is_empty() {
        tags.push(format!(
            "<meta name=\"twitter:description\" content=\"{}\">",
            escape_attr(description)
        ));
    }

    if !has_meta_tag(html, "twitter:image") {
        let image = extract_existing_meta(html, "og:image");
        let image = if image.is_empty() {
            extract_first_content_image(html)
        } else {
            image
        };
        if !image.is_empty() {
            tags.push(format!(
                "<meta name=\"twitter:image\" content=\"{}\">",
                escape_attr(&image)
            ));
        }
    }

    tags
}

/// Builds the meta description tag if missing from the HTML.
fn build_meta_description(html: &str, description: &str) -> Option<String> {
    if !has_meta_tag(html, "description") && !description.is_empty() {
        Some(format!(
            "<meta name=\"description\" content=\"{}\">",
            escape_attr(description)
        ))
    } else {
        None
    }
}

/// Inject missing SEO meta tags into an HTML string, returning the modified HTML.
fn inject_seo_tags_html(html: &str) -> Result<String> {
    let title = extract_title(html);
    let description = extract_description(html, 160);
    let canonical = extract_canonical(html);

    let is_article = html.contains("<article");
    let og_type = if is_article { "article" } else { "website" };
    let twitter_card = if is_article {
        "summary_large_image"
    } else {
        "summary"
    };

    let mut tags = Vec::new();

    if let Some(meta_desc) = build_meta_description(html, &description) {
        tags.push(meta_desc);
    }
    tags.extend(build_og_tags(
        html,
        &title,
        &description,
        &canonical,
        og_type,
    ));
    tags.extend(build_twitter_tags(html, &title, &description, twitter_card));

    if tags.is_empty() {
        return Ok(html.to_string());
    }

    let injection = tags.join("\n");
    let result = if let Some(pos) = html.find("</head>") {
        format!("{}{}\n{}", &html[..pos], injection, &html[pos..])
    } else {
        html.to_string()
    };

    Ok(result)
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

    #[test]
    fn name_is_stable() {
        assert_eq!(SeoPlugin.name(), "seo");
    }

    #[test]
    fn no_op_when_site_dir_missing() {
        let dir = tempdir().unwrap();
        SeoPlugin
            .after_compile(&ctx(&dir.path().join("nope")))
            .unwrap();
    }

    // ── build_meta_description ──────────────────────────────────

    #[test]
    fn meta_description_built_when_missing_and_text_provided() {
        let html = r#"<html><head><title>X</title></head><body></body></html>"#;
        let out = build_meta_description(html, "A cool page");
        assert_eq!(
            out.as_deref(),
            Some(r#"<meta name="description" content="A cool page">"#)
        );
    }

    #[test]
    fn meta_description_skipped_when_empty_text() {
        let html = "<html><head></head></html>";
        assert!(build_meta_description(html, "").is_none());
    }

    #[test]
    fn meta_description_skipped_when_already_present() {
        let html = r#"<html><head><meta name="description" content="X"></head></html>"#;
        assert!(build_meta_description(html, "Override?").is_none());
    }

    #[test]
    fn meta_description_escapes_attribute_value() {
        let html = "<html><head></head></html>";
        let out = build_meta_description(html, r#"X & "Y" <Z>"#).unwrap();
        // No raw `&`, raw `"` between content="...", or raw `<` in attribute.
        assert!(out.contains("content="));
        assert!(!out.contains(r#"content="X & ""#));
    }

    // ── build_og_tags ───────────────────────────────────────────

    #[test]
    fn og_tags_includes_title_description_type_url() {
        let html = "<html lang=\"en\"><head></head></html>";
        let tags = build_og_tags(
            html,
            "Hello",
            "World",
            "https://example.com/page",
            "website",
        );
        let joined = tags.join("\n");
        assert!(joined.contains(r#"property="og:title" content="Hello""#));
        assert!(joined.contains(r#"property="og:description" content="World""#));
        assert!(joined.contains(r#"property="og:type" content="website""#));
        assert!(joined.contains(
            r#"property="og:url" content="https://example.com/page""#
        ));
        assert!(joined.contains(r#"property="og:locale" content="en""#));
    }

    #[test]
    fn og_tags_skips_existing_tags() {
        let html = r#"<html lang="en"><head>
            <meta property="og:title" content="Existing">
            <meta property="og:type" content="article">
        </head></html>"#;
        let tags = build_og_tags(
            html,
            "Hello",
            "World",
            "https://example.com",
            "website",
        );
        let joined = tags.join("\n");
        assert!(
            !joined.contains(r#"property="og:title""#),
            "should not duplicate og:title: {joined}"
        );
        assert!(
            !joined.contains(r#"property="og:type""#),
            "should not duplicate og:type"
        );
    }

    #[test]
    fn og_tags_falls_back_from_twitter_image_when_og_image_missing() {
        let html = r#"<html><head>
            <meta name="twitter:image" content="/twit.png">
        </head></html>"#;
        let tags = build_og_tags(html, "T", "D", "", "website");
        let joined = tags.join("\n");
        assert!(
            joined.contains(r#"property="og:image" content="/twit.png""#),
            "should reuse twitter:image when og:image absent: {joined}"
        );
        // and emit explicit dimensions for fast social card render
        assert!(joined.contains(r#"property="og:image:width" content="1200""#));
        assert!(joined.contains(r#"property="og:image:height" content="630""#));
    }

    #[test]
    fn og_tags_locale_translates_html_lang_dashes_to_underscores() {
        let html = "<html lang=\"en-GB\"><head></head></html>";
        let tags = build_og_tags(html, "T", "D", "", "website");
        let joined = tags.join("\n");
        assert!(
            joined.contains(r#"property="og:locale" content="en_GB""#),
            "lang=\"en-GB\" should produce og:locale=\"en_GB\", got: {joined}"
        );
    }

    #[test]
    fn og_tags_omits_locale_when_html_has_no_lang() {
        let html = "<html><head></head></html>";
        let tags = build_og_tags(html, "T", "D", "", "website");
        let joined = tags.join("\n");
        assert!(
            !joined.contains("og:locale"),
            "no html lang → no og:locale, got: {joined}"
        );
    }

    // ── build_twitter_tags ──────────────────────────────────────

    #[test]
    fn twitter_tags_includes_card_title_description() {
        let html = "<html><head></head></html>";
        let tags = build_twitter_tags(html, "T", "D", "summary");
        let joined = tags.join("\n");
        assert!(joined.contains(r#"name="twitter:card" content="summary""#));
        assert!(joined.contains(r#"name="twitter:title" content="T""#));
        assert!(joined.contains(r#"name="twitter:description" content="D""#));
    }

    #[test]
    fn twitter_tags_falls_back_to_og_image_when_twitter_image_missing() {
        let html = r#"<html><head>
            <meta property="og:image" content="/og.png">
        </head></html>"#;
        let tags = build_twitter_tags(html, "T", "D", "summary");
        let joined = tags.join("\n");
        assert!(
            joined.contains(r#"name="twitter:image" content="/og.png""#),
            "should reuse og:image when twitter:image absent: {joined}"
        );
    }

    // ── inject_seo_tags integration via after_compile ───────────

    #[test]
    fn transform_html_injects_tags() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());

        let html = r#"<!doctype html><html lang="en"><head><title>Hello</title></head>
            <body><p>World is wide.</p></body></html>"#;

        let after = SeoPlugin
            .transform_html(html, Path::new("page.html"), &c)
            .unwrap();
        assert!(after.contains("og:title"));
        assert!(after.contains("twitter:card"));
        assert!(after.contains("name=\"description\""));
    }

    #[test]
    fn transform_html_uses_article_type_when_article_tag_present() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());

        let html = r#"<!doctype html><html lang="en"><head><title>P</title></head>
            <body><article><p>Content.</p></article></body></html>"#;

        let after = SeoPlugin
            .transform_html(html, Path::new("post.html"), &c)
            .unwrap();
        assert!(
            after.contains(r#"og:type" content="article""#),
            "presence of <article> should set og:type=article: {after}"
        );
        assert!(
            after.contains(r#"twitter:card" content="summary_large_image""#),
            "article should use summary_large_image twitter card: {after}"
        );
    }

    #[test]
    fn transform_html_is_idempotent() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());

        let html = r#"<html lang="en"><head><title>Y</title></head><body>Z</body></html>"#;

        let first = SeoPlugin
            .transform_html(html, Path::new("x.html"), &c)
            .unwrap();
        let second = SeoPlugin
            .transform_html(&first, Path::new("x.html"), &c)
            .unwrap();
        assert_eq!(first, second, "second run must not duplicate meta tags");
    }

    #[test]
    fn after_compile_no_op_when_no_html_files() {
        let dir = tempdir().unwrap();
        // Site dir exists but is empty.
        SeoPlugin.after_compile(&ctx(dir.path())).unwrap();
    }

    #[test]
    fn transform_html_handles_html_without_head_tag() {
        let dir = tempdir().unwrap();
        let c = ctx(dir.path());
        let raw = "<!doctype html><html><body>only</body></html>";
        let after = SeoPlugin
            .transform_html(raw, Path::new("frag.html"), &c)
            .unwrap();
        assert_eq!(after, raw);
    }
}
