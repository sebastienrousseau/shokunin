// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Asset optimization: fingerprinting, SRI hashes, and basic minification.
//!
//! Provides cache-busting via content-hash filenames and Subresource
//! Integrity attributes for CSS and JS files.

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Plugin that fingerprints CSS/JS assets and rewrites HTML references.
///
/// Runs in `after_compile`:
/// 1. Hash each `.css` and `.js` file (SHA-256, first 8 hex chars)
/// 2. Rename: `style.css` → `style.a1b2c3d4.css`
/// 3. Rewrite all HTML `<link>` and `<script>` references
/// 4. Add `integrity` and `crossorigin` attributes (SRI)
#[derive(Debug, Clone, Copy)]
pub struct FingerprintPlugin;

impl Plugin for FingerprintPlugin {
    fn name(&self) -> &'static str {
        "fingerprint"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let all_assets = collect_assets(&ctx.site_dir)?;
        if all_assets.is_empty() {
            return Ok(());
        }

        // Three-pass fingerprinting (resolves the CSS-url() problem
        // surfaced in the v0.0.39 audit):
        //
        //   1. Hash + rename non-CSS assets first (images, fonts,
        //      JS). Build the first-stage manifest.
        //   2. Walk every CSS file. Patch any `url(...)` references
        //      that resolve to an entry in the first-stage manifest
        //      so they point at the new fingerprinted name. THEN
        //      hash + rename the CSS — its SRI hash now covers the
        //      post-rewrite content.
        //   3. Walk every HTML file and rewrite `<link href>`,
        //      `<script src>`, `<img src>`, etc. against the full
        //      manifest, attaching `integrity` + `crossorigin` for
        //      CSS/JS where SRI is meaningful.
        //
        // Without this split, CSS `url(/images/logo.png)` would
        // 404 after `logo.png` was renamed to `logo.<hash>.png`.

        let (css_files, non_css): (Vec<_>, Vec<_>) = all_assets
            .into_iter()
            .partition(|p| p.extension().is_some_and(|e| e == "css"));

        let mut manifest = fingerprint_assets(&non_css, &ctx.site_dir)?;

        for css_path in &css_files {
            rewrite_css_urls_inplace(css_path, &ctx.site_dir, &manifest)?;
        }

        let css_manifest = fingerprint_assets(&css_files, &ctx.site_dir)?;
        manifest.extend(css_manifest);

        rewrite_html_references(&ctx.site_dir, &manifest)?;

        log::info!(
            "[fingerprint] Processed {} asset(s) across {} CSS + {} other",
            manifest.len(),
            css_files.len(),
            manifest.len() - css_files.len()
        );
        Ok(())
    }
}

/// Fingerprints all asset files: computes hash, renames, and builds the manifest.
fn fingerprint_assets(
    assets: &[PathBuf],
    site_dir: &Path,
) -> Result<HashMap<String, AssetInfo>, SsgError> {
    let mut manifest = HashMap::new();

    for asset_path in assets {
        let info = fingerprint_file(asset_path, site_dir)?;
        let _ = manifest.insert(info.0, info.1);
    }

    Ok(manifest)
}

/// Fingerprints a single asset file: hash, rename, return (`old_rel`, `AssetInfo`).
fn fingerprint_file(
    asset_path: &Path,
    site_dir: &Path,
) -> Result<(String, AssetInfo), SsgError> {
    let mut content = fs::read(asset_path).with_path(asset_path)?;
    let ext = asset_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut minified = false;

    if ext == "css" {
        if let Ok(css_str) = std::str::from_utf8(&content) {
            content = minify_css(css_str).into_bytes();
            minified = true;
        }
    } else if ext == "js" || ext == "mjs" {
        if let Ok(js_str) = std::str::from_utf8(&content) {
            content = minify_js(js_str).into_bytes();
            minified = true;
        }
    }

    let hash = sha256_hex(&content);
    let short_hash = &hash[..8];

    let stem = asset_path.file_stem().unwrap_or_default().to_string_lossy();
    let new_name = format!("{stem}.{short_hash}.{ext}");
    let new_path = asset_path.with_file_name(&new_name);

    let sri = format!("sha256-{}", sri_base64(&content));

    if minified {
        fs::write(&new_path, &content).with_path(&new_path)?;
        fs::remove_file(asset_path).with_path(asset_path)?;
    } else {
        fs::rename(asset_path, &new_path).with_path(asset_path)?;
    }

    let rel_old = asset_path
        .strip_prefix(site_dir)
        .unwrap_or(asset_path)
        .to_string_lossy()
        .replace('\\', "/");
    let rel_new = new_path
        .strip_prefix(site_dir)
        .unwrap_or(&new_path)
        .to_string_lossy()
        .replace('\\', "/");

    Ok((
        rel_old,
        AssetInfo {
            fingerprinted: rel_new,
            sri,
        },
    ))
}

/// Rewrites HTML files to use fingerprinted asset references.
fn rewrite_html_references(
    site_dir: &Path,
    manifest: &HashMap<String, AssetInfo>,
) -> Result<(), SsgError> {
    let html_files = collect_html_files(site_dir)?;
    for html_path in &html_files {
        let html = fs::read_to_string(html_path).with_path(html_path)?;
        let rewritten = rewrite_asset_refs(&html, manifest);
        if rewritten != html {
            fs::write(html_path, rewritten).with_path(html_path)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct AssetInfo {
    fingerprinted: String,
    sri: String,
}

/// Rewrites every `url(...)` reference in a CSS file in place,
/// pointing each one at the fingerprinted name from `manifest` if
/// the URL resolves to a known asset.
///
/// Resolution rules:
///
/// - `url(/foo.png)` — absolute from the site root; lookup key is
///   `foo.png`.
/// - `url(./foo.png)`, `url(../images/foo.png)` — resolved against
///   the CSS file's parent directory, then made site-relative.
/// - `url(images/foo.png)` (bare, no `/` or `./`) — same as the
///   relative case above.
/// - URLs containing `://` (full URLs) and `data:` URIs are left
///   untouched.
///
/// Output URLs are written as **absolute, site-rooted paths**
/// (`/foo.<hash>.png`) regardless of the original form. This is
/// valid CSS and unambiguous; it deliberately trades a tiny bit of
/// stylistic preservation for correctness.
///
/// Quote forms handled: `url(x)`, `url("x")`, `url('x')`. URLs with
/// query strings or fragments (e.g. `url(foo.svg#icon)`,
/// `url(foo.css?v=1)`) preserve the suffix on the rewritten URL.
fn rewrite_css_urls(
    css: &str,
    css_path: &Path,
    site_dir: &Path,
    manifest: &HashMap<String, AssetInfo>,
) -> String {
    let css_dir = css_path.parent().unwrap_or(css_path);
    let mut out = String::with_capacity(css.len());
    let mut remaining = css;

    while let Some(idx) = remaining.find("url(") {
        out.push_str(&remaining[..idx]);
        let after_open = &remaining[idx + 4..]; // past "url("
        let Some(close_idx) = after_open.find(')') else {
            // Unterminated url(...) — leave the rest unchanged.
            out.push_str("url(");
            out.push_str(after_open);
            return out;
        };
        let raw = &after_open[..close_idx];
        let rest = &after_open[close_idx + 1..];

        // Strip optional surrounding quotes.
        let trimmed = raw.trim();
        let (quote, inner) = if let Some(s) = trimmed.strip_prefix('"') {
            ('"', s.strip_suffix('"').unwrap_or(s))
        } else if let Some(s) = trimmed.strip_prefix('\'') {
            ('\'', s.strip_suffix('\'').unwrap_or(s))
        } else {
            ('\0', trimmed)
        };

        // Split off ?query or #fragment so we don't try to resolve them.
        let (url, suffix) = if let Some(i) = inner.find(['?', '#']) {
            (&inner[..i], &inner[i..])
        } else {
            (inner, "")
        };

        let resolved = resolve_css_url(url, css_dir, site_dir);
        let hit = resolved.and_then(|key| manifest.get(&key).map(|i| (key, i)));

        out.push_str("url(");
        if let Some((_, info)) = hit {
            // Emit absolute /<fingerprinted>(suffix).
            let new_url = format!("/{}{}", info.fingerprinted, suffix);
            if quote != '\0' {
                out.push(quote);
            }
            out.push_str(&new_url);
            if quote != '\0' {
                out.push(quote);
            }
        } else {
            // No manifest hit — emit the original verbatim.
            out.push_str(raw);
        }
        out.push(')');

        remaining = rest;
    }

    out.push_str(remaining);
    out
}

/// Resolves a CSS URL to a site-relative manifest key.
///
/// Returns `None` for full URLs (`http://`, `https://`, `//`),
/// `data:` URIs, and paths that escape the site directory.
fn resolve_css_url(
    url: &str,
    css_dir: &Path,
    site_dir: &Path,
) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("data:")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("//")
    {
        return None;
    }

    // Build the absolute on-disk path.
    let candidate = if let Some(stripped) = trimmed.strip_prefix('/') {
        site_dir.join(stripped)
    } else {
        css_dir.join(trimmed)
    };

    // Logical canonicalisation: collapse `..` and `.` without
    // touching the filesystem so non-existent (already-renamed)
    // targets still resolve.
    let mut components: Vec<&std::ffi::OsStr> = Vec::new();
    for c in candidate.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _ = components.pop();
            }
            std::path::Component::Normal(s) => components.push(s),
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                components.clear();
            }
        }
    }
    let mut resolved = PathBuf::new();
    for c in components {
        resolved.push(c);
    }

    // The manifest stores keys relative to site_dir (no leading slash).
    let site_components: Vec<&std::ffi::OsStr> = site_dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s),
            _ => None,
        })
        .collect();
    let resolved_components: Vec<&std::ffi::OsStr> = resolved
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s),
            _ => None,
        })
        .collect();

    if resolved_components.len() < site_components.len()
        || resolved_components[..site_components.len()] != site_components[..]
    {
        return None;
    }

    let rel: PathBuf = resolved_components[site_components.len()..]
        .iter()
        .collect();
    Some(rel.to_string_lossy().replace('\\', "/"))
}

/// Reads, rewrites, and writes a CSS file in place if any of its
/// `url(...)` references resolve to a manifest entry.
fn rewrite_css_urls_inplace(
    css_path: &Path,
    site_dir: &Path,
    manifest: &HashMap<String, AssetInfo>,
) -> Result<(), SsgError> {
    let css = fs::read_to_string(css_path).with_path(css_path)?;
    let rewritten = rewrite_css_urls(&css, css_path, site_dir, manifest);
    if rewritten != css {
        fs::write(css_path, rewritten).with_path(css_path)?;
    }
    Ok(())
}

/// Rewrites asset references in HTML and adds SRI attributes.
fn rewrite_asset_refs(
    html: &str,
    manifest: &HashMap<String, AssetInfo>,
) -> String {
    let mut result = html.to_string();
    for (old_path, info) in manifest {
        // Replace href="old" with href="new" integrity="..." crossorigin="anonymous"
        let old_ref = format!("\"{old_path}\"");
        let old_ref_slash = format!("\"/{old_path}\"");
        let new_ref = format!(
            "\"{}\" integrity=\"{}\" crossorigin=\"anonymous\"",
            info.fingerprinted, info.sri
        );
        let new_ref_slash = format!(
            "\"/{}\" integrity=\"{}\" crossorigin=\"anonymous\"",
            info.fingerprinted, info.sri
        );

        result = result.replace(&old_ref, &new_ref);
        result = result.replace(&old_ref_slash, &new_ref_slash);
    }
    result
}

/// SHA-256 hash as a 64-char hex string.
///
/// Used for the cache-busting fingerprint suffix (`name.<hash>.ext`).
/// The first 8 hex characters of this output are taken as the
/// short content fingerprint; the full digest is also reused as the
/// raw input to [`sri_base64`] for the `integrity="sha256-..."`
/// attribute.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Returns the canonical Subresource Integrity payload for `data`.
///
/// Browsers compare the `integrity` attribute against `base64(SHA-256(body))`
/// per the [W3C SRI spec](https://www.w3.org/TR/SRI/#the-integrity-attribute).
/// Combined with the `sha256-` prefix at the call site, the resulting
/// attribute is exactly what a browser will validate the response body
/// against.
fn sri_base64(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    BASE64.encode(bytes)
}

/// Minimal CSS minifier that removes comments and compresses whitespace.
fn minify_css(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    let mut in_comment = false;
    let mut in_string = None;

    while let Some(ch) = chars.next() {
        if in_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_comment = false;
            }
            continue;
        }

        if let Some(quote) = in_string {
            result.push(ch);
            if ch == quote {
                let mut backslashes = 0;
                let mut temp = result.len() as isize - 2;
                while temp >= 0 && result.as_bytes()[temp as usize] == b'\\' {
                    backslashes += 1;
                    temp -= 1;
                }
                if backslashes % 2 == 0 {
                    in_string = None;
                }
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_comment = true;
            continue;
        }

        if ch == '\'' || ch == '"' {
            in_string = Some(ch);
            result.push(ch);
            continue;
        }

        if ch.is_whitespace() {
            result.push(' ');
            continue;
        }

        result.push(ch);
    }

    let mut clean = String::with_capacity(result.len());
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == ' ' {
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let next = if i + 1 < chars.len() {
                Some(chars[i + 1])
            } else {
                None
            };

            let is_needed = match (prev, next) {
                (Some(p), Some(n)) => {
                    let is_p_word = p.is_alphanumeric()
                        || p == '-'
                        || p == '_'
                        || p == '#'
                        || p == '.'
                        || p == '@'
                        || p == '%'
                        || p == '$';
                    let is_n_word = n.is_alphanumeric()
                        || n == '-'
                        || n == '_'
                        || n == '#'
                        || n == '.'
                        || n == '@'
                        || n == '%'
                        || n == '$';
                    is_p_word && is_n_word
                }
                _ => false,
            };
            if is_needed {
                clean.push(' ');
            }
        } else {
            clean.push(ch);
        }
        i += 1;
    }

    clean.trim().to_string()
}

/// Minimal JS minifier that removes comments and compresses whitespace/newlines safely.
fn minify_js(js: &str) -> String {
    let mut result = String::with_capacity(js.len());
    let mut chars = js.chars().peekable();
    let mut in_multi_comment = false;
    let mut in_single_comment = false;
    let mut in_string = None;

    while let Some(ch) = chars.next() {
        if in_multi_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_multi_comment = false;
            }
            continue;
        }

        if in_single_comment {
            if ch == '\n' || ch == '\r' {
                in_single_comment = false;
                result.push('\n');
            }
            continue;
        }

        if let Some(quote) = in_string {
            result.push(ch);
            if ch == quote {
                let mut backslashes = 0;
                let mut temp = result.len() as isize - 2;
                while temp >= 0 && result.as_bytes()[temp as usize] == b'\\' {
                    backslashes += 1;
                    temp -= 1;
                }
                if backslashes % 2 == 0 {
                    in_string = None;
                }
            }
            continue;
        }

        if ch == '/' {
            if chars.peek() == Some(&'*') {
                let _ = chars.next();
                in_multi_comment = true;
                continue;
            } else if chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_single_comment = true;
                continue;
            }
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            in_string = Some(ch);
            result.push(ch);
            continue;
        }

        if ch.is_whitespace() {
            if ch == '\n' || ch == '\r' {
                if !result.ends_with('\n') && !result.is_empty() {
                    result.push('\n');
                }
            } else if !result.ends_with(' ')
                && !result.ends_with('\n')
                && !result.is_empty()
            {
                result.push(' ');
            }
            continue;
        }

        result.push(ch);
    }

    let mut clean = String::with_capacity(result.len());
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == ' ' || ch == '\n' {
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };
            let next = if i + 1 < chars.len() {
                Some(chars[i + 1])
            } else {
                None
            };

            let is_needed = match (prev, next) {
                (Some(p), Some(n)) => {
                    let is_p_word = p.is_alphanumeric() || p == '_' || p == '$';
                    let is_n_word = n.is_alphanumeric() || n == '_' || n == '$';
                    is_p_word && is_n_word
                }
                _ => false,
            };
            if is_needed {
                clean.push(ch);
            }
        } else {
            clean.push(ch);
        }
        i += 1;
    }
    clean.trim().to_string()
}

/// Asset extensions we content-fingerprint. Matches the
/// "content-addressable asset pipeline" intent of issue #468:
/// CSS/JS for code, common raster + vector image formats for art,
/// font formats for typography. Each gets a `name.hash.ext` rename
/// and an SRI hash; deploy configs serve them with
/// `Cache-Control: public, max-age=31536000, immutable`.
const FINGERPRINTED_EXTENSIONS: &[&str] = &[
    "css", "js", "mjs", "png", "jpg", "jpeg", "webp", "avif", "gif", "svg",
    "woff", "woff2", "ttf", "otf",
];

/// Helper to map anyhow errors from path walkers to `SsgError`.
fn map_anyhow_to_io(err: anyhow::Error, path: &Path) -> SsgError {
    let io_err = err.downcast::<std::io::Error>().unwrap_or_else(|e| {
        std::io::Error::other(e.to_string())
    });
    SsgError::Io {
        path: path.to_path_buf(),
        source: io_err,
    }
}

/// Collects every fingerprintable asset from site dir.
fn collect_assets(dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    crate::walk::walk_files_multi(dir, FINGERPRINTED_EXTENSIONS)
        .map_err(|e| map_anyhow_to_io(e, dir))
}

fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>, SsgError> {
    crate::walk::walk_files(dir, "html").map_err(|e| map_anyhow_to_io(e, dir))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_minify_css() {
        let input = "body {\n  color: red;\n  background-color: #ffffff; /* comment */\n}";
        let expected = "body{color:red;background-color:#ffffff;}";
        assert_eq!(minify_css(input), expected);
    }

    #[test]
    fn test_minify_js() {
        let input = "const x = 5; // comment\n/* multi\ncomment */\nconst y = 10;\nconsole.log(x + y);";
        let expected = "const x=5;const y=10;console.log(x+y);";
        assert_eq!(minify_js(input), expected);
    }

    #[test]
    fn test_sha256_hex_deterministic() {
        let h1 = sha256_hex(b"hello");
        let h2 = sha256_hex(b"hello");
        assert_eq!(h1, h2);
        // Real SHA-256 is 32 bytes → 64 hex chars.
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_sha256_hex_known_vectors() {
        // Verifies real SHA-256 is in use, not an FNV placeholder.
        // Empty input — well-known SHA-256("") digest.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // "abc" — the canonical NIST test vector.
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sri_base64_known_vector() {
        // SHA-256("") base64-encoded is the canonical 47ZHRWlj-... value.
        assert_eq!(
            sri_base64(b""),
            "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
        );
    }

    #[test]
    fn test_sha256_hex_varies() {
        let h1 = sha256_hex(b"hello");
        let h2 = sha256_hex(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_fingerprint_plugin() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        // Create a CSS file
        fs::write(site.join("style.css"), "body { color: red; }").unwrap();

        // Create HTML that references it
        let html = r#"<html><head><link rel="stylesheet" href="style.css"></head><body></body></html>"#;
        fs::write(site.join("index.html"), html).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        FingerprintPlugin.after_compile(&ctx).unwrap();

        // Original file should be gone
        assert!(!site.join("style.css").exists());

        // Fingerprinted file should exist
        let entries: Vec<_> = fs::read_dir(&site)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| {
                e.path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("style.")
                    && e.path().extension().is_some_and(|e| e == "css")
            })
            .collect();
        assert_eq!(entries.len(), 1);

        // HTML should reference the fingerprinted file
        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(output.contains("integrity="));
        assert!(output.contains("crossorigin=\"anonymous\""));
        assert!(!output.contains("href=\"style.css\""));
    }

    #[test]
    fn name_returns_static_fingerprint_identifier() {
        assert_eq!(FingerprintPlugin.name(), "fingerprint");
    }

    #[test]
    fn after_compile_missing_site_dir_returns_ok() {
        // Line 34: `!ctx.site_dir.exists()` early return.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let ctx =
            PluginContext::new(dir.path(), dir.path(), &missing, dir.path());
        FingerprintPlugin.after_compile(&ctx).unwrap();
        assert!(!missing.exists());
    }

    #[test]
    fn after_compile_no_assets_short_circuits() {
        // Line 40: `assets.is_empty()` early return — site with
        // HTML but no CSS/JS.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("index.html"), "<p></p>").unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        FingerprintPlugin.after_compile(&ctx).unwrap();
        // HTML untouched.
        assert_eq!(
            fs::read_to_string(site.join("index.html")).unwrap(),
            "<p></p>"
        );
    }

    #[test]
    fn after_compile_fingerprint_absolute_path_href() {
        // Covers the `old_ref_slash` variant (with leading /) in
        // rewrite_asset_refs — absolute-path stylesheet links.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("app.js"), "console.log(1);").unwrap();
        fs::write(
            site.join("index.html"),
            r#"<html><head><script src="/app.js"></script></head></html>"#,
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        FingerprintPlugin.after_compile(&ctx).unwrap();
        let html = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(html.contains("integrity="));
    }

    #[test]
    fn collect_assets_picks_up_fingerprintable_extensions() {
        // Issue #468 widened the fingerprinted set from {css, js}
        // to also include images and fonts. css/js/png/woff2 are in;
        // html/txt/md are out.
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.css"), "").unwrap();
        fs::write(dir.path().join("b.js"), "").unwrap();
        fs::write(dir.path().join("c.html"), "").unwrap();
        fs::write(dir.path().join("d.png"), "").unwrap();
        fs::write(dir.path().join("e.woff2"), "").unwrap();
        fs::write(dir.path().join("f.txt"), "").unwrap();
        let files = collect_assets(dir.path()).unwrap();
        // 4 fingerprintable: css, js, png, woff2.
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn collect_assets_recurses_into_subdirectories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("vendor");
        fs::create_dir(&nested).unwrap();
        fs::write(dir.path().join("top.css"), "").unwrap();
        fs::write(nested.join("lib.js"), "").unwrap();
        let files = collect_assets(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn collect_html_files_filters_non_html() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();
        let files = collect_html_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn sha256_hex_produces_64_hex_chars() {
        assert_eq!(sha256_hex(b"abc").len(), 64);
        assert_eq!(sha256_hex(b"").len(), 64);
    }

    #[test]
    fn sri_base64_is_nonempty_for_input() {
        assert!(!sri_base64(b"hello").is_empty());
    }

    #[test]
    fn sri_base64_emits_44_char_payload() {
        // SHA-256 → 32 raw bytes → base64 with padding = 44 chars.
        assert_eq!(sri_base64(b"hello").len(), 44);
        assert_eq!(sri_base64(b"").len(), 44);
    }

    #[test]
    fn test_rewrite_asset_refs() {
        let mut manifest = HashMap::new();
        let _ = manifest.insert(
            "style.css".to_string(),
            AssetInfo {
                fingerprinted: "style.abc12345.css".to_string(),
                sri: "sha256-xyz".to_string(),
            },
        );

        let html = r#"<link rel="stylesheet" href="style.css">"#;
        let result = rewrite_asset_refs(html, &manifest);
        assert!(result.contains("style.abc12345.css"));
        assert!(result.contains("integrity=\"sha256-xyz\""));
    }

    // ── CSS url() rewriting (resolves audit item #2) ───────────────

    fn css_manifest() -> HashMap<String, AssetInfo> {
        let mut m = HashMap::new();
        let _ = m.insert(
            "images/logo.png".to_string(),
            AssetInfo {
                fingerprinted: "images/logo.deadbeef.png".to_string(),
                sri: String::new(),
            },
        );
        let _ = m.insert(
            "fonts/sans.woff2".to_string(),
            AssetInfo {
                fingerprinted: "fonts/sans.cafef00d.woff2".to_string(),
                sri: String::new(),
            },
        );
        m
    }

    #[test]
    fn rewrite_css_urls_handles_absolute_path() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("assets/style.css");
        let css = "body { background: url(/images/logo.png); }";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(out.contains("url(/images/logo.deadbeef.png)"));
        assert!(!out.contains("logo.png)"));
    }

    #[test]
    fn rewrite_css_urls_handles_relative_path() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("assets/style.css");
        let css = "body { background: url(../images/logo.png); }";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(out.contains("url(/images/logo.deadbeef.png)"));
    }

    #[test]
    fn rewrite_css_urls_handles_double_quotes() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = r#"body { background: url("/images/logo.png"); }"#;
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(out.contains(r#"url("/images/logo.deadbeef.png")"#));
    }

    #[test]
    fn rewrite_css_urls_handles_single_quotes() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = "body { background: url('/images/logo.png'); }";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(out.contains("url('/images/logo.deadbeef.png')"));
    }

    #[test]
    fn rewrite_css_urls_preserves_query_and_fragment() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = "@font-face { src: url(/fonts/sans.woff2?v=1#hint); }";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(out.contains("/fonts/sans.cafef00d.woff2?v=1#hint"));
    }

    #[test]
    fn rewrite_css_urls_skips_external_and_data_urls() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = r#"
            a { background: url(https://cdn.example.com/x.png); }
            b { background: url(//cdn.example.com/y.png); }
            c { background: url(data:image/svg+xml,<svg/>); }
        "#;
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        // All three URLs are left untouched.
        assert!(out.contains("https://cdn.example.com/x.png"));
        assert!(out.contains("//cdn.example.com/y.png"));
        assert!(out.contains("data:image/svg+xml"));
    }

    #[test]
    fn rewrite_css_urls_no_change_when_url_not_in_manifest() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = "body { background: url(/images/missing.png); }";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert_eq!(out, css);
    }

    #[test]
    fn rewrite_css_urls_unterminated_url_does_not_panic() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("style.css");
        let css = "body { background: url(/images/logo.png";
        let out = rewrite_css_urls(css, &css_path, dir.path(), &css_manifest());
        assert!(!out.is_empty());
    }

    #[test]
    fn after_compile_rewrites_css_url_to_fingerprinted_image() {
        // End-to-end: drop a CSS file referencing a PNG, run the
        // plugin, and confirm the produced CSS points at the
        // fingerprinted PNG name.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(site.join("images")).unwrap();
        // 1×1 transparent PNG (the smallest valid PNG bytes).
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00,
            0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63,
            0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4,
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60,
            0x82,
        ];
        fs::write(site.join("images/logo.png"), png_bytes).unwrap();
        fs::write(
            site.join("style.css"),
            "body { background: url(/images/logo.png); }",
        )
        .unwrap();
        fs::write(
            site.join("index.html"),
            r#"<html><head><link rel="stylesheet" href="style.css"></head><body></body></html>"#,
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        FingerprintPlugin.after_compile(&ctx).unwrap();

        // Find the renamed CSS and verify its url() points at the
        // renamed PNG (not the original `logo.png` filename).
        let mut css_text = None;
        for entry in fs::read_dir(&site).unwrap().flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "css") {
                css_text = Some(fs::read_to_string(&p).unwrap());
            }
        }
        let css_text = css_text.expect("renamed CSS file present");
        assert!(
            css_text.contains("/images/logo."),
            "rewritten CSS should reference renamed PNG: {css_text}"
        );
        assert!(css_text.contains(".png"), "still ends in .png: {css_text}");
        // Crucial: the URL is no longer the original `/images/logo.png`
        // — it's `/images/logo.<hash>.png`.
        assert!(
            !css_text.contains("/images/logo.png)"),
            "must no longer point at the un-fingerprinted PNG: {css_text}"
        );
    }

    #[test]
    fn test_fingerprint_file_missing_returns_io_error() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.css");
        let res = fingerprint_file(&missing, dir.path());
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, SsgError::Io { .. }));
        if let SsgError::Io { path, .. } = err {
            assert_eq!(path, missing);
        }
    }

    #[test]
    fn test_rewrite_css_urls_inplace_missing_returns_io_error() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.css");
        let manifest = HashMap::new();
        let res = rewrite_css_urls_inplace(&missing, dir.path(), &manifest);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, SsgError::Io { .. }));
        if let SsgError::Io { path, .. } = err {
            assert_eq!(path, missing);
        }
    }
}
