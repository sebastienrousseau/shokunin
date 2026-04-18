// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! robots.txt generation plugin.

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Generates a `robots.txt` file in the site directory.
///
/// The file allows all user agents and references the sitemap at
/// `{base_url}/sitemap.xml`. If a `robots.txt` already exists, it is
/// not overwritten.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::RobotsPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(RobotsPlugin::new("https://example.com"));
/// ```
#[derive(Debug, Clone)]
pub struct RobotsPlugin {
    base_url: String,
}

impl RobotsPlugin {
    /// Creates a new `RobotsPlugin` with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Plugin for RobotsPlugin {
    fn name(&self) -> &'static str {
        "robots"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let robots_path = ctx.site_dir.join("robots.txt");
        if robots_path.exists() {
            return Ok(());
        }

        let content = format!(
            "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
            self.base_url.trim_end_matches('/')
        );

        fs::write(&robots_path, content).with_context(|| {
            format!("cannot write {}", robots_path.display())
        })?;

        Ok(())
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

    #[test]
    fn name_is_stable() {
        // Plugin name is part of the public contract — log lines and
        // PluginManager APIs key off it. Pin the value.
        assert_eq!(RobotsPlugin::new("https://x.example").name(), "robots");
    }

    #[test]
    fn new_accepts_string_or_str() {
        // Both `&str` and `String` should work via `impl Into<String>`.
        let _from_str = RobotsPlugin::new("https://a.example");
        let _from_string = RobotsPlugin::new(String::from("https://b.example"));
    }

    #[test]
    fn writes_robots_txt_when_missing() {
        let dir = tempdir().unwrap();
        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx(dir.path())).unwrap();

        let body = fs::read_to_string(dir.path().join("robots.txt")).unwrap();
        assert_eq!(
            body,
            "User-agent: *\nAllow: /\nSitemap: https://example.com/sitemap.xml\n"
        );
    }

    #[test]
    fn trims_trailing_slash_from_base_url() {
        let dir = tempdir().unwrap();
        let plugin = RobotsPlugin::new("https://example.com/");
        plugin.after_compile(&ctx(dir.path())).unwrap();

        let body = fs::read_to_string(dir.path().join("robots.txt")).unwrap();
        assert!(
            body.contains("Sitemap: https://example.com/sitemap.xml\n"),
            "trailing slash on base_url should be trimmed before joining \
             /sitemap.xml, got: {body}"
        );
        assert!(
            !body.contains("//sitemap.xml"),
            "should not produce double-slash"
        );
    }

    #[test]
    fn does_not_overwrite_existing_robots_txt() {
        let dir = tempdir().unwrap();
        let custom = "User-agent: GPTBot\nDisallow: /\n";
        fs::write(dir.path().join("robots.txt"), custom).unwrap();

        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx(dir.path())).unwrap();

        let body = fs::read_to_string(dir.path().join("robots.txt")).unwrap();
        assert_eq!(body, custom, "existing robots.txt must be left untouched");
    }

    #[test]
    fn no_op_when_site_dir_missing() {
        // Site dir doesn't exist — plugin must succeed silently.
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("nope");
        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx(&nonexistent)).unwrap();
        assert!(
            !nonexistent.join("robots.txt").exists(),
            "plugin should not create files in a missing site dir"
        );
    }
}
