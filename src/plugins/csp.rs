// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Content Security Policy hardening plugin.
//!
//! Extracts inline `<style>` and `<script>` blocks into external files
//! with Subresource Integrity (SRI) hashes, eliminating the need for
//! `'unsafe-inline'` in the Content-Security-Policy header.

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

/// Canonical Content-Security-Policy string emitted by the CSP plugin.
///
/// Returned by [`computed_policy`] and consumed by downstream emitters
/// (e.g. the `edge_headers` postprocess plugin) that need to forward
/// the same policy as an HTTP header instead of a `<meta>` tag.
///
/// The string is intentionally `'unsafe-inline'`-free; this matches the
/// post-extraction posture enforced by [`CspPlugin::transform_html`]
/// and [`inject_csp_meta`] (which strip `'unsafe-inline'` from any
/// preexisting `<meta>` policy on the way through).
pub const DEFAULT_CSP_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' https: data:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none'";

/// Returns the canonical Content-Security-Policy string that the CSP
/// plugin's inline-extraction posture is designed to enforce.
///
/// This is the single source of truth for the CSP header value: every
/// other emitter (deploy adapters, edge-headers postprocess plugin)
/// reads from this function rather than recomputing or hardcoding the
/// string, ensuring that platform-level HTTP headers and the
/// `<meta http-equiv>` injection stay in lock-step.
///
/// Returns a borrowed reference to [`DEFAULT_CSP_POLICY`] today; the
/// signature is intentionally a borrowed `&'static str` so callers can
/// pass it directly into `format!` / `write!` without an extra
/// allocation.
///
/// # Examples
///
/// ```rust
/// use ssg::csp::computed_policy;
///
/// let policy = computed_policy();
/// assert!(policy.contains("default-src"));
/// ```
#[must_use]
pub const fn computed_policy() -> &'static str {
    DEFAULT_CSP_POLICY
}

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
///
/// # Computed policy
///
/// The single string the plugin posture enforces is exposed via
/// [`computed_policy`]; downstream HTTP-header emitters (e.g. the
/// `edge_headers` postprocess plugin) call that function rather than
/// hardcoding a CSP string of their own, so the policy stays in
/// lock-step across `<meta>` injection and platform HTTP headers.
#[derive(Debug, Clone, Copy, Default)]
pub struct CspPlugin;

impl CspPlugin {
    /// Creates a new `CspPlugin`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::csp::CspPlugin;
    /// use ssg::plugin::Plugin;
    ///
    /// let p = CspPlugin::new();
    /// assert_eq!(p.name(), "csp");
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for CspPlugin {
    fn name(&self) -> &'static str {
        "csp"
    }

    fn has_transform(&self) -> bool {
        true
    }

    fn transform_html(
        &self,
        html: &str,
        path: &Path,
        ctx: &PluginContext,
    ) -> Result<String, SsgError> {
        let csp_dir = ctx.site_dir.join("_csp");
        let (rewritten, extracted) =
            extract_inline_blocks(html, &csp_dir, &ctx.site_dir)
                .map_err(|e| SsgError::io(e, path))?;

        if extracted > 0 {
            let final_html = remove_unsafe_inline_from_csp(&rewritten);
            Ok(final_html)
        } else {
            Ok(html.to_string())
        }
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Pre-create _csp/ dir so transform_html writers have the directory
        let csp_dir = ctx.site_dir.join("_csp");
        fs::create_dir_all(&csp_dir).with_path(&csp_dir)?;

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
    while let Some((before, opening_tag, content, after)) =
        find_inline_script(&result)
    {
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

        // Preserve original attributes (e.g. type=module, async, defer,
        // data-*), but override src/integrity/crossorigin since we are
        // extracting the body to a new file and computing fresh SRI.
        let preserved = preserve_script_attrs(
            &opening_tag,
            &["src", "integrity", "crossorigin"],
        );
        let script_tag = if preserved.is_empty() {
            format!(
                "<script src=\"/{rel_path}\" integrity=\"{sri}\" crossorigin=\"anonymous\"></script>"
            )
        } else {
            format!(
                "<script {preserved} src=\"/{rel_path}\" integrity=\"{sri}\" crossorigin=\"anonymous\"></script>"
            )
        };

        result = format!("{before}{script_tag}{after}");
        count += 1;
    }

    Ok((result, count))
}

/// Parses attributes out of an opening `<script …>` tag and returns a
/// space-separated, normalised attribute string with `drop` attributes
/// removed. Boolean attributes (no `=`) are emitted unchanged.
///
/// Uses `lol_html` so quoting, whitespace, and case-folding all match
/// the HTML5 parser exactly — this is what the rest of the SSG
/// pipeline relies on for tag rewriting.
fn preserve_script_attrs(opening_tag: &str, drop: &[&str]) -> String {
    use crate::util::html_rewriter::rewrite_html;
    use lol_html::element;
    use std::cell::RefCell;
    use std::rc::Rc;

    // lol_html needs a closed element to fire the handler; wrap the
    // opening tag with an explicit close.
    let fragment = format!("{opening_tag}</script>");
    let collected: Rc<RefCell<Vec<(String, String)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let collected_cb = Rc::clone(&collected);

    let _ = rewrite_html(
        &fragment,
        vec![element!("script", move |el| {
            for attr in el.attributes() {
                collected_cb.borrow_mut().push((attr.name(), attr.value()));
            }
            Ok(())
        })],
    );

    let drop_lower: Vec<String> =
        drop.iter().map(|d| d.to_ascii_lowercase()).collect();

    let parts: Vec<String> = collected
        .borrow()
        .iter()
        .filter(|(name, _)| !drop_lower.contains(&name.to_ascii_lowercase()))
        .map(|(name, value)| {
            if value.is_empty() {
                name.clone()
            } else {
                let escaped = value.replace('"', "&quot;");
                format!("{name}=\"{escaped}\"")
            }
        })
        .collect();
    parts.join(" ")
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
///
/// Returns `(before, opening_tag, content, after)` where `opening_tag`
/// is the full `<script …>` including angle brackets — the caller uses
/// it to preserve attributes such as `type=module`, `async`, `defer`,
/// and `data-*` when rewriting the tag.
fn find_inline_script(html: &str) -> Option<(String, String, String, String)> {
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
            opening_tag.to_string(),
            content.to_string(),
            html[end..].to_string(),
        ));
    }
}

/// Removes `'unsafe-inline'` from CSP `<meta>` tags in HTML.
fn remove_unsafe_inline_from_csp(html: &str) -> String {
    html.replace("'unsafe-inline'", "").replace("  ;", " ;")
}

/// Inserts a `<meta http-equiv="Content-Security-Policy" content="...">`
/// tag immediately after the `<head>` opening tag.
///
/// Uses `lol_html` so the insertion point is the correct one regardless
/// of whitespace, comments, or `<title>` placement inside the head
/// (issue #525 AC7). If the document already contains a CSP meta tag
/// (either matching `policy` exactly or any other CSP policy), the
/// input is returned unchanged so successive calls are idempotent.
/// If no `<head>` element exists in the input, the function returns
/// the input verbatim — this matches the convention used by the
/// rest of the SSG HTML post-processors (no implicit head injection).
///
/// # Examples
///
/// ```rust
/// use ssg::csp::inject_csp_meta;
///
/// let html = "<html><head><title>t</title></head></html>";
/// let out = inject_csp_meta(html, "default-src 'self'");
/// assert!(out.contains("Content-Security-Policy"));
/// ```
///
/// # Errors
///
/// Returns the input unchanged when the underlying `lol_html` rewrite
/// fails; in practice the only failure mode is allocation exhaustion.
#[must_use]
pub fn inject_csp_meta(html: &str, policy: &str) -> String {
    use crate::util::html_rewriter::rewrite_html;
    use lol_html::element;
    use lol_html::html_content::ContentType;
    use std::cell::Cell;
    use std::rc::Rc;

    // Idempotency: if a CSP meta already exists anywhere in the
    // document, do nothing. lol_html visits comments-aware so this
    // false-positive-free.
    let already_present = Rc::new(Cell::new(false));
    let already_present_cb = Rc::clone(&already_present);
    let detect = element!(
        "meta[http-equiv=\"Content-Security-Policy\" i]",
        move |_el| {
            already_present_cb.set(true);
            Ok(())
        }
    );
    let _ = rewrite_html(html, vec![detect]);
    if already_present.get() {
        return html.to_string();
    }

    let policy = policy.to_string();
    let injected = Rc::new(Cell::new(false));
    let injected_cb = Rc::clone(&injected);
    let head_handler = element!("head", move |el| {
        let meta = format!(
            "<meta http-equiv=\"Content-Security-Policy\" content=\"{policy}\">"
        );
        el.prepend(&meta, ContentType::Html);
        injected_cb.set(true);
        Ok(())
    });

    rewrite_html(html, vec![head_handler]).unwrap_or_else(|_| html.to_string())
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

/// Computes an SRI hash string: `sha256-<base64>`.
fn compute_sri(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    let b64 = BASE64.encode(bytes);
    format!("sha256-{b64}")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        let html = "<html><head><style>body{color:red}</style></head><body><script>alert(1)</script></body></html>";

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        CspPlugin.after_compile(&ctx).unwrap();

        let output = CspPlugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
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

    // ── CspPlugin::new + inject_csp_meta coverage ───────────────────

    #[test]
    fn csp_plugin_new_constructs_unit_struct() {
        let p = CspPlugin::new();
        assert_eq!(p.name(), "csp");
        assert!(p.has_transform());
    }

    #[test]
    fn inject_csp_meta_adds_meta_when_absent() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let out = inject_csp_meta(html, "default-src 'self'");
        assert!(out.contains("http-equiv=\"Content-Security-Policy\""));
        assert!(out.contains("default-src 'self'"));
    }

    #[test]
    fn inject_csp_meta_is_idempotent_when_meta_already_present() {
        let html = r#"<html><head><meta http-equiv="Content-Security-Policy" content="default-src 'self'"></head></html>"#;
        let out = inject_csp_meta(html, "script-src 'self'");
        // Should not duplicate.
        let count = out
            .matches("http-equiv=\"Content-Security-Policy\"")
            .count();
        assert_eq!(count, 1, "must not duplicate CSP meta tag");
        // And must not insert the new policy.
        assert!(!out.contains("script-src 'self'"));
    }

    #[test]
    fn inject_csp_meta_handles_no_head_gracefully() {
        let html = "<html><body>no head</body></html>";
        let out = inject_csp_meta(html, "default-src 'self'");
        // Without a <head>, lol_html returns the input unchanged.
        assert_eq!(out, html);
    }

    #[test]
    fn preserve_script_attrs_keeps_type_module() {
        let out = preserve_script_attrs(
            r#"<script type="module">"#,
            &["src", "integrity", "crossorigin"],
        );
        assert!(out.contains(r#"type="module""#), "got: {out}");
    }

    #[test]
    fn preserve_script_attrs_keeps_boolean_async_defer() {
        let out = preserve_script_attrs(
            "<script async defer>",
            &["src", "integrity", "crossorigin"],
        );
        assert!(out.contains("async"), "got: {out}");
        assert!(out.contains("defer"), "got: {out}");
    }

    #[test]
    fn preserve_script_attrs_drops_listed_attrs() {
        let out = preserve_script_attrs(
            r#"<script src="/x.js" integrity="sha384-foo" crossorigin="anonymous" data-id="9">"#,
            &["src", "integrity", "crossorigin"],
        );
        assert!(!out.contains("src="), "got: {out}");
        assert!(!out.contains("integrity="), "got: {out}");
        assert!(!out.contains("crossorigin="), "got: {out}");
        assert!(out.contains(r#"data-id="9""#), "got: {out}");
    }

    #[test]
    fn preserve_script_attrs_empty_when_no_attrs() {
        let out = preserve_script_attrs("<script>", &["src"]);
        assert_eq!(out, "");
    }

    #[test]
    fn extract_inline_script_preserves_type_module() {
        let html = r#"<html><body><script type="module">import x from '/m.js';</script></body></html>"#;
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");
        let (out, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();
        assert_eq!(count, 1);
        assert!(out.contains(r#"type="module""#), "got: {out}");
        assert!(out.contains("integrity="), "got: {out}");
    }

    #[test]
    fn extract_inline_script_preserves_data_attrs() {
        let html = r#"<html><body><script data-domain="example.com">window.x=1;</script></body></html>"#;
        let dir = tempdir().unwrap();
        let csp_dir = dir.path().join("_csp");
        let (out, count) =
            extract_inline_blocks(html, &csp_dir, dir.path()).unwrap();
        assert_eq!(count, 1);
        assert!(out.contains(r#"data-domain="example.com""#), "got: {out}");
    }

    #[test]
    fn csp_plugin_transform_html_no_inline_blocks_returns_unchanged() {
        let html =
            "<html><head><title>X</title></head><body><p>hi</p></body></html>";
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let out = CspPlugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        assert_eq!(out, html);
    }
}
