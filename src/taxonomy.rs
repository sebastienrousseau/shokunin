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
#[derive(Debug)]
pub struct TaxonomyPlugin;

impl Plugin for TaxonomyPlugin {
    fn name(&self) -> &str {
        "taxonomy"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let sidecar_dir = ctx.build_dir.join(".meta");
        if !sidecar_dir.exists() {
            return Ok(());
        }

        let sidecars = collect_json_files(&sidecar_dir)?;
        let mut tags: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut categories: HashMap<String, Vec<(String, String)>> =
            HashMap::new();

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

            // Derive URL from sidecar path
            let rel = sidecar_path
                .strip_prefix(&sidecar_dir)
                .unwrap_or(sidecar_path)
                .with_extension("")
                .with_extension("html");
            let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));

            // Extract tags
            if let Some(tag_arr) = meta.get("tags") {
                if let Some(arr) = tag_arr.as_array() {
                    for tag in arr {
                        if let Some(t) = tag.as_str() {
                            tags.entry(t.to_string())
                                .or_default()
                                .push((title.clone(), url.clone()));
                        }
                    }
                }
            }

            // Extract categories
            if let Some(cat_arr) = meta.get("categories") {
                if let Some(arr) = cat_arr.as_array() {
                    for cat in arr {
                        if let Some(c) = cat.as_str() {
                            categories
                                .entry(c.to_string())
                                .or_default()
                                .push((title.clone(), url.clone()));
                        }
                    }
                }
            }
        }

        // Generate taxonomy pages
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
             <title>{}</title></head>\n<body>\n<main>\n\
             <h1>{}</h1>\n<ul>\n",
            term, term,
        );
        for (title, url) in *pages {
            term_html.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                url, title
            ));
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
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "json") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Rust Programming"), "rust-programming");
        assert_eq!(slugify("C++"), "c");
        assert_eq!(slugify("hello world!"), "hello-world");
    }

    #[test]
    fn test_taxonomy_generation() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        let meta_dir = build.join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&meta_dir).unwrap();

        // Write sidecar with tags
        fs::write(
            meta_dir.join("post1.meta.json"),
            r#"{"title": "Post 1", "tags": ["rust", "web"]}"#,
        )
        .unwrap();
        fs::write(
            meta_dir.join("post2.meta.json"),
            r#"{"title": "Post 2", "tags": ["rust"], "categories": ["tutorials"]}"#,
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        TaxonomyPlugin.after_compile(&ctx).unwrap();

        // Check tag pages
        assert!(site.join("tags/index.html").exists());
        assert!(site.join("tags/rust/index.html").exists());
        assert!(site.join("tags/web/index.html").exists());

        // Check category pages
        assert!(site.join("categories/index.html").exists());
        assert!(site.join("categories/tutorials/index.html").exists());

        // Verify content
        let rust_page =
            fs::read_to_string(site.join("tags/rust/index.html")).unwrap();
        assert!(rust_page.contains("Post 1"));
        assert!(rust_page.contains("Post 2"));

        let tags_index =
            fs::read_to_string(site.join("tags/index.html")).unwrap();
        assert!(tags_index.contains("rust"));
        assert!(tags_index.contains("(2)")); // rust has 2 posts
    }

    #[test]
    fn test_no_sidecars_noop() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&build).unwrap();
        // No .meta dir

        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        TaxonomyPlugin.after_compile(&ctx).unwrap();
        // Should succeed with no output
        assert!(!site.join("tags").exists());
    }
}
