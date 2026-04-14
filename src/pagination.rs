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

        let mut entries = collect_page_entries(&sidecar_dir)?;
        if entries.is_empty() {
            return Ok(());
        }

        entries.sort_by(|a, b| b.date.cmp(&a.date));

        let total_pages = entries.len().div_ceil(self.per_page);
        if total_pages <= 1 {
            return Ok(());
        }

        let page_dir = ctx.site_dir.join("page");
        for page_num in 2..=total_pages {
            let start = (page_num - 1) * self.per_page;
            let end = (start + self.per_page).min(entries.len());
            let page_entries = &entries[start..end];

            write_pagination_page(
                &page_dir,
                page_num,
                total_pages,
                page_entries,
            )?;
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

/// Collects page entries with dates from sidecar JSON files.
fn collect_page_entries(sidecar_dir: &Path) -> Result<Vec<PageEntry>> {
    let sidecars = collect_json_files(sidecar_dir)?;
    let mut entries = Vec::new();

    for sidecar_path in &sidecars {
        if let Some(entry) = parse_page_entry(sidecar_path, sidecar_dir) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parses a single sidecar JSON file into a `PageEntry`, if it has a date.
fn parse_page_entry(
    sidecar_path: &Path,
    sidecar_dir: &Path,
) -> Option<PageEntry> {
    let content = fs::read_to_string(sidecar_path).ok()?;
    let meta: HashMap<String, serde_json::Value> =
        serde_json::from_str(&content).ok()?;

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

    if date.is_empty() {
        return None;
    }

    let rel = sidecar_path
        .strip_prefix(sidecar_dir)
        .unwrap_or(sidecar_path)
        .with_extension("")
        .with_extension("html");
    let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));

    Some(PageEntry { title, url, date })
}

/// Writes a single pagination page to disk.
fn write_pagination_page(
    page_dir: &Path,
    page_num: usize,
    total_pages: usize,
    page_entries: &[PageEntry],
) -> Result<()> {
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
    Ok(())
}

fn collect_json_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::init_logger;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    /// Builds a fresh temp dir layout: `<root>/site`, `<root>/build/.meta`,
    /// and a `PluginContext` pointing at it. Returns the temp dir guard
    /// (must outlive the test), the site path, the meta sidecar path,
    /// and the context.
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

    /// Writes a sidecar JSON file shaped `{"title": ..., "date": ...}`.
    fn write_sidecar(meta: &Path, name: &str, title: &str, date: &str) {
        let json = if date.is_empty() {
            format!(r#"{{"title": "{title}"}}"#)
        } else {
            format!(r#"{{"title": "{title}", "date": "{date}"}}"#)
        };
        fs::write(meta.join(format!("{name}.meta.json")), json)
            .expect("write sidecar");
    }

    /// Writes `n` dated posts numbered 1..=n with monotonically
    /// increasing dates so sort order is well-defined.
    fn write_n_dated_posts(meta: &Path, n: usize) {
        for i in 1..=n {
            write_sidecar(
                meta,
                &format!("post{i:03}"),
                &format!("Post {i}"),
                &format!("2026-01-{i:02}"),
            );
        }
    }

    // -------------------------------------------------------------------
    // Constructor + derive surface
    // -------------------------------------------------------------------

    #[test]
    fn default_uses_default_per_page_constant() {
        // The Default impl is the public ergonomic — assert it matches
        // the documented constant rather than a magic number, so the
        // test stays correct if DEFAULT_PER_PAGE is ever retuned.
        let plugin = PaginationPlugin::default();
        assert_eq!(plugin.per_page, DEFAULT_PER_PAGE);
    }

    #[test]
    fn with_per_page_stores_supplied_value() {
        let plugin = PaginationPlugin::with_per_page(7);
        assert_eq!(plugin.per_page, 7);
    }

    #[test]
    fn with_per_page_zero_clamps_to_one() {
        // Zero would cause a divide-by-zero in `div_ceil`. The
        // constructor must clamp it.
        let plugin = PaginationPlugin::with_per_page(0);
        assert_eq!(plugin.per_page, 1);
    }

    #[test]
    fn with_per_page_one_is_valid_lower_bound() {
        let plugin = PaginationPlugin::with_per_page(1);
        assert_eq!(plugin.per_page, 1);
    }

    #[test]
    fn with_per_page_table_driven_values() {
        // Table-driven sanity check across a spread of valid sizes.
        let cases: &[(usize, usize)] = &[
            (1, 1),
            (5, 5),
            (10, 10),
            (100, 100),
            (usize::MAX, usize::MAX),
        ];
        for &(input, expected) in cases {
            let plugin = PaginationPlugin::with_per_page(input);
            assert_eq!(
                plugin.per_page, expected,
                "with_per_page({input}) should store {expected}"
            );
        }
    }

    #[test]
    fn pagination_plugin_is_copy_after_move() {
        // Guards the `Copy` derive added in v0.0.34.
        let plugin = PaginationPlugin::with_per_page(3);
        let _copy = plugin;
        assert_eq!(plugin.per_page, 3);
    }

    #[test]
    fn name_returns_static_pagination_identifier() {
        let plugin = PaginationPlugin::default();
        assert_eq!(plugin.name(), "pagination");
    }

    // -------------------------------------------------------------------
    // after_compile — early-return paths
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_missing_meta_dir_returns_ok_without_writing() {
        // No `.meta` directory under build/ — must short-circuit, not
        // error, and must not create the page/ directory.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let build = dir.path().join("build");
        fs::create_dir_all(&site).expect("mkdir site");
        fs::create_dir_all(&build).expect("mkdir build");
        let ctx = PluginContext::new(dir.path(), &build, &site, dir.path());

        PaginationPlugin::default()
            .after_compile(&ctx)
            .expect("missing meta dir is not an error");

        assert!(!site.join("page").exists());
    }

    #[test]
    fn after_compile_empty_meta_dir_returns_ok_without_writing() {
        let (_tmp, site, _meta, ctx) = make_layout();
        PaginationPlugin::default()
            .after_compile(&ctx)
            .expect("empty meta is fine");
        assert!(!site.join("page").exists());
    }

    #[test]
    fn after_compile_only_undated_pages_returns_ok_without_writing() {
        // Pages without `date` are skipped — see line 91. Only undated
        // entries means `entries.is_empty()` short-circuit at line 105.
        let (_tmp, site, meta, ctx) = make_layout();
        write_sidecar(&meta, "about", "About", "");
        write_sidecar(&meta, "contact", "Contact", "");

        PaginationPlugin::default().after_compile(&ctx).unwrap();
        assert!(!site.join("page").exists());
    }

    #[test]
    fn after_compile_single_page_skips_pagination() {
        // 5 dated posts at default per_page=10 → 1 page total → no
        // /page/N/ directories produced (line 114 `total_pages <= 1`).
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 5);

        PaginationPlugin::default().after_compile(&ctx).unwrap();
        assert!(!site.join("page").exists());
    }

    // -------------------------------------------------------------------
    // after_compile — sidecar parsing fallbacks
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_skips_invalid_json_sidecars() {
        // The JSON parser error branch at line 76 must not propagate —
        // bad sidecars are silently skipped so a single corrupt file
        // can't poison the whole build.
        let (_tmp, site, meta, ctx) = make_layout();
        fs::write(meta.join("broken.meta.json"), "{not valid json").unwrap();
        // Add 11 valid posts so we still cross the pagination threshold
        // (default per_page=10 → 2 pages).
        write_n_dated_posts(&meta, 11);

        PaginationPlugin::default()
            .after_compile(&ctx)
            .expect("broken sidecar must not error");
        assert!(site.join("page/2/index.html").exists());
    }

    #[test]
    fn after_compile_missing_title_defaults_to_untitled() {
        // Pages without a `title` field but with a `date` are still
        // paginated; the title falls back to "Untitled" (line 82).
        let (_tmp, site, meta, ctx) = make_layout();
        // 11 entries with NO title field → "Untitled" fallback used.
        for i in 1..=11 {
            fs::write(
                meta.join(format!("post{i}.meta.json")),
                format!(r#"{{"date": "2026-01-{i:02}"}}"#),
            )
            .unwrap();
        }

        PaginationPlugin::default().after_compile(&ctx).unwrap();
        let page2 = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(
            page2.contains("Untitled"),
            "missing title must fall back to \"Untitled\":\n{page2}"
        );
    }

    #[test]
    fn after_compile_skips_pages_with_empty_date_string() {
        // A `date` field present but empty must be treated the same
        // as a missing date (line 91-93).
        let (_tmp, site, meta, ctx) = make_layout();
        write_sidecar(&meta, "draft", "Draft", ""); // empty date branch
        write_n_dated_posts(&meta, 11);

        PaginationPlugin::default().after_compile(&ctx).unwrap();
        // Only the 11 dated posts paginate; the empty-date entry is
        // ignored, so we get exactly one /page/2/ (11 → 2 pages).
        assert!(site.join("page/2/index.html").exists());
        assert!(!site.join("page/3/index.html").exists());
    }

    // -------------------------------------------------------------------
    // after_compile — page slicing arithmetic
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_exact_multiple_yields_full_pages() {
        // 10 posts at per_page=5 → exactly 2 full pages, no remainder.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 10);

        PaginationPlugin::with_per_page(5)
            .after_compile(&ctx)
            .unwrap();

        let page2 = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        // Page 2 of 2: should contain exactly 5 list items.
        let li_count = page2.matches("<li>").count();
        assert_eq!(li_count, 5, "page 2 should have 5 entries:\n{page2}");
        assert!(!site.join("page/3/index.html").exists());
    }

    #[test]
    fn after_compile_non_multiple_yields_partial_last_page() {
        // 11 posts at per_page=5 → 3 pages: 5 + 5 + 1.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);

        PaginationPlugin::with_per_page(5)
            .after_compile(&ctx)
            .unwrap();

        assert!(site.join("page/2/index.html").exists());
        assert!(site.join("page/3/index.html").exists());
        assert!(!site.join("page/4/index.html").exists());

        let page3 = fs::read_to_string(site.join("page/3/index.html")).unwrap();
        let li_count = page3.matches("<li>").count();
        assert_eq!(li_count, 1, "last page should have 1 entry:\n{page3}");
    }

    #[test]
    fn after_compile_per_page_one_yields_one_page_per_post() {
        // per_page=1 boundary: 5 posts → 5 pages → /page/2 .. /page/5.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 5);

        PaginationPlugin::with_per_page(1)
            .after_compile(&ctx)
            .unwrap();

        for n in 2..=5 {
            assert!(
                site.join(format!("page/{n}/index.html")).exists(),
                "page/{n}/ should exist"
            );
        }
        assert!(!site.join("page/6/index.html").exists());
    }

    // -------------------------------------------------------------------
    // after_compile — sort order
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_sorts_entries_by_date_descending() {
        // Posts written out of order — newest must appear first on
        // page 1 (which is unwritten by this plugin), so the remainder
        // on page 2 must be the *oldest* entries.
        let (_tmp, site, meta, ctx) = make_layout();
        // Write posts with dates that are NOT in filename order:
        // file `a` → 2026-01-01 (oldest)
        // file `z` → 2026-01-11 (newest)
        let dates = [
            ("a", "2026-01-01"),
            ("m", "2026-01-05"),
            ("z", "2026-01-11"),
            ("b", "2026-01-02"),
            ("y", "2026-01-10"),
            ("c", "2026-01-03"),
            ("x", "2026-01-09"),
            ("d", "2026-01-04"),
            ("w", "2026-01-08"),
            ("e", "2026-01-06"),
            ("f", "2026-01-07"),
        ];
        for (name, date) in dates {
            write_sidecar(&meta, name, &format!("Post {name}"), date);
        }

        PaginationPlugin::with_per_page(10)
            .after_compile(&ctx)
            .unwrap();

        // 11 entries / 10 per page → page 2 has the single OLDEST entry.
        let page2 = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(
            page2.contains("2026-01-01"),
            "page 2 should contain the oldest entry:\n{page2}"
        );
        assert!(
            !page2.contains("2026-01-11"),
            "page 2 should NOT contain the newest entry:\n{page2}"
        );
    }

    // -------------------------------------------------------------------
    // after_compile — HTML structure & navigation
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_emits_doctype_lang_and_charset() {
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        let html = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
    }

    #[test]
    fn after_compile_emits_pagination_nav_landmark() {
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        let html = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(html.contains("<nav aria-label=\"Pagination\">"));
    }

    #[test]
    fn after_compile_page_two_prev_link_points_at_root() {
        // The "previous" link from page 2 must point to "/" — page 1
        // is the home/root, not /page/1/. Guards line 129-130.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        let html = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(
            html.contains(r#"<a href="/" rel="prev">"#),
            "page 2's prev should point to root:\n{html}"
        );
    }

    #[test]
    fn after_compile_page_three_prev_link_points_at_page_two() {
        // Beyond page 2 the prev link uses /page/N-1/ form (line 132).
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::with_per_page(5)
            .after_compile(&ctx)
            .unwrap();

        let html = fs::read_to_string(site.join("page/3/index.html")).unwrap();
        assert!(
            html.contains(r#"<a href="/page/2/" rel="prev">"#),
            "page 3's prev should point to /page/2/:\n{html}"
        );
    }

    #[test]
    fn after_compile_last_page_has_no_next_link() {
        // The Next link is omitted on the final page — guards
        // the `if let Some(next)` branch at line 159.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::with_per_page(5)
            .after_compile(&ctx)
            .unwrap();

        let last = fs::read_to_string(site.join("page/3/index.html")).unwrap();
        assert!(
            !last.contains(r#"rel="next""#),
            "last page must not emit a Next link:\n{last}"
        );
    }

    #[test]
    fn after_compile_middle_page_has_both_prev_and_next() {
        // 16 posts / per_page=5 → 4 pages. Page 2 and page 3 are both
        // "middle" — assert they have BOTH prev and next.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 16);
        PaginationPlugin::with_per_page(5)
            .after_compile(&ctx)
            .unwrap();

        let page3 = fs::read_to_string(site.join("page/3/index.html")).unwrap();
        assert!(page3.contains(r#"rel="prev""#));
        assert!(page3.contains(r#"rel="next""#));
    }

    #[test]
    fn after_compile_renders_time_element_per_entry() {
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        PaginationPlugin::default().after_compile(&ctx).unwrap();

        let html = fs::read_to_string(site.join("page/2/index.html")).unwrap();
        assert!(
            html.contains("<time>2026-01-01</time>"),
            "page 2 should render a <time> element:\n{html}"
        );
    }

    #[test]
    fn after_compile_idempotent_overwrites_existing_pages() {
        // Re-running must not error and must leave the page directories
        // intact. Guards against any future use of `create_new`.
        let (_tmp, site, meta, ctx) = make_layout();
        write_n_dated_posts(&meta, 11);
        let plugin = PaginationPlugin::default();
        plugin.after_compile(&ctx).expect("first run");
        plugin.after_compile(&ctx).expect("second run");
        assert!(site.join("page/2/index.html").exists());
    }

    // -------------------------------------------------------------------
    // collect_json_files — recursion + filtering
    // -------------------------------------------------------------------

    #[test]
    fn collect_json_files_returns_empty_for_missing_directory() {
        // Non-existent path: the inner `is_dir()` check at line 183
        // means we just `continue`, ending with an empty Vec — no Err.
        let dir = tempdir().expect("tempdir");
        let result =
            collect_json_files(&dir.path().join("does-not-exist")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn collect_json_files_returns_empty_for_empty_directory() {
        let dir = tempdir().expect("tempdir");
        let result = collect_json_files(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn collect_json_files_filters_non_json_extensions() {
        // Only `.json` files are returned. The `is_some_and` filter at
        // line 191 must reject `.txt`, `.md`, extensionless files, etc.
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("a.json"), "{}").unwrap();
        fs::write(dir.path().join("b.txt"), "x").unwrap();
        fs::write(dir.path().join("c.md"), "x").unwrap();
        fs::write(dir.path().join("noext"), "x").unwrap();

        let result = collect_json_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].file_name().unwrap() == "a.json");
    }

    #[test]
    fn collect_json_files_recurses_into_subdirectories() {
        // Walks nested directories — guards line 189-190 (push subdir
        // onto stack).
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("top.json"), "{}").unwrap();
        fs::write(dir.path().join("a").join("mid.json"), "{}").unwrap();
        fs::write(nested.join("deep.json"), "{}").unwrap();

        let result = collect_json_files(dir.path()).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn collect_json_files_returns_results_sorted() {
        // The `files.sort()` at line 196 must yield deterministic output.
        let dir = tempdir().expect("tempdir");
        for name in ["zebra.json", "apple.json", "mango.json"] {
            fs::write(dir.path().join(name), "{}").unwrap();
        }
        let result = collect_json_files(dir.path()).unwrap();
        let names: Vec<&str> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.json", "mango.json", "zebra.json"]);
    }
}
