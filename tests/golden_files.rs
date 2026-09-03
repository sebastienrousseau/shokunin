// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Golden-file regression framework (resolves #466 — framework phase).
//!
//! ## How it works
//!
//! 1. Each test scaffolds a deterministic input under a tempdir using
//!    `ssg::scaffold::scaffold_project_at`, runs `compile_site`, then
//!    walks the produced site directory.
//! 2. Each generated artifact is normalised (whitespace folded, build
//!    timestamps stripped, ISO 8601 dates and content-fingerprint
//!    hashes regex-replaced with placeholders) before comparison.
//! 3. The normalised artifact is compared against a checked-in
//!    "golden" file in `tests/golden/`. Any diff fails the test.
//!
//! ## Updating goldens
//!
//! Set `UPDATE_GOLDEN=1` in the environment:
//!
//! ```sh
//! UPDATE_GOLDEN=1 cargo test --test golden_files
//! ```
//!
//! The test will overwrite the golden file with the current
//! normalised output instead of asserting equality. Review the diff
//! in `git diff tests/golden/` before committing.
//!
//! ## Phase scope
//!
//! The framework ships with golden files for every file the
//! scaffold writes (11 deterministic templates) plus an end-to-end
//! compile-and-emit golden that runs `compile_site` on a minimal
//! one-page fixture and goldens the produced `sitemap.xml`. Further
//! example-driven goldens (atom.xml, news-sitemap.xml, json-feed.json,
//! search-index.json, sbom.cdx.json, accessibility-report.json) are
//! emitted as separate goldens against the bundled `examples/`
//! output in `tests/example_outputs.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// Returns the path to the `tests/golden/` directory.
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Replaces non-deterministic substrings with stable placeholders so
/// goldens are comparable across runs and machines.
///
/// Substitutions (in order):
/// - ISO 8601 datetimes (`2026-05-10T12:34:56Z` → `<DATE>`)
/// - ISO 8601 dates (`2026-05-10` → `<DATE>`)
/// - 8-char hex content hashes (`a1b2c3d4` between `.` and `.`)
///   → `<HASH>`
/// - SHA-* SRI hashes (`sha256-...`, `sha384-...`) → `<SRI>`
/// - Trailing whitespace stripped per line.
/// - CRLF → LF (Windows runners).
fn normalise(input: &str) -> String {
    // Cheap, regex-free passes — keeps the framework dep-light.
    let mut s = input.replace("\r\n", "\n");

    // ISO datetimes (must run before bare dates).
    s = strip_iso_datetimes(&s);
    s = strip_iso_dates(&s);

    // Content fingerprint: <stem>.<8 hex>.<ext>
    s = strip_fingerprint_hashes(&s);

    // SRI hashes: sha{256,384,512}-<base64-or-hex>
    s = strip_sri(&s);

    // Trailing whitespace.
    let mut out = String::with_capacity(s.len());
    for line in s.split('\n') {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    while out.ends_with("\n\n") {
        let _ = out.pop();
    }
    out
}

fn strip_iso_datetimes(s: &str) -> String {
    // YYYY-MM-DDTHH:MM:SS[.fff][Z|+HH:MM]
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Only attempt the ASCII slice if the 19-byte window is fully
        // ASCII AND is a char boundary on both ends — otherwise the
        // byte slice would split a multibyte UTF-8 char (em-dash etc).
        if i + 19 <= bytes.len()
            && s.is_char_boundary(i)
            && s.is_char_boundary(i + 19)
            && looks_like_iso_datetime(&s[i..i + 19])
        {
            out.push_str("<DATE>");
            let mut j = 19;
            // Optional fractional seconds .fff
            if bytes.get(i + j) == Some(&b'.') {
                j += 1;
                while i + j < bytes.len() && bytes[i + j].is_ascii_digit() {
                    j += 1;
                }
            }
            // Optional Z or ±HH:MM
            if bytes.get(i + j) == Some(&b'Z') {
                j += 1;
            } else if (bytes.get(i + j) == Some(&b'+')
                || bytes.get(i + j) == Some(&b'-'))
                && i + j + 6 <= bytes.len()
            {
                j += 6;
            }
            i += j;
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn looks_like_iso_datetime(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 19
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[10] == b'T'
        && b[11..13].iter().all(u8::is_ascii_digit)
        && b[13] == b':'
        && b[14..16].iter().all(u8::is_ascii_digit)
        && b[16] == b':'
        && b[17..19].iter().all(u8::is_ascii_digit)
}

fn strip_iso_dates(s: &str) -> String {
    // YYYY-MM-DD with no time component following.
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 10 <= chars.len() && looks_like_iso_date(&chars[i..i + 10]) {
            // Don't double-strip if surrounded by `<DATE>` already.
            out.push_str("<DATE>");
            i += 10;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn looks_like_iso_date(c: &[char]) -> bool {
    c.len() == 10
        && c[..4].iter().all(char::is_ascii_digit)
        && c[4] == '-'
        && c[5..7].iter().all(char::is_ascii_digit)
        && c[7] == '-'
        && c[8..10].iter().all(char::is_ascii_digit)
}

fn strip_fingerprint_hashes(s: &str) -> String {
    // Match `.<8 hex>.<ext>` where ext is one of our fingerprinted
    // extensions. Cheap: walk by '.' anchor.
    let exts = [
        "css", "js", "mjs", "png", "jpg", "jpeg", "webp", "avif", "gif", "svg",
        "woff", "woff2", "ttf", "otf",
    ];
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 9 < bytes.len() {
            let hex = &s[i + 1..i + 9];
            if hex.bytes().all(|b| b.is_ascii_hexdigit())
                && bytes.get(i + 9) == Some(&b'.')
            {
                let after = &s[i + 10..];
                if let Some(ext_end) =
                    after.find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                {
                    let ext = &after[..ext_end];
                    if exts.contains(&ext) {
                        out.push_str(".<HASH>.");
                        out.push_str(ext);
                        i += 10 + ext.len();
                        continue;
                    }
                } else if exts.contains(&after) {
                    out.push_str(".<HASH>.");
                    out.push_str(after);
                    i = bytes.len();
                    continue;
                }
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn strip_sri(s: &str) -> String {
    // Replaces every `sha{256,384,512}-<value>` occurrence with `<SRI>`.
    // The value runs until a quote, whitespace, or angle bracket.
    //
    // Implementation: find each prefix via str::find in turn, copy
    // the prefix-free chunk verbatim, emit `<SRI>`, skip the value.
    // No interleaved index control between fast/slow paths — every
    // iteration consumes either an SRI hash or zero characters
    // (then advances by one to make progress).
    const PREFIXES: &[&str] = &["sha256-", "sha384-", "sha512-"];
    const VALUE_TERMINATORS: &[char] = &['"', '\'', ' ', '\n', '\t', '<', '>'];

    let mut out = String::with_capacity(s.len());
    let mut remaining = s;
    loop {
        // Find the earliest match across all three prefixes.
        let next_match = PREFIXES
            .iter()
            .filter_map(|p| remaining.find(p).map(|i| (i, *p)))
            .min_by_key(|(i, _)| *i);

        match next_match {
            None => {
                out.push_str(remaining);
                return out;
            }
            Some((idx, prefix)) => {
                out.push_str(&remaining[..idx]);
                out.push_str("<SRI>");
                let after_prefix = &remaining[idx + prefix.len()..];
                let value_end = after_prefix
                    .find(VALUE_TERMINATORS)
                    .unwrap_or(after_prefix.len());
                remaining = &after_prefix[value_end..];
            }
        }
    }
}

/// Compares `actual` (as normalised) against the golden file at
/// `tests/golden/<name>`. On `UPDATE_GOLDEN=1`, overwrites the
/// golden instead of asserting.
fn assert_or_update_golden(name: &str, actual: &str) {
    let normalised = normalise(actual);
    let golden_path = golden_dir().join(name);

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::create_dir_all(golden_dir()).unwrap();
        fs::write(&golden_path, &normalised).unwrap_or_else(|e| {
            panic!("UPDATE_GOLDEN write failed for {name}: {e}")
        });
        eprintln!(
            "[golden] updated {} ({} bytes)",
            golden_path.display(),
            normalised.len()
        );
        return;
    }

    let expected = fs::read_to_string(&golden_path).unwrap_or_else(|e| {
        panic!(
            "golden file {} missing or unreadable: {e}\n\
             Run `UPDATE_GOLDEN=1 cargo test --test golden_files` to seed.",
            golden_path.display()
        )
    });

    if expected != normalised {
        // Inline diff that prints both sides truncated to the first
        // 60 differing lines so CI logs stay scannable.
        let mut msg = format!(
            "golden mismatch for {}\n\n\
             === expected ===\n{}\n\
             === actual ===\n{}\n",
            golden_path.display(),
            expected.chars().take(2_000).collect::<String>(),
            normalised.chars().take(2_000).collect::<String>(),
        );
        if expected.len() > 2_000 || normalised.len() > 2_000 {
            msg.push_str("...(truncated; review the full files)\n");
        }
        panic!("{msg}");
    }
}

// =====================================================================
// One end-to-end golden: scaffold_project_at output stability
// =====================================================================
//
// Issue #466 asks for 50+ golden files spread across the 8 examples.
// We seed exactly one here as proof the framework works; the other
// 49 land incrementally so reviewers can sign off on each batch's
// diff without one mega-PR.

/// Scaffolds a deterministic project tree under a fresh tempdir and
/// returns its root, used by every scaffold-output golden test.
fn scaffold_into_tempdir() -> (tempfile::TempDir, PathBuf) {
    use ssg::scaffold::scaffold_project_at;
    let dir = tempfile::tempdir().unwrap();
    scaffold_project_at("golden-test-site", dir.path())
        .expect("scaffold project");
    let root = dir.path().join("golden-test-site");
    (dir, root)
}

/// Goldens one file under the scaffold root.
fn golden_scaffold_file(rel: &str, golden_name: &str) {
    let (_keep, root) = scaffold_into_tempdir();
    let path = root.join(rel);
    let body = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("scaffold did not produce {}: {e}", path.display())
    });
    assert_or_update_golden(golden_name, &body);
}

#[test]
fn scaffold_config_toml_stays_stable() {
    golden_scaffold_file("config.toml", "scaffold_config_toml.golden");
}

#[test]
fn scaffold_content_index_md_stays_stable() {
    golden_scaffold_file(
        "content/index.md",
        "scaffold_content_index_md.golden",
    );
}

#[test]
fn scaffold_content_about_md_stays_stable() {
    golden_scaffold_file(
        "content/about.md",
        "scaffold_content_about_md.golden",
    );
}

#[test]
fn scaffold_content_blog_first_post_md_stays_stable() {
    golden_scaffold_file(
        "content/blog/first-post.md",
        "scaffold_content_blog_first_post_md.golden",
    );
}

#[test]
fn scaffold_template_base_html_stays_stable() {
    golden_scaffold_file(
        "templates/tera/base.html",
        "scaffold_template_base_html.golden",
    );
}

#[test]
fn scaffold_template_page_html_stays_stable() {
    golden_scaffold_file(
        "templates/tera/page.html",
        "scaffold_template_page_html.golden",
    );
}

#[test]
fn scaffold_template_post_html_stays_stable() {
    golden_scaffold_file(
        "templates/tera/post.html",
        "scaffold_template_post_html.golden",
    );
}

#[test]
fn scaffold_template_index_html_stays_stable() {
    golden_scaffold_file(
        "templates/tera/index.html",
        "scaffold_template_index_html.golden",
    );
}

#[test]
fn scaffold_static_css_style_stays_stable() {
    golden_scaffold_file(
        "static/css/style.css",
        "scaffold_static_css_style.golden",
    );
}

#[test]
fn scaffold_data_nav_toml_stays_stable() {
    golden_scaffold_file("data/nav.toml", "scaffold_data_nav_toml.golden");
}

/// End-to-end: scaffold a project, run `compile_site`, golden a stable
/// output artefact. Uses `sitemap.xml` because it's pure XML built from
/// content + frontmatter and contains no machine-specific paths.
#[test]
fn end_to_end_compile_site_sitemap_xml_stays_stable() {
    use ssg::compile_site;
    let (_keep, root) = scaffold_into_tempdir();

    let content = root.join("content");
    let build = root.join("build");
    let site = root.join("public");

    // `compile_site` does not create its own output directories, and the
    // scaffold's own templates do not drive it — see the note above the
    // test. Both are why the old body's skip triggered on every run.
    fs::create_dir_all(&build).expect("create build dir");
    fs::create_dir_all(&site).expect("create site dir");

    // Use the bundled template set rather than the scaffolded one. The
    // scaffold writes MiniJinja templates under `templates/tera/` and
    // nothing at the template-dir root, while this pipeline needs the
    // root set (`template.html`, `main.js`, `sw.js`, ...). Pointing at a
    // template set that actually renders is what lets this test assert
    // something; goldening the scaffold's own build has to wait for the
    // scaffold to produce a buildable project.
    let template =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/templates/en");
    assert!(
        template.join("template.html").is_file(),
        "bundled templates missing at {}",
        template.display()
    );

    compile_site(&build, &content, &site, &template)
        .expect("compile_site on the deterministic scaffold");

    // Every artefact #466 names that this scaffold actually produces.
    // Asserting the set is non-empty is what stops this silently
    // becoming a no-op again if the pipeline stops emitting them.
    let artefacts: &[(&str, &str)] = &[
        ("sitemap.xml", "end_to_end_sitemap_xml.golden"),
        ("robots.txt", "end_to_end_robots_txt.golden"),
        ("manifest.json", "end_to_end_manifest_json.golden"),
        ("index.html", "end_to_end_index_html.golden"),
        ("rss.xml", "end_to_end_rss_xml.golden"),
        ("news-sitemap.xml", "end_to_end_news_sitemap_xml.golden"),
        ("humans.txt", "end_to_end_humans_txt.golden"),
    ];

    let mut goldened = 0_usize;
    for (rel, golden) in artefacts {
        let path = site.join(rel);
        if !path.exists() {
            continue;
        }
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert_or_update_golden(golden, &body);
        goldened += 1;
    }

    assert!(
        goldened >= 3,
        "only {goldened} build artefact(s) were goldened. The scaffold \
         build is deterministic, so this means the pipeline stopped \
         emitting output the golden suite was pinning — which is exactly \
         the regression #466 exists to catch, not a reason to skip."
    );
}

// =====================================================================
// Unit tests for the normalisation helpers
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_strips_iso_datetimes() {
        let input = "<lastBuildDate>2026-05-10T12:34:56Z</lastBuildDate>";
        let out = normalise(input);
        assert!(out.contains("<DATE>"));
        assert!(!out.contains("2026-05-10T"));
    }

    #[test]
    fn normalise_strips_bare_iso_dates() {
        let input = "Published 2026-05-10 today";
        let out = normalise(input);
        assert!(out.contains("<DATE>"));
        assert!(!out.contains("2026-05-10"));
    }

    #[test]
    fn normalise_strips_fingerprint_hashes() {
        let input = "<link href=\"/style.a1b2c3d4.css\">";
        let out = normalise(input);
        assert!(out.contains("/style.<HASH>.css"));
    }

    #[test]
    fn normalise_strips_sri_hashes() {
        let input = "integrity=\"sha256-abcDEF123456==\"";
        let out = normalise(input);
        assert!(out.contains("<SRI>"));
        assert!(!out.contains("abcDEF123456"));
    }

    #[test]
    fn normalise_collapses_crlf_to_lf() {
        assert_eq!(normalise("a\r\nb\r\n"), "a\nb\n");
    }

    #[test]
    fn normalise_strips_trailing_whitespace() {
        assert_eq!(normalise("a   \nb\t\nc"), "a\nb\nc\n");
    }
}
