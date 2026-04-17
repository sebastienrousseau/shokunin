// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    dead_code,
    unused
)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Site generation benchmarks.
//!
//! Measures end-to-end compile performance at varying page counts to
//! track regressions and compare against external SSGs.
//!
//! Default tiers (run in CI):
//!   - 10, 50, 100, `1_000`, `10_000` pages
//!
//! Heavy tiers (uncomment for local profiling):
//!   - `50_000`, `100_000` pages

use criterion::{BenchmarkId, Criterion};
use std::fs;
use std::hint::black_box;
use std::path::Path;
use tempfile::TempDir;

/// Markdown frontmatter template used for synthetic pages.
const PAGE_TEMPLATE: &str = r#"---
title: "Benchmark page {N}"
description: "Synthetic page for performance measurement."
date: "2026-01-01"
layout: "page"
language: "en-GB"
charset: "utf-8"
permalink: "https://example.com/page-{N}"
author: "bench@example.com (Bench)"
banner_alt: "banner"
banner_height: "398"
banner_width: "1440"
banner: "https://example.com/banner.webp"
cdn: "https://example.com"
changefreq: "weekly"
hreflang: "en"
icon: "https://example.com/icon.svg"
id: "https://example.com"
image_alt: "image"
image_height: "630"
image_width: "1200"
image: "https://example.com/image.webp"
keywords: "bench, test, ssg"
locale: "en_GB"
logo_alt: "logo"
logo_height: "33"
logo_width: "100"
logo: "https://example.com/logo.svg"
name: "Bench"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "bench"
subtitle: "Benchmark subtitle"
tags: "bench, test"
theme-color: "143, 250, 113"
url: "https://example.com"
viewport: "width=device-width, initial-scale=1, shrink-to-fit=no"
---

## Page {N}

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea
commodo consequat.

### Features

- Feature alpha for page {N}
- Feature beta with extended description for testing content length
- Feature gamma with **bold** and *italic* formatting

### Code Example

```rust
fn page_{N}_example() -> &'static str {
    "Hello from page {N}"
}
```

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum
dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
"#;

/// Scaffold `n` synthetic Markdown pages into `content_dir`.
fn generate_pages(content_dir: &Path, n: usize) {
    fs::create_dir_all(content_dir).unwrap();
    for i in 0..n {
        let body = PAGE_TEMPLATE.replace("{N}", &i.to_string());
        let filename = if i == 0 {
            "index.md".to_string()
        } else {
            format!("page-{i}.md")
        };
        fs::write(content_dir.join(&filename), &body).unwrap();
    }
}

/// Benchmark: compile `n` pages through the staticdatagen pipeline.
fn bench_compile(c: &mut Criterion, n: usize, label: &str) {
    let _ = c.bench_function(label, |b| {
        b.iter_with_setup(
            || {
                let tmp = TempDir::new().unwrap();
                let content = tmp.path().join("content");
                let build = tmp.path().join("build");
                let site = tmp.path().join("site");
                let tpl = tmp.path().join("templates");

                generate_pages(&content, n);

                // Copy a minimal template so the compiler has something to render
                fs::create_dir_all(&tpl).unwrap();
                let src_tpl = Path::new("examples/templates/en");
                if src_tpl.exists() {
                    for entry in fs::read_dir(src_tpl).unwrap() {
                        let entry = entry.unwrap();
                        let _ =
                            fs::copy(entry.path(), tpl.join(entry.file_name()))
                                .unwrap();
                    }
                }

                (tmp, content, build, site, tpl)
            },
            |(_tmp, content, build, site, tpl)| {
                let result = staticdatagen::compiler::service::compile(
                    &build, &content, &site, &tpl,
                );
                let _ = black_box(result);
            },
        );
    });
}

/// Parameterised benchmark group across multiple page counts.
fn bench_compile_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("site_generation");

    // CI-friendly tiers
    for &n in &[10, 50, 100, 1_000, 10_000] {
        let _ = group.bench_with_input(
            BenchmarkId::new("compile", n),
            &n,
            |b, &n| {
                b.iter_with_setup(
                    || {
                        let tmp = TempDir::new().unwrap();
                        let content = tmp.path().join("content");
                        let build = tmp.path().join("build");
                        let site = tmp.path().join("site");
                        let tpl = tmp.path().join("templates");

                        generate_pages(&content, n);

                        fs::create_dir_all(&tpl).unwrap();
                        let src_tpl = Path::new("examples/templates/en");
                        if src_tpl.exists() {
                            for entry in fs::read_dir(src_tpl).unwrap() {
                                let entry = entry.unwrap();
                                let _ = fs::copy(
                                    entry.path(),
                                    tpl.join(entry.file_name()),
                                )
                                .unwrap();
                            }
                        }

                        (tmp, content, build, site, tpl)
                    },
                    |(_tmp, content, build, site, tpl)| {
                        let result = staticdatagen::compiler::service::compile(
                            &build, &content, &site, &tpl,
                        );
                        let _ = black_box(result);
                    },
                );
            },
        );
    }

    // Heavy tiers -- uncomment for local profiling (too slow for CI):
    // for &n in &[50_000, 100_000] {
    //     group.sample_size(10);
    //     group.bench_with_input(
    //         BenchmarkId::new("compile", n),
    //         &n,
    //         |b, &n| { /* same body as above */ },
    //     );
    // }

    group.finish();
}

/// Entry point for site generation benchmarks.
#[allow(unreachable_pub)]
pub fn bench_site_generation(c: &mut Criterion) {
    // Legacy individual benchmarks (kept for backwards compatibility)
    bench_compile(c, 10, "compile 10 pages");
    bench_compile(c, 50, "compile 50 pages");
    bench_compile(c, 100, "compile 100 pages");

    // Parameterised group with 1K and 10K tiers
    bench_compile_group(c);
}
