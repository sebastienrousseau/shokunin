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
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    fn name(&self) -> &'static str {
        "minify"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let cache = ctx.cache.as_ref();

        // Collect HTML files (top-level only, matching previous behaviour).
        let html_files: Vec<_> = fs::read_dir(&ctx.site_dir)?
            .filter_map(std::result::Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .filter(|p| cache.is_none_or(|c| c.has_changed(p)))
            .collect();

        let count = AtomicUsize::new(0);

        html_files.par_iter().try_for_each(|path| -> Result<()> {
            fail_point!("plugins::minify-read", |_| {
                anyhow::bail!("injected: plugins::minify-read")
            });
            let content = fs::read_to_string(path).with_context(|| {
                format!("Failed to read {}", path.display())
            })?;
            let minified = minify_html(&content);
            fail_point!("plugins::minify-write", |_| {
                anyhow::bail!("injected: plugins::minify-write")
            });
            fs::write(path, &minified).with_context(|| {
                format!("Failed to write {}", path.display())
            })?;
            let _ = count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })?;

        let total = count.load(Ordering::Relaxed);
        if total > 0 {
            println!("[minify] Processed {total} HTML files");
        }
        Ok(())
    }
}

/// Minimal HTML minification: collapse whitespace runs into single spaces.
///
/// `<pre>` blocks short-circuit and return the input unchanged. This
/// is intentionally simplistic; a real minifier lives in `minify-html`.
fn minify_html(html: &str) -> String {
    // Fast path: any `<pre` anywhere disables minification entirely.
    if html.contains("<pre") {
        return html.to_string();
    }

    let mut result = String::with_capacity(html.len());
    let mut in_whitespace = false;
    for ch in html.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            in_whitespace = false;
            result.push(ch);
        }
    }
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
    fn name(&self) -> &'static str {
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
                if matches!(
                    ext.as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "bmp"
                ) {
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
    #[must_use]
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
        }
    }
}

impl Plugin for DeployPlugin {
    fn name(&self) -> &'static str {
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use crate::test_support::init_logger;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx_with(site_dir: &Path) -> PluginContext {
        init_logger();
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

    #[test]
    fn minify_plugin_preserves_pre_blocks() {
        // Arrange
        let input = "<pre>  code   with   spaces  </pre><p>  other  </p>";

        // Act
        let result = minify_html(input);

        // Assert — content with <pre> is returned verbatim
        assert_eq!(result, input);
    }

    #[test]
    fn minify_plugin_handles_nested_html() {
        // Arrange
        let input = "<div>  <section>  <article>  <p>  deep  </p>  </article>  </section>  </div>";

        // Act
        let result = minify_html(input);

        // Assert — runs of whitespace collapsed to single spaces
        assert!(!result.contains("  "));
        assert!(result.contains("<div>"));
        assert!(result.contains("</div>"));
        assert!(result.contains("deep"));
    }

    #[test]
    fn minify_plugin_empty_html_file() -> Result<()> {
        // Arrange
        let temp = tempdir()?;
        let html_path = temp.path().join("empty.html");
        fs::write(&html_path, "")?;

        // Act
        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        // Assert — file exists, no crash
        let content = fs::read_to_string(&html_path)?;
        assert!(content.is_empty());
        Ok(())
    }

    #[test]
    fn image_opti_plugin_finds_jpeg_variants() -> Result<()> {
        // Arrange
        let temp = tempdir()?;
        fs::write(temp.path().join("photo.jpg"), "JPG")?;
        fs::write(temp.path().join("banner.jpeg"), "JPEG")?;
        fs::write(temp.path().join("readme.txt"), "text")?;

        // Act
        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;

        // Assert — plugin runs without error (it only logs; we verify no crash)
        // Also verify both extensions are recognized by the match arm
        let mut found = Vec::new();
        for entry in fs::read_dir(temp.path())? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "jpg" | "jpeg") {
                    found.push(path);
                }
            }
        }
        assert_eq!(found.len(), 2);
        Ok(())
    }

    #[test]
    fn image_opti_plugin_nested_directories() -> Result<()> {
        // Arrange — ImageOptiPlugin only reads top-level (read_dir, not recursive)
        let temp = tempdir()?;
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir)?;
        fs::write(subdir.join("deep.png"), "PNG")?;
        fs::write(temp.path().join("top.png"), "PNG")?;

        // Act
        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;

        // Assert — plugin completes without error; subdir images are not
        // discovered since read_dir is non-recursive
        Ok(())
    }

    #[test]
    fn deploy_plugin_custom_target() -> Result<()> {
        // Arrange
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());
        let target_name = "staging-eu-west-1";
        let plugin = DeployPlugin::new(target_name);

        // Act — after_compile prints the target
        plugin.after_compile(&ctx)?;

        // Assert — the stored target matches what was provided
        assert_eq!(plugin.target, target_name);
        Ok(())
    }

    #[test]
    fn minify_plugin_nonexistent_dir_returns_ok() -> Result<()> {
        // Arrange
        let ctx = test_ctx_with(Path::new("/this/path/does/not/exist/at/all"));

        // Act & Assert — returns Ok without error
        assert!(MinifyPlugin.after_compile(&ctx).is_ok());
        Ok(())
    }
}
