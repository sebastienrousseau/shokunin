// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Regression guard: no placeholder default may reach a rendered site.
//!
//! ## What went wrong
//!
//! `DEFAULT_SITE_TITLE` was `"My SSG Site"`. A build with no config file falls
//! back to [`ssg::cmd::default_config`], the taxonomy plugin copies
//! `site_title` into `site.title`, and `templates/tera/base.html` appends it to
//! every page title:
//!
//! ```text
//! <title>{% block title %}{{ page.title }}{% if site.title %} — {{ site.title }}{% endif %}{% endblock %}</title>
//! ```
//!
//! On sebastienrousseau.com that shipped **7,189 generated tag pages** titled
//! `Tag: <term> — My SSG Site`, live and indexable, on a site whose own name
//! appears nowhere in them.
//!
//! ## Why nothing caught it
//!
//! Every existing assertion about these constants is *positive*:
//!
//! ```text
//! assert_eq!(props["site_title"]["default"], "My SSG Site");
//! assert_eq!(cfg.site_name, DEFAULT_SITE_NAME);
//! ```
//!
//! Those pin the placeholder as *correct*. Not one asked the only question
//! that mattered: does it escape into output? A default that is never rendered
//! is fine; a default that is rendered is a brand, and a placeholder brand is
//! a bug. That distinction had no test.
//!
//! ## What this guards
//!
//! The class, not the instance. [`FORBIDDEN_IN_OUTPUT`] lists every string the
//! tool may substitute on the user's behalf, and this scans a whole rendered
//! tree for all of them.
//!
//! The narrow, decisive proof lives beside the code it guards, in
//! `src/plugins/taxonomy.rs`:
//! `tag_page_never_renders_a_placeholder_site_title` drives the exact render
//! path through a config that leaves `site_title` unset, with
//! `tag_page_renders_a_configured_site_title` as the positive control so
//! emptying the default cannot "pass" by silently dropping the feature. Both
//! were observed failing against the old constant before passing against the
//! new one.
//!
//! This file is the wide net: it catches a placeholder escaping through any
//! *other* route, in any rendered file type. It asserts the build actually
//! rendered something first — the first draft of this test passed with the bug
//! present, because the fixture aborted before the taxonomy plugin ran.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

use clap::{arg, Command};
use ssg::process;
use tempfile::TempDir;

/// Strings the tool substitutes when the user supplied nothing.
///
/// A build is allowed to *use* these internally (a scaffold directory name, a
/// log line). It is never allowed to render one into a page a reader or a
/// crawler will see.
const FORBIDDEN_IN_OUTPUT: &[(&str, &str)] = &[
    (
        "My SSG Site",
        "DEFAULT_SITE_TITLE — reached 7,189 live tag-page titles",
    ),
    (
        "MySsgSite",
        "DEFAULT_SITE_NAME — scaffold name, must stay internal",
    ),
    ("A site built with SSG", "default site_description"),
];

/// Extensions a human or a crawler actually reads.
const RENDERED: &[&str] = &[
    "html",
    "htm",
    "xml",
    "json",
    "txt",
    "rss",
    "atom",
    "webmanifest",
];

fn rendered_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect(root, &mut out);
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => collect(&path, out),
            Ok(ft) if ft.is_file() => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if RENDERED.contains(&ext.as_str()) {
                    out.push(path);
                }
            }
            _ => {}
        }
    }
}

fn make_matches(
    content: &Path,
    output: &Path,
    site: &Path,
    template: &Path,
) -> clap::ArgMatches {
    Command::new("ssg")
        .arg(arg!(--"content" <CONTENT> "Content directory"))
        .arg(arg!(--"output"  <OUTPUT>  "Output directory"))
        .arg(arg!(--"new"     <NEW>     "Site directory"))
        .arg(arg!(--"template" <TEMPLATE> "Template directory"))
        .get_matches_from(vec![
            "ssg",
            "--content",
            content.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--new",
            site.to_str().unwrap(),
            "--template",
            template.to_str().unwrap(),
        ])
}

/// A site with tagged content in several scripts, and **no config file** —
/// the exact condition that produced the shipped placeholder.
fn write_unconfigured_site(content: &Path) {
    fs::create_dir_all(content).unwrap();
    fs::write(
        content.join("index.md"),
        "---\ntitle: \"Home\"\ndescription: \"Index\"\n\
         date: \"2026-01-01\"\nlayout: \"index\"\ntags: \"rust, ssg\"\n---\nHome.\n",
    )
    .unwrap();
    // Tagged posts drive the taxonomy plugin, which is what rendered the
    // placeholder. Non-ASCII separators are included so this test also covers
    // the term-splitting path (ssg_core::TERM_SEPARATORS).
    for (name, tags) in [
        ("post-en.md", "banking, payments"),
        ("post-ar.md", "المصرفية، المدفوعات"),
        ("post-ja.md", "銀行、決済"),
        ("post-zh.md", "银行，支付"),
    ] {
        fs::write(
            content.join(name),
            format!(
                "---\ntitle: \"Post {name}\"\ndescription: \"D\"\n\
                 date: \"2026-01-01\"\nlayout: \"index\"\ntags: \"{tags}\"\n---\nBody.\n"
            ),
        )
        .unwrap();
    }
}

fn write_minimal_template(template: &Path) {
    fs::create_dir_all(template).unwrap();
    fs::write(
        template.join("index.html"),
        "<!doctype html><html><head><meta charset=\"utf-8\">\
         <title>{{title}}</title></head><body><main>{{content}}</main></body></html>",
    )
    .unwrap();
}

#[test]
fn unconfigured_build_renders_no_placeholder_default() {
    let tmp = TempDir::new().unwrap();
    let content = tmp.path().join("content");
    let output = tmp.path().join("output");
    let site = tmp.path().join("site");
    let template = tmp.path().join("templates");

    write_unconfigured_site(&content);
    write_minimal_template(&template);

    // Deliberately ignore the build's own result. Whether the pipeline
    // completes is another test's job; this one asks only what reached disk.
    let _ = process::args(&make_matches(&content, &output, &site, &template));

    // GUARD AGAINST VACUOUS SUCCESS. The first version of this test passed
    // even with the bug present, because the synthesised site aborted before
    // the taxonomy plugin ran, so there was nothing to scan. A test that can
    // pass for the wrong reason is worse than no test. Require that the build
    // actually rendered something before drawing any conclusion from it.
    let rendered_any = [&output, &site]
        .iter()
        .any(|d| !rendered_files(d).is_empty());
    assert!(
        rendered_any,
        "the build produced no rendered files, so this test proves nothing. \
         Fix the fixture rather than letting it pass vacuously. \
         output={} site={}",
        output.display(),
        site.display()
    );

    let mut offences: Vec<String> = Vec::new();
    for dir in [&output, &site] {
        for file in rendered_files(dir) {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            for (needle, why) in FORBIDDEN_IN_OUTPUT {
                if text.contains(needle) {
                    offences.push(format!(
                        "{}: contains {needle:?} ({why})",
                        file.display()
                    ));
                }
            }
        }
    }

    assert!(
        offences.is_empty(),
        "a placeholder default reached rendered output — a build with no \
         config must brand nothing:\n  {}",
        offences.join("\n  ")
    );
}

#[test]
fn default_site_title_is_empty_so_templates_omit_the_suffix() {
    // base.html guards the suffix with `{% if site.title %}`, so an empty
    // title renders `<title>{page}</title>` rather than a branded one. If this
    // ever becomes non-empty again, the guard above is the safety net — but
    // this states the intent directly, where someone changing the constant
    // will see it.
    assert_eq!(
        ssg::cmd::DEFAULT_SITE_TITLE,
        "",
        "DEFAULT_SITE_TITLE is rendered into every page title by the default \
         templates; a non-empty default brands every unconfigured site"
    );
}

#[test]
fn forbidden_list_covers_the_rendered_config_defaults() {
    // Guards the guard: if a default that reaches output is added to
    // SsgConfig, it must be listed above. Checked by asserting the two known
    // rendered defaults are present, so deleting one fails loudly.
    let listed: Vec<&str> = FORBIDDEN_IN_OUTPUT
        .iter()
        .map(|(needle, _)| *needle)
        .collect();
    assert!(
        listed.contains(&"MySsgSite"),
        "DEFAULT_SITE_NAME must stay covered"
    );
    assert!(
        listed.iter().any(|n| n.contains("built with SSG")),
        "the default site_description must stay covered"
    );
}
