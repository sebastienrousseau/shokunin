// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Taxonomy generation plugin.
//!
//! Reads `tags` and `categories` from frontmatter sidecars and
//! generates index pages for each taxonomy term.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
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

/// Plugin that generates taxonomy index pages for tags and categories.
///
/// Runs in `after_compile`. Reads `.meta.json` sidecars to find
/// `tags` and `categories` arrays, then generates:
/// - `/tags/index.html` — list of all tags with page counts
/// - `/tags/{slug}/index.html` — list of pages for each tag
/// - `/categories/index.html` and `/categories/{slug}/index.html`
#[derive(Debug, Clone, Copy)]
pub struct TaxonomyPlugin;

impl Plugin for TaxonomyPlugin {
    fn name(&self) -> &'static str {
        "taxonomy"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let sidecar_dir = ctx.build_dir.join(".meta");
        if !sidecar_dir.exists() {
            return Ok(());
        }

        let (tags, categories) = collect_taxonomy_entries(&sidecar_dir)?;

        if !tags.is_empty() {
            generate_taxonomy_pages(&ctx.site_dir, "tags", &tags)?;
            log::info!("[taxonomy] Generated {} tag page(s)", tags.len());
        }

        if !categories.is_empty() {
            generate_taxonomy_pages(&ctx.site_dir, "categories", &categories)?;
            log::info!(
                "[taxonomy] Generated {} category page(s)",
                categories.len()
            );
        }

        Ok(())
    }
}

/// Extracts string terms from a JSON array value into the given map.
fn extract_terms_from_array(
    value: &serde_json::Value,
    map: &mut HashMap<String, Vec<(String, String)>>,
    title: &str,
    url: &str,
) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                map.entry(s.to_string())
                    .or_default()
                    .push((title.to_string(), url.to_string()));
            }
        }
    }
}

/// Collects taxonomy entries (tags and categories) from sidecar JSON files.
fn collect_taxonomy_entries(
    sidecar_dir: &Path,
) -> Result<(TaxonomyMap, TaxonomyMap)> {
    let sidecars = collect_json_files(sidecar_dir)?;
    let mut tags: TaxonomyMap = HashMap::new();
    let mut categories: TaxonomyMap = HashMap::new();

    for sidecar_path in &sidecars {
        let content = fs::read_to_string(sidecar_path)?;
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
            extract_terms_from_array(tag_arr, &mut tags, &title, &url);
        }
        if let Some(cat_arr) = meta.get("categories") {
            extract_terms_from_array(cat_arr, &mut categories, &title, &url);
        }
    }

    Ok((tags, categories))
}

/// Generates index and term pages for a taxonomy.
fn generate_taxonomy_pages(
    site_dir: &Path,
    taxonomy_name: &str,
    terms: &HashMap<String, Vec<(String, String)>>,
) -> Result<()> {
    let tax_dir = site_dir.join(taxonomy_name);
    fs::create_dir_all(&tax_dir)?;

    // Generate index page listing all terms
    let mut index_html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
         <meta charset=\"utf-8\">\
         <title>{}</title></head>\n<body>\n<main>\n\
         <h1>{}</h1>\n<ul>\n",
        capitalize(taxonomy_name),
        capitalize(taxonomy_name),
    );

    let mut sorted_terms: Vec<_> = terms.iter().collect();
    sorted_terms.sort_by_key(|(name, _)| name.to_lowercase());

    for (term, pages) in &sorted_terms {
        let slug = slugify(term);
        index_html.push_str(&format!(
            "<li><a href=\"/{}/{}/\">{}</a> ({})</li>\n",
            taxonomy_name,
            slug,
            term,
            pages.len()
        ));

        // Generate individual term page
        let term_dir = tax_dir.join(&slug);
        fs::create_dir_all(&term_dir)?;

        let mut term_html = format!(
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <title>{term}</title></head>\n<body>\n<main>\n\
             <h1>{term}</h1>\n<ul>\n",
        );
        for (title, url) in *pages {
            term_html
                .push_str(&format!("<li><a href=\"{url}\">{title}</a></li>\n"));
        }
        term_html.push_str("</ul>\n</main>\n</body>\n</html>\n");

        fs::write(term_dir.join("index.html"), term_html)?;
    }

    index_html.push_str("</ul>\n</main>\n</body>\n</html>\n");
    fs::write(tax_dir.join("index.html"), index_html)?;

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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn collect_json_files(dir: &Path) -> Result<Vec<PathBuf>> {
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
        // Guards the `to_lowercase` step at line 180 — case
        // normalization is the entire reason slug routing is stable.
        assert_eq!(slugify("RUST"), "rust");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }

    // -------------------------------------------------------------------
    // capitalize — table-driven (covers None/Some(_) match arms)
    // -------------------------------------------------------------------

    #[test]
    fn capitalize_table_driven_inputs_produce_expected_output() {
        let cases: &[(&str, &str)] = &[
            // empty -> empty (None arm at line 193)
            ("", ""),
            // ASCII single char
            ("a", "A"),
            // word
            ("tags", "Tags"),
            ("categories", "Categories"),
            // already capitalized
            ("Tags", "Tags"),
            // single non-letter passes through
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
        // Guards the `Copy` derive added in v0.0.34.
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
        // A page with neither `tags` nor `categories` arrays must
        // not trigger the `!tags.is_empty()` / `!categories.is_empty()`
        // branches at lines 105 and 110.
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
        // The `Err(_) => continue` arm at line 59 must not poison
        // the build. A valid sibling sidecar should still produce
        // taxonomy pages.
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
        // The `if let Some(t) = tag.as_str()` filter at line 80 must
        // skip integers, objects, etc., without erroring.
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(
            meta.join("mixed.meta.json"),
            r#"{"title": "Mixed", "tags": ["rust", 42, null, "web", {"x":1}]}"#,
        )
        .unwrap();

        TaxonomyPlugin.after_compile(&ctx).unwrap();
        // Only the two string entries should produce term pages.
        assert!(site.join("tags/rust/index.html").exists());
        assert!(site.join("tags/web/index.html").exists());
    }

    #[test]
    fn after_compile_ignores_non_array_categories_field() {
        // Symmetric to `after_compile_ignores_non_array_tags_field`:
        // a `categories: "tutorials"` (string, not array) must take
        // the `as_array()` None branch without panicking. Closes
        // line 100 (the closing brace of the inner if-let).
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
        // The `if let Some(c) = cat.as_str()` filter at line 93
        // must skip ints/objects/nulls.
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
        // The `if let Some(arr) = tag_arr.as_array()` guard at line 78
        // must reject string/object values silently.
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
    // after_compile — tags and categories generation
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
    fn after_compile_index_shows_page_count_per_term() {
        // The `({})` count rendering at line 151 must reflect the
        // actual number of pages tagged with that term.
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
        // The sort key at line 142 is `name.to_lowercase()`, so
        // `Apple` must precede `banana` despite uppercase coming
        // earlier in ASCII.
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
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<h1>Tags</h1>"));
    }

    #[test]
    fn after_compile_term_page_links_back_to_source_url() {
        // The url derived from the sidecar path (line 74) must be
        // present in the term-page list item.
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
}
