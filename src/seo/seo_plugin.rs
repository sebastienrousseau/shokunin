// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SEO meta tag injection plugin.

use super::helpers::{
    collect_html_files, escape_attr, extract_canonical, extract_description,
    extract_existing_meta, extract_first_content_image, extract_html_lang,
    extract_title, has_meta_tag,
};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;
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

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        let cache = ctx.cache.as_ref();
        let files: Vec<_> = html_files
            .into_iter()
            .filter(|p| cache.is_none_or(|c| c.has_changed(p)))
            .collect();

        files
            .par_iter()
            .try_for_each(|path| inject_seo_tags(path))?;

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

/// Inject missing SEO meta tags into a single HTML file.
///
/// Fix 5: checks for actual `<meta>` tags rather than comment markers,
/// and injects a comprehensive set of OG/Twitter tags for all page types.
fn inject_seo_tags(path: &Path) -> Result<()> {
    let html = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    let title = extract_title(&html);
    let description = extract_description(&html, 160);
    let canonical = extract_canonical(&html);

    let is_article = html.contains("<article");
    let og_type = if is_article { "article" } else { "website" };
    let twitter_card = if is_article {
        "summary_large_image"
    } else {
        "summary"
    };

    let mut tags = Vec::new();

    if let Some(meta_desc) = build_meta_description(&html, &description) {
        tags.push(meta_desc);
    }
    tags.extend(build_og_tags(
        &html,
        &title,
        &description,
        &canonical,
        og_type,
    ));
    tags.extend(build_twitter_tags(
        &html,
        &title,
        &description,
        twitter_card,
    ));

    if tags.is_empty() {
        return Ok(());
    }

    let injection = tags.join("\n");
    let result = if let Some(pos) = html.find("</head>") {
        format!("{}{}\n{}", &html[..pos], injection, &html[pos..])
    } else {
        html
    };

    fs::write(path, result)
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}
