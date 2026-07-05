// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::content_stager` permalink derivation
//! (spec A2/B1, plan §2 item 1.2, issue #586).
//!
//! The stager guarantees every staged `.md` page carries a
//! `permalink:` front-matter key before `staticdatagen::compile`
//! consumes the staged tree, making `rss-gen`'s "channel.link is
//! missing" hard-fail unreachable.

use std::fs;

use ssg::content_stager::{
    inject_permalink_if_missing, stage_content_with_site_defaults,
};
use ssg::urls::derive_permalink;
use tempfile::tempdir;

const BASE: &str = "https://example.com";

/// Plan §2 1.2 fixture: three pages, ZERO of which declare
/// `permalink:`. Staging must inject a derived permalink for each,
/// equal to `derive_permalink(base_url, content_relative_path)` —
/// the same single code path the canonical plugin uses.
#[test]
fn three_page_fixture_without_permalinks_gets_derived_permalinks() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("content");
    let build = tmp.path().join("build");
    fs::create_dir_all(src.join("posts")).unwrap();

    let pages = ["index.md", "about.md", "posts/first.md"];
    for page in pages {
        let path = src.join(page);
        fs::write(&path, "---\ntitle: T\ndescription: D\n---\nbody").unwrap();
    }

    let staged =
        stage_content_with_site_defaults(&src, &build, &[], Some(BASE))
            .unwrap();

    for page in pages {
        let body = fs::read_to_string(staged.join(page)).unwrap();
        let expected = derive_permalink(BASE, page);
        assert!(
            body.contains(&format!("permalink: \"{expected}\"")),
            "{page}: expected derived permalink {expected}, got:\n{body}"
        );
    }
}

#[test]
fn author_specified_permalink_is_kept_verbatim() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("content");
    let build = tmp.path().join("build");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("pinned.md"),
        "---\npermalink: \"https://example.com/legacy-spot/\"\ntitle: P\n---\nbody",
    )
    .unwrap();

    let staged =
        stage_content_with_site_defaults(&src, &build, &[], Some(BASE))
            .unwrap();
    let body = fs::read_to_string(staged.join("pinned.md")).unwrap();
    assert!(body.contains("permalink: \"https://example.com/legacy-spot/\""));
    assert_eq!(
        body.matches("permalink:").count(),
        1,
        "author permalink must not be duplicated: {body}"
    );
}

/// `index.md` publishes at the site root (`index.html`), and nested
/// `dir/index.md` publishes at `dir/index.html` — both collapse to
/// directory URLs with a trailing slash (the Atom feed convention).
#[test]
fn index_md_convention_maps_to_directory_urls() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("content");
    let build = tmp.path().join("build");
    fs::create_dir_all(src.join("docs")).unwrap();
    fs::write(src.join("index.md"), "---\ntitle: Home\n---\nh").unwrap();
    fs::write(src.join("docs/index.md"), "---\ntitle: Docs\n---\nd").unwrap();

    // Trailing-slash base URL must not produce `//`.
    let staged = stage_content_with_site_defaults(
        &src,
        &build,
        &[],
        Some("https://example.com/"),
    )
    .unwrap();

    let home = fs::read_to_string(staged.join("index.md")).unwrap();
    assert!(
        home.contains("permalink: \"https://example.com/\""),
        "root index.md → bare base URL with trailing slash: {home}"
    );
    let docs = fs::read_to_string(staged.join("docs/index.md")).unwrap();
    assert!(
        docs.contains("permalink: \"https://example.com/docs/\""),
        "nested index.md → enclosing directory URL: {docs}"
    );
}

#[test]
fn inject_permalink_if_missing_respects_url_alias() {
    let with_url = "---\nurl: /already/\ntitle: T\n---\nbody";
    assert_eq!(
        inject_permalink_if_missing(with_url, "https://example.com/x/"),
        with_url,
        "a front-matter `url:` key counts as author-specified"
    );
}
