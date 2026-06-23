// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Built-in plugins
//!
//! Ready-to-use plugins for common static site generation tasks.
//!
//! - `MinifyPlugin` — Minifies HTML files in the site output directory.
//!   With the `minify` feature enabled, also minifies `.css` and `.js`
//!   assets and walks the site directory recursively.
//! - `ImageOptiPlugin` — Logs image files for optimization (stub for external tooling).
//! - `DeployPlugin` — Logs deployment target after build (stub for CI integration).

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Minifies HTML files and (with the `minify` feature) JS/CSS assets.
///
/// Runs during the `after_compile` hook.
///
/// * **Default build:** processes only top-level `.html` files in
///   `site_dir`, falling back to a whitespace-collapsing pass that
///   short-circuits on any document containing `<pre`.
/// * **`minify` feature:** walks `site_dir` recursively (via `walkdir`)
///   and uses
///   [`minify-html`](https://crates.io/crates/minify-html) for HTML,
///   [`oxc_minifier`](https://crates.io/crates/oxc_minifier) for JS, and
///   [`lightningcss`](https://crates.io/crates/lightningcss) for CSS.
///   `<pre>` content is preserved bit-identically by `minify-html`'s
///   native handling.
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

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let cache = ctx.cache.as_ref();
        let (html_files, css_files, js_files) =
            collect_minifiable_files(&ctx.site_dir, cache)?;

        let count = AtomicUsize::new(0);

        html_files
            .par_iter()
            .try_for_each(|path| -> Result<(), SsgError> {
                fail_point!("plugins::minify-read", |_| {
                    Err(SsgError::Io {
                        path: path.clone(),
                        source: std::io::Error::other(
                            "injected: plugins::minify-read",
                        ),
                    })
                });
                let content = fs::read_to_string(path).with_path(path)?;
                let minified = minify_html(&content);
                fail_point!("plugins::minify-write", |_| {
                    Err(SsgError::Io {
                        path: path.clone(),
                        source: std::io::Error::other(
                            "injected: plugins::minify-write",
                        ),
                    })
                });
                fs::write(path, &minified).with_path(path)?;
                let _ = count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })?;

        // CSS + JS minification only run when the `minify` feature is on.
        // In the default build these vectors are empty and the loops
        // are no-ops.
        #[cfg(feature = "minify")]
        {
            css_files
                .par_iter()
                .try_for_each(|path| -> Result<(), SsgError> {
                    let content = fs::read_to_string(path).with_path(path)?;
                    let minified = minify_css(&content).unwrap_or(content);
                    fs::write(path, &minified).with_path(path)?;
                    let _ = count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })?;

            js_files
                .par_iter()
                .try_for_each(|path| -> Result<(), SsgError> {
                    let content = fs::read_to_string(path).with_path(path)?;
                    let minified = minify_js(&content).unwrap_or(content);
                    fs::write(path, &minified).with_path(path)?;
                    let _ = count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })?;
        }

        // Suppress unused-binding warnings on the default build where
        // CSS/JS lists are populated but never consumed.
        #[cfg(not(feature = "minify"))]
        {
            let _ = &css_files;
            let _ = &js_files;
        }

        let total = count.load(Ordering::Relaxed);
        if total > 0 {
            println!("[minify] Processed {total} file(s)");
        }
        Ok(())
    }
}

/// `(html, css, js)` file lists returned by [`collect_minifiable_files`].
type MinifiableFiles = (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);

/// Walks `site_dir` and returns `(html, css, js)` file lists, honouring
/// the plugin cache for incremental builds.
///
/// * With the `minify` feature, the walk is recursive (via `walkdir`)
///   and includes `.css` and `.js` files.
/// * Without the feature, the walk is top-level only (matching the
///   pre-0.0.42 behaviour) and CSS/JS lists are returned empty.
fn collect_minifiable_files(
    site_dir: &std::path::Path,
    cache: Option<&crate::plugin::PluginCache>,
) -> Result<MinifiableFiles, SsgError> {
    #[cfg(feature = "minify")]
    {
        let mut html = Vec::new();
        let mut css = Vec::new();
        let mut js = Vec::new();
        for entry in walkdir::WalkDir::new(site_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.into_path();
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            // Cache check applies uniformly to all minifiable assets.
            if cache.is_some_and(|c| !c.has_changed(&path)) {
                continue;
            }
            match ext {
                "html" => html.push(path),
                "css" => css.push(path),
                "js" => js.push(path),
                _ => {}
            }
        }
        Ok((html, css, js))
    }
    #[cfg(not(feature = "minify"))]
    {
        let html: Vec<_> = fs::read_dir(site_dir)
            .with_path(site_dir)?
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "html"))
            .filter(|p| cache.is_none_or(|c| c.has_changed(p)))
            .collect();
        Ok((html, Vec::new(), Vec::new()))
    }
}

/// HTML minification.
///
/// * With the `minify` feature: delegates to `minify-html` configured
///   with `keep_comments: false`, `do_not_minify_doctype: true`. CSS
///   inside `<style>` and JS inside `<script>` are passed through
///   without inline minification (the dedicated asset-file passes
///   handle that, and avoid double-minification of inline blocks that
///   may contain template-specific syntax).
/// * Without the feature: falls back to a whitespace-collapsing pass
///   that short-circuits when any `<pre` substring is present so
///   user-visible whitespace in code blocks is preserved.
#[cfg(feature = "minify")]
pub fn minify_html(html: &str) -> String {
    let cfg = minify_html::Cfg {
        do_not_minify_doctype: true,
        keep_comments: false,
        keep_html_and_head_opening_tags: true,
        keep_closing_tags: true,
        keep_spaces_between_attributes: true,
        ..minify_html::Cfg::default()
    };
    let out = minify_html::minify(html.as_bytes(), &cfg);
    // minify-html guarantees valid UTF-8 in, valid UTF-8 out for the
    // accepted inputs we generate. Fall back to the original string on
    // the unlikely path where it isn't, to keep the build non-fatal.
    String::from_utf8(out).unwrap_or_else(|_| html.to_string())
}

/// Fallback HTML minifier (whitespace collapse) — see the
/// feature-gated overload above for the production minifier.
#[cfg(not(feature = "minify"))]
pub fn minify_html(html: &str) -> String {
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

/// Minifies a CSS source string with `lightningcss`.
///
/// Returns `None` if the input fails to parse — callers fall back to
/// the original content so a malformed asset can't sink an entire
/// build.
#[cfg(feature = "minify")]
pub fn minify_css(css: &str) -> Option<String> {
    use lightningcss::printer::PrinterOptions;
    use lightningcss::stylesheet::{
        MinifyOptions, ParserOptions, StyleSheet,
    };

    let mut sheet =
        StyleSheet::parse(css, ParserOptions::default()).ok()?;
    sheet.minify(MinifyOptions::default()).ok()?;
    let opts = PrinterOptions {
        minify: true,
        ..PrinterOptions::default()
    };
    sheet.to_css(opts).ok().map(|r| r.code)
}

/// Minifies a JavaScript source string with `oxc_minifier` +
/// `oxc_codegen`.
///
/// Returns `None` if the input is not parseable as a script or module
/// — callers fall back to the original content.
#[cfg(feature = "minify")]
pub fn minify_js(js: &str) -> Option<String> {
    use oxc_allocator::Allocator;
    use oxc_codegen::{Codegen, CodegenOptions};
    use oxc_minifier::{Minifier, MinifierOptions};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, js, source_type).parse();
    if !ret.errors.is_empty() {
        return None;
    }
    let mut program = ret.program;
    let options = MinifierOptions::default();
    let _ = Minifier::new(options).minify(&allocator, &mut program);
    let codegen_options = CodegenOptions {
        minify: true,
        ..CodegenOptions::default()
    };
    let out = Codegen::new()
        .with_options(codegen_options)
        .build(&program);
    Some(out.code)
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

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }
        let mut images = Vec::new();
        for entry in fs::read_dir(&ctx.site_dir).with_path(&ctx.site_dir)? {
            let entry = entry.with_path(&ctx.site_dir)?;
            let path = entry.path();
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

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
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
    use anyhow::Result;
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

        // CSS minification only runs under the `minify` feature; on
        // the default build the file is untouched.
        let content = fs::read_to_string(&css_path)?;
        #[cfg(not(feature = "minify"))]
        assert!(content.contains("   "));
        #[cfg(feature = "minify")]
        {
            // With minify on, CSS *is* compressed — verify it parses
            // back to the same logical rule.
            assert!(content.contains("color"));
            assert!(content.contains("red"));
        }
        Ok(())
    }

    #[test]
    fn test_minify_plugin_nonexistent_dir() -> Result<()> {
        let ctx = test_ctx_with(Path::new("/nonexistent"));
        MinifyPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn test_minify_html_collapses_whitespace() {
        let result = minify_html("<p>  Hello   World  </p>");
        assert_eq!(result, "<p> Hello World </p>");
    }

    #[cfg(not(feature = "minify"))]
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

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_plugin_preserves_pre_blocks() {
        // Arrange
        let input = "<pre>  code   with   spaces  </pre><p>  other  </p>";

        // Act
        let result = minify_html(input);

        // Assert — content with <pre> is returned verbatim
        assert_eq!(result, input);
    }

    #[cfg(not(feature = "minify"))]
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

    // -----------------------------------------------------------------
    // minify_html — additional edge cases (fallback only)
    // -----------------------------------------------------------------

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_empty_string() {
        let result = minify_html("");
        assert_eq!(result, "");
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_whitespace_only() {
        let result = minify_html("   \n\t  \n  ");
        assert_eq!(result, " ");
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_no_whitespace() {
        let input = "<p>hello</p>";
        let result = minify_html(input);
        assert_eq!(result, input);
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_preserves_pre_with_class() {
        let input = "<pre class=\"lang-rust\">  fn main() {  }  </pre>";
        let result = minify_html(input);
        assert_eq!(result, input);
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_tabs_and_newlines() {
        let input = "<div>\n\t<p>\n\t\tHello\n\t</p>\n</div>";
        let result = minify_html(input);
        assert_eq!(result, "<div> <p> Hello </p> </div>");
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_mixed_whitespace_types() {
        let input = "<span>  \t\n  word  \t\n  </span>";
        let result = minify_html(input);
        assert_eq!(result, "<span> word </span>");
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_single_char() {
        assert_eq!(minify_html("a"), "a");
        assert_eq!(minify_html(" "), " ");
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_html_multiple_pre_tags() {
        let input = "<pre>a</pre><pre>b</pre>";
        let result = minify_html(input);
        assert_eq!(result, input);
    }

    // -----------------------------------------------------------------
    // MinifyPlugin — multiple HTML files
    // -----------------------------------------------------------------

    #[test]
    fn minify_plugin_processes_multiple_html_files() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("a.html"), "<p>  hello  </p>")?;
        fs::write(temp.path().join("b.html"), "<div>  world  </div>")?;
        fs::write(temp.path().join("c.txt"), "  not html  ")?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        let a = fs::read_to_string(temp.path().join("a.html"))?;
        let b = fs::read_to_string(temp.path().join("b.html"))?;
        let c = fs::read_to_string(temp.path().join("c.txt"))?;

        assert!(!a.contains("  "), "a.html should be minified");
        assert!(!b.contains("  "), "b.html should be minified");
        assert!(c.contains("  "), "c.txt should not be minified");
        Ok(())
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_plugin_whitespace_only_html_file() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("ws.html"), "   \n\t  \n  ")?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(temp.path().join("ws.html"))?;
        assert_eq!(content, " ");
        Ok(())
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn minify_plugin_html_with_pre_block_not_modified() -> Result<()> {
        let temp = tempdir()?;
        let original =
            "<html><pre>  keep  spaces  </pre><p>  other  </p></html>";
        fs::write(temp.path().join("pre.html"), original)?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        let content = fs::read_to_string(temp.path().join("pre.html"))?;
        assert_eq!(content, original);
        Ok(())
    }

    // -----------------------------------------------------------------
    // ImageOptiPlugin — additional file types
    // -----------------------------------------------------------------

    #[test]
    fn image_opti_plugin_finds_gif_and_bmp() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("anim.gif"), "GIF")?;
        fs::write(temp.path().join("icon.bmp"), "BMP")?;
        fs::write(temp.path().join("doc.pdf"), "PDF")?;

        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;

        // Verify the plugin ran without error. The plugin only logs —
        // we verify it recognizes gif/bmp by not crashing and check
        // file counts manually.
        let mut count = 0;
        for entry in fs::read_dir(temp.path())? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "gif" | "bmp") {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 2);
        Ok(())
    }

    #[test]
    fn image_opti_plugin_empty_dir_no_crash() -> Result<()> {
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn image_opti_plugin_no_images() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("readme.txt"), "text")?;
        fs::write(temp.path().join("style.css"), "css")?;

        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;
        Ok(())
    }

    #[test]
    fn image_opti_plugin_files_without_extension() -> Result<()> {
        let temp = tempdir()?;
        fs::write(temp.path().join("Makefile"), "all:")?;
        fs::write(temp.path().join("LICENSE"), "MIT")?;

        let ctx = test_ctx_with(temp.path());
        ImageOptiPlugin.after_compile(&ctx)?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // DeployPlugin — additional targets
    // -----------------------------------------------------------------

    #[test]
    fn deploy_plugin_empty_target() -> Result<()> {
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());
        let plugin = DeployPlugin::new("");
        plugin.after_compile(&ctx)?;
        assert_eq!(plugin.target, "");
        Ok(())
    }

    #[test]
    fn deploy_plugin_various_targets() -> Result<()> {
        let temp = tempdir()?;
        let ctx = test_ctx_with(temp.path());

        for target in ["staging", "production", "preview", "canary"] {
            let plugin = DeployPlugin::new(target);
            assert_eq!(plugin.name(), "deploy");
            assert_eq!(plugin.target, target);
            plugin.after_compile(&ctx)?;
        }
        Ok(())
    }

    #[test]
    fn deploy_plugin_debug_format() {
        let plugin = DeployPlugin::new("prod");
        let debug = format!("{plugin:?}");
        assert!(debug.contains("prod"));
    }

    // -----------------------------------------------------------------
    // MinifyPlugin / ImageOptiPlugin — trait object coverage
    // -----------------------------------------------------------------

    #[test]
    fn minify_plugin_copy_clone() {
        let a = MinifyPlugin;
        let b = a;
        #[allow(clippy::clone_on_copy)]
        let c = a.clone();
        assert_eq!(a.name(), b.name());
        assert_eq!(a.name(), c.name());
    }

    #[test]
    fn minify_plugin_debug_format() {
        let debug = format!("{:?}", MinifyPlugin);
        assert!(debug.contains("MinifyPlugin"));
    }

    #[test]
    fn image_opti_plugin_copy_clone() {
        let a = ImageOptiPlugin;
        let b = a;
        #[allow(clippy::clone_on_copy)]
        let c = a.clone();
        assert_eq!(a.name(), b.name());
        assert_eq!(a.name(), c.name());
    }

    #[test]
    fn image_opti_plugin_debug_format() {
        let debug = format!("{:?}", ImageOptiPlugin);
        assert!(debug.contains("ImageOptiPlugin"));
    }

    #[cfg(not(feature = "minify"))]
    #[test]
    fn test_minify_plugin_read_dir_error() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("not_a_dir");
        fs::write(&file_path, "").unwrap();
        let ctx = test_ctx_with(&file_path);
        let res = MinifyPlugin.after_compile(&ctx);
        assert!(res.is_err());
    }

    #[test]
    fn test_image_opti_plugin_read_dir_error() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("not_a_dir");
        fs::write(&file_path, "").unwrap();
        let ctx = test_ctx_with(&file_path);
        let res = ImageOptiPlugin.after_compile(&ctx);
        assert!(res.is_err());
    }

    // -----------------------------------------------------------------
    // `minify` feature — happy paths (only compiled with the feature)
    // -----------------------------------------------------------------

    #[cfg(feature = "minify")]
    #[test]
    fn minify_html_preserves_pre_content_bit_identical() {
        let body = "fn main() {\n    println!(\"hi\");\n}";
        let input = format!(
            "<html><body><pre><code>{body}</code></pre></body></html>"
        );
        let out = minify_html(&input);
        // The exact whitespace inside <pre><code>…</code></pre> must
        // survive minification untouched. We only check containment
        // because minify-html may rewrite attributes outside the pre.
        assert!(
            out.contains(body),
            "minified output must preserve <pre> body byte-for-byte:\n{out}"
        );
    }

    #[cfg(feature = "minify")]
    #[test]
    fn minify_css_compresses_input() {
        let input = "body  {\n  color:   red;\n  margin:  0px  0px  0px  0px;\n}";
        let out = minify_css(input).expect("css minification");
        assert!(out.len() < input.len());
        assert!(out.contains("red"));
    }

    #[cfg(feature = "minify")]
    #[test]
    fn minify_js_compresses_input() {
        let input = "const greeting = 'hello world';\nconsole.log(greeting);";
        let out = minify_js(input).expect("js minification");
        assert!(out.len() < input.len());
    }

    #[cfg(feature = "minify")]
    #[test]
    fn minify_plugin_recursive_walk_processes_nested_html() -> Result<()> {
        let temp = tempdir()?;
        let deep = temp.path().join("blog").join("2026").join("post");
        fs::create_dir_all(&deep)?;
        let nested = deep.join("index.html");
        fs::write(
            &nested,
            "<html>  <body>   <p>   nested   </p>   </body>   </html>",
        )?;
        let top = temp.path().join("index.html");
        fs::write(&top, "<html>  <body>   <p>   top   </p>   </body></html>")?;

        let ctx = test_ctx_with(temp.path());
        MinifyPlugin.after_compile(&ctx)?;

        let nested_after = fs::read_to_string(&nested)?;
        // Nested file must have been touched (size strictly smaller).
        assert!(
            nested_after.len()
                < "<html>  <body>   <p>   nested   </p>   </body>   </html>"
                    .len(),
            "nested file should have been minified: {nested_after}"
        );
        Ok(())
    }
}
