// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::cmd::{LanguageCode, CliError}`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::cmd::LanguageCode;

#[test]
fn language_code_accepts_well_formed_bcp47_pair() {
    let lc = LanguageCode::new("en-GB").expect("en-GB");
    assert_eq!(lc.to_string(), "en-GB");
}

#[test]
fn language_code_rejects_lowercase_region_segment() {
    assert!(LanguageCode::new("en-gb").is_err());
}

#[test]
fn language_code_rejects_uppercase_language_segment() {
    assert!(LanguageCode::new("EN-GB").is_err());
}

#[test]
fn language_code_rejects_wrong_separator_count() {
    assert!(LanguageCode::new("eng").is_err());
}
