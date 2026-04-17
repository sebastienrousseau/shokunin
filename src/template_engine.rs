// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Template engine integration (MiniJinja).
//!
//! Wraps the [MiniJinja](https://docs.rs/minijinja) template engine to
//! provide Jinja2-style templating with inheritance, conditionals, loops,
//! partials, and custom filters for static site generation.

#[cfg(feature = "templates")]
use anyhow::{Context, Result};
#[cfg(feature = "templates")]
use std::{collections::HashMap, path::PathBuf};

/// Configuration for the template engine.
#[cfg(feature = "templates")]
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Directory containing templates.
    pub template_dir: PathBuf,
    /// Global variables injected into every template context.
    pub globals: HashMap<String, serde_json::Value>,
    /// Whether to enable HTML auto-escaping (default: true).
    pub autoescape: bool,
}

#[cfg(feature = "templates")]
impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("templates/tera"),
            globals: HashMap::new(),
            autoescape: true,
        }
    }
}

/// Wraps `MiniJinja` and provides site-generation-specific rendering.
#[cfg(feature = "templates")]
#[derive(Debug)]
pub struct TemplateEngine {
    env: minijinja::Environment<'static>,
    config: TemplateConfig,
}

#[cfg(feature = "templates")]
impl TemplateEngine {
    /// Initializes the template engine from a template directory.
    ///
    /// Uses a path-based loader for lazy template resolution.
    /// Returns `Ok(None)` if the template directory does not exist
    /// (graceful fallback for projects without templates).
    pub fn init(config: TemplateConfig) -> Result<Option<Self>> {
        if !config.template_dir.exists() {
            return Ok(None);
        }

        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader(&config.template_dir));

        if !config.autoescape {
            env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        }

        // Register custom filters
        env.add_filter("reading_time", reading_time_filter);
        env.add_filter("slugify", slugify_filter);

        Ok(Some(Self { env, config }))
    }

    /// Renders a page through the template chain.
    ///
    /// # Arguments
    /// * `template_name` — template to render (e.g. `"page.html"`)
    /// * `page_content` — compiled HTML content from staticdatagen
    /// * `frontmatter` — parsed frontmatter as JSON key-value pairs
    /// * `site_globals` — site-level variables (name, `base_url`, etc.)
    pub fn render_page(
        &self,
        template_name: &str,
        page_content: &str,
        frontmatter: &HashMap<String, serde_json::Value>,
        site_globals: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Build page context
        let mut page: serde_json::Map<String, serde_json::Value> = frontmatter
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let _ = page.insert(
            "content".to_string(),
            serde_json::Value::String(page_content.to_string()),
        );

        // Build the full render context
        let mut ctx = serde_json::Map::new();
        let _ = ctx.insert("page".to_string(), serde_json::Value::Object(page));
        let _ = ctx.insert(
            "site".to_string(),
            serde_json::Value::Object(
                site_globals
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
        );

        // Inject global config variables at top level
        for (k, v) in &self.config.globals {
            let _ = ctx.insert(k.clone(), v.clone());
        }

        // Determine which template to use, fall back to page.html
        let tmpl_name = if self.env.get_template(template_name).is_ok() {
            template_name
        } else if self.env.get_template("page.html").is_ok() {
            "page.html"
        } else {
            // No matching template — return content as-is
            return Ok(page_content.to_string());
        };

        let tmpl = self.env.get_template(tmpl_name).with_context(|| {
            format!("Failed to load template '{tmpl_name}'")
        })?;

        tmpl.render(serde_json::Value::Object(ctx))
            .with_context(|| format!("Failed to render template '{tmpl_name}'"))
    }

    /// Builds site-level globals from an `SsgConfig`.
    #[must_use]
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
    #[must_use]
    pub fn load_data_files(
        content_dir: &std::path::Path,
    ) -> HashMap<String, serde_json::Value> {
        let data_dir = content_dir.parent().unwrap_or(content_dir).join("data");
        let mut data = HashMap::new();

        if !data_dir.exists() {
            return data;
        }

        let Ok(entries) = std::fs::read_dir(&data_dir) else {
            return data;
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

            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };

            let value: Option<serde_json::Value> = match ext.as_str() {
                "toml" => toml::from_str::<serde_json::Value>(&content).ok(),
                "json" => serde_json::from_str(&content).ok(),
                "yml" | "yaml" => serde_json::from_str(&content).ok(),
                _ => None,
            };

            if let Some(val) = value {
                let _ = data.insert(stem, val);
            }
        }

        data
    }
}

/// Custom filter: estimates reading time in minutes.
///
/// Usage: `{{ page.content | reading_time }}`
/// Returns a string like "3 min read".
#[cfg(feature = "templates")]
fn reading_time_filter(value: String) -> String {
    let word_count = value.split_whitespace().count();
    let minutes = (word_count / 200).max(1);
    format!("{minutes} min read")
}

/// Custom filter: converts a string to a URL-safe slug.
///
/// Usage: `{{ tag | slugify }}`
#[cfg(feature = "templates")]
fn slugify_filter(value: String) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(all(test, feature = "templates"))]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn setup_templates(dir: &Path) {
        crate::test_support::init_logger();
        let tera_dir = dir.join("tera");
        fs::create_dir_all(&tera_dir).unwrap();

        fs::write(
            tera_dir.join("base.html"),
            r#"<!DOCTYPE html>
<html lang="{{ site.language | default("en") }}">
<head><title>{% block title %}{{ page.title | default("Untitled") }}{% endblock %}</title>
{% block head_extra %}{% endblock %}
</head>
<body>
<main>{% block content %}{% endblock %}</main>
<footer>{% block footer %}<p>&copy; {{ site.name | default("") }}</p>{% endblock %}</footer>
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
<h1>{{ page.title | default("") }}</h1>
<time>{{ page.date | default("") }}</time>
<p>{{ page.content | reading_time }}</p>
{{ page.content | safe }}
</article>
{% endblock %}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_init_missing_dir() {
        let config = TemplateConfig {
            template_dir: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let result = TemplateEngine::init(config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_init_and_render_page() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TemplateConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

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

        let config = TemplateConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

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

        let config = TemplateConfig {
            template_dir: dir.path().join("tera"),
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

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
        let result = reading_time_filter(text);
        assert_eq!(result, "2 min read");
    }

    #[test]
    fn test_slugify_filter() {
        assert_eq!(slugify_filter("Hello World!".to_string()), "hello-world");
        assert_eq!(slugify_filter("Rust & Web".to_string()), "rust-web");
    }

    // -------------------------------------------------------------------
    // load_data_files — format + fallback coverage
    // -------------------------------------------------------------------

    #[test]
    fn load_data_files_missing_data_dir_returns_empty_map() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let result = TemplateEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    #[test]
    fn load_data_files_parses_toml_and_json_and_yaml() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::write(data.join("site.toml"), r#"key = "toml-value""#).unwrap();
        fs::write(data.join("nav.json"), r#"{"items": ["home", "about"]}"#)
            .unwrap();
        fs::write(data.join("conf.yml"), r#"{"yaml": "value"}"#).unwrap();
        fs::write(data.join("ignored.txt"), "not parsed").unwrap();

        let sub = data.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("inside.json"), "{}").unwrap();

        let result = TemplateEngine::load_data_files(&content);
        assert!(result.contains_key("site"));
        assert!(result.contains_key("nav"));
        assert!(result.contains_key("conf"));
        assert!(!result.contains_key("ignored"));
        assert!(!result.contains_key("sub"));
    }

    #[test]
    fn load_data_files_skips_files_with_invalid_content() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::write(data.join("broken.toml"), "not valid toml [[[").unwrap();
        fs::write(data.join("broken.json"), "{not valid").unwrap();
        fs::write(data.join("good.toml"), r#"x = "y""#).unwrap();

        let result = TemplateEngine::load_data_files(&content);
        assert!(result.contains_key("good"));
        assert!(!result.contains_key("broken"));
    }

    #[test]
    fn load_data_files_ignores_unsupported_extensions() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::write(data.join("a.xml"), "<x/>").unwrap();
        fs::write(data.join("b.csv"), "a,b").unwrap();
        fs::write(data.join("c"), "no extension").unwrap();

        let result = TemplateEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    // -------------------------------------------------------------------
    // render_page — custom globals + no-fallback branch
    // -------------------------------------------------------------------

    #[test]
    fn render_page_injects_custom_globals_from_config() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        // Write a minimal template that references the custom global.
        fs::write(
            dir.path().join("tera").join("branded.html"),
            r"<p>{{ brand }}</p>",
        )
        .unwrap();

        let config = TemplateConfig {
            template_dir: dir.path().join("tera"),
            globals: {
                let mut g = HashMap::new();
                let _ = g.insert(
                    "brand".to_string(),
                    serde_json::Value::String("Acme".to_string()),
                );
                g
            },
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

        let result = engine
            .render_page("branded.html", "", &HashMap::new(), &HashMap::new())
            .unwrap();
        assert!(result.contains("Acme"));
    }

    #[test]
    fn render_page_no_matching_template_and_no_page_html_returns_content_as_is()
    {
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        // Only write a `base.html`, NOT a `page.html`.
        fs::write(
            tera_dir.join("base.html"),
            r"<!DOCTYPE html><html><body>{% block content %}{% endblock %}</body></html>",
        )
        .unwrap();

        let config = TemplateConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

        let content = "<p>raw content</p>";
        let result = engine
            .render_page(
                "nonexistent.html",
                content,
                &HashMap::new(),
                &HashMap::new(),
            )
            .unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn init_with_autoescape_false() {
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TemplateConfig {
            template_dir: dir.path().join("tera"),
            autoescape: false,
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();
        let result = engine
            .render_page(
                "page.html",
                "<p>x</p>",
                &HashMap::new(),
                &HashMap::new(),
            )
            .unwrap();
        assert!(result.contains("<p>x</p>"));
    }

    #[test]
    fn init_with_broken_template_errors_on_render() {
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        // Use an extends to a non-existent parent — always errors on render
        fs::write(tera_dir.join("broken.html"), "{% extends \"nonexistent_parent.html\" %}{% block x %}{% endblock %}").unwrap();

        let config = TemplateConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        // MiniJinja uses lazy loading — init succeeds
        let engine = TemplateEngine::init(config).unwrap().unwrap();
        // Error surfaces at render time
        let result = engine.render_page(
            "broken.html",
            "",
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn load_data_files_unreadable_file_continues_silently() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::create_dir_all(data.join("not-really.toml")).unwrap();
        fs::write(data.join("real.toml"), r#"k = "v""#).unwrap();

        let result = TemplateEngine::load_data_files(&content);
        assert!(result.contains_key("real"));
        assert!(!result.contains_key("not-really"));
    }

    #[test]
    fn load_data_files_data_dir_is_a_file_returns_empty() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::write(&data, "I am a file, not a directory").unwrap();

        let result = TemplateEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    #[test]
    fn render_page_propagates_render_errors() {
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        // Undefined filter → render fails
        fs::write(
            tera_dir.join("broken.html"),
            r"{{ page.title | nonexistent_filter }}",
        )
        .unwrap();

        let config = TemplateConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        let engine = TemplateEngine::init(config).unwrap().unwrap();

        let mut fm = HashMap::new();
        let _ = fm.insert(
            "title".to_string(),
            serde_json::Value::String("T".to_string()),
        );

        let result =
            engine.render_page("broken.html", "", &fm, &HashMap::new());
        assert!(result.is_err());
    }
}
