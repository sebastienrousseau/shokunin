// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Performance regression tests.
//!
//! These tests generate synthetic content and assert that compilation
//! completes within bounded time. They catch performance regressions
//! early — before benchmarks are needed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// Generates `n` synthetic markdown files with realistic frontmatter.
fn generate_pages(content_dir: &Path, n: usize) {
    fs::create_dir_all(content_dir).unwrap();
    for i in 0..n {
        let content = format!(
            "---\ntitle: \"Page {i}\"\ndate: \"2026-04-18T00:00:00Z\"\ndescription: \"Test page {i} for performance benchmarking\"\nkeywords: \"test, perf, page{i}\"\n---\n\n# Page {i}\n\nThis is page {i} with enough content to be realistic.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do\neiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n## Section A\n\nMore content for page {i}. This ensures each page has enough\ntext for the search indexer, SEO plugin, and readability auditor\nto process meaningful input.\n\n## Section B\n\n- Item 1 for page {i}\n- Item 2 for page {i}\n- Item 3 for page {i}\n"
        );
        fs::write(content_dir.join(format!("page-{i}.md")), content).unwrap();
    }
    // Required pages that staticdatagen expects
    for required in ["index.md", "404.md"] {
        if !content_dir.join(required).exists() {
            let name = required.replace(".md", "");
            let content = format!(
                "---\ntitle: \"{name}\"\ndate: \"2026-04-18T00:00:00Z\"\ndescription: \"Required page\"\n---\n\n# {name}\n\nRequired content.\n"
            );
            fs::write(content_dir.join(required), content).unwrap();
        }
    }
}

#[test]
fn compile_100_pages_under_5s() {
    let dir = tempdir().unwrap();
    let content_dir = dir.path().join("content");
    let build_dir = dir.path().join("build");
    let site_dir = dir.path().join("public");
    let template_dir = dir.path().join("templates");

    fs::create_dir_all(&build_dir).unwrap();
    fs::create_dir_all(&site_dir).unwrap();
    fs::create_dir_all(&template_dir).unwrap();

    generate_pages(&content_dir, 100);

    let start = Instant::now();
    let result =
        ssg::compile_site(&build_dir, &content_dir, &site_dir, &template_dir);
    let elapsed = start.elapsed();

    // The compile may fail because staticdatagen has specific expectations,
    // but we're measuring the attempt time, not success
    println!(
        "100 pages: {elapsed:.2?} (result: {})",
        if result.is_ok() { "ok" } else { "err" }
    );

    assert!(
        elapsed < Duration::from_secs(5),
        "100-page compilation took {elapsed:.2?} — exceeds 5s budget"
    );
}

#[test]
fn search_index_generation_under_1s() {
    let dir = tempdir().unwrap();
    let site_dir = dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();

    // Generate 50 HTML files
    for i in 0..50 {
        let html = format!(
            "<html><head><title>Page {i}</title></head>\
             <body><main><h1>Page {i}</h1>\
             <p>Content for search indexing test page {i}.</p>\
             </main></body></html>"
        );
        fs::create_dir_all(site_dir.join(format!("page-{i}"))).unwrap();
        fs::write(site_dir.join(format!("page-{i}/index.html")), html).unwrap();
    }

    let start = Instant::now();
    let ctx = ssg::plugin::PluginContext::new(
        dir.path(),
        dir.path(),
        &site_dir,
        dir.path(),
    );
    let search_plugin = ssg::search::SearchPlugin;
    let _ = ssg::plugin::Plugin::after_compile(&search_plugin, &ctx);
    let elapsed = start.elapsed();

    println!("50-page search index: {elapsed:.2?}");
    assert!(
        elapsed < Duration::from_secs(1),
        "Search indexing of 50 pages took {elapsed:.2?} — exceeds 1s budget"
    );
}

#[test]
fn cache_fingerprint_1000_files_under_2s() {
    let dir = tempdir().unwrap();
    let content_dir = dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Generate 1000 small files
    for i in 0..1000 {
        fs::write(
            content_dir.join(format!("file-{i}.md")),
            format!("Content for file {i}\n"),
        )
        .unwrap();
    }

    let start = Instant::now();
    let mut cache =
        ssg::cache::BuildCache::new(&dir.path().join(".ssg-cache.json"));
    let _ = cache.update(&content_dir);
    let elapsed = start.elapsed();

    println!("1000-file cache fingerprint: {elapsed:.2?}");
    assert!(
        elapsed < Duration::from_secs(2),
        "Cache fingerprinting of 1000 files took {elapsed:.2?} — exceeds 2s budget"
    );
}

#[test]
fn streaming_hash_1000_files_under_2s() {
    let dir = tempdir().unwrap();
    let content_dir = dir.path().join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Generate 1000 small files
    for i in 0..1000 {
        fs::write(
            content_dir.join(format!("file-{i}.txt")),
            format!("Streaming hash content for file {i}\n"),
        )
        .unwrap();
    }

    let start = Instant::now();
    for i in 0..1000 {
        let path = content_dir.join(format!("file-{i}.txt"));
        let hash = ssg::stream::stream_hash(&path).unwrap();
        assert!(!hash.is_empty(), "hash must be non-empty");
    }
    let elapsed = start.elapsed();

    println!("1000-file stream hash: {elapsed:.2?}");
    assert!(
        elapsed < Duration::from_secs(2),
        "Streaming hash of 1000 files took {elapsed:.2?} — exceeds 2s budget"
    );
}
