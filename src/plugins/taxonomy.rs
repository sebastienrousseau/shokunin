// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Taxonomy generation plugin.
//!
//! Reads `tags` and `categories` from frontmatter sidecars and
//! generates index pages for each taxonomy term, rendered through
//! the same template engine (`MiniJinja`) that drives normal page
//! rendering. Built-in fallback templates extend `base.html` so the
//! pages share the site's layout, CSS, nav, and footer (#542).

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// A mapping from taxonomy term to a list of (title, URL) pairs.
type TaxonomyMap = HashMap<String, Vec<(String, String)>>;

/// A taxonomy term with its associated pages.
#[derive(Debug, Clone)]
pub struct TaxonomyTerm {
    /// The term name (e.g. "rust", "web").
    pub name: String,
    /// The URL slug (e.g. "rust", "web").
    pub slug: String,
    /// Pages with this term: (title, url).
    pub pages: Vec<(String, String)>,
}

// =====================================================================
// Built-in templates (embedded so the binary works without scaffold)
// =====================================================================

/// Built-in tag term-page template (#542).
const BUILTIN_TAG_HTML: &str = include_str!("builtin_templates/tag.html");
/// Built-in category term-page template (#542).
const BUILTIN_CATEGORY_HTML: &str =
    include_str!("builtin_templates/category.html");
/// Built-in archive/topic term-page template (#542).
const BUILTIN_ARCHIVE_HTML: &str =
    include_str!("builtin_templates/archive.html");
/// Built-in taxonomy index-page template (lists all terms) (#542).
const BUILTIN_TAXONOMY_INDEX_HTML: &str =
    include_str!("builtin_templates/taxonomy_index.html");
/// Built-in minimal `base.html` for sites that ship none of their own.
/// User-provided `base.html` is preferred via the path loader (#542).
const BUILTIN_BASE_HTML: &str = include_str!("builtin_templates/base.html");

/// Plugin that generates taxonomy index pages for tags and categories.
///
/// Runs in `after_compile`. Reads `.meta.json` sidecars to find
/// `tags` and `categories` arrays, then generates:
/// - `/tags/index.html` — list of all tags with page counts
/// - `/tags/{slug}/index.html` — list of pages for each tag
/// - `/categories/index.html` and `/categories/{slug}/index.html`
/// - `/topics/index.html` and `/topics/{slug}/index.html`
///
/// All pages render through the site's `MiniJinja` template engine so
/// they share the site's `base.html`, CSS, nav, footer, and lang
/// attribute (issue #542).
#[derive(Debug, Clone, Copy)]
pub struct TaxonomyPlugin;

impl Plugin for TaxonomyPlugin {
    fn name(&self) -> &'static str {
        "taxonomy"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        let sidecar_dir = ctx.build_dir.join(".meta");
        if !sidecar_dir.exists() {
            return Ok(());
        }

        let (tags, categories, topics) =
            collect_taxonomy_entries(&sidecar_dir)?;

        // Lazily build the template engine once per run; reused across
        // tags, categories, and topics.
        let renderer = TaxonomyRenderer::new(ctx);

        if !tags.is_empty() {
            generate_taxonomy_pages(
                &ctx.site_dir,
                "tags",
                "Tags",
                &tags,
                TaxonomyKind::Tag,
                &renderer,
            )?;
            log::info!("[taxonomy] Generated {} tag page(s)", tags.len());
        }

        if !categories.is_empty() {
            generate_taxonomy_pages(
                &ctx.site_dir,
                "categories",
                "Categories",
                &categories,
                TaxonomyKind::Category,
                &renderer,
            )?;
            log::info!(
                "[taxonomy] Generated {} category page(s)",
                categories.len()
            );
        }

        if !topics.is_empty() {
            generate_taxonomy_pages(
                &ctx.site_dir,
                "topics",
                "Topics",
                &topics,
                TaxonomyKind::Archive,
                &renderer,
            )?;
            log::info!("[taxonomy] Generated {} topic page(s)", topics.len());
        }

        Ok(())
    }
}

/// Which built-in template family to use for a taxonomy.
#[derive(Debug, Clone, Copy)]
enum TaxonomyKind {
    Tag,
    Category,
    Archive,
}

impl TaxonomyKind {
    /// User-overridable template filename (looked up in the user's
    /// `templates/tera/` directory first).
    const fn template_name(self) -> &'static str {
        match self {
            Self::Tag => "tag.html",
            Self::Category => "category.html",
            Self::Archive => "archive.html",
        }
    }

    /// Variable name the template uses to address the current term
    /// (`tag`, `category`, `term`).
    const fn term_var(self) -> &'static str {
        match self {
            Self::Tag => "tag",
            Self::Category => "category",
            Self::Archive => "term",
        }
    }
}

// =====================================================================
// Template engine integration
// =====================================================================

/// Encapsulates `MiniJinja` rendering for taxonomy pages.
///
/// Holds a `MiniJinja` environment whose loader prefers the user's
/// `templates/tera/` files and falls back to embedded built-in sources
/// so pages always extend a real `base.html` (issue #542).
struct TaxonomyRenderer<'a> {
    #[cfg(feature = "templates")]
    env: minijinja::Environment<'static>,
    ctx: &'a PluginContext,
}

#[cfg(feature = "templates")]
impl<'a> TaxonomyRenderer<'a> {
    fn new(ctx: &'a PluginContext) -> Self {
        let user_dir = resolve_user_template_dir(ctx);

        let mut env = minijinja::Environment::new();
        env.set_loader(
            move |name| -> Result<Option<String>, minijinja::Error> {
                // 1) Try the user's templates/tera/<name> if present.
                if let Some(dir) = user_dir.as_ref() {
                    let candidate = dir.join(name);
                    match fs::read_to_string(&candidate) {
                        Ok(s) => return Ok(Some(s)),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => {
                            return Err(minijinja::Error::new(
                                minijinja::ErrorKind::InvalidOperation,
                                format!(
                                    "failed to read user template {}: {e}",
                                    candidate.display()
                                ),
                            ))
                        }
                    }
                }
                // 2) Embedded fallbacks for the templates this plugin owns.
                Ok(match name {
                    "base.html" => Some(BUILTIN_BASE_HTML.to_string()),
                    "tag.html" => Some(BUILTIN_TAG_HTML.to_string()),
                    "category.html" => Some(BUILTIN_CATEGORY_HTML.to_string()),
                    "archive.html" => Some(BUILTIN_ARCHIVE_HTML.to_string()),
                    "taxonomy_index.html" => {
                        Some(BUILTIN_TAXONOMY_INDEX_HTML.to_string())
                    }
                    _ => None,
                })
            },
        );

        Self { env, ctx }
    }

    /// Renders a term page (e.g. `/tags/rust/index.html`).
    fn render_term_page(
        &self,
        kind: TaxonomyKind,
        taxonomy_name: &str,
        taxonomy_title: &str,
        term: &str,
        slug: &str,
        pages: &[(String, String)],
    ) -> Result<String, SsgError> {
        let tmpl =
            self.env.get_template(kind.template_name()).map_err(|e| {
                SsgError::Io {
                    path: PathBuf::from(kind.template_name()),
                    source: std::io::Error::other(e.to_string()),
                }
            })?;

        let mut ctx_map = self.base_context();
        let _ = ctx_map.insert(
            kind.term_var().to_string(),
            serde_json::Value::String(term.to_string()),
        );
        // Always expose `term` as well so generic templates can use it.
        let _ = ctx_map.insert(
            "term".to_string(),
            serde_json::Value::String(term.to_string()),
        );
        let _ = ctx_map.insert(
            "slug".to_string(),
            serde_json::Value::String(slug.to_string()),
        );
        let _ = ctx_map.insert(
            "taxonomy_name".to_string(),
            serde_json::Value::String(taxonomy_name.to_string()),
        );
        let _ = ctx_map.insert(
            "taxonomy_title".to_string(),
            serde_json::Value::String(taxonomy_title.to_string()),
        );
        let _ = ctx_map.insert(
            "page_url".to_string(),
            serde_json::Value::String(format!("/{taxonomy_name}/{slug}/")),
        );
        let _ = ctx_map.insert(
            "posts".to_string(),
            serde_json::Value::Array(pages_to_json(pages)),
        );

        tmpl.render(serde_json::Value::Object(ctx_map))
            .map(|mut s| {
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                s
            })
            .map_err(|e| SsgError::Io {
                path: PathBuf::from(kind.template_name()),
                source: std::io::Error::other(e.to_string()),
            })
    }

    /// Renders the taxonomy index page (lists all terms).
    fn render_index_page(
        &self,
        taxonomy_name: &str,
        taxonomy_title: &str,
        sorted_terms: &[(&String, &Vec<(String, String)>)],
    ) -> Result<String, SsgError> {
        let tmpl =
            self.env.get_template("taxonomy_index.html").map_err(|e| {
                SsgError::Io {
                    path: PathBuf::from("taxonomy_index.html"),
                    source: std::io::Error::other(e.to_string()),
                }
            })?;

        let mut ctx_map = self.base_context();
        let _ = ctx_map.insert(
            "taxonomy_name".to_string(),
            serde_json::Value::String(taxonomy_name.to_string()),
        );
        let _ = ctx_map.insert(
            "taxonomy_title".to_string(),
            serde_json::Value::String(taxonomy_title.to_string()),
        );
        let _ = ctx_map.insert(
            "page_url".to_string(),
            serde_json::Value::String(format!("/{taxonomy_name}/")),
        );

        let term_entries: Vec<serde_json::Value> = sorted_terms
            .iter()
            .map(|(term, pages)| {
                let mut obj = serde_json::Map::new();
                let _ = obj.insert(
                    "name".to_string(),
                    serde_json::Value::String((*term).clone()),
                );
                let _ = obj.insert(
                    "slug".to_string(),
                    serde_json::Value::String(slugify(term)),
                );
                let _ = obj.insert(
                    "count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        pages.len(),
                    )),
                );
                serde_json::Value::Object(obj)
            })
            .collect();
        let _ = ctx_map.insert(
            "terms".to_string(),
            serde_json::Value::Array(term_entries),
        );

        tmpl.render(serde_json::Value::Object(ctx_map))
            .map(|mut s| {
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                s
            })
            .map_err(|e| SsgError::Io {
                path: PathBuf::from("taxonomy_index.html"),
                source: std::io::Error::other(e.to_string()),
            })
    }

    /// Builds the `{ site: { name, title, language, ... } }` context
    /// shared by every taxonomy page.
    fn base_context(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut site = serde_json::Map::new();
        if let Some(cfg) = self.ctx.config.as_ref() {
            let _ = site.insert(
                "name".to_string(),
                serde_json::Value::String(cfg.site_name.clone()),
            );
            let _ = site.insert(
                "title".to_string(),
                serde_json::Value::String(cfg.site_title.clone()),
            );
            let _ = site.insert(
                "description".to_string(),
                serde_json::Value::String(cfg.site_description.clone()),
            );
            let _ = site.insert(
                "base_url".to_string(),
                serde_json::Value::String(cfg.base_url.clone()),
            );
            let _ = site.insert(
                "language".to_string(),
                serde_json::Value::String(cfg.language.clone()),
            );
        } else {
            // Sensible defaults when the plugin runs without a config
            // (tests, ad-hoc invocations).
            let _ = site.insert(
                "language".to_string(),
                serde_json::Value::String("en".to_string()),
            );
        }

        let mut ctx_map = serde_json::Map::new();
        let _ =
            ctx_map.insert("site".to_string(), serde_json::Value::Object(site));
        ctx_map
    }
}

/// Fallback shim so the module still compiles when the `templates`
/// feature is disabled. The MiniJinja crate is gated on that feature
/// in `Cargo.toml`; the shim falls back to a minimal escaped HTML
/// renderer that still respects `site.language` and per-page metadata.
#[cfg(not(feature = "templates"))]
impl<'a> TaxonomyRenderer<'a> {
    fn new(ctx: &'a PluginContext) -> Self {
        Self { ctx }
    }

    fn render_term_page(
        &self,
        _kind: TaxonomyKind,
        taxonomy_name: &str,
        taxonomy_title: &str,
        term: &str,
        _slug: &str,
        pages: &[(String, String)],
    ) -> Result<String, SsgError> {
        let lang = self.lang();
        let mut out = format!(
            "<!DOCTYPE html>\n<html lang=\"{lang}\">\n<head>\
             <meta charset=\"utf-8\">\
             <title>{taxonomy_title}: {term}</title></head>\n\
             <body>\n<main>\n<h1>{taxonomy_title}: {term}</h1>\n<ul>\n"
        );
        let _ = taxonomy_name; // unused in fallback
        for (title, url) in pages {
            out.push_str(&format!("<li><a href=\"{url}\">{title}</a></li>\n"));
        }
        out.push_str("</ul>\n</main>\n</body>\n</html>\n");
        Ok(out)
    }

    fn render_index_page(
        &self,
        taxonomy_name: &str,
        taxonomy_title: &str,
        sorted_terms: &[(&String, &Vec<(String, String)>)],
    ) -> Result<String, SsgError> {
        let lang = self.lang();
        let mut out = format!(
            "<!DOCTYPE html>\n<html lang=\"{lang}\">\n<head>\
             <meta charset=\"utf-8\">\
             <title>{taxonomy_title}</title></head>\n\
             <body>\n<main>\n<h1>{taxonomy_title}</h1>\n<ul>\n"
        );
        for (term, pages) in sorted_terms {
            let slug = slugify(term);
            out.push_str(&format!(
                "<li><a href=\"/{taxonomy_name}/{slug}/\">{term}</a> ({})</li>\n",
                pages.len()
            ));
        }
        out.push_str("</ul>\n</main>\n</body>\n</html>\n");
        Ok(out)
    }

    fn lang(&self) -> String {
        self.ctx
            .config
            .as_ref()
            .map(|c| c.language.clone())
            .unwrap_or_else(|| "en".to_string())
    }
}

/// Resolves the user's template directory, preferring
/// `<template_dir>/tera/` (the canonical layout) but falling back to
/// `<template_dir>/` if `tera/` is absent.
fn resolve_user_template_dir(ctx: &PluginContext) -> Option<PathBuf> {
    let tera = ctx.template_dir.join("tera");
    if tera.is_dir() {
        return Some(tera);
    }
    if ctx.template_dir.is_dir() {
        return Some(ctx.template_dir.clone());
    }
    None
}

/// Converts a list of (title, url) pairs into JSON page objects with
/// `title` and `url` keys, suitable for template iteration.
fn pages_to_json(pages: &[(String, String)]) -> Vec<serde_json::Value> {
    pages
        .iter()
        .map(|(title, url)| {
            let mut obj = serde_json::Map::new();
            let _ = obj.insert(
                "title".to_string(),
                serde_json::Value::String(title.clone()),
            );
            let _ = obj.insert(
                "url".to_string(),
                serde_json::Value::String(url.clone()),
            );
            serde_json::Value::Object(obj)
        })
        .collect()
}

/// Extracts string terms from a JSON value (array of strings or comma-separated string) into the given map.
fn extract_terms_from_value(
    value: &serde_json::Value,
    map: &mut HashMap<String, Vec<(String, String)>>,
    title: &str,
    url: &str,
    allow_string: bool,
) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                for part in s.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        map.entry(trimmed.to_string())
                            .or_default()
                            .push((title.to_string(), url.to_string()));
                    }
                }
            }
        }
    } else if allow_string {
        if let Some(s) = value.as_str() {
            for part in s.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    map.entry(trimmed.to_string())
                        .or_default()
                        .push((title.to_string(), url.to_string()));
                }
            }
        }
    }
}

/// Collects taxonomy entries (tags, categories, topics) from sidecar JSON files.
fn collect_taxonomy_entries(
    sidecar_dir: &Path,
) -> Result<(TaxonomyMap, TaxonomyMap, TaxonomyMap), SsgError> {
    let sidecars = collect_json_files(sidecar_dir)?;
    let mut tags: TaxonomyMap = HashMap::new();
    let mut categories: TaxonomyMap = HashMap::new();
    let mut topics: TaxonomyMap = HashMap::new();

    for sidecar_path in &sidecars {
        let content =
            fs::read_to_string(sidecar_path).with_path(sidecar_path)?;
        let meta: HashMap<String, serde_json::Value> =
            match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };

        let title = meta
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let rel = sidecar_path
            .strip_prefix(sidecar_dir)
            .unwrap_or(sidecar_path)
            .with_extension("")
            .with_extension("html");
        let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));

        if let Some(tag_arr) = meta.get("tags") {
            extract_terms_from_value(tag_arr, &mut tags, &title, &url, false);
        }
        if let Some(cat_arr) = meta.get("categories") {
            extract_terms_from_value(
                cat_arr,
                &mut categories,
                &title,
                &url,
                false,
            );
        }
        if let Some(topic_arr) = meta.get("topic_clusters") {
            extract_terms_from_value(
                topic_arr,
                &mut topics,
                &title,
                &url,
                true,
            );
        }
    }

    Ok((tags, categories, topics))
}

/// Generates index and term pages for a taxonomy via the template engine.
fn generate_taxonomy_pages(
    site_dir: &Path,
    taxonomy_name: &str,
    taxonomy_title: &str,
    terms: &HashMap<String, Vec<(String, String)>>,
    kind: TaxonomyKind,
    renderer: &TaxonomyRenderer<'_>,
) -> Result<(), SsgError> {
    let tax_dir = site_dir.join(taxonomy_name);
    fs::create_dir_all(&tax_dir).with_path(&tax_dir)?;

    let mut sorted_terms: Vec<_> = terms.iter().collect();
    sorted_terms.sort_by_key(|(name, _)| name.to_lowercase());

    // Per-term pages.
    for (term, pages) in &sorted_terms {
        let slug = slugify(term);
        let term_dir = tax_dir.join(&slug);
        fs::create_dir_all(&term_dir).with_path(&term_dir)?;

        let term_html = renderer.render_term_page(
            kind,
            taxonomy_name,
            taxonomy_title,
            term,
            &slug,
            pages,
        )?;
        let out_file = term_dir.join("index.html");
        fs::write(&out_file, term_html).with_path(&out_file)?;
    }

    // Taxonomy index page.
    let index_html = renderer.render_index_page(
        taxonomy_name,
        taxonomy_title,
        &sorted_terms,
    )?;
    let out_index = tax_dir.join("index.html");
    fs::write(&out_index, index_html).with_path(&out_index)?;

    Ok(())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn collect_json_files(dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    crate::walk::walk_files(dir, "json")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::test_support::init_logger;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    /// Builds a fresh temp dir layout: `<root>/site`, `<root>/build/.meta`
    /// and a `PluginContext`.
    fn make_layout() -> (TempDir, PathBuf, PathBuf, PluginContext) {
        init_logger();
        let dir = tempdir().expect("create tempdir");
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        let meta = build.join(".meta");
        fs::create_dir_all(&site).expect("mkdir site");
        fs::create_dir_all(&meta).expect("mkdir meta");
        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        (dir, site, meta, ctx)
    }

    // -------------------------------------------------------------------
    // slugify — table-driven coverage of the character classes
    // -------------------------------------------------------------------

    #[test]
    fn slugify_table_driven_inputs_produce_expected_slugs() {
        let cases: &[(&str, &str)] = &[
            // basic — alphanumeric + space
            ("Rust Programming", "rust-programming"),
            // punctuation collapsing
            ("C++", "c"),
            ("hello world!", "hello-world"),
            // multiple consecutive non-alphanumerics collapse to one dash
            ("a !! b", "a-b"),
            ("a___b", "a-b"),
            // leading and trailing punctuation are stripped
            ("---rust---", "rust"),
            ("!!!hello!!!", "hello"),
            // unicode letters survive (alphanumeric)
            ("café", "café"),
            // pure punctuation collapses to empty
            ("!!!", ""),
            // already-slug stays the same
            ("rust-web", "rust-web"),
            // mixed digits and letters
            ("Rust 2024", "rust-2024"),
            // empty input
            ("", ""),
        ];
        for &(input, expected) in cases {
            assert_eq!(
                slugify(input),
                expected,
                "slugify({input:?}) should be {expected:?}"
            );
        }
    }

    #[test]
    fn slugify_lowercases_uppercase_input() {
        assert_eq!(slugify("RUST"), "rust");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }

    // -------------------------------------------------------------------
    // capitalize — table-driven (covers None/Some(_) match arms)
    // -------------------------------------------------------------------

    #[test]
    fn capitalize_table_driven_inputs_produce_expected_output() {
        let cases: &[(&str, &str)] = &[
            ("", ""),
            ("a", "A"),
            ("tags", "Tags"),
            ("categories", "Categories"),
            ("Tags", "Tags"),
            ("1", "1"),
        ];
        for &(input, expected) in cases {
            assert_eq!(
                capitalize(input),
                expected,
                "capitalize({input:?}) should be {expected:?}"
            );
        }
    }

    // -------------------------------------------------------------------
    // TaxonomyPlugin — derive surface
    // -------------------------------------------------------------------

    #[test]
    fn taxonomy_plugin_is_copy_after_move() {
        let plugin = TaxonomyPlugin;
        let _copy = plugin;
        assert_eq!(plugin.name(), "taxonomy");
    }

    #[test]
    fn name_returns_static_taxonomy_identifier() {
        assert_eq!(TaxonomyPlugin.name(), "taxonomy");
    }

    // -------------------------------------------------------------------
    // after_compile — early-return paths
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_missing_meta_dir_returns_ok_without_writing() {
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        fs::create_dir_all(&site).expect("mkdir site");
        fs::create_dir_all(&build).expect("mkdir build");
        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());

        TaxonomyPlugin
            .after_compile(&ctx)
            .expect("missing meta is fine");
        assert!(!site.join("tags").exists());
        assert!(!site.join("categories").exists());
    }

    #[test]
    fn after_compile_empty_meta_dir_returns_ok_without_writing() {
        let (_tmp, site, _meta, ctx) = make_layout();
        TaxonomyPlugin
            .after_compile(&ctx)
            .expect("empty meta is fine");
        assert!(!site.join("tags").exists());
        assert!(!site.join("categories").exists());
    }

    #[test]
    fn after_compile_pages_without_taxonomies_emit_no_output() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(meta.join("about.meta.json"), r#"{"title": "About"}"#)
            .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("tags").exists());
        assert!(!site.join("categories").exists());
    }

    // -------------------------------------------------------------------
    // after_compile — sidecar parsing fallbacks
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_skips_invalid_json_sidecars() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(meta.join("broken.meta.json"), "{not valid").unwrap();
        fs::write(
            meta.join("good.meta.json"),
            r#"{"title": "Good", "tags": ["rust"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("tags/rust/index.html").exists());
    }

    #[test]
    fn after_compile_missing_title_falls_back_to_untitled() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(meta.join("notitle.meta.json"), r#"{"tags": ["rust"]}"#)
            .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        let html =
            fs::read_to_string(site.join("tags/rust/index.html")).unwrap();
        assert!(html.contains("Untitled"));
    }

    #[test]
    fn after_compile_ignores_non_string_tag_values() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("mixed.meta.json"),
            r#"{"title": "Mixed", "tags": ["rust", 42, null, "web", {"x":1}]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("tags/rust/index.html").exists());
        assert!(site.join("tags/web/index.html").exists());
    }

    #[test]
    fn after_compile_ignores_non_array_categories_field() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("badcats.meta.json"),
            r#"{"title": "BadCats", "categories": "not-an-array"}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("categories").exists());
    }

    #[test]
    fn after_compile_ignores_non_string_category_values() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("mixed-cats.meta.json"),
            r#"{"title": "Mixed", "categories": ["blog", 42, null, {"x":1}]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("categories/blog/index.html").exists());
    }

    #[test]
    fn after_compile_ignores_non_array_tags_field() {
        let (_tmp, site, _meta_dir, ctx) = make_layout();
        let meta_dir = ctx.build_dir.join(".meta");
        fs::write(
            meta_dir.join("badtype.meta.json"),
            r#"{"title": "BadType", "tags": "not-an-array"}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("tags").exists());
    }

    // -------------------------------------------------------------------
    // after_compile — tags and categories generation (built-in templates)
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_generates_index_and_term_pages_for_tags() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p1.meta.json"),
            r#"{"title": "P1", "tags": ["rust", "web"]}"#,
        )
        .unwrap();
        fs::write(
            meta.join("p2.meta.json"),
            r#"{"title": "P2", "tags": ["rust"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();

        assert!(site.join("tags/index.html").exists());
        assert!(site.join("tags/rust/index.html").exists());
        assert!(site.join("tags/web/index.html").exists());

        let rust =
            fs::read_to_string(site.join("tags/rust/index.html")).unwrap();
        assert!(rust.contains("P1"));
        assert!(rust.contains("P2"));

        let web = fs::read_to_string(site.join("tags/web/index.html")).unwrap();
        assert!(web.contains("P1"));
        assert!(!web.contains("P2"));
    }

    #[test]
    fn after_compile_generates_index_and_term_pages_for_categories() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p1.meta.json"),
            r#"{"title": "P1", "categories": ["tutorials"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("categories/index.html").exists());
        assert!(site.join("categories/tutorials/index.html").exists());
    }

    #[test]
    fn after_compile_generates_index_and_term_pages_for_topics() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p1.meta.json"),
            r#"{"title": "P1", "topic_clusters": "cloud-native-banking"}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("topics/index.html").exists());
        assert!(site.join("topics/cloud-native-banking/index.html").exists());
    }

    #[test]
    fn after_compile_index_shows_page_count_per_term() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("a.meta.json"),
            r#"{"title": "A", "tags": ["rust"]}"#,
        )
        .unwrap();
        fs::write(
            meta.join("b.meta.json"),
            r#"{"title": "B", "tags": ["rust"]}"#,
        )
        .unwrap();
        fs::write(
            meta.join("c.meta.json"),
            r#"{"title": "C", "tags": ["rust", "web"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        let index = fs::read_to_string(site.join("tags/index.html")).unwrap();
        assert!(index.contains("(3)"), "rust should have 3 posts:\n{index}");
        assert!(index.contains("(1)"), "web should have 1 post:\n{index}");
    }

    #[test]
    fn after_compile_index_lists_terms_alphabetically_case_insensitive() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p.meta.json"),
            r#"{"title": "P", "tags": ["banana", "Apple", "cherry"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        let index = fs::read_to_string(site.join("tags/index.html")).unwrap();
        let apple = index.find("Apple").expect("Apple in index");
        let banana = index.find("banana").expect("banana in index");
        let cherry = index.find("cherry").expect("cherry in index");
        assert!(apple < banana, "Apple should sort before banana");
        assert!(banana < cherry, "banana should sort before cherry");
    }

    #[test]
    fn after_compile_tags_and_categories_coexist_independently() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p.meta.json"),
            r#"{"title": "P", "tags": ["rust"], "categories": ["tutorials"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("tags/rust/index.html").exists());
        assert!(site.join("categories/tutorials/index.html").exists());
    }

    #[test]
    fn after_compile_idempotent_overwrites_existing_pages() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p.meta.json"),
            r#"{"title": "P", "tags": ["rust"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).expect("first run");
        TaxonomyPlugin.after_compile(&ctx).expect("second run");
        assert!(site.join("tags/rust/index.html").exists());
    }

    #[test]
    fn after_compile_emits_doctype_lang_charset_in_index() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("p.meta.json"),
            r#"{"title": "P", "tags": ["rust"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        let html = fs::read_to_string(site.join("tags/index.html")).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        // Built-in base.html renders `lang="en"` when no config supplies one.
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("Tags"));
    }

    #[test]
    fn after_compile_term_page_links_back_to_source_url() {
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("hello.meta.json"),
            r#"{"title": "Hello", "tags": ["rust"]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        let html =
            fs::read_to_string(site.join("tags/rust/index.html")).unwrap();
        assert!(
            html.contains(r#"href="/hello.html""#),
            "term page should link back to /hello.html:\n{html}"
        );
    }

    // -------------------------------------------------------------------
    // collect_json_files — recursion + filtering
    // -------------------------------------------------------------------

    #[test]
    fn collect_json_files_returns_empty_for_missing_directory() {
        let dir = tempdir().expect("tempdir");
        let result = collect_json_files(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn collect_json_files_filters_non_json_extensions() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("a.json"), "{}").unwrap();
        fs::write(dir.path().join("b.txt"), "x").unwrap();
        fs::write(dir.path().join("c"), "x").unwrap();

        let result = collect_json_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn collect_json_files_recurses_into_nested_subdirectories() {
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("top.json"), "{}").unwrap();
        fs::write(nested.join("deep.json"), "{}").unwrap();

        let result = collect_json_files(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn collect_json_files_returns_results_sorted() {
        let dir = tempdir().expect("tempdir");
        for name in ["zebra.json", "apple.json", "mango.json"] {
            fs::write(dir.path().join(name), "{}").unwrap();
        }
        let result = collect_json_files(dir.path()).unwrap();
        let names: Vec<_> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.json", "mango.json", "zebra.json"]);
    }

    // -------------------------------------------------------------------
    // TaxonomyTerm — public type smoke test
    // -------------------------------------------------------------------

    #[test]
    fn taxonomy_term_can_be_constructed_and_cloned() {
        let term = TaxonomyTerm {
            name: "Rust".to_string(),
            slug: "rust".to_string(),
            pages: vec![("Hello".to_string(), "/hello.html".to_string())],
        };
        let copy = term;
        assert_eq!(copy.name, "Rust");
        assert_eq!(copy.slug, "rust");
        assert_eq!(copy.pages.len(), 1);
    }

    #[test]
    fn test_generate_taxonomy_pages_invalid_dir_returns_io_error() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file");
        fs::write(&file_path, "").unwrap();

        let mut terms = HashMap::new();
        let _ = terms.insert(
            "rust".to_string(),
            vec![("Title".to_string(), "/hello.html".to_string())],
        );

        let ctx =
            PluginContext::new(tmp.path(), tmp.path(), tmp.path(), tmp.path());
        let renderer = TaxonomyRenderer::new(&ctx);
        let res = generate_taxonomy_pages(
            &file_path,
            "tags",
            "Tags",
            &terms,
            TaxonomyKind::Tag,
            &renderer,
        );
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, SsgError::Io { .. }));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// `slugify` output must contain only (Unicode) alphanumerics and
        /// hyphens, with no leading/trailing/consecutive hyphens.
        ///
        /// NOTE: proptest discovered that `slugify` preserves Unicode
        /// alphanumeric characters (e.g. `𐞀`). This is intentional —
        /// the existing test suite asserts `"café"` -> `"café"`.
        #[test]
        fn slugify_valid_chars(input in "\\PC*") {
            let slug = slugify(&input);
            for ch in slug.chars() {
                prop_assert!(
                    ch.is_alphanumeric() || ch == '-',
                    "unexpected char {:?} in slug {:?}", ch, slug,
                );
            }
            prop_assert!(
                !slug.starts_with('-'),
                "slug must not start with hyphen: {:?}", slug,
            );
            prop_assert!(
                !slug.ends_with('-'),
                "slug must not end with hyphen: {:?}", slug,
            );
            prop_assert!(
                !slug.contains("--"),
                "slug must not contain consecutive hyphens: {:?}", slug,
            );
        }
    }
}
