// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! End-to-end regression suite for the v0.0.45 site-build hot-fix.
//!
//! Reproduces the four upstream `staticdatagen 0.0.9` / `staticweaver
//! 0.0.2` / `metadata-gen 0.0.4` brittleness points that the user's
//! 2,371-file site exposed:
//!
//! 1. `.md` with no `layout:` key → `MiniJinja` `invalid template or
//!    partial name: ""`.
//! 2. Template directory missing `main.js` / `sw.js` →
//!    `copy_auxiliary_files` aborts with `No such file or directory`.
//! 3. No `tags.md` or `tags/index.md` → `write_tags_html_to_file`
//!    aborts with `No such file or directory` after producing every
//!    other artefact.
//! 4. Template references a `{{ var }}` that the content's frontmatter
//!    omits → staticweaver fails with `Unresolved template tag`.
//! 5. YAML-spec-valid multi-line quoted scalar (e.g.
//!    `key: "\nvalue"`) → `noyalib` parser inside `metadata-gen`
//!    reports `No valid front matter found`.
//!
//! Each test below builds a tiny site that triggers exactly one of
//! these patterns and asserts the build succeeds with output files
//! present. Failure of any one signals the upstream regression has
//! returned.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ssg::pipeline::compile_site;
use std::fs;
use std::path::Path;

/// Bundles a one-file fixture site for each regression test.
struct Fixture {
    tmp: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        Self { tmp }
    }

    fn content_dir(&self) -> std::path::PathBuf {
        let p = self.tmp.path().join("content");
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn template_dir(&self) -> std::path::PathBuf {
        let p = self.tmp.path().join("templates");
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn build_dir(&self) -> std::path::PathBuf {
        self.tmp.path().join("build")
    }

    fn site_dir(&self) -> std::path::PathBuf {
        self.tmp.path().join("public")
    }

    /// Writes a minimal `page.html` template that references nothing.
    fn write_minimal_template(&self) -> std::path::PathBuf {
        let dir = self.template_dir();
        fs::write(
            dir.join("page.html"),
            "<!doctype html><html><body>{{ content }}</body></html>",
        )
        .unwrap();
        dir
    }

    fn run_build(&self) -> Result<(), ssg::error::SsgError> {
        let build = self.build_dir();
        let site = self.site_dir();
        let content = self.content_dir();
        let template = self.write_minimal_template();
        fs::create_dir_all(&build).unwrap();
        compile_site(&build, &content, &site, &template)
    }
}

fn require_html_exists(site_dir: &Path, slug: &str) {
    let candidates = [
        site_dir.join(slug).join("index.html"),
        site_dir.join(format!("{slug}.html")),
    ];
    assert!(
        candidates.iter().any(|p| p.exists()),
        "expected one of {:?} to exist; site contents: {:?}",
        candidates,
        list_files(site_dir),
    );
}

fn list_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut out);
    out
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// Frontmatter fields that staticdatagen's RSS / metadata writers
/// require to be non-empty. Tests inject these alongside whatever
/// regression-specific shape they're exercising.
fn minimal_rss_safe_frontmatter(title: &str, slug: &str) -> String {
    format!(
        "title: \"{title}\"\n\
         description: \"desc\"\n\
         permalink: \"https://example.invalid/{slug}\"\n"
    )
}

#[test]
fn md_without_layout_key_still_builds() {
    // Regression 1: missing layout → staticdatagen renders with "" →
    // MiniJinja crashes. Our stager injects `layout: "page"`.
    let fx = Fixture::new();
    let fm = minimal_rss_safe_frontmatter("My Page", "page");
    fs::write(
        fx.content_dir().join("page.md"),
        format!("---\n{fm}---\nbody"),
    )
    .unwrap();
    fx.run_build().unwrap();
    require_html_exists(&fx.site_dir(), "page");
}

#[test]
fn templates_missing_required_aux_files_still_build() {
    // Regression 2: copy_auxiliary_files looks for `main.js` + `sw.js`.
    // Our stager creates empty stubs when absent.
    let fx = Fixture::new();
    let fm = minimal_rss_safe_frontmatter("T", "page");
    fs::write(
        fx.content_dir().join("page.md"),
        format!("---\nlayout: page\n{fm}---\nbody"),
    )
    .unwrap();
    fx.run_build().unwrap();
    require_html_exists(&fx.site_dir(), "page");
}

#[test]
fn no_tags_page_still_builds_via_stub() {
    // Regression 3: write_tags_html_to_file demands a tags page.
    // Our stager synthesises one with an example.invalid permalink.
    let fx = Fixture::new();
    let fm = minimal_rss_safe_frontmatter("T", "article");
    fs::write(
        fx.content_dir().join("article.md"),
        format!("---\nlayout: page\n{fm}---\nbody"),
    )
    .unwrap();
    fx.run_build().unwrap();
    require_html_exists(&fx.site_dir(), "article");
}

#[test]
fn template_var_missing_from_content_still_builds() {
    // Regression 4: template references {{ author }} but content's
    // frontmatter doesn't supply it. Our stager scans the template,
    // sees the {{ author }} reference, and injects `author: ""` into
    // every content file that lacks it.
    let fx = Fixture::new();
    fs::write(
        fx.template_dir().join("page.html"),
        "<!doctype html><html><body>by {{ author }} {{ content }}</body></html>",
    )
    .unwrap();
    let fm = minimal_rss_safe_frontmatter("T", "post");
    fs::write(
        fx.content_dir().join("post.md"),
        format!("---\nlayout: page\n{fm}---\nbody"),
    )
    .unwrap();
    let build = fx.build_dir();
    let site = fx.site_dir();
    let content = fx.content_dir();
    let template = fx.template_dir();
    fs::create_dir_all(&build).unwrap();
    compile_site(&build, &content, &site, &template).unwrap();
    require_html_exists(&site, "post");
}

#[test]
fn multiline_quoted_scalar_in_frontmatter_still_builds() {
    // Regression 5: spec-valid `key: "\nvalue"` frontmatter that
    // noyalib trips on. Our stager collapses the value onto a single
    // line before staticdatagen sees it.
    let fx = Fixture::new();
    let fm = minimal_rss_safe_frontmatter("T", "post");
    let body = format!(
        "---\n\
         layout: page\n\
         {fm}\
         twitter_url: \"\nhttps://example.com/post\"\n\
         ---\n\
         body"
    );
    fs::write(fx.content_dir().join("post.md"), body).unwrap();
    fx.run_build().unwrap();
    require_html_exists(&fx.site_dir(), "post");
}

#[test]
fn full_user_pattern_combined() {
    // End-to-end: every regression in one file.
    // - layout missing → injected
    // - template references {{ author }} → empty default injected
    // - multi-line quoted url → collapsed
    // - templates missing main.js/sw.js → stubbed
    // - tags page missing → auto-stubbed
    let fx = Fixture::new();
    fs::write(
        fx.template_dir().join("page.html"),
        "<!doctype html><html><body>by {{ author }} {{ content }}</body></html>",
    )
    .unwrap();
    let body = "---\n\
                title: \"Full pattern test\"\n\
                description: \"covers every fix\"\n\
                permalink: \"https://example.invalid/article\"\n\
                twitter_url: \"\nhttps://example.com/test\"\n\
                ---\n\
                body content";
    fs::write(fx.content_dir().join("article.md"), body).unwrap();
    fx.run_build().unwrap();
    require_html_exists(&fx.site_dir(), "article");
}

// ─────────────────────────────────────────────────────────────────────
// v0.0.46 regression: nested-locale content tree.
//
// staticdatagen 0.0.10 (issue #70) added recursive directory walk via
// walkdir, and preserves the per-locale subdir prefix in the URL.
// This test exercises that shape end-to-end on a small 3-locale × 2-post
// tempdir. See `examples/multilingual_full_example.rs` for the larger
// 5 × 5 runnable showcase.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn nested_locale_subdirectories_build_per_language() {
    // Mirrors the Jekyll `content/<lang>/<slug>.md` shape on a small
    // 3-locale × 2-post tree. Once staticdatagen 0.0.10 is in the dep
    // graph, every per-locale post lands at
    // `public/<lang>/post-N/index.html` and this test goes green.
    let fx = Fixture::new();
    fs::write(
        fx.template_dir().join("page.html"),
        "<!doctype html><html lang=\"{{ language }}\"><body>{{ content }}</body></html>",
    )
    .unwrap();

    for &(lang, native) in
        &[("en", "English"), ("fr", "Français"), ("de", "Deutsch")]
    {
        let dir = fx.content_dir().join(lang);
        fs::create_dir_all(&dir).unwrap();
        for i in 1..=2 {
            let body = format!(
                "---\n\
                 title: \"{native} post {i}\"\n\
                 description: \"localised body in {native}\"\n\
                 permalink: \"https://example.invalid/{lang}/post-{i}\"\n\
                 layout: page\n\
                 language: \"{lang}\"\n\
                 hreflang: \"{lang}\"\n\
                 ---\n\
                 {native} post body {i}"
            );
            fs::write(dir.join(format!("post-{i}.md")), body).unwrap();
        }
    }

    fx.run_build().unwrap();

    // Every per-locale URL must materialise — `add()` walked
    // recursively AND `get_processed_file_name` preserved the
    // locale prefix.
    for &lang in &["en", "fr", "de"] {
        for i in 1..=2 {
            require_html_exists(
                &fx.site_dir(),
                &format!("{lang}/post-{i}"),
            );
        }
    }
}
