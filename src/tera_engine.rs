// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tera templating engine integration.
//!
//! Wraps the [Tera](https://keats.github.io/tera/) template engine to
//! provide Jinja2-style templating with inheritance, conditionals, loops,
//! partials, and custom filters for static site generation.

#[cfg(feature = "tera-templates")]
use anyhow::{Context, Result};
#[cfg(feature = "tera-templates")]
use std::{collections::HashMap, path::PathBuf};

/// Configuration for the Tera templating engine.
#[cfg(feature = "tera-templates")]
#[derive(Debug, Clone)]
pub struct TeraConfig {
    /// Directory containing Tera templates.
    pub template_dir: PathBuf,
    /// Global variables injected into every template context.
    pub globals: HashMap<String, serde_json::Value>,
    /// Whether to enable HTML auto-escaping (default: true).
    pub autoescape: bool,
}

#[cfg(feature = "tera-templates")]
impl Default for TeraConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("templates/tera"),
            globals: HashMap::new(),
            autoescape: true,
        }
    }
}

/// Wraps Tera and provides site-generation-specific rendering.
#[cfg(feature = "tera-templates")]
#[derive(Debug)]
pub struct TeraEngine {
    tera: tera::Tera,
    config: TeraConfig,
}

#[cfg(feature = "tera-templates")]
impl TeraEngine {
    /// Initializes the Tera engine from a template directory.
    ///
    /// Loads all `*.html` files recursively from the template directory.
    /// Returns `Ok(None)` if the template directory does not exist
    /// (graceful fallback for projects without Tera templates).
    pub fn init(config: TeraConfig) -> Result<Option<Self>> {
        if !config.template_dir.exists() {
            return Ok(None);
        }

        let glob = config
            .template_dir
            .join("**/*.html")
            .to_string_lossy()
            .to_string();

        let mut tera = tera::Tera::new(&glob).with_context(|| {
            format!(
                "Failed to load Tera templates from {:?}",
                config.template_dir
            )
        })?;

        // Register custom filters
        tera.register_filter("reading_time", reading_time_filter);

        if !config.autoescape {
            tera.autoescape_on(vec![]);
        }

        Ok(Some(Self { tera, config }))
    }

    /// Renders a page through the Tera template chain.
    ///
    /// # Arguments
    /// * `template_name` — template to render (e.g. `"page.html"`)
    /// * `page_content` — compiled HTML content from staticdatagen
    /// * `frontmatter` — parsed frontmatter as JSON key-value pairs
    /// * `site_globals` — site-level variables (name, base_url, etc.)
    pub fn render_page(
        &self,
        template_name: &str,
        page_content: &str,
        frontmatter: &HashMap<String, serde_json::Value>,
        site_globals: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let mut context = tera::Context::new();

        // Inject page-level variables under `page.*`
        let mut page = HashMap::new();
        for (k, v) in frontmatter {
            let _ = page.insert(k.clone(), v.clone());
        }
        let _ = page.insert(
            "content".to_string(),
            serde_json::Value::String(page_content.to_string()),
        );
        context.insert("page", &page);

        // Inject site-level variables under `site.*`
        context.insert("site", site_globals);

        // Inject global config variables
        for (k, v) in &self.config.globals {
            context.insert(k, v);
        }

        // Determine which template to use, fall back to page.html
        let tmpl = if self.tera.get_template(template_name).is_ok() {
            template_name.to_string()
        } else if self.tera.get_template("page.html").is_ok() {
            "page.html".to_string()
        } else {
            // No matching template — return content as-is
            return Ok(page_content.to_string());
        };

        self.tera
            .render(&tmpl, &context)
            .with_context(|| format!("Failed to render template '{}'", tmpl))
    }

    /// Builds site-level globals from an `SsgConfig`.
    pub fn site_globals_from_config(
        config: &crate::cmd::SsgConfig,
    ) -> HashMap<String, serde_json::Value> {
        let mut globals = HashMap::new();
        let _ = globals.insert(
            "name".to_string(),
            serde_json::Value::String(config.site_name.clone()),
        );
        let _ = globals.insert(
            "title".to_string(),
            serde_json::Value::String(config.site_title.clone()),
        );
        let _ = globals.insert(
            "description".to_string(),
            serde_json::Value::String(config.site_description.clone()),
        );
        let _ = globals.insert(
            "base_url".to_string(),
            serde_json::Value::String(config.base_url.clone()),
        );
        let _ = globals.insert(
            "language".to_string(),
            serde_json::Value::String(config.language.clone()),
        );
        globals
    }

    /// Loads data files from a `data/` directory into the context.
    ///
    /// Supports `.toml`, `.json`, and `.yml`/`.yaml` files.
    /// Files are accessible as `{{ data.filename }}` in templates.
    ///
    /// Example: `data/nav.toml` → `{{ data.nav.links }}`
    pub fn load_data_files(
        content_dir: &std::path::Path,
    ) -> HashMap<String, serde_json::Value> {
        let data_dir = content_dir.parent().unwrap_or(content_dir).join("data");
        let mut data = HashMap::new();

        if !data_dir.exists() {
            return data;
        }

        let entries = match std::fs::read_dir(&data_dir) {
            Ok(e) => e,
            Err(_) => return data,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let value: Option<serde_json::Value> = match ext.as_str() {
                "toml" => toml::from_str::<serde_json::Value>(&content).ok(),
                "json" => serde_json::from_str(&content).ok(),
                "yml" | "yaml" => {
                    // Parse YAML via serde_json round-trip
                    serde_json::from_str(&content).ok()
                }
                _ => None,
            };

            if let Some(val) = value {
                let _ = data.insert(stem, val);
            }
        }

        data
    }
}

/// Custom Tera filter: estimates reading time in minutes.
///
/// Usage: `{{ page.content | reading_time }}`
/// Returns a string like "3 min read".
#[cfg(feature = "tera-templates")]
fn reading_time_filter(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let text = value.as_str().unwrap_or("");
    let word_count = text.split_whitespace().count();
    let minutes = (word_count / 200).max(1);
    Ok(tera::Value::String(format!("{} min read", minutes)))
}

#[cfg(all(test, feature = "tera-templates"))]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn setup_templates(dir: &Path) {
        let tera_dir = dir.join("tera");
        fs::create_dir_all(&tera_dir).unwrap();

        fs::write(
            tera_dir.join("base.html"),
            r#"<!DOCTYPE html>
<html lang="{{ site.language | default(value="en") }}">
<head><title>{% block title %}{{ page.title | default(value="Untitled") }}{% endblock %}</title>
{% block head_extra %}{% endblock %}
</head>
<body>
<main>{% block content %}{% endblock %}</main>
<footer>{% block footer %}<p>&copy; {{ site.name | default(value="") }}</p>{% endblock %}</footer>
</body>
</html>"#,
        )
        .unwrap();

        fs::write(
            tera_dir.join("page.html"),
            r#"{% extends "base.html" %}
{% block content %}{{ page.content | safe }}{% endblock %}"#,
        )
        .unwrap();

        fs::write(
            tera_dir.join("post.html"),
            r#"{% extends "base.html" %}
{% block content %}
<article>
<h1>{{ page.title | default(value="") }}</h1>
<time>{{ page.date | default(value="") }}</time>
<p>{{ page.content | reading_time }}</p>
{{ page.content | safe }}
</article>
{% endblock %}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_init_missing_dir() {
        let config = TeraConfig {
            template_dir: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let result = TeraEngine::init(config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_init_and_render_page() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TeraConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();

        let mut fm = HashMap::new();
        let _ = fm.insert(
            "title".to_string(),
            serde_json::Value::String("Hello".to_string()),
        );

        let mut site = HashMap::new();
        let _ = site.insert(
            "name".to_string(),
            serde_json::Value::String("My Site".to_string()),
        );
        let _ = site.insert(
            "language".to_string(),
            serde_json::Value::String("en-GB".to_string()),
        );

        let result = engine
            .render_page("page.html", "<p>Body</p>", &fm, &site)
            .unwrap();

        assert!(result.contains("Hello"));
        assert!(result.contains("<p>Body</p>"));
        assert!(result.contains("My Site"));
        assert!(result.contains("en-GB"));
    }

    #[test]
    fn test_render_post_with_reading_time() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TeraConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();

        let content = "word ".repeat(600); // ~3 min read
        let mut fm = HashMap::new();
        let _ = fm.insert(
            "title".to_string(),
            serde_json::Value::String("Post".to_string()),
        );
        let _ = fm.insert(
            "date".to_string(),
            serde_json::Value::String("2026-01-01".to_string()),
        );

        let site = HashMap::new();
        let result = engine
            .render_page("post.html", &content, &fm, &site)
            .unwrap();

        assert!(result.contains("3 min read"));
        assert!(result.contains("<article>"));
    }

    #[test]
    fn test_fallback_to_page_html() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TeraConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();

        let fm = HashMap::new();
        let site = HashMap::new();
        let result = engine
            .render_page("nonexistent.html", "<p>fallback</p>", &fm, &site)
            .unwrap();

        assert!(result.contains("<p>fallback</p>"));
    }

    #[test]
    fn test_reading_time_filter_direct() {
        let text = "word ".repeat(400);
        let val = tera::Value::String(text);
        let result = reading_time_filter(&val, &HashMap::new()).unwrap();
        assert_eq!(result, tera::Value::String("2 min read".to_string()));
    }
}
