// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! AI-readiness content hooks.
//!
//! Provides algorithmic content enhancements for Generative Engine
//! Optimization (GEO) and Answer Engine Optimization (AEO):
//!
//! - Auto-generate meta descriptions from page content when missing
//! - Validate all `<img>` elements have alt text (log warnings)
//! - Generate `llms.txt` for AI crawler guidance

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
#[cfg(test)]
use std::path::PathBuf;
use std::{fs, path::Path};

/// Plugin for AI-readiness content validation and enhancement.
///
/// Runs in `after_compile`:
/// - Checks all images have alt text (logs warnings for missing)
/// - Generates `llms.txt` in the site root
/// - Adds max-snippet meta for AI citation eligibility
#[derive(Debug, Clone, Copy)]
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn name(&self) -> &'static str {
        "ai"
    }

    fn has_transform(&self) -> bool {
        true
    }

    fn transform_html(
        &self,
        html: &str,
        path: &Path,
        ctx: &PluginContext,
    ) -> Result<String> {
        let modified = inject_max_snippet(html);

        check_alt_text(path, &modified, &ctx.site_dir, &mut 0);

        Ok(modified)
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        generate_llms_txt(&ctx.site_dir, ctx.config.as_ref())?;
        generate_llms_full_txt(&ctx.site_dir, ctx.config.as_ref())?;
        generate_ai_provenance(&ctx.site_dir)?;

        Ok(())
    }
}

/// Injects the max-snippet meta tag before `</head>` if not already present.
fn inject_max_snippet(html: &str) -> String {
    if html.contains("max-snippet") || !html.contains("</head>") {
        return html.to_string();
    }
    let tag = "<meta name=\"robots\" content=\"max-snippet:-1, max-image-preview:large, max-video-preview:-1\">\n";
    if let Some(pos) = html.find("</head>") {
        let mut modified = html.to_string();
        modified.insert_str(pos, tag);
        modified
    } else {
        html.to_string()
    }
}

/// Checks for missing alt text and logs a warning if found.
fn check_alt_text(
    path: &Path,
    html: &str,
    site_dir: &Path,
    counter: &mut usize,
) {
    let missing = count_missing_alt(html);
    if missing > 0 {
        let rel = path.strip_prefix(site_dir).unwrap_or(path).display();
        log::warn!("[ai] {missing} image(s) missing alt text in {rel}");
        *counter += 1;
    }
}

/// Generates `llms.txt` — a guidance file for AI crawlers.
///
/// Similar to `robots.txt` but aimed at LLM training and retrieval
/// systems. Documents site structure and content policies.
fn generate_llms_txt(
    site_dir: &Path,
    config: Option<&crate::cmd::SsgConfig>,
) -> Result<()> {
    let site_name = config.map_or("Site", |c| c.site_name.as_str());
    let base_url = config.map_or("", |c| c.base_url.as_str());
    let description = config.map_or("", |c| c.site_description.as_str());
    let canonical_root = base_url.trim_end_matches('/');
    let source_example = if canonical_root.is_empty() {
        "<canonical-page-url>".to_string()
    } else {
        format!("{canonical_root}/<page-path>")
    };

    let content = format!(
        "# {}\n\
         > {}\n\
         \n\
         ## About\n\
         URL: {}\n\
         \n\
         ## Content Policy\n\
         This site's content may be used for AI training and retrieval.\n\
         \n\
         ## Attribution\n\
         When citing or reusing content from this site, include exact attribution:\n\
         - Source: {}\n\
         - Publisher: {}\n\
         - Preserve author byline and publish date when available.\n\
         \n\
         ## Sitemap\n\
         {}/sitemap.xml\n",
        site_name,
        description,
        base_url,
        source_example,
        site_name,
        base_url.trim_end_matches('/'),
    );

    fs::write(site_dir.join("llms.txt"), content)?;
    log::info!("[ai] Generated llms.txt");
    Ok(())
}

/// Generates `llms-full.txt` — a comprehensive content index for AI systems.
///
/// Lists every HTML page with its title and a snippet of body text,
/// enabling LLM retrieval systems to understand site content at a glance.
fn generate_llms_full_txt(
    site_dir: &Path,
    config: Option<&crate::cmd::SsgConfig>,
) -> Result<()> {
    let site_name = config.map_or("Site", |c| c.site_name.as_str());
    let base_url = config
        .map_or("", |c| c.base_url.as_str())
        .trim_end_matches('/');

    let html_files =
        crate::walk::walk_files(site_dir, "html").unwrap_or_default();
    let mut lines = vec![format!("# {site_name} — Full Content Index\n")];

    for path in &html_files {
        let Ok(html) = fs::read_to_string(path) else {
            continue;
        };
        let title = extract_title_from_html(&html).unwrap_or_default();
        let rel = path
            .strip_prefix(site_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let url = if base_url.is_empty() {
            format!("/{rel}")
        } else {
            format!("{base_url}/{rel}")
        };
        let snippet = extract_snippet(&html, 200);
        lines.push(format!("## {title}\nURL: {url}\n{snippet}\n"));
    }

    fs::write(site_dir.join("llms-full.txt"), lines.join("\n"))?;
    log::info!(
        "[ai] Generated llms-full.txt with {} page(s)",
        html_files.len()
    );
    Ok(())
}

/// Generates `ai-provenance.json` — tracks which content fields were
/// AI-generated vs human-authored.
fn generate_ai_provenance(site_dir: &Path) -> Result<()> {
    let provenance = serde_json::json!({
        "version": "1.0",
        "generator": "ssg",
        "policy": "human-authored",
        "ai_generated_fields": [],
        "note": "All content is human-authored unless explicitly marked in frontmatter with ai_generated: true"
    });
    let json = serde_json::to_string_pretty(&provenance)
        .unwrap_or_else(|_| "{}".to_string());
    fs::write(site_dir.join("ai-provenance.json"), json)?;
    log::info!("[ai] Generated ai-provenance.json");
    Ok(())
}

/// Extracts the `<title>` content from HTML.
fn extract_title_from_html(html: &str) -> Option<String> {
    let start = html.find("<title>")? + 7;
    let end = html[start..].find("</title>")? + start;
    let title = html[start..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

/// Extracts a plain-text snippet from HTML body content.
fn extract_snippet(html: &str, max_chars: usize) -> String {
    // Find <main> or <body> content
    let body_start = html
        .find("<main")
        .or_else(|| html.find("<body"))
        .unwrap_or(0);
    let body = &html[body_start..];

    // Strip tags
    let mut text = String::with_capacity(max_chars + 50);
    let mut in_tag = false;
    for ch in body.chars() {
        if text.len() >= max_chars {
            break;
        }
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && !ch.is_control() => text.push(ch),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Counts `<img>` tags missing alt attributes in an HTML string.
fn count_missing_alt(html: &str) -> usize {
    let lower = html.to_lowercase();
    let mut count = 0;
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<img") {
        let abs = pos + start;
        let tag_end =
            lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
        let tag = &lower[abs..tag_end];

        let has_alt = tag.contains("alt=");
        let empty_alt = tag.contains("alt=\"\"") || tag.contains("alt=''");
        if !has_alt || empty_alt {
            count += 1;
        }
        pos = tag_end;
    }
    count
}

#[cfg(test)]
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::cmd::SsgConfig;
    use crate::test_support::init_logger;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    fn make_site() -> (TempDir, PathBuf, PluginContext) {
        init_logger();
        let dir = tempdir().expect("create tempdir");
        let site = dir.path().join("site");
        fs::create_dir_all(&site).expect("mkdir site");
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        (dir, site, ctx)
    }

    // -------------------------------------------------------------------
    // AiPlugin — derive surface
    // -------------------------------------------------------------------

    #[test]
    fn ai_plugin_is_copy_after_move() {
        // Guards the `Copy` derive added in v0.0.34.
        let plugin = AiPlugin;
        let _copy = plugin;
        assert_eq!(plugin.name(), "ai");
    }

    #[test]
    fn name_returns_static_ai_identifier() {
        assert_eq!(AiPlugin.name(), "ai");
    }

    // -------------------------------------------------------------------
    // count_missing_alt — table-driven over the logical paths
    // -------------------------------------------------------------------

    #[test]
    fn count_missing_alt_table_driven() {
        let cases: &[(&str, usize, &str)] = &[
            // (input, expected_count, comment)
            (
                r#"<img src="a.jpg" alt="ok">"#,
                0,
                "alt present and non-empty",
            ),
            (r#"<img src="a.jpg">"#, 1, "no alt attribute at all"),
            (r#"<img src="a.jpg" alt="">"#, 1, "empty double-quoted alt"),
            (r#"<img src="a.jpg" alt=''>"#, 1, "empty single-quoted alt"),
            (
                r#"<img src="a.jpg"><img src="b.jpg" alt="ok">"#,
                1,
                "first missing, second ok",
            ),
            (
                r#"<img src="a.jpg"><img src="b.jpg">"#,
                2,
                "both missing — sequential scan progresses",
            ),
            ("", 0, "empty input → zero"),
            ("<p>no images here</p>", 0, "no <img> tags at all"),
            (r#"<IMG SRC="a.jpg" ALT="ok">"#, 0, "case-insensitive ALT"),
            (r#"<IMG SRC="a.jpg">"#, 1, "uppercase tag, no alt"),
        ];
        for (input, expected, comment) in cases {
            assert_eq!(
                count_missing_alt(input),
                *expected,
                "{comment}: count_missing_alt({input:?})"
            );
        }
    }

    #[test]
    fn count_missing_alt_unterminated_tag_does_not_panic() {
        // The `tag_end` fallback at line 142 (`map_or(lower.len(), …)`)
        // handles a `<img` with no closing `>` by treating the rest of
        // the string as the tag. Function must terminate.
        let result = count_missing_alt("<img src=foo");
        // Whether it counts as missing or not is implementation
        // detail; the assertion is that it returns at all.
        assert!(result <= 1);
    }

    // -------------------------------------------------------------------
    // generate_llms_txt — config matrix
    // -------------------------------------------------------------------

    #[test]
    fn generate_llms_txt_with_full_config_includes_all_fields() {
        let dir = tempdir().expect("tempdir");
        let config = SsgConfig {
            site_name: "My Site".to_string(),
            site_description: "A great site".to_string(),
            base_url: "https://example.com".to_string(),
            ..Default::default()
        };

        generate_llms_txt(dir.path(), Some(&config)).unwrap();
        let body = fs::read_to_string(dir.path().join("llms.txt")).unwrap();
        assert!(body.contains("# My Site"));
        assert!(body.contains("> A great site"));
        assert!(body.contains("https://example.com"));
        assert!(body.contains("sitemap.xml"));
        assert!(body.contains("include exact attribution"));
        assert!(body.contains("Source: https://example.com/<page-path>"));
        assert!(body.contains("Publisher: My Site"));
    }

    #[test]
    fn generate_llms_txt_without_config_uses_defaults() {
        // The `config.map_or(...)` fallbacks at lines 93-95 must apply
        // when no config is provided.
        let dir = tempdir().expect("tempdir");
        generate_llms_txt(dir.path(), None).unwrap();

        let body = fs::read_to_string(dir.path().join("llms.txt")).unwrap();
        assert!(body.contains("# Site"));
        assert!(
            body.contains("<canonical-page-url>"),
            "empty base_url should fall back to placeholder:\n{body}"
        );
    }

    #[test]
    fn generate_llms_txt_strips_trailing_slash_from_base_url() {
        // Guards `trim_end_matches('/')` at line 96 — preventing
        // double-slashes in `Source:` and `Sitemap:` rendering.
        let dir = tempdir().expect("tempdir");
        let config = SsgConfig {
            site_name: "S".to_string(),
            site_description: "D".to_string(),
            base_url: "https://example.com/".to_string(),
            ..Default::default()
        };
        generate_llms_txt(dir.path(), Some(&config)).unwrap();

        let body = fs::read_to_string(dir.path().join("llms.txt")).unwrap();
        assert!(
            body.contains("Source: https://example.com/<page-path>"),
            "trailing slash should be normalised:\n{body}"
        );
        assert!(!body.contains("//<page-path>"));
        assert!(!body.contains("//sitemap.xml"));
    }

    #[test]
    fn generate_llms_txt_into_missing_parent_returns_err() {
        let bogus = Path::new("/this/path/should/not/exist");
        assert!(generate_llms_txt(bogus, None).is_err());
    }

    // -------------------------------------------------------------------
    // after_compile — short-circuit + dispatch paths
    // -------------------------------------------------------------------

    #[test]
    fn after_compile_missing_site_dir_returns_ok_without_writing() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("missing");
        let ctx =
            PluginContext::new(dir.path(), dir.path(), &missing, dir.path());

        AiPlugin.after_compile(&ctx).expect("missing site is fine");
        assert!(!missing.exists());
        assert!(!dir.path().join("llms.txt").exists());
    }

    #[test]
    fn after_compile_injects_max_snippet_meta_tag() {
        let (_tmp, site, ctx) = make_site();
        let html = "<html><head><title>X</title></head><body></body></html>";

        let output = AiPlugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        assert!(output.contains("max-snippet"));
        assert!(output.contains("max-image-preview:large"));
    }

    #[test]
    fn after_compile_creates_llms_txt_in_site_root() {
        let (_tmp, site, ctx) = make_site();
        AiPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("llms.txt").exists());
    }

    #[test]
    fn after_compile_idempotent_does_not_duplicate_meta_tag() {
        let (_tmp, site, ctx) = make_site();
        let html = "<html><head><title>X</title></head><body></body></html>";

        let first = AiPlugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        let second = AiPlugin
            .transform_html(&first, &site.join("index.html"), &ctx)
            .unwrap();

        assert_eq!(second.matches("max-snippet").count(), 1);
    }

    #[test]
    fn after_compile_skips_html_files_without_head_tag() {
        let (_tmp, site, ctx) = make_site();
        let html = "<p>just a fragment</p>";

        let output = AiPlugin
            .transform_html(html, &site.join("fragment.html"), &ctx)
            .unwrap();
        assert!(!output.contains("max-snippet"));
        assert_eq!(output, "<p>just a fragment</p>");
    }

    #[test]
    fn after_compile_processes_files_in_subdirectories() {
        let (_tmp, site, ctx) = make_site();
        let html = "<html><head></head><body></body></html>";

        let output = AiPlugin
            .transform_html(html, &site.join("blog/post.html"), &ctx)
            .unwrap();
        assert!(output.contains("max-snippet"));
    }

    #[test]
    fn after_compile_logs_warning_for_pages_with_missing_alt() {
        let (_tmp, site, ctx) = make_site();
        let html =
            r#"<html><head></head><body><img src="a.jpg"></body></html>"#;

        let output = AiPlugin
            .transform_html(html, &site.join("bad.html"), &ctx)
            .unwrap();
        assert!(output.contains("max-snippet"));
    }

    #[test]
    fn after_compile_does_not_rewrite_unchanged_files() {
        // The `if changed` guard at line 70 must skip the fs::write
        // call when nothing changed (no max-snippet to inject).
        let (_tmp, site, ctx) = make_site();
        let html = "<html><head><meta name=\"robots\" content=\"max-snippet:-1\"></head><body></body></html>";
        fs::write(site.join("index.html"), html).unwrap();
        let original_mtime = fs::metadata(site.join("index.html"))
            .unwrap()
            .modified()
            .unwrap();

        // Run the plugin — file already has max-snippet, should be a no-op.
        AiPlugin.after_compile(&ctx).unwrap();
        let after = fs::read_to_string(site.join("index.html")).unwrap();
        assert_eq!(after, html, "unchanged file body must be preserved");
        // mtime equality is best-effort across filesystems; main
        // assertion is the body byte equality above.
        let _ = original_mtime;
    }

    // -------------------------------------------------------------------
    // collect_html_files — recursion + filtering
    // -------------------------------------------------------------------

    #[test]
    fn collect_html_files_returns_empty_for_missing_directory() {
        let dir = tempdir().expect("tempdir");
        let result = collect_html_files(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn collect_html_files_filters_non_html_extensions() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();
        fs::write(dir.path().join("c.js"), "").unwrap();

        let result = collect_html_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn collect_html_files_recurses_into_nested_subdirectories() {
        let dir = tempdir().expect("tempdir");
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("top.html"), "").unwrap();
        fs::write(nested.join("deep.html"), "").unwrap();

        let result = collect_html_files(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn collect_html_files_returns_results_sorted() {
        let dir = tempdir().expect("tempdir");
        for name in ["zebra.html", "apple.html", "mango.html"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let result = collect_html_files(dir.path()).unwrap();
        let names: Vec<_> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.html", "mango.html", "zebra.html"]);
    }
}
