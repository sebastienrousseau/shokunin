// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Built-in plugins
//!
//! Ready-to-use plugins for common static site generation tasks.
//!
//! - `MinifyPlugin` — Minifies HTML files in the site output directory.
//! - `ImageOptiPlugin` — Logs image files for optimization (stub for external tooling).
//! - `DeployPlugin` — Logs deployment target after build (stub for CI integration).

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Minifies HTML files by removing unnecessary whitespace.
///
/// Runs during the `after_compile` hook. Processes all `.html` files
/// in the site directory.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::plugins::MinifyPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(MinifyPlugin);
/// ```
#[derive(Debug, Copy, Clone)]
pub struct MinifyPlugin;

impl Plugin for MinifyPlugin {
    fn name(&self) -> &str {
        "minify"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }
        let mut count = 0usize;
        for entry in fs::read_dir(&ctx.site_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |e| e == "html") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                let minified = minify_html(&content);
                fs::write(&path, &minified)
                    .with_context(|| format!("Failed to write {}", path.display()))?;
                count += 1;
            }
        }
        if count > 0 {
            println!("[minify] Processed {} HTML files", count);
        }
        Ok(())
    }
}

/// Minimal HTML minification: collapse whitespace runs into single spaces.
fn minify_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_whitespace = false;
    let in_pre = false;

    for ch in html.chars() {
        if html.contains("<pre") {
            // Simple pre-tag detection — skip minification if any <pre> exists
            return html.to_string();
        }
        if ch.is_whitespace() {
            if !in_whitespace && !in_pre {
                result.push(' ');
                in_whitespace = true;
            } else if in_pre {
                result.push(ch);
            }
        } else {
            in_whitespace = false;
            result.push(ch);
        }
    }
    let _ = in_pre; // suppress unused warning
    result
}

/// Image optimization plugin stub.
///
/// Scans the site directory for image files and logs them.
/// Actual optimization requires external tools (e.g., `cwebp`, `avifenc`).
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::plugins::ImageOptiPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(ImageOptiPlugin);
/// ```
#[derive(Debug, Copy, Clone)]
pub struct ImageOptiPlugin;

impl Plugin for ImageOptiPlugin {
    fn name(&self) -> &str {
        "image-opti"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }
        let mut images = Vec::new();
        for entry in fs::read_dir(&ctx.site_dir)? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp") {
                    images.push(path);
                }
            }
        }
        if !images.is_empty() {
            println!(
                "[image-opti] Found {} images for optimization",
                images.len()
            );
        }
        Ok(())
    }
}

/// Deployment plugin stub.
///
/// Logs the deployment target after a successful build.
/// Extend with actual deployment logic for Vercel, Netlify, or Cloudflare.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::plugins::DeployPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(DeployPlugin::new("production"));
/// ```
#[derive(Debug)]
pub struct DeployPlugin {
    target: String,
}

impl DeployPlugin {
    /// Creates a new deployment plugin for the given target environment.
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
        }
    }
}

impl Plugin for DeployPlugin {
    fn name(&self) -> &str {
        "deploy"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        println!(
            "[deploy] Site at {} ready for deployment to '{}'",
            ctx.site_dir.display(),
            self.target
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx_with(site_dir: &Path) -> PluginContext {
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
        )
    }

    #[test]
    fn test_minify_plugin_name() {
        assert_eq!(MinifyPlugin.name(), "minify");
    }

    #[test]
    fn test_minify_plugin_empty_dir() -> Result<()> {
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_minify_plugin_processes_html() -> Result<()> {
        let temp = tempdir()?;
        let html_path = temp.path().join("index.html");
        fs::write(&html_path, "<h1>  Hello   World  </h1>")?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(&html_path)?;
        assert!(!content.contains("  "));
        Ok(())
    }

    #[test]
    fn test_minify_plugin_skips_non_html() -> Result<()> {
        let temp = tempdir()?;
        let css_path = temp.path().join("style.css");
        fs::write(&css_path, "body {   color: red;   }")?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        // CSS should be unchanged
        let content = fs::read_to_string(&css_path)?;
        assert!(content.contains("   "));
        Ok(())
    }

    #[test]
    fn test_minify_plugin_nonexistent_dir() -> Result<()> {
        let ctx = test_ctx_with(Path::new("/nonexistent"));
        MinifyPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_minify_html_collapses_whitespace() {
        let result = minify_html("<p>  Hello   World  </p>");
        assert_eq!(result, "<p> Hello World </p>");
    }

    #[test]
    fn test_minify_html_preserves_pre() {
        let input = "<pre>  keep   spaces  </pre>";
        let result = minify_html(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_image_opti_plugin_name() {
        assert_eq!(ImageOptiPlugin.name(), "image-opti");
    }

    #[test]
    fn test_image_opti_plugin_finds_images() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("photo.png"), "PNG")?;
        fs::write(temp.path().join("logo.jpg"), "JPG")?;
        fs::write(temp.path().join("style.css"), "CSS")?;

        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_image_opti_plugin_nonexistent_dir() -> Result<()> {
        let ctx = test_ctx_with(Path::new("/nonexistent"));
        ImageOptiPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_deploy_plugin_name() {
        let p = DeployPlugin::new("staging");
        assert_eq!(p.name(), "deploy");
    }

    #[test]
    fn test_deploy_plugin_prints_target() -> Result<()> {
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());
        let p = DeployPlugin::new("production");
        p.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn test_all_plugins_register() {
        use crate::plugin::PluginManager;
        let mut pm = PluginManager::new();
        pm.register(MinifyPlugin);
        pm.register(ImageOptiPlugin);
        pm.register(DeployPlugin::new("test"));
        assert_eq!(pm.len(), 3);
        assert_eq!(pm.names(), vec!["minify", "image-opti", "deploy"]);
    }
}
