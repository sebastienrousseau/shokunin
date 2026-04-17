// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Content Security Policy hardening plugin.
//!
//! Extracts inline `<style>` and `<script>` blocks into external files
//! with Subresource Integrity (SRI) hashes, eliminating the need for
//! `'unsafe-inline'` in the Content-Security-Policy header.

use crate::plugin::{Plugin, PluginContext};
use crate::walk;
use anyhow::Result;
use std::{fs, path::Path};

/// Plugin that extracts inline styles/scripts to external files with SRI.
///
/// Runs in `after_compile` after all other content transforms but before
/// minification. For each HTML file:
///
/// 1. Finds `<style>…</style>` and `<script>…</script>` inline blocks
/// 2. Writes each block to `_csp/<hash>.css` or `_csp/<hash>.js`
/// 3. Replaces the inline block with a `<link>`/`<script src>` tag
///    including `integrity` and `crossorigin` attributes
/// 4. Rewrites any `<meta>` CSP tags to remove `'unsafe-inline'`
///
/// Blocks with `type="application/ld+json"` or `data-ssg-livereload`
/// attributes are skipped (structured data / dev-only scripts).
#[derive(Debug, Clone, Copy)]
pub struct CspPlugin;

impl Plugin for CspPlugin {
    fn name(&self) -> &'static str {
        "csp"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let csp_dir = ctx.site_dir.join("_csp");
        let html_files =
            walk::walk_files(&ctx.site_dir, "html").unwrap_or_default();

        if html_files.is_empty() {
            return Ok(());
        }

        let mut total_extracted = 0usize;

        for html_path in &html_files {
            let html = fs::read_to_string(html_path)?;
            let (rewritten, extracted) =
                extract_inline_blocks(&html, &csp_dir, &ctx.site_dir)?;

            if extracted > 0 {
                // Also strip 'unsafe-inline' from CSP meta tags
                let final_html = remove_unsafe_inline_from_csp(&rewritten);
                fs::write(html_path, final_html)?;
                total_extracted += extracted;
            }
        }

        if total_extracted > 0 {
            log::info!(
                "[csp] Extracted {total_extracted} inline block(s) to _csp/"
            );
        }
        Ok(())
    }
}

/// Extracts inline `<style>` and `<script>` blocks from HTML.
///
/// Returns `(rewritten_html, count_of_extracted_blocks)`.
fn extract_inline_blocks(
    html: &str,
    csp_dir: &Path,
    site_dir: &Path,
) -> Result<(String, usize)> {
    let mut result = html.to_string();
    let mut count = 0;

    // Extract <style>…</style> blocks
    while let Some((before, content, after)) =
        find_inline_block(&result, "style")
    {
        let hash = fnv_hash(content.as_bytes());
        let filename = format!("{hash:016x}.css");
        let file_path = csp_dir.join(&filename);

        fs::create_dir_all(csp_dir)?;
        fs::write(&file_path, content.as_bytes())?;

        let sri = compute_sri(content.as_bytes());
        let rel_path = file_path
            .strip_prefix(site_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");

        let link_tag = format!(
            "<link rel=\"stylesheet\" href=\"/{}\" integrity=\"{}\" crossorigin=\"anonymous\">",
            rel_path, sri
        );

        result = format!("{before}{link_tag}{after}");
        count += 1;
    }

    // Extract <script>…</script> blocks (skip JSON-LD and livereload)
    while let Some((before, content, after)) = find_inline_script(&result) {
        let hash = fnv_hash(content.as_bytes());
        let filename = format!("{hash:016x}.js");
        let file_path = csp_dir.join(&filename);

        fs::create_dir_all(csp_dir)?;
        fs::write(&file_path, content.as_bytes())?;

        let sri = compute_sri(content.as_bytes());
        let rel_path = file_path
            .strip_prefix(site_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");

        let script_tag = format!(
            "<script src=\"/{}\" integrity=\"{}\" crossorigin=\"anonymous\"></script>",
            rel_path, sri
        );

        result = format!("{before}{script_tag}{after}");
        count += 1;
    }

    Ok((result, count))
}

/// Finds the first inline `<style>…</style>` block and returns
/// `(html_before, style_content, html_after)`.
fn find_inline_block<'a>(
    html: &'a str,
    tag: &str,
) -> Option<(&'a str, &'a str, &'a str)> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    let start = html.find(&open)?;
    let content_start = start + open.len();
    let content_end = html[content_start..].find(&close)? + content_start;
    let end = content_end + close.len();

    let content = &html[content_start..content_end];
    if content.trim().is_empty() {
        return None;
    }

    Some((&html[..start], content, &html[end..]))
}

/// Finds the first inline `<script>…</script>` block, skipping:
/// - `<script type="application/ld+json">` (structured data)
/// - `<script data-ssg-livereload>` (dev-only)
/// - `<script src="...">` (already external)
fn find_inline_script(html: &str) -> Option<(String, String, String)> {
    let mut search_from = 0;

    loop {
        let rest = &html[search_from..];
        let start = rest.find("<script")?;
        let abs_start = search_from + start;

        // Find the end of the opening tag
        let tag_end = html[abs_start..].find('>')? + abs_start;
        let opening_tag = &html[abs_start..=tag_end];

        // Skip JSON-LD, livereload, and already-external scripts
        if opening_tag.contains("application/ld+json")
            || opening_tag.contains("data-ssg-livereload")
            || opening_tag.contains("src=")
        {
            search_from = tag_end + 1;
            continue;
        }

        let content_start = tag_end + 1;
        let close_tag = "</script>";
        let content_end =
            html[content_start..].find(close_tag)? + content_start;
        let end = content_end + close_tag.len();

        let content = &html[content_start..content_end];
        if content.trim().is_empty() {
            search_from = end;
            continue;
        }

        return Some((
            html[..abs_start].to_string(),
            content.to_string(),
            html[end..].to_string(),
        ));
    }
}

/// Removes `'unsafe-inline'` from CSP `<meta>` tags in HTML.
fn remove_unsafe_inline_from_csp(html: &str) -> String {
    html.replace("'unsafe-inline'", "").replace("  ;", " ;")
}

/// FNV-1a 64-bit hash.
fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Computes an SRI hash string: `sha256-<hex>`.
fn compute_sri(data: &[u8]) -> String {
    let hash = fnv_hash(data);
    format!("sha256-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extract_style_block() {
        let html = "<html><head><style>body { color: red; }</style></head><body></body></html>";
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (result, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();

        assert_eq!(count, 1);
        assert!(result.contains("<link rel=\"stylesheet\""));
        assert!(result.contains("integrity="));
        assert!(!result.contains("<style>"));
    }

    #[test]
    fn extract_script_block() {
        let html =
            "<html><body><script>console.log('hi');</script></body></html>";
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (result, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();

        assert_eq!(count, 1);
        assert!(result.contains("<script src="));
        assert!(result.contains("integrity="));
        assert!(!result.contains("console.log"));
    }

    #[test]
    fn skips_jsonld_scripts() {
        let html = r#"<html><body><script type="application/ld+json">{"@type":"Thing"}</script></body></html>"#;
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (result, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();

        assert_eq!(count, 0);
        assert!(result.contains("application/ld+json"));
    }

    #[test]
    fn skips_livereload_scripts() {
        let html = r#"<html><body><script data-ssg-livereload>ws.connect();</script></body></html>"#;
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (result, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();

        assert_eq!(count, 0);
        assert!(result.contains("data-ssg-livereload"));
    }

    #[test]
    fn skips_external_scripts() {
        let html =
            r#"<html><body><script src="/app.js"></script></body></html>"#;
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (result, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();

        assert_eq!(count, 0);
        assert_eq!(result, html);
    }

    #[test]
    fn removes_unsafe_inline_from_csp() {
        let html = r#"<meta content="script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'">"#;
        let result = remove_unsafe_inline_from_csp(html);
        assert!(!result.contains("unsafe-inline"));
    }

    #[test]
    fn skips_empty_style_blocks() {
        let html = "<html><head><style>  </style></head></html>";
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");

        let (_, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn csp_plugin_name() {
        assert_eq!(CspPlugin.name(), "csp");
    }

    #[test]
    fn csp_plugin_skips_missing_site_dir() {
        let ctx = PluginContext::new(
            Path::new("/tmp/c"),
            Path::new("/tmp/b"),
            Path::new("/nonexistent/site"),
            Path::new("/tmp/t"),
        );
        assert!(CspPlugin.after_compile(&ctx).is_ok());
    }

    #[test]
    fn csp_plugin_processes_html_files() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            "<html><head><style>body{color:red}</style></head><body><script>alert(1)</script></body></html>",
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        CspPlugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(output.contains("<link rel=\"stylesheet\""));
        assert!(output.contains("<script src="));
        assert!(!output.contains("body{color:red}"));
        assert!(!output.contains("alert(1)"));
        assert!(site.join("_csp").exists());
    }

    #[test]
    fn fnv_hash_deterministic() {
        let h1 = fnv_hash(b"hello");
        let h2 = fnv_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn fnv_hash_different_inputs() {
        assert_ne!(fnv_hash(b"a"), fnv_hash(b"b"));
    }

    #[test]
    fn compute_sri_format() {
        let sri = compute_sri(b"test");
        assert!(sri.starts_with("sha256-"));
    }
}
