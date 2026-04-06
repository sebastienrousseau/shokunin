// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Pagination plugin.
//!
//! Generates paginated index pages (`/page/2/`, `/page/3/`, etc.)
//! from frontmatter sidecars when `paginate` is specified.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Default number of items per page.
const DEFAULT_PER_PAGE: usize = 10;

/// Page metadata for pagination.
#[derive(Debug, Clone)]
struct PageEntry {
    title: String,
    url: String,
    date: String,
}

/// Plugin that generates paginated listing pages.
///
/// Runs in `after_compile`. Reads `.meta.json` sidecars, collects
/// pages with dates, sorts by date descending, and generates
/// `/page/N/index.html` files.
#[derive(Debug, Clone, Copy)]
pub struct PaginationPlugin {
    per_page: usize,
}

impl Default for PaginationPlugin {
    fn default() -> Self {
        Self {
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl PaginationPlugin {
    /// Creates a pagination plugin with a custom page size.
    #[must_use]
    pub fn with_per_page(per_page: usize) -> Self {
        Self {
            per_page: per_page.max(1),
        }
    }
}

impl Plugin for PaginationPlugin {
    fn name(&self) -> &'static str {
        "pagination"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let sidecar_dir = ctx.build_dir.join(".meta");
        if !sidecar_dir.exists() {
            return Ok(());
        }

        // Collect all pages with dates from sidecars
        let sidecars = collect_json_files(&sidecar_dir)?;
        let mut entries = Vec::new();

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
            let date = meta
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Only paginate pages that have dates (blog posts)
            if date.is_empty() {
                continue;
            }

            let rel = sidecar_path
                .strip_prefix(&sidecar_dir)
                .unwrap_or(sidecar_path)
                .with_extension("")
                .with_extension("html");
            let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));

            entries.push(PageEntry { title, url, date });
        }

        if entries.is_empty() {
            return Ok(());
        }

        // Sort by date descending
        entries.sort_by(|a, b| b.date.cmp(&a.date));

        let total_pages = entries.len().div_ceil(self.per_page);

        if total_pages <= 1 {
            return Ok(());
        }

        // Generate /page/N/index.html for pages 2..=total
        let page_dir = ctx.site_dir.join("page");

        for page_num in 2..=total_pages {
            let start = (page_num - 1) * self.per_page;
            let end = (start + self.per_page).min(entries.len());
            let page_entries = &entries[start..end];

            let dir = page_dir.join(page_num.to_string());
            fs::create_dir_all(&dir)?;

            let prev_url = if page_num == 2 {
                "/".to_string()
            } else {
                format!("/page/{}/", page_num - 1)
            };
            let next_url = if page_num < total_pages {
                Some(format!("/page/{}/", page_num + 1))
            } else {
                None
            };

            let mut html = format!(
                "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
                 <meta charset=\"utf-8\">\
                 <title>Page {page_num} of {total_pages}</title></head>\n\
                 <body>\n<main>\n\
                 <h1>Page {page_num} of {total_pages}</h1>\n<ul>\n",
            );

            for entry in page_entries {
                html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a> <time>{}</time></li>\n",
                    entry.url, entry.title, entry.date
                ));
            }

            html.push_str("</ul>\n<nav aria-label=\"Pagination\">\n");
            html.push_str(&format!(
                "<a href=\"{prev_url}\" rel=\"prev\">&larr; Previous</a>\n"
            ));
            if let Some(next) = &next_url {
                html.push_str(&format!(
                    "<a href=\"{next}\" rel=\"next\">Next &rarr;</a>\n"
                ));
            }
            html.push_str("</nav>\n</main>\n</body>\n</html>\n");

            fs::write(dir.join("index.html"), html)?;
        }

        log::info!(
            "[pagination] Generated {} page(s) ({} entries, {} per page)",
            total_pages - 1,
            entries.len(),
            self.per_page
        );
        Ok(())
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
    fn test_pagination_generates_pages() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        let meta_dir = build.join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&meta_dir).unwrap();

        // Create 5 dated posts with per_page=2 → 3 pages (page 1 = main, 2, 3)
        for i in 1..=5 {
            fs::write(
                meta_dir.join(format!("post{}.meta.json", i)),
                format!(
                    r#"{{"title": "Post {}", "date": "2026-01-{:02}"}}"#,
                    i, i
                ),
            )
            .unwrap();
        }

        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        PaginationPlugin::with_per_page(2)
            .after_compile(&ctx)
            .unwrap();

        assert!(site.join("page/2/index.html").exists());
        assert!(site.join("page/3/index.html").exists());
        assert!(!site.join("page/4/index.html").exists());

        let page2 = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(page2.contains("Page 2 of 3"));
        assert!(page2.contains("Previous"));
        assert!(page2.contains("Next"));
    }

    #[test]
    fn test_pagination_no_dated_pages() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        let meta_dir = build.join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&meta_dir).unwrap();

        fs::write(meta_dir.join("about.meta.json"), r#"{"title": "About"}"#)
            .unwrap();

        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        assert!(!site.join("page").exists());
    }

    #[test]
    fn test_pagination_single_page_no_output() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        let meta_dir = build.join(".meta");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&meta_dir).unwrap();

        // Only 2 posts with per_page=10 → 1 page → no pagination needed
        for i in 1..=2 {
            fs::write(
                meta_dir.join(format!("post{}.meta.json", i)),
                format!(
                    r#"{{"title": "Post {}", "date": "2026-01-{:02}"}}"#,
                    i, i
                ),
            )
            .unwrap();
        }

        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        assert!(!site.join("page").exists());
    }
}
