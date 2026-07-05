// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::agent_api` (issue #586, port 3) —
//! exercises the plugin against a small fixture site the way the
//! pipeline does.

use ssg::agent_api::AgentApiPlugin;
use ssg::cmd::SsgConfig;
use ssg::plugin::{Plugin, PluginContext};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Builds a fixture site: three posts (one draft) with sidecars under
/// `<build>/.meta/` and rendered HTML under `<site>/`.
fn fixture_site() -> (TempDir, PluginContext) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let build = tmp.path().join("build");
    let site = tmp.path().join("public");
    let meta = build.join(".meta");
    fs::create_dir_all(meta.join("blog")).unwrap();
    fs::create_dir_all(site.join("blog")).unwrap();

    let write =
        |meta_rel: &str, meta_json: &str, html_rel: &str, html: &str| {
            fs::write(meta.join(meta_rel), meta_json).unwrap();
            fs::write(site.join(html_rel), html).unwrap();
        };

    write(
        "index.meta.json",
        r#"{
            "title": "Home",
            "description": "Fixture home",
            "author": "jane@fixture.test (Jane Fixture)",
            "date": "2026-01-01",
            "tags": "home, fixture",
            "word_count": 100
        }"#,
        "index.html",
        "<html><head><title>Home</title></head><body>home</body></html>",
    );
    write(
        "blog/post.meta.json",
        r#"{
            "title": "Post",
            "description": "Fixture post",
            "author": "jane@fixture.test (Jane Fixture)",
            "date": "2026-02-02",
            "tags": ["rust", "fixture"],
            "word_count": 250
        }"#,
        "blog/post.html",
        "<html><head><title>Post</title></head><body>post</body></html>",
    );
    write(
        "blog/draft.meta.json",
        r#"{"title": "Draft", "draft": true, "tags": "rust"}"#,
        "blog/draft.html",
        "<html><head><title>Draft</title></head><body>draft</body></html>",
    );

    let cfg = SsgConfig::builder()
        .site_name("Fixture".to_string())
        .base_url("https://fixture.test".to_string())
        .build()
        .expect("config");
    let ctx =
        PluginContext::with_config(tmp.path(), &build, &site, tmp.path(), cfg);
    (tmp, ctx)
}

fn read_json(site: &Path, rel: &str) -> serde_json::Value {
    let body = fs::read_to_string(site.join(rel)).expect(rel);
    serde_json::from_str(&body).expect(rel)
}

#[test]
fn plugin_name_is_stable() {
    assert_eq!(AgentApiPlugin::default().name(), "agent-api");
}

#[test]
fn fixture_site_gets_all_four_documents() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();

    let site = ctx.site_dir.clone();
    assert!(site.join("api/agents/index.json").exists());
    assert!(site.join("api/agents/posts.json").exists());
    assert!(site.join("api/agents/topics.json").exists());
    assert!(site.join("api/agents/person.json").exists());
}

#[test]
fn posts_json_excludes_drafts_and_sorts_by_url() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();

    let posts = read_json(&ctx.site_dir, "api/agents/posts.json");
    let arr = posts.as_array().unwrap();
    assert_eq!(arr.len(), 2, "draft must be excluded: {posts}");
    assert_eq!(arr[0]["url"], "https://fixture.test/blog/post.html");
    assert_eq!(arr[1]["url"], "https://fixture.test/index.html");
    assert_eq!(arr[0]["wordCount"], 250);
    assert_eq!(arr[0]["tags"], serde_json::json!(["fixture", "rust"]));
}

#[test]
fn topics_json_maps_terms_to_member_urls() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();

    let topics = read_json(&ctx.site_dir, "api/agents/topics.json");
    assert_eq!(
        topics["rust"],
        serde_json::json!(["https://fixture.test/blog/post.html"])
    );
    assert_eq!(
        topics["fixture"],
        serde_json::json!([
            "https://fixture.test/blog/post.html",
            "https://fixture.test/index.html"
        ])
    );
}

#[test]
fn person_json_is_schema_org_person() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();

    let person = read_json(&ctx.site_dir, "api/agents/person.json");
    assert_eq!(person["@context"], "https://schema.org");
    assert_eq!(person["@type"], "Person");
    assert_eq!(person["name"], "Jane Fixture");
    assert_eq!(person["email"], "jane@fixture.test");
    assert_eq!(person["url"], "https://fixture.test");
}

#[test]
fn index_json_links_resolve_within_the_site() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();

    let index = read_json(&ctx.site_dir, "api/agents/index.json");
    assert_eq!(index["counts"]["posts"], 2);
    for doc in ["index", "posts", "topics", "person"] {
        let link = index["links"][doc].as_str().unwrap();
        let rel = link
            .strip_prefix("https://fixture.test/")
            .expect("absolute link");
        assert!(
            ctx.site_dir.join(rel).exists(),
            "{link} must resolve to an emitted file"
        );
    }
}

#[test]
fn rebuild_is_byte_deterministic() {
    let (_tmp, ctx) = fixture_site();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();
    let read_all = || {
        ["index.json", "posts.json", "topics.json", "person.json"].map(|d| {
            fs::read_to_string(ctx.site_dir.join("api/agents").join(d)).unwrap()
        })
    };
    let first = read_all();
    AgentApiPlugin::default().after_compile(&ctx).unwrap();
    assert_eq!(first, read_all());
}
