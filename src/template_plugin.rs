// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Template rendering plugin.
//!
//! Post-processes compiled HTML through templates, enabling
//! template inheritance, conditionals, loops, and filters.

#[cfg(feature = "templates")]
use crate::{
    frontmatter,
    plugin::{Plugin, PluginContext},
    template_engine::{TemplateConfig, TemplateEngine},
    MAX_DIR_DEPTH,
};
#[cfg(feature = "templates")]
use anyhow::Result;
#[cfg(feature = "templates")]
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Plugin that post-processes compiled HTML through templates.
///
/// Runs in the `after_compile` phase. For each HTML file in `site_dir`:
/// 1. Reads the companion `.meta.json` sidecar (from frontmatter extraction)
/// 2. Determines the layout from frontmatter (`layout` field, default: `page`)
/// 3. Renders the HTML through the template chain
/// 4. Writes the rendered result back to the same file
///
/// Falls back gracefully if no templates directory exists.
#[cfg(feature = "templates")]
#[derive(Debug)]
pub struct TemplatePlugin {
    config: TemplateConfig,
}

#[cfg(feature = "templates")]
impl TemplatePlugin {
    /// Creates a new `TemplatePlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: TemplateConfig) -> Self {
        Self { config }
    }

    /// Creates a `TemplatePlugin` that looks for templates in the standard
    /// `templates/tera/` subdirectory of the template dir.
    #[must_use]
    pub fn from_template_dir(template_dir: &Path) -> Self {
        Self {
            config: TemplateConfig {
                template_dir: template_dir.join("tera"),
                ..Default::default()
            },
        }
    }
}

#[cfg(feature = "templates")]
impl Plugin for TemplatePlugin {
    fn name(&self) -> &'static str {
        "templates"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        // Emit .meta.json sidecars for all markdown content
        let sidecar_dir = ctx.build_dir.join(".meta");
        let count = frontmatter::emit_sidecars(&ctx.content_dir, &sidecar_dir)?;
        if count > 0 {
            log::info!("[templates] Emitted {count} frontmatter sidecar(s)");
        }
        Ok(())
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let Some(engine) = TemplateEngine::init(self.config.clone())? else {
            log::info!(
                "[templates] No templates at {}, skipping",
                self.config.template_dir.display()
            );
            return Ok(());
        };

        // Build site-level globals from config
        let mut site_globals = ctx
            .config
            .as_ref()
            .map(TemplateEngine::site_globals_from_config)
            .unwrap_or_default();

        // Load data files (data/*.toml, data/*.json) into context
        let data_files = TemplateEngine::load_data_files(&ctx.content_dir);
        if !data_files.is_empty() {
            let _ = site_globals.insert(
                "data".to_string(),
                serde_json::Value::Object(data_files.into_iter().collect()),
            );
        }

        let sidecar_dir = ctx.build_dir.join(".meta");
        let html_files = collect_html_files(&ctx.site_dir)?;

        let mut rendered = 0usize;
        for html_path in &html_files {
            let content = fs::read_to_string(html_path)?;

            // Read frontmatter sidecar
            let fm = read_frontmatter_for_html(
                html_path,
                &ctx.site_dir,
                &sidecar_dir,
            );

            // Determine template from `layout` field
            let layout =
                fm.get("layout").and_then(|v| v.as_str()).unwrap_or("page");
            let template_name = format!("{layout}.html");

            match engine.render_page(
                &template_name,
                &content,
                &fm,
                &site_globals,
            ) {
                Ok(output) => {
                    fs::write(html_path, output)?;
                    rendered += 1;
                }
                Err(e) => {
                    log::warn!(
                        "[templates] Failed to render {}: {e}",
                        html_path.display()
                    );
                }
            }
        }

        if rendered > 0 {
            log::info!("[templates] Rendered {rendered} page(s)");
        }
        Ok(())
    }
}

/// Reads frontmatter for an HTML file, trying sidecar then falling back to empty.
#[cfg(feature = "templates")]
fn read_frontmatter_for_html(
    html_path: &Path,
    site_dir: &Path,
    sidecar_dir: &Path,
) -> HashMap<String, serde_json::Value> {
    let rel = html_path.strip_prefix(site_dir).unwrap_or(html_path);
    let sidecar = sidecar_dir.join(rel).with_extension("meta.json");
    if sidecar.exists() {
        if let Ok(content) = fs::read_to_string(&sidecar) {
            if let Ok(meta) = serde_json::from_str(&content) {
                return meta;
            }
        }
    }
    HashMap::new()
}

/// Recursively collects `.html` files (delegates to `crate::walk`).
#[cfg(feature = "templates")]
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files_bounded_depth(dir, "html", MAX_DIR_DEPTH)
}

#[cfg(all(test, feature = "templates"))]
mod tests {
    use super::*;
    use crate::cmd::SsgConfig;
    use crate::test_support::init_logger;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    fn layout() -> (TempDir, PathBuf, PathBuf, PathBuf, PathBuf) {
        init_logger();
        let dir = tempdir().expect("tempdir");
        let content = dir.path().join("content");
        let build = dir.path().join("build");
        let site = dir.path().join("site");
        let templates = dir.path().join("templates/tera");
        for d in [&content, &build, &site, &templates] {
            fs::create_dir_all(d).expect("mkdir");
        }
        (dir, content, build, site, templates)
    }

    fn make_config(root: &Path) -> SsgConfig {
        SsgConfig {
            site_name: "Test".to_string(),
            site_title: "Test Site".to_string(),
            site_description: "Desc".to_string(),
            base_url: "http://localhost".to_string(),
            language: "en-GB".to_string(),
            content_dir: root.join("content"),
            output_dir: root.join("build"),
            template_dir: root.join("templates"),
            serve_dir: None,
            i18n: None,
        }
    }

    fn setup_project(dir: &Path) {
        let content = dir.join("content");
        let build = dir.join("build");
        let site = dir.join("site");
        let templates = dir.join("templates/tera");
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(&build).unwrap();
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&templates).unwrap();

        fs::write(
            templates.join("base.html"),
            r#"<!DOCTYPE html>
<html><head><title>{{ page.title | default("") }}</title></head>
<body>{% block content %}{% endblock %}</body></html>"#,
        )
        .unwrap();

        fs::write(
            templates.join("page.html"),
            r#"{% extends "base.html" %}
{% block content %}{{ page.content | safe }}{% endblock %}"#,
        )
        .unwrap();

        fs::write(
            content.join("index.md"),
            "---\ntitle: Home\nlayout: page\n---\n# Welcome\n",
        )
        .unwrap();

        fs::write(site.join("index.html"), "<h1>Welcome</h1>").unwrap();

        let meta_dir = build.join(".meta");
        fs::create_dir_all(&meta_dir).unwrap();
        fs::write(
            meta_dir.join("index.meta.json"),
            r#"{"title": "Home", "layout": "page"}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_template_plugin_renders() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("templates/tera"),
            ..Default::default()
        });

        let config = SsgConfig {
            site_name: "Test".to_string(),
            site_title: "Test Site".to_string(),
            site_description: "Desc".to_string(),
            base_url: "http://localhost".to_string(),
            language: "en-GB".to_string(),
            content_dir: dir.path().join("content"),
            output_dir: dir.path().join("build"),
            template_dir: dir.path().join("templates"),
            serve_dir: None,
            i18n: None,
        };

        let content_dir = config.content_dir.clone();
        let output_dir = config.output_dir.clone();
        let template_dir = config.template_dir.clone();
        let site = dir.path().join("site");
        let ctx = PluginContext::with_config(
            &content_dir,
            &output_dir,
            &site,
            &template_dir,
            config,
        );

        plugin.after_compile(&ctx).unwrap();

        let output =
            fs::read_to_string(dir.path().join("site/index.html")).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("Home"));
        assert!(output.contains("<h1>Welcome</h1>"));
    }

    #[test]
    fn test_template_plugin_skips_missing_templates() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("index.html"), "<p>hello</p>").unwrap();

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("nonexistent"),
            ..Default::default()
        });

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());

        plugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert_eq!(output, "<p>hello</p>");
    }

    #[test]
    fn name_returns_templates_identifier() {
        let plugin = TemplatePlugin::new(TemplateConfig::default());
        assert_eq!(plugin.name(), "templates");
    }

    #[test]
    fn new_stores_supplied_config() {
        let cfg = TemplateConfig {
            template_dir: std::env::temp_dir().join("ssg_template_fake"),
            ..Default::default()
        };
        let plugin = TemplatePlugin::new(cfg.clone());
        assert_eq!(plugin.config.template_dir, cfg.template_dir);
    }

    #[test]
    fn from_template_dir_nests_under_tera_subdirectory() {
        let plugin =
            TemplatePlugin::from_template_dir(Path::new("/my/templates"));
        assert!(plugin.config.template_dir.ends_with("templates/tera"));
    }

    #[test]
    fn before_compile_emits_sidecars_from_content_markdown() {
        let (_tmp, content, build, _site, templates) = layout();
        fs::write(content.join("index.md"), "---\ntitle: Test\n---\nbody")
            .unwrap();

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: templates,
            ..Default::default()
        });
        let ctx = PluginContext::new(&content, &build, &content, &content);

        plugin.before_compile(&ctx).unwrap();
        assert!(build.join(".meta").join("index.meta.json").exists());
    }

    #[test]
    fn before_compile_no_markdown_files_still_returns_ok() {
        let (_tmp, content, build, _site, templates) = layout();
        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: templates,
            ..Default::default()
        });
        let ctx = PluginContext::new(&content, &build, &content, &content);
        plugin.before_compile(&ctx).unwrap();
    }

    #[test]
    fn after_compile_without_config_uses_empty_site_globals() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("templates/tera"),
            ..Default::default()
        });
        let ctx = PluginContext::new(
            &dir.path().join("content"),
            &dir.path().join("build"),
            &dir.path().join("site"),
            &dir.path().join("templates"),
        );

        plugin.after_compile(&ctx).unwrap();
        let output =
            fs::read_to_string(dir.path().join("site").join("index.html"))
                .unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn after_compile_loads_data_files_into_context() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("nav.toml"), r#"site = "demo""#).unwrap();

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("templates/tera"),
            ..Default::default()
        });
        let config = make_config(dir.path());
        let ctx = PluginContext::with_config(
            &config.content_dir.clone(),
            &config.output_dir.clone(),
            &dir.path().join("site"),
            &config.template_dir.clone(),
            config,
        );

        plugin.after_compile(&ctx).unwrap();
        let output =
            fs::read_to_string(dir.path().join("site").join("index.html"))
                .unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn after_compile_unknown_layout_does_not_propagate_error() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let meta_dir = dir.path().join("build").join(".meta");
        fs::write(
            meta_dir.join("index.meta.json"),
            r#"{"title": "Home", "layout": "unknown_layout_999"}"#,
        )
        .unwrap();

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("templates/tera"),
            ..Default::default()
        });
        let ctx = PluginContext::new(
            &dir.path().join("content"),
            &dir.path().join("build"),
            &dir.path().join("site"),
            &dir.path().join("templates"),
        );

        plugin
            .after_compile(&ctx)
            .expect("render failure must not propagate");
    }

    #[test]
    fn after_compile_default_layout_is_page_when_missing_field() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let meta_dir = dir.path().join("build").join(".meta");
        fs::write(meta_dir.join("index.meta.json"), r#"{"title": "Home"}"#)
            .unwrap();

        let plugin = TemplatePlugin::new(TemplateConfig {
            template_dir: dir.path().join("templates/tera"),
            ..Default::default()
        });
        let ctx = PluginContext::new(
            &dir.path().join("content"),
            &dir.path().join("build"),
            &dir.path().join("site"),
            &dir.path().join("templates"),
        );

        plugin.after_compile(&ctx).unwrap();
        let out =
            fs::read_to_string(dir.path().join("site").join("index.html"))
                .unwrap();
        assert!(out.contains("<!DOCTYPE html>"));
    }

    // -------------------------------------------------------------------
    // read_frontmatter_for_html — three branches
    // -------------------------------------------------------------------

    #[test]
    fn read_frontmatter_for_html_direct_sidecar_match() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let sidecars = dir.path().join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("post.html");
        fs::write(&html, "").unwrap();
        fs::write(sidecars.join("post.meta.json"), r#"{"title": "Direct"}"#)
            .unwrap();

        let meta = read_frontmatter_for_html(&html, &site, &sidecars);
        assert_eq!(meta.get("title").and_then(|v| v.as_str()), Some("Direct"));
    }

    #[test]
    fn read_frontmatter_for_html_invalid_sidecar_returns_empty() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let sidecars = dir.path().join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("post.html");
        fs::write(&html, "").unwrap();
        fs::write(sidecars.join("post.meta.json"), "{not valid").unwrap();

        let meta = read_frontmatter_for_html(&html, &site, &sidecars);
        assert!(meta.is_empty());
    }

    #[test]
    fn read_frontmatter_for_html_no_match_returns_empty_map() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let sidecars = dir.path().join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("ghost.html");
        fs::write(&html, "").unwrap();

        let meta = read_frontmatter_for_html(&html, &site, &sidecars);
        assert!(meta.is_empty());
    }

    // -------------------------------------------------------------------
    // collect_html_files
    // -------------------------------------------------------------------

    #[test]
    fn collect_html_files_filters_non_html_extensions() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();
        fs::write(dir.path().join("c.js"), "").unwrap();

        let files = collect_html_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn collect_html_files_recurses_into_subdirectories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("blog").join("2026");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("index.html"), "").unwrap();
        fs::write(nested.join("post.html"), "").unwrap();

        let files = collect_html_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn collect_html_files_returns_empty_for_missing_directory() {
        let dir = tempdir().unwrap();
        let result = collect_html_files(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
    }
}
