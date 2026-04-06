// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tera template rendering plugin.
//!
//! Post-processes compiled HTML through Tera templates, enabling
//! template inheritance, conditionals, loops, and filters.

#[cfg(feature = "tera-templates")]
use crate::{
    frontmatter,
    plugin::{Plugin, PluginContext},
    tera_engine::{TeraConfig, TeraEngine},
    MAX_DIR_DEPTH,
};
#[cfg(feature = "tera-templates")]
use anyhow::Result;
#[cfg(feature = "tera-templates")]
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Plugin that post-processes compiled HTML through Tera templates.
///
/// Runs in the `after_compile` phase. For each HTML file in `site_dir`:
/// 1. Reads the companion `.meta.json` sidecar (from frontmatter extraction)
/// 2. Determines the layout from frontmatter (`layout` field, default: `page`)
/// 3. Renders the HTML through the Tera template chain
/// 4. Writes the rendered result back to the same file
///
/// Falls back gracefully if no Tera templates directory exists.
#[cfg(feature = "tera-templates")]
#[derive(Debug)]
pub struct TeraPlugin {
    config: TeraConfig,
}

#[cfg(feature = "tera-templates")]
impl TeraPlugin {
    /// Creates a new `TeraPlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: TeraConfig) -> Self {
        Self { config }
    }

    /// Creates a `TeraPlugin` that looks for templates in the standard
    /// `templates/tera/` subdirectory of the template dir.
    #[must_use]
    pub fn from_template_dir(template_dir: &Path) -> Self {
        Self {
            config: TeraConfig {
                template_dir: template_dir.join("tera"),
                ..Default::default()
            },
        }
    }
}

#[cfg(feature = "tera-templates")]
impl Plugin for TeraPlugin {
    fn name(&self) -> &'static str {
        "tera"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        // Emit .meta.json sidecars for all markdown content
        let sidecar_dir = ctx.build_dir.join(".meta");
        let count = frontmatter::emit_sidecars(&ctx.content_dir, &sidecar_dir)?;
        if count > 0 {
            log::info!("[tera] Emitted {count} frontmatter sidecar(s)");
        }
        Ok(())
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let engine = if let Some(e) = TeraEngine::init(self.config.clone())? {
            e
        } else {
            log::info!(
                "[tera] No templates at {:?}, skipping",
                self.config.template_dir
            );
            return Ok(());
        };

        // Build site-level globals from config
        let mut site_globals = ctx
            .config
            .as_ref()
            .map(TeraEngine::site_globals_from_config)
            .unwrap_or_default();

        // Load data files (data/*.toml, data/*.json) into context
        let data_files = TeraEngine::load_data_files(&ctx.content_dir);
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
                    log::warn!("[tera] Failed to render {html_path:?}: {e}");
                }
            }
        }

        if rendered > 0 {
            log::info!("[tera] Rendered {rendered} page(s)");
        }
        Ok(())
    }
}

/// Reads frontmatter for an HTML file, trying sidecar then falling back to empty.
#[cfg(feature = "tera-templates")]
fn read_frontmatter_for_html(
    html_path: &Path,
    site_dir: &Path,
    sidecar_dir: &Path,
) -> HashMap<String, serde_json::Value> {
    // Try to find a matching sidecar by relative path
    let rel = html_path.strip_prefix(site_dir).unwrap_or(html_path);

    // HTML files map to .md sidecars: index.html → index.meta.json
    let sidecar = sidecar_dir.join(rel).with_extension("meta.json");
    if sidecar.exists() {
        if let Ok(content) = fs::read_to_string(&sidecar) {
            if let Ok(meta) = serde_json::from_str(&content) {
                return meta;
            }
        }
    }

    // Try with .md extension: about.html → about.md.meta.json
    let md_name = rel.with_extension("md");
    let md_sidecar = sidecar_dir.join(&md_name).with_extension("meta.json");
    if md_sidecar.exists() {
        if let Ok(content) = fs::read_to_string(&md_sidecar) {
            if let Ok(meta) = serde_json::from_str(&content) {
                return meta;
            }
        }
    }

    HashMap::new()
}

/// Recursively collects `.html` files from a directory.
#[cfg(feature = "tera-templates")]
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];

    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DIR_DEPTH || !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if path.extension().is_some_and(|ext| ext == "html") {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(all(test, feature = "tera-templates"))]
mod tests {
    use super::*;
    use crate::cmd::SsgConfig;
    use std::fs;
    use tempfile::tempdir;

    fn setup_project(dir: &Path) {
        let content = dir.join("content");
        let build = dir.join("build");
        let site = dir.join("site");
        let templates = dir.join("templates/tera");
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(&build).unwrap();
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&templates).unwrap();

        // Write Tera templates
        fs::write(
            templates.join("base.html"),
            r#"<!DOCTYPE html>
<html><head><title>{{ page.title | default(value="") }}</title></head>
<body>{% block content %}{% endblock %}</body></html>"#,
        )
        .unwrap();

        fs::write(
            templates.join("page.html"),
            r#"{% extends "base.html" %}
{% block content %}{{ page.content | safe }}{% endblock %}"#,
        )
        .unwrap();

        // Write content
        fs::write(
            content.join("index.md"),
            "---\ntitle: Home\nlayout: page\n---\n# Welcome\n",
        )
        .unwrap();

        // Write compiled HTML (simulating staticdatagen output)
        fs::write(site.join("index.html"), "<h1>Welcome</h1>").unwrap();

        // Write sidecar
        let meta_dir = build.join(".meta");
        fs::create_dir_all(&meta_dir).unwrap();
        fs::write(
            meta_dir.join("index.meta.json"),
            r#"{"title": "Home", "layout": "page"}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_tera_plugin_renders() {
        let dir = tempdir().unwrap();
        setup_project(dir.path());

        let plugin = TeraPlugin::new(TeraConfig {
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
    fn test_tera_plugin_skips_missing_templates() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("index.html"), "<p>hello</p>").unwrap();

        let plugin = TeraPlugin::new(TeraConfig {
            template_dir: dir.path().join("nonexistent"),
            ..Default::default()
        });

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());

        // Should succeed (graceful skip)
        plugin.after_compile(&ctx).unwrap();

        // Content should be unchanged
        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert_eq!(output, "<p>hello</p>");
    }
}
