// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::cmd::SsgConfig` + builder.

use std::path::PathBuf;

use ssg::cmd::SsgConfig;

#[test]
fn builder_constructs_with_minimum_fields() {
    let cfg = SsgConfig::builder()
        .site_name("x".into())
        .base_url("https://example.com".into())
        .site_title("Title".into())
        .site_description("Desc".into())
        .language("en-US".into())
        .content_dir(PathBuf::from("content"))
        .output_dir(PathBuf::from("public"))
        .template_dir(PathBuf::from("templates"))
        .build()
        .expect("config");
    assert_eq!(cfg.site_name, "x");
}

#[test]
fn validate_passes_for_well_formed_config() {
    let cfg = SsgConfig::builder()
        .site_name("x".into())
        .base_url("https://example.com".into())
        .site_title("Title".into())
        .site_description("Desc".into())
        .language("en-US".into())
        .content_dir(PathBuf::from("content"))
        .output_dir(PathBuf::from("public"))
        .template_dir(PathBuf::from("templates"))
        .build()
        .unwrap();
    assert!(cfg.validate().is_ok());
}
