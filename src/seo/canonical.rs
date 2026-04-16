// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Canonical URL injection plugin.

use super::helpers::{collect_html_files, escape_attr};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;

/// Injects `<link rel="canonical">` tags into HTML files.
///
/// For each HTML file missing a canonical link, this plugin computes
/// the canonical URL from the base URL and the file's relative path,
/// then injects the tag before `</head>`.
///
/// The plugin is idempotent — it will not add a duplicate canonical
/// link if one already exists.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::CanonicalPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(CanonicalPlugin::new("https://example.com"));
/// ```
#[derive(Debug, Clone)]
pub struct CanonicalPlugin {
    base_url: String,
}

impl CanonicalPlugin {
    /// Creates a new `CanonicalPlugin` with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Plugin for CanonicalPlugin {
    fn name(&self) -> &'static str {
        "canonical"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        let base = self.base_url.trim_end_matches('/');
        let site_dir = &ctx.site_dir;

        html_files.par_iter().try_for_each(|path| -> Result<()> {
            let html = fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;

            let rel_path = path
                .strip_prefix(site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let tag = build_canonical_tag(base, &rel_path);

            let mut result = remove_existing_canonicals(&html);

            // Inject the correct canonical before </head>
            result = if let Some(pos) = result.find("</head>") {
                format!("{}{}\n{}", &result[..pos], tag, &result[pos..])
            } else {
                result
            };

            if result != html {
                fs::write(path, &result).with_context(|| {
                    format!("cannot write {}", path.display())
                })?;
            }
            Ok(())
        })?;

        Ok(())
    }
}

/// Builds a `<link rel="canonical">` tag for the given base URL and path.
fn build_canonical_tag(base: &str, rel_path: &str) -> String {
    let canonical_url = format!("{base}/{rel_path}");
    format!(
        "<link rel=\"canonical\" href=\"{}\">",
        escape_attr(&canonical_url)
    )
}

/// Removes all existing canonical link tags from HTML.
fn remove_existing_canonicals(html: &str) -> String {
    let has_canonical = html.contains("rel=\"canonical\"")
        || html.contains("rel='canonical'")
        || html.contains("rel=canonical");
    if !has_canonical {
        return html.to_string();
    }

    let mut result = html.to_string();
    for pat in &["rel=\"canonical\"", "rel='canonical'", "rel=canonical"] {
        while let Some(pos) = result.find(pat) {
            let start = result[..pos].rfind('<').unwrap_or(pos);
            let end = result[pos..]
                .find('>')
                .map_or(result.len(), |i| pos + i + 1);
            let end = if result.as_bytes().get(end) == Some(&b'\n') {
                end + 1
            } else {
                end
            };
            result.replace_range(start..end, "");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
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
        assert_eq!(CanonicalPlugin::new("https://x").name(), "canonical");
    }

    #[test]
    fn new_accepts_string_or_str() {
        let _ = CanonicalPlugin::new("https://a");
        let _ = CanonicalPlugin::new(String::from("https://b"));
    }

    #[test]
    fn no_op_when_site_dir_missing() {
        let dir = tempdir().unwrap();
        CanonicalPlugin::new("https://x")
            .after_compile(&ctx(&dir.path().join("nope")))
            .unwrap();
    }

    #[test]
    fn build_canonical_tag_joins_base_and_rel_path() {
        let tag = build_canonical_tag("https://example.com", "blog/post.html");
        assert_eq!(
            tag,
            r#"<link rel="canonical" href="https://example.com/blog/post.html">"#
        );
    }

    #[test]
    fn build_canonical_tag_escapes_href_attribute_value() {
        let tag = build_canonical_tag("https://example.com", "x?a=1&b=2");
        // & in href must be escaped to &amp; (what escape_attr does)
        assert!(
            tag.contains("&amp;"),
            "ampersand in URL must be HTML-escaped: {tag}"
        );
    }

    #[test]
    fn remove_existing_canonicals_no_op_when_none_present() {
        let html = "<head><title>x</title></head>";
        assert_eq!(remove_existing_canonicals(html), html);
    }

    #[test]
    fn remove_existing_canonicals_strips_double_quoted() {
        let html = r#"<head><link rel="canonical" href="/old"><title>x</title></head>"#;
        let out = remove_existing_canonicals(html);
        assert!(!out.contains("rel=\"canonical\""));
        assert!(out.contains("<title>x</title>"));
    }

    #[test]
    fn remove_existing_canonicals_strips_single_quoted() {
        let html = "<head><link rel='canonical' href='/old'></head>";
        let out = remove_existing_canonicals(html);
        assert!(!out.contains("rel='canonical'"));
    }

    #[test]
    fn remove_existing_canonicals_strips_unquoted() {
        let html = "<head><link rel=canonical href=/old></head>";
        let out = remove_existing_canonicals(html);
        assert!(!out.contains("rel=canonical"));
    }

    #[test]
    fn remove_existing_canonicals_strips_multiple() {
        let html = r#"<head>
            <link rel="canonical" href="/a">
            <link rel="canonical" href="/b">
        </head>"#;
        let out = remove_existing_canonicals(html);
        assert!(!out.contains("rel=\"canonical\""));
    }

    #[test]
    fn after_compile_injects_canonical_into_html() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("page.html"),
            "<html><head></head><body></body></html>",
        )
        .unwrap();
        CanonicalPlugin::new("https://example.com")
            .after_compile(&ctx(dir.path()))
            .unwrap();
        let after = fs::read_to_string(dir.path().join("page.html")).unwrap();
        assert!(
            after.contains(r#"<link rel="canonical""#),
            "canonical link should be injected: {after}"
        );
    }

    #[test]
    fn after_compile_replaces_existing_canonical_with_correct_one() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("page.html"),
            r#"<html><head><link rel="canonical" href="/wrong"></head></html>"#,
        )
        .unwrap();
        CanonicalPlugin::new("https://example.com")
            .after_compile(&ctx(dir.path()))
            .unwrap();
        let after = fs::read_to_string(dir.path().join("page.html")).unwrap();
        assert!(
            after.contains("https://example.com"),
            "wrong canonical replaced with correct: {after}"
        );
        assert!(
            !after.contains("/wrong"),
            "old canonical should be gone: {after}"
        );
    }

    #[test]
    fn after_compile_trims_trailing_slash_on_base_url() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("page.html"), "<html><head></head></html>")
            .unwrap();
        CanonicalPlugin::new("https://example.com/")
            .after_compile(&ctx(dir.path()))
            .unwrap();
        let after = fs::read_to_string(dir.path().join("page.html")).unwrap();
        assert!(
            !after.contains("com//page.html"),
            "no double-slash after trim: {after}"
        );
    }

    #[test]
    fn after_compile_handles_html_without_head_tag() {
        // Edge: file with no </head> — should not panic, leave as-is.
        let dir = tempdir().unwrap();
        let raw = "<!doctype html><html><body>only</body></html>";
        fs::write(dir.path().join("frag.html"), raw).unwrap();
        CanonicalPlugin::new("https://example.com")
            .after_compile(&ctx(dir.path()))
            .unwrap();
        let after = fs::read_to_string(dir.path().join("frag.html")).unwrap();
        assert_eq!(after, raw);
    }
}
