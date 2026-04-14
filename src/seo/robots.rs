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
