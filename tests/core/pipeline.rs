// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::pipeline`.

use std::path::PathBuf;

use ssg::cmd::SsgConfig;
use ssg::pipeline::{clear_error_message, resolve_build_and_site_dirs};

fn minimal_config() -> SsgConfig {
    SsgConfig::builder()
        .site_name("x".into())
        .base_url("https://example.com".into())
        .site_title("y".into())
        .site_description("z".into())
        .language("en-US".into())
        .content_dir(PathBuf::from("content"))
        .output_dir(PathBuf::from("public"))
        .template_dir(PathBuf::from("templates"))
        .build()
        .expect("config")
}

#[test]
fn clear_error_message_returns_a_string() {
    let _ = clear_error_message();
}

#[test]
fn resolve_build_and_site_dirs_returns_two_paths() {
    let cfg = minimal_config();
    let (build, site) = resolve_build_and_site_dirs(&cfg);
    assert!(!build.as_os_str().is_empty());
    assert!(!site.as_os_str().is_empty());
}
