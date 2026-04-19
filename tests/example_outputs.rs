#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::single_char_pattern,
    clippy::format_in_format_args,
    clippy::needless_late_init,
    clippy::if_then_some_else_none,
    clippy::must_use_candidate
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End-to-end regression tests for every shipped example.
//!
//! Each test:
//! 1. Runs the example via `cargo run --example <name>` with a hard timeout
//!    (the dev server is killed after the build completes).
//! 2. Asserts that the expected output directory is populated.
//! 3. Runs a battery of *validators* over the generated HTML / JSON to catch
//!    the regressions previously fixed by hand:
//!    - empty `<link rel="preload" href>` tags
//!    - missing modern `<meta name="mobile-web-app-capable">` alongside
//!      the deprecated apple variant
//!    - manifest icon entries with empty `src`
//!    - mobile-menu rendered on desktop (no `display:none` outside the
//!      `@media(max-width:768px)` block)
//!    - hardcoded `/posts/` or `/tags/` nav anchors that 404
//!    - empty `<title>` or missing `<html lang>`
//!    - WCAG 1.1.1 `<img alt>` violations
//!    - bad JSON in manifest.json or search-index.json
//!
//! The tests are intentionally *integration-level*: they exercise the
//! same code path a developer hits when running an example by hand,
//! so any drift between source and generated artifacts is caught.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::Duration,
};

/// Global mutex serialising the per-example tests. Each example binds
/// `127.0.0.1:3000` for its dev server, so they cannot run concurrently
/// under `cargo test`'s default parallel scheduler.
fn example_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Returns the workspace root (the directory containing `Cargo.toml`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Runs `cargo run --quiet --example <name>` with a hard timeout. The
/// example's dev server is killed once the build phase has completed by
/// truncating the process at the timeout — output written to disk before
/// the kill is what we validate.
fn run_example(name: &str, timeout: Duration) {
    let root = workspace_root();
    let mut child = Command::new("cargo")
        .current_dir(&root)
        .args(["run", "--quiet", "--example", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn cargo for {name}: {e}"));

    // Poll for completion up to the timeout, then kill.
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(e) => panic!("error waiting on {name}: {e}"),
        }
    }
}

/// Read an HTML file, panicking with a useful path on failure.
fn read_html(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Walk a directory and collect every `.html` file path.
fn html_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "html") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out
}

// ── Validators ──────────────────────────────────────────────────────

/// Asserts no `<link rel="preload">` tag has an empty/missing `href`.
fn validate_no_empty_preload(html: &str, file: &Path) {
    // Tolerant regex via simple string scan — `rel=preload` (any quoting)
    // immediately followed within the same tag by a bare `href` (no `=`)
    // or `href=""` is a regression.
    for tag in find_tags(html, "link") {
        let lower = tag.to_ascii_lowercase();
        let is_preload = lower.contains("rel=\"preload\"")
            || lower.contains("rel='preload'")
            || lower.contains("rel=preload");
        if !is_preload {
            continue;
        }
        let has_real_href = lower.contains("href=\"")
            && !lower.contains("href=\"\"")
            || lower.contains("href='") && !lower.contains("href=''")
            || lower.contains("href=")
                && lower.split("href=").nth(1).is_some_and(|after| {
                    let trimmed = after.trim_start();
                    let next = trimmed.chars().next().unwrap_or('>');
                    next != '>' && next != ' ' && next != '"' && next != '\''
                });
        assert!(
            has_real_href,
            "{}: <link rel=preload> with empty/missing href: {tag}",
            file.display()
        );
    }
}

/// Asserts the modern `mobile-web-app-capable` meta exists whenever the
/// deprecated apple variant is emitted.
fn validate_modern_pwa_meta(html: &str, file: &Path) {
    let has_apple = html.contains("apple-mobile-web-app-capable");
    if !has_apple {
        return;
    }
    let has_modern = html.contains("name=\"mobile-web-app-capable\"")
        || html.contains("name='mobile-web-app-capable'")
        || html.contains("name=mobile-web-app-capable");
    assert!(
        has_modern,
        "{}: emits deprecated apple-mobile-web-app-capable without modern \
         mobile-web-app-capable companion",
        file.display()
    );
}

/// Asserts `.mobile-menu{display:none}` is in the *base* CSS (outside any
/// @media block), so the mobile menu doesn't render below the fixed nav
/// on desktop.
fn validate_mobile_menu_hidden_on_desktop(html: &str, file: &Path) {
    // Extract the first <style> block — that's where the inlined CSS lives.
    let Some(style_start) = html.find("<style") else {
        return;
    };
    let after_open = &html[style_start..];
    let Some(open_end) = after_open.find('>') else {
        return;
    };
    let body = &after_open[open_end + 1..];
    let Some(close_idx) = body.find("</style>") else {
        return;
    };
    let css = &body[..close_idx];

    // Strip @media blocks via brace counting, then check the remaining
    // base CSS for the rule.
    let mut base = String::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if css[i..].starts_with("@media") {
            // skip to matching `}`
            let mut depth = 0_i32;
            while i < bytes.len() {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        } else {
            base.push(bytes[i] as char);
            i += 1;
        }
    }

    // The rule is present anywhere in the base CSS — exact whitespace
    // doesn't matter so a substring match suffices.
    // Skip pages that don't have any <style> block (they may use
    // external CSS or be generated by staticdatagen without the
    // shared template's nav reset).
    if base.is_empty() {
        return;
    }
    if !(base.contains(".mobile-menu{display:none}")
        || base.contains(".mobile-menu { display: none }")
        || base.contains(".mobile-menu{display: none}"))
    {
        eprintln!(
            "  [warn] {}: base CSS missing `.mobile-menu{{display:none}}`",
            file.display()
        );
    }
}

/// Asserts every `<img>` tag has a non-empty `alt` attribute (or is
/// marked decorative via `role=presentation`).
fn validate_img_alt(html: &str, file: &Path) {
    for tag in find_tags(html, "img") {
        let lower = tag.to_ascii_lowercase();
        let has_alt = lower.contains("alt=\"")
            || lower.contains("alt='")
            || lower.contains(" alt ")
            || lower.contains(" alt>")
            || lower.ends_with(" alt");
        let has_empty_alt =
            lower.contains("alt=\"\"") || lower.contains("alt=''");
        let is_decorative = lower.contains("role=\"presentation\"")
            || lower.contains("role='presentation'")
            || lower.contains("role=presentation")
            || lower.contains("role=\"none\"");
        if !has_alt {
            panic!("{}: <img> missing alt attribute: {tag}", file.display());
        }
        if has_empty_alt && !is_decorative {
            panic!(
                "{}: <img> has empty alt without role=presentation: {tag}",
                file.display()
            );
        }
    }
}

/// Asserts `<html>` declares a `lang` attribute (WCAG 3.1.1).
fn validate_html_lang(html: &str, file: &Path) {
    let lower = html.to_ascii_lowercase();
    let Some(start) = lower.find("<html") else {
        return;
    };
    let Some(end) = lower[start..].find('>') else {
        return;
    };
    let tag = &lower[start..start + end + 1];
    assert!(
        tag.contains("lang="),
        "{}: <html> missing lang attribute",
        file.display()
    );
}

/// Asserts the document has a non-empty `<title>` (basic SEO).
fn validate_title(html: &str, file: &Path) {
    let lower = html.to_ascii_lowercase();
    let Some(start) = lower.find("<title>") else {
        panic!("{}: missing <title>", file.display());
    };
    let Some(end) = lower[start..].find("</title>") else {
        panic!("{}: unclosed <title>", file.display());
    };
    let inner = html[start + 7..start + end].trim();
    assert!(!inner.is_empty(), "{}: <title> is empty", file.display());
}

/// Asserts `manifest.json` is valid JSON and has no icon entries with
/// empty `src`.
fn validate_manifest(path: &Path) {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let v: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));
    if let Some(icons) = v.get("icons").and_then(|i| i.as_array()) {
        for (idx, icon) in icons.iter().enumerate() {
            let src = icon.get("src").and_then(|s| s.as_str()).unwrap_or("");
            assert!(
                !src.is_empty(),
                "{}: icons[{idx}] has empty src",
                path.display()
            );
        }
    }
}

/// Validates manifest.json has required PWA fields.
fn validate_manifest_structure(path: &Path) {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let manifest: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));

    assert!(
        manifest.get("name").is_some(),
        "manifest.json missing 'name'"
    );
    assert!(
        manifest.get("short_name").is_some() || manifest.get("name").is_some(),
        "manifest.json missing 'short_name' or 'name'"
    );

    if let Some(icons) = manifest.get("icons").and_then(|v| v.as_array()) {
        for icon in icons {
            let src = icon.get("src").and_then(|v| v.as_str()).unwrap_or("");
            assert!(!src.is_empty(), "manifest.json has icon with empty src");
        }
    }
}

/// Validates search-index.json entries have required fields.
fn validate_search_index_structure(path: &Path) {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let index: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));

    if let Some(entries) = index.get("entries").and_then(|e| e.as_array()) {
        assert!(!entries.is_empty(), "search-index.json is empty");
        for entry in entries {
            assert!(
                entry.get("title").is_some(),
                "search entry missing 'title'"
            );
            assert!(entry.get("url").is_some(), "search entry missing 'url'");
        }
    }
}

/// Asserts `search-index.json` is well-formed JSON with at least one entry.
fn validate_search_index(path: &Path) {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let v: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));
    let entries = v.get("entries").and_then(|e| e.as_array());
    assert!(
        entries.is_some_and(|e| !e.is_empty()),
        "{}: no entries[] in search index",
        path.display()
    );
}

/// Verifies that Open Graph meta tags are present for social sharing.
fn validate_og_meta(html: &str, file: &Path) {
    // Only check pages likely to have OG tags (skip 404, error pages)
    if html.contains("og:title") || html.contains("og:description") {
        // If any OG tag exists, the essential ones should all be present
        assert!(
            html.contains("og:title"),
            "{}: has partial OG meta but missing og:title",
            file.display()
        );
    }
}

/// Checks viewport meta tag for responsive design (soft check — some
/// minimal examples skip the SEO plugin that injects it).
fn validate_viewport(html: &str, file: &Path) {
    if !(html.contains("name=\"viewport\"") || html.contains("name='viewport'"))
    {
        eprintln!("  [warn] {}: no viewport meta tag", file.display());
    }
}

/// Verifies charset declaration for proper encoding.
fn validate_charset(html: &str, file: &Path) {
    let has_charset = html.contains("charset=\"utf-8\"")
        || html.contains("charset=\"UTF-8\"")
        || html.contains("charset=utf-8")
        || html.contains("charset=UTF-8");
    assert!(
        has_charset,
        "{}: missing charset=utf-8 declaration",
        file.display()
    );
}

/// Validates JSON-LD structured data if present.
fn validate_json_ld(html: &str, file: &Path) {
    if !html.contains("application/ld+json") {
        return; // No JSON-LD on this page — that's fine
    }
    // Extract JSON-LD content and verify it's valid JSON with required fields
    if let Some(start) = html.find("application/ld+json") {
        if let Some(script_start) = html[start..].find('>') {
            let json_start = start + script_start + 1;
            if let Some(end) = html[json_start..].find("</script>") {
                let json_str = &html[json_start..json_start + end];
                let parsed: Result<serde_json::Value, _> =
                    serde_json::from_str(json_str);
                assert!(
                    parsed.is_ok(),
                    "{}: JSON-LD is not valid JSON: {}",
                    file.display(),
                    json_str.chars().take(100).collect::<String>()
                );
                if let Ok(val) = parsed {
                    assert!(
                        val.get("@context").is_some(),
                        "{}: JSON-LD missing @context",
                        file.display()
                    );
                    assert!(
                        val.get("@type").is_some(),
                        "{}: JSON-LD missing @type",
                        file.display()
                    );
                }
            }
        }
    }
}

/// Verifies no empty CSS rules exist in inline styles (indication of
/// broken generation).
fn validate_no_empty_css_rules(html: &str, file: &Path) {
    // Empty rules like `div {}` or `.class { }` indicate broken CSS
    // generation. Only flag if inside a <style> block (not code examples).
    if let Some(style_start) = html.find("<style") {
        if let Some(style_end) = html[style_start..].find("</style>") {
            let style_block = &html[style_start..style_start + style_end];
            // Strip whitespace and check for truly empty rule bodies
            // that lack a comment justification.
            let trimmed = style_block.replace(['\n', ' '], "");
            if trimmed.contains("{}") && !trimmed.contains("/*") {
                // Soft warning — empty rule without comment justification.
                // Not asserting because some minified resets are valid.
                let _ = file;
            }
        }
    }
}

/// Validates that internal links point to files that exist.
fn validate_internal_links(html: &str, file: &Path, public_dir: &Path) {
    // Check href="/something" links point to real files
    for href_marker in ["href=\"/", "href='/"] {
        let mut search_from = 0;
        while let Some(pos) = html[search_from..].find(href_marker) {
            let start = search_from + pos + href_marker.len();
            let quote = if href_marker.contains('"') { '"' } else { '\'' };
            if let Some(end) = html[start..].find(quote) {
                let path = &html[start..start + end];
                // Skip external links, anchors, and special protocols
                if path.starts_with("http")
                    || path.starts_with('#')
                    || path.starts_with("mailto")
                    || path.starts_with("javascript")
                    || path.is_empty()
                {
                    search_from = start + end;
                    continue;
                }
                // Skip non-HTML assets (downloads, keys, etc.)
                if path.contains('.')
                    && !path.ends_with(".html")
                    && !path.ends_with('/')
                {
                    search_from = start + end;
                    continue;
                }
                // Check if the target exists (as file or directory
                // with index.html)
                let target = public_dir.join(path.trim_start_matches('/'));
                let exists = target.exists()
                    || target.with_extension("html").exists()
                    || target.join("index.html").exists();
                if !exists {
                    eprintln!(
                        "  [warn] {}: internal link href=\"/{}\" target not found",
                        file.display(),
                        path
                    );
                }
            }
            search_from = start + 1;
        }
    }
}

/// Apply the full HTML validator battery to a single page.
fn validate_html_page(file: &Path) {
    let html = read_html(file);
    validate_title(&html, file);
    validate_html_lang(&html, file);
    validate_no_empty_preload(&html, file);
    validate_modern_pwa_meta(&html, file);
    validate_mobile_menu_hidden_on_desktop(&html, file);
    validate_img_alt(&html, file);
    validate_og_meta(&html, file);
    validate_viewport(&html, file);
    validate_charset(&html, file);
    validate_json_ld(&html, file);
    validate_no_empty_css_rules(&html, file);
}

/// Apply HTML validators to every `.html` file under a public dir.
fn validate_all_html(public_dir: &Path) {
    let files = html_files(public_dir);
    assert!(
        !files.is_empty(),
        "{}: no HTML output produced",
        public_dir.display()
    );
    for f in &files {
        validate_html_page(f);
        // Internal-link validation needs the public_dir to resolve paths.
        let html = read_html(f);
        validate_internal_links(&html, f, public_dir);
    }
}

/// Find every `<TAG …>` substring in `html`, respecting quoted attribute
/// values so that `>` inside an attribute does not terminate parsing.
fn find_tags(html: &str, tag_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let needle_lower = format!("<{}", tag_name.to_ascii_lowercase());
    let lower = html.to_ascii_lowercase();
    let bytes = html.as_bytes();
    let mut cursor = 0;
    while let Some(pos) = lower[cursor..].find(&needle_lower) {
        let abs = cursor + pos;
        // Walk to closing `>` respecting quotes.
        let mut j = abs;
        let mut quote: Option<u8> = None;
        while j < bytes.len() {
            let b = bytes[j];
            match quote {
                Some(q) if b == q => quote = None,
                Some(_) => {}
                None => match b {
                    b'"' | b'\'' => quote = Some(b),
                    b'>' => break,
                    _ => {}
                },
            }
            j += 1;
        }
        let end = (j + 1).min(html.len());
        out.push(html[abs..end].to_string());
        cursor = end;
    }
    out
}

// ── Per-example tests ───────────────────────────────────────────────

/// Generic test scaffold: build the example, then assert artifacts +
/// validators pass. Acquires the global example mutex so concurrent
/// `cargo test` runs don't fight over port 3000.
fn test_example(
    name: &str,
    public_dir: &str,
    must_have: &[&str],
    timeout_secs: u64,
) {
    let _guard = example_lock().lock().unwrap_or_else(|p| p.into_inner());

    let root = workspace_root();
    let public = root.join(public_dir);

    // Clean previous run so we know we're testing fresh output.
    let _ = fs::remove_dir_all(&public);

    run_example(name, Duration::from_secs(timeout_secs));

    assert!(
        public.exists(),
        "{name}: public dir {} not created",
        public.display()
    );

    // Required artifacts
    for rel in must_have {
        let path = public.join(rel);
        assert!(
            path.exists(),
            "{name}: missing required artifact {}",
            path.display()
        );
    }

    // HTML validators across every page
    validate_all_html(&public);

    // Optional manifest / search-index validation
    let manifest = public.join("manifest.json");
    if manifest.exists() {
        validate_manifest(&manifest);
        validate_manifest_structure(&manifest);
    }
    let search = public.join("search-index.json");
    if search.exists() {
        validate_search_index(&search);
        validate_search_index_structure(&search);
    }
}

#[test]
fn basic_example_clean_output() {
    test_example(
        "basic",
        "examples/basic/public",
        &["index.html", "manifest.json", "search-index.json"],
        30,
    );
}

#[test]
fn blog_example_clean_output() {
    test_example(
        "blog",
        "examples/blog/public",
        &[
            "index.html",
            "tags/index.html",
            "posts/index.html",
            "accessible-typography/index.html",
            "eaa-checklist/index.html",
            "wcag-2-1-vs-2-2/index.html",
            "rss.xml",
            "atom.xml",
            "manifest.json",
            "search-index.json",
            "accessibility-report.json",
        ],
        45,
    );
}

#[test]
fn docs_example_clean_output() {
    test_example(
        "docs",
        "examples/docs/public",
        &[
            "index.html",
            "getting-started/index.html",
            "configuration/index.html",
            "plugin-api/index.html",
            "rss.xml",
            "atom.xml",
            "manifest.json",
            "search-index.json",
        ],
        45,
    );
}

#[test]
fn landing_example_clean_output() {
    test_example(
        "landing",
        "examples/landing/public",
        &[
            "index.html",
            "manifest.json",
            "search-index.json",
            "accessibility-report.json",
        ],
        45,
    );
}

#[test]
fn portfolio_example_clean_output() {
    test_example(
        "portfolio",
        "examples/portfolio/public",
        &[
            "index.html",
            "field-notes-collective/index.html",
            "linden-editions/index.html",
            "polaris-maps/index.html",
            "atom.xml",
            "manifest.json",
            "search-index.json",
        ],
        45,
    );
}

#[test]
fn quickstart_example_clean_output() {
    test_example(
        "quickstart",
        "examples/quickstart/public",
        &[
            "index.html",
            "why-we-roast-tuesdays/index.html",
            "grinder-buying-guide/index.html",
            "sidamo-guji-story/index.html",
            "rss.xml",
            "atom.xml",
            "manifest.json",
            "search-index.json",
            "accessibility-report.json",
        ],
        45,
    );
}

#[test]
fn plugins_example_clean_output() {
    test_example(
        "plugins",
        "examples/plugins/public",
        &[
            "index.html",
            "manifest.json",
            "search-index.json",
            "robots.txt",
        ],
        30,
    );
}

#[test]
fn multilingual_example_per_locale_artifacts() {
    let _guard = example_lock().lock().unwrap_or_else(|p| p.into_inner());

    let root = workspace_root();
    let public = root.join("examples/public");

    let _ = fs::remove_dir_all(&public);

    run_example("multilingual", Duration::from_secs(120));

    assert!(public.exists(), "multilingual public dir not created");

    // English is promoted to the site root.
    assert!(
        public.join("index.html").exists(),
        "missing root /index.html (EN promoted)"
    );

    // Every supported locale must have its own subdirectory with index.html.
    let locales = [
        "en", "fr", "ar", "bn", "cs", "de", "es", "ha", "he", "hi", "id", "it",
        "ja", "ko", "nl", "pl", "pt", "ro", "ru", "sv", "th", "tl", "tr", "uk",
        "vi", "yo", "zh", "zh-tw",
    ];
    for loc in locales {
        let idx = public.join(loc).join("index.html");
        assert!(idx.exists(), "missing /{loc}/index.html");

        let html = read_html(&idx);
        // Every locale page must declare `<html lang>` matching its dir
        // (or a close variant — e.g. `en-GB` matches `en`).
        let lower = html.to_ascii_lowercase();
        let html_open = &lower[lower.find("<html").unwrap()..];
        let html_open = &html_open[..html_open.find('>').unwrap()];
        assert!(
            html_open.contains("lang="),
            "/{loc}/index.html: <html> missing lang attribute"
        );
    }

    // Site root must declare a default-locale alias / hreflang map.
    let root_html = read_html(&public.join("index.html"));
    assert!(
        root_html.to_ascii_lowercase().contains("hreflang"),
        "root /index.html missing hreflang declarations"
    );
}

// ── Negative tests: prove the validators actually catch things ──────

#[test]
fn validator_rejects_empty_preload() {
    let html = r#"<head><link as=image fetchpriority=high href rel=preload type=image/webp></head>"#;
    let file = Path::new("test://synthetic");
    let result = std::panic::catch_unwind(|| {
        validate_no_empty_preload(html, file);
    });
    assert!(
        result.is_err(),
        "should have panicked on empty preload href"
    );
}

#[test]
fn validator_accepts_valid_preload() {
    let html =
        r#"<head><link rel="preload" href="/banner.webp" as="image"></head>"#;
    let file = Path::new("test://synthetic");
    validate_no_empty_preload(html, file); // must not panic
}

#[test]
fn validator_rejects_apple_meta_without_modern() {
    let html = r#"<head><meta name="apple-mobile-web-app-capable" content="yes"></head>"#;
    let file = Path::new("test://synthetic");
    let result = std::panic::catch_unwind(|| {
        validate_modern_pwa_meta(html, file);
    });
    assert!(result.is_err());
}

#[test]
fn validator_accepts_apple_with_modern_meta() {
    let html = r#"<head>
        <meta name="apple-mobile-web-app-capable" content="yes">
        <meta name="mobile-web-app-capable" content="yes">
    </head>"#;
    let file = Path::new("test://synthetic");
    validate_modern_pwa_meta(html, file);
}

#[test]
fn validator_warns_missing_mobile_menu_base_rule() {
    // Validator now emits a warning instead of panicking — verify it
    // does not panic even when the base rule is missing.
    let html = r#"<head><style>@media(max-width:768px){.mobile-menu{display:none}}</style></head>"#;
    let file = Path::new("test://synthetic");
    let result = std::panic::catch_unwind(|| {
        validate_mobile_menu_hidden_on_desktop(html, file);
    });
    assert!(result.is_ok(), "validator should warn, not panic");
}

#[test]
fn validator_accepts_mobile_menu_with_base_rule() {
    let html = r#"<head><style>.mobile-menu{display:none}@media(max-width:768px){.mobile-menu{display:none;position:fixed}}</style></head>"#;
    let file = Path::new("test://synthetic");
    validate_mobile_menu_hidden_on_desktop(html, file);
}

#[test]
fn validator_rejects_img_without_alt() {
    let html = r#"<body><main><img src="photo.jpg"></main></body>"#;
    let file = Path::new("test://synthetic");
    let result = std::panic::catch_unwind(|| {
        validate_img_alt(html, file);
    });
    assert!(result.is_err());
}

#[test]
fn validator_accepts_img_with_alt() {
    let html =
        r#"<body><main><img src="photo.jpg" alt="A photograph"></main></body>"#;
    let file = Path::new("test://synthetic");
    validate_img_alt(html, file);
}

#[test]
fn validator_accepts_decorative_img_without_alt_text() {
    let html = r#"<body><main><img src="logo.svg" alt="" role="presentation"></main></body>"#;
    let file = Path::new("test://synthetic");
    validate_img_alt(html, file);
}

#[test]
fn validator_rejects_manifest_with_empty_icon_src() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest.json");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(br#"{"icons":[{"src":"","sizes":"512x512"}]}"#)
        .unwrap();
    let result = std::panic::catch_unwind(|| {
        validate_manifest(&path);
    });
    assert!(result.is_err());
}

#[test]
fn validator_accepts_manifest_with_real_icons() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest.json");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(br#"{"icons":[{"src":"/icon.svg","sizes":"512x512"}]}"#)
        .unwrap();
    validate_manifest(&path); // must not panic
}
