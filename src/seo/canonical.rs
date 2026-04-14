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
