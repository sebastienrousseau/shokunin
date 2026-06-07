// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::cmd::validation`.

use ssg::cmd::{is_valid_url, validate_url};

#[test]
fn is_valid_url_accepts_https_scheme() {
    assert!(is_valid_url("https://example.com"));
}

#[test]
fn is_valid_url_rejects_bare_string() {
    assert!(!is_valid_url("not a url"));
}

#[test]
fn validate_url_rejects_data_uri_scheme() {
    assert!(validate_url("data:text/html,<script>alert(1)</script>").is_err());
}

#[test]
fn validate_url_rejects_vbscript_scheme() {
    assert!(validate_url("vbscript:msgbox(1)").is_err());
}

#[test]
fn validate_url_accepts_well_formed_https() {
    assert!(validate_url("https://example.com/path").is_ok());
}
