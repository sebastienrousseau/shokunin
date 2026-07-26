// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::oembed` (issue #586, port 4) — runs the
//! plugin against a fixture site through both lifecycle hooks
//! (`after_compile` for the JSON documents, `transform_html` for the
//! discovery link) via the real `PluginManager` fused-transform path.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::oembed::OembedPlugin;
use ssg::plugin::{Plugin, PluginContext, PluginManager};
use std::fs;
use tempfile::TempDir;

fn fixture_site() -> (TempDir, PluginContext) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let build = tmp.path().join("build");
    let site = tmp.path().join("public");
    fs::create_dir_all(build.join(".meta")).unwrap();
    fs::create_dir_all(&site).unwrap();

    fs::write(
        build.join(".meta/hello.meta.json"),
        r#"{"title": "Hello & Welcome", "author": "a@b.test (Ann)"}"#,
    )
    .unwrap();
    fs::write(
        site.join("hello.html"),
        "<html><head><title>Hello</title></head><body>hi</body></html>",
    )
    .unwrap();

    let cfg = ssg::cmd::SsgConfig::builder()
        .site_name("Fixture".to_string())
        .base_url("https://fixture.test".to_string())
        .build()
        .expect("config");
    let ctx =
        PluginContext::with_config(tmp.path(), &build, &site, tmp.path(), cfg);
    (tmp, ctx)
}

#[test]
fn plugin_name_is_stable() {
    assert_eq!(OembedPlugin.name(), "oembed");
}

#[test]
fn full_lifecycle_emits_document_and_discovery_link() {
    let (_tmp, mut ctx) = fixture_site();

    let mut pm = PluginManager::new();
    pm.register(OembedPlugin);

    // Same order the pipeline uses: after_compile, then the fused
    // transform pass over the (pre-cached) HTML file list.
    ctx.cache_html_files();
    pm.run_after_compile(&ctx).unwrap();
    pm.run_fused_transforms(&ctx).unwrap();

    // 1. Sibling oEmbed 1.0 document.
    let doc: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ctx.site_dir.join("hello.oembed.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["version"], "1.0");
    assert_eq!(doc["type"], "link");
    assert_eq!(doc["title"], "Hello & Welcome");
    assert_eq!(doc["provider_name"], "Fixture");
    assert_eq!(doc["provider_url"], "https://fixture.test");
    assert_eq!(doc["author_name"], "Ann");

    // 2. Discovery link injected into <head>.
    let html = fs::read_to_string(ctx.site_dir.join("hello.html")).unwrap();
    assert!(html.contains("application/json+oembed"), "{html}");
    assert!(
        html.contains("href=\"https://fixture.test/hello.oembed.json\""),
        "{html}"
    );
    // Title attribute is escaped.
    assert!(html.contains("title=\"Hello &amp; Welcome\""), "{html}");
}

#[test]
fn second_run_is_idempotent() {
    let (_tmp, mut ctx) = fixture_site();
    let mut pm = PluginManager::new();
    pm.register(OembedPlugin);
    ctx.cache_html_files();

    pm.run_after_compile(&ctx).unwrap();
    pm.run_fused_transforms(&ctx).unwrap();
    let first = fs::read_to_string(ctx.site_dir.join("hello.html")).unwrap();

    pm.run_after_compile(&ctx).unwrap();
    pm.run_fused_transforms(&ctx).unwrap();
    let second = fs::read_to_string(ctx.site_dir.join("hello.html")).unwrap();

    assert_eq!(first, second, "discovery link must not duplicate");
    assert_eq!(first.matches("json+oembed").count(), 1);
}

#[test]
fn not_registered_means_no_output() {
    // Opt-in contract: a pipeline without the plugin emits nothing.
    let (_tmp, mut ctx) = fixture_site();
    let pm = PluginManager::new();
    ctx.cache_html_files();
    pm.run_after_compile(&ctx).unwrap();
    pm.run_fused_transforms(&ctx).unwrap();
    assert!(!ctx.site_dir.join("hello.oembed.json").exists());
}
