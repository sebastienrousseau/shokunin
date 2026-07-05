// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::urls` — the shared page-URL
//! derivation feeding permalink injection, canonical `<link>`, and
//! feed links (spec A2/B1, plan §2 item 1.2, issue #586).

use ssg::urls::{derive_output_rel_path, derive_page_url, derive_permalink};

#[test]
fn derive_page_url_root_index_is_base_with_trailing_slash() {
    assert_eq!(
        derive_page_url("https://example.com", "index.html"),
        "https://example.com/"
    );
}

#[test]
fn derive_page_url_tolerates_trailing_slash_base() {
    assert_eq!(
        derive_page_url("https://example.com/", "posts/foo/index.html"),
        "https://example.com/posts/foo/"
    );
}

#[test]
fn derive_page_url_collapses_index_html_to_directory() {
    assert_eq!(
        derive_page_url("https://example.com", "about/index.html"),
        "https://example.com/about/"
    );
}

#[test]
fn derive_page_url_keeps_non_index_file_names() {
    assert_eq!(
        derive_page_url("https://example.com", "rss.xml"),
        "https://example.com/rss.xml"
    );
}

#[test]
fn derive_page_url_normalises_windows_separators() {
    assert_eq!(
        derive_page_url("https://example.com", "a\\b\\index.html"),
        "https://example.com/a/b/"
    );
}

#[test]
fn derive_page_url_leaves_percent_encoding_untouched() {
    assert_eq!(
        derive_page_url("https://example.com", "caf%C3%A9/index.html"),
        "https://example.com/caf%C3%A9/"
    );
}

/// Locks the source→output mapping to the compiler convention
/// (`staticdatagen 0.0.9` `utilities/write.rs`, mirrored by
/// `isr_manifest::derive_url`): `posts/foo.md → posts/foo/index.html`,
/// `index.md → index.html`, `about/index.md → about/index.html`.
#[test]
fn derive_output_rel_path_matches_compiler_convention() {
    assert_eq!(
        derive_output_rel_path("posts/foo.md"),
        "posts/foo/index.html"
    );
    assert_eq!(derive_output_rel_path("index.md"), "index.html");
    assert_eq!(derive_output_rel_path("about/index.md"), "about/index.html");
}

#[test]
fn derive_permalink_end_to_end() {
    assert_eq!(
        derive_permalink("https://example.com/", "posts/foo.md"),
        "https://example.com/posts/foo/"
    );
    assert_eq!(
        derive_permalink("https://example.com", "index.md"),
        "https://example.com/"
    );
}
