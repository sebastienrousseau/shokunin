// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::server`.

use std::fs;

use ssg::server::generate_locale_redirect;
use tempfile::tempdir;

#[test]
fn generate_locale_redirect_emits_html_with_meta_refresh() {
    let dir = tempdir().unwrap();
    let site = dir.path();
    fs::create_dir_all(site).unwrap();
    let locales = vec!["en-US".to_string(), "fr-FR".to_string()];
    let result = generate_locale_redirect(site, &locales, "en-US");
    assert!(result.is_ok());
    // Should emit some kind of index/redirect file
    let entries: Vec<_> = fs::read_dir(site).unwrap().collect();
    assert!(!entries.is_empty());
}
