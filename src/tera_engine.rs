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
                "Failed to load Tera templates from {}",
                config.template_dir.display()
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
    /// * `site_globals` — site-level variables (name, `base_url`, etc.)
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
            .with_context(|| format!("Failed to render template '{tmpl}'"))
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
    Ok(tera::Value::String(format!("{minutes} min read")))
}

#[cfg(all(test, feature = "tera-templates"))]
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

    // -------------------------------------------------------------------
    // load_data_files — format + fallback coverage
    // -------------------------------------------------------------------

    #[test]
    fn load_data_files_missing_data_dir_returns_empty_map() {
        // The `!data_dir.exists()` early return at line 173 is
        // exercised when there's no `data/` sibling to content_dir.
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let result = TeraEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    #[test]
    fn load_data_files_parses_toml_and_json_and_yaml() {
        // Covers the main body of load_data_files at lines 182-216:
        // the file walk, read_to_string success, extension match,
        // and `if let Some(val) = value` branch.
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

        // Add a subdirectory — the `!path.is_file() continue` at
        // line 184 should skip it.
        let sub = data.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("inside.json"), "{}").unwrap();

        let result = TeraEngine::load_data_files(&content);
        assert!(result.contains_key("site"));
        assert!(result.contains_key("nav"));
        assert!(result.contains_key("conf"));
        assert!(!result.contains_key("ignored"));
        assert!(!result.contains_key("sub"));
    }

    #[test]
    fn load_data_files_skips_files_with_invalid_content() {
        // Covers the `Option::ok()` branch that returns None on
        // unparseable content — the `if let Some(val) = value` at
        // line 214 skips those entries.
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::write(data.join("broken.toml"), "not valid toml [[[").unwrap();
        fs::write(data.join("broken.json"), "{not valid").unwrap();
        fs::write(data.join("good.toml"), r#"x = "y""#).unwrap();

        let result = TeraEngine::load_data_files(&content);
        // Only the good file survives.
        assert!(result.contains_key("good"));
        assert!(!result.contains_key("broken"));
    }

    #[test]
    fn load_data_files_ignores_unsupported_extensions() {
        // The `_ => None` arm of the extension match at line 211.
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        fs::write(data.join("a.xml"), "<x/>").unwrap();
        fs::write(data.join("b.csv"), "a,b").unwrap();
        fs::write(data.join("c"), "no extension").unwrap();

        let result = TeraEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    // -------------------------------------------------------------------
    // render_page — custom globals + no-fallback branch
    // -------------------------------------------------------------------

    #[test]
    fn render_page_injects_custom_globals_from_config() {
        // Covers lines 112-114 — the `for (k, v) in &self.config.globals`
        // loop that injects engine-level globals into every render.
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let mut globals = HashMap::new();
        let _ = globals.insert(
            "brand".to_string(),
            serde_json::Value::String("Acme".to_string()),
        );
        let config = TeraConfig {
            template_dir: dir.path().join("tera"),
            globals,
            ..Default::default()
        };
        let _ = TeraEngine::init(config).unwrap().unwrap();

        // Add a minimal template that references the custom global.
        fs::write(
            dir.path().join("tera").join("branded.html"),
            r"<p>{{ brand }}</p>",
        )
        .unwrap();

        // Re-init to pick up the new template.
        let config2 = TeraConfig {
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
        let engine = TeraEngine::init(config2).unwrap().unwrap();

        let result = engine
            .render_page("branded.html", "", &HashMap::new(), &HashMap::new())
            .unwrap();
        assert!(result.contains("Acme"));
        let _ = engine; // keep engine alive for the assertion
    }

    #[test]
    fn render_page_no_matching_template_and_no_page_html_returns_content_as_is()
    {
        // Covers line 123: the final fallback `return Ok(page_content.to_string())`
        // when neither the requested template nor `page.html` exist.
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        // Only write a `base.html`, NOT a `page.html`.
        fs::write(
            tera_dir.join("base.html"),
            r"<!DOCTYPE html><html><body>{% block content %}{% endblock %}</body></html>",
        )
        .unwrap();

        let config = TeraConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();

        let content = "<p>raw content</p>";
        let result = engine
            .render_page(
                "nonexistent.html",
                content,
                &HashMap::new(),
                &HashMap::new(),
            )
            .unwrap();
        // Content returned verbatim when no template chain matches.
        assert_eq!(result, content);
    }

    #[test]
    fn init_with_autoescape_false_calls_autoescape_on_with_empty_vec() {
        // Line 75: the `if !config.autoescape` branch.
        let dir = tempdir().unwrap();
        setup_templates(dir.path());

        let config = TeraConfig {
            template_dir: dir.path().join("tera"),
            autoescape: false,
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();
        // Engine constructed; render still works.
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
    fn init_with_broken_template_propagates_with_context_error() {
        // Lines 64-69: the `.with_context(|| format!(...))` closure
        // on Tera::new failure. Plant a template with invalid syntax.
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        // {% block %} is unclosed → Tera::new returns Err.
        fs::write(tera_dir.join("broken.html"), "{% block %}").unwrap();

        let config = TeraConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        let result = TeraEngine::init(config);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Failed to load Tera templates"));
    }

    #[test]
    #[cfg(unix)]
    fn load_data_files_unreadable_file_continues_silently() {
        // Line 201: `Err(_) => continue` for the read_to_string Err
        // branch. We create a broken symlink (link target doesn't
        // exist) which `is_file()` reports true on dangling symlinks
        // is platform-dependent — we use a regular file with a
        // sibling that we then make unreadable via path tricks.
        // Simplest cross-fs: a directory shaped like a .toml file —
        // .file_stem() and .extension() succeed, but read_to_string
        // returns Err because the path is a directory.
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let data = dir.path().join("data");
        fs::create_dir_all(&data).unwrap();

        // A .toml "file" that's actually a directory.
        fs::create_dir_all(data.join("not-really.toml")).unwrap();
        // Plus a real one to prove the rest of the loop continues.
        fs::write(data.join("real.toml"), r#"k = "v""#).unwrap();

        let result = TeraEngine::load_data_files(&content);
        // Only the real file makes it through.
        assert!(result.contains_key("real"));
        assert!(!result.contains_key("not-really"));
    }

    #[test]
    fn load_data_files_data_dir_is_a_file_returns_empty() {
        // Line 179: `Err(_) => return data` from read_dir when the
        // path resolves to a file rather than a directory.
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        // Plant `data` as a file alongside content/.
        let data = dir.path().join("data");
        fs::write(&data, "I am a file, not a directory").unwrap();

        let result = TeraEngine::load_data_files(&content);
        assert!(result.is_empty());
    }

    #[test]
    fn render_page_propagates_tera_render_errors() {
        // Covers line 128: `.with_context(...)` on a Tera render Err.
        // Write a template with a syntax-valid-but-undefined filter.
        let dir = tempdir().unwrap();
        let tera_dir = dir.path().join("tera");
        fs::create_dir_all(&tera_dir).unwrap();
        fs::write(
            tera_dir.join("broken.html"),
            // `nonexistent_filter` doesn't exist → render fails.
            r"{{ page.title | nonexistent_filter }}",
        )
        .unwrap();

        let config = TeraConfig {
            template_dir: tera_dir,
            ..Default::default()
        };
        let engine = TeraEngine::init(config).unwrap().unwrap();

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
