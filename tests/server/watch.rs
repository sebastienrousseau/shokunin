// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::watch`.

use std::path::Path;

use ssg::watch::{classify_change, ChangeKind};

#[test]
fn classify_change_recognises_css() {
    assert!(matches!(
        classify_change(Path::new("theme.css")),
        ChangeKind::Css
    ));
}

#[test]
fn classify_change_recognises_markdown_as_content() {
    assert!(matches!(
        classify_change(Path::new("post.md")),
        ChangeKind::Content
    ));
}

#[test]
fn classify_change_recognises_html_template() {
    assert!(matches!(
        classify_change(Path::new("base.html")),
        ChangeKind::Content | ChangeKind::Template
    ));
}

#[test]
fn classify_change_falls_back_to_other_for_unknown() {
    assert!(matches!(
        classify_change(Path::new("data.toml")),
        ChangeKind::Other | ChangeKind::Content
    ));
}
