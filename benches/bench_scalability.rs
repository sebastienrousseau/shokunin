#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs, unused)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Scalability benchmarks: measures build time at 100, 1K, 10K, and 100K pages.

use criterion::{criterion_group, BenchmarkId, Criterion, SamplingMode};
use std::fs;
use std::hint::black_box;
use std::path::Path;
use tempfile::TempDir;

/// Generates `n` synthetic Markdown files with realistic frontmatter.
fn generate_corpus(dir: &Path, n: usize) {
    fs::create_dir_all(dir).expect("create content dir");
    for i in 0..n {
        let content = format!(
            "---\n\
             title: \"Page {i}\"\n\
             date: \"2026-01-15T09:00:00+00:00\"\n\
             description: \"Benchmark page number {i} for scalability testing\"\n\
             language: \"en-GB\"\n\
             layout: \"page\"\n\
             permalink: \"https://example.com/page-{i}\"\n\
             ---\n\n\
             # Page {i}\n\n\
             This is benchmark content for page number {i}. \
             The static site generator processes each page through \
             the full pipeline including template rendering and \
             plugin transforms.\n\n\
             ## Features\n\n\
             - Fast compilation with Rayon parallelism\n\
             - Content-addressed caching for incremental builds\n\
             - Fused transform pipeline for minimal I/O\n\n\
             The build system handles {n} pages efficiently.\n"
        );
        let filename = if i == 0 {
            "index.md".to_string()
        } else {
            format!("page-{i}.md")
        };
        fs::write(dir.join(filename), content).expect("write benchmark page");
    }
}

fn bench_build_at_scale(c: &mut Criterion) {
    let tiers: &[(usize, &str)] = &[
        (100, "100 pages"),
        (1_000, "1K pages"),
        (10_000, "10K pages"),
    ];

    let mut group = c.benchmark_group("scalability");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);

    for &(n, label) in tiers {
        let _ = group.bench_with_input(
            BenchmarkId::new("compile", label),
            &n,
            |b, &n| {
                b.iter_with_setup(
                    || {
                        let tmp = TempDir::new().expect("tempdir");
                        let content = tmp.path().join("content");
                        let build = tmp.path().join("build");
                        let site = tmp.path().join("site");
                        let template = tmp.path().join("templates");
                        fs::create_dir_all(&content).unwrap();
                        fs::create_dir_all(&build).unwrap();
                        fs::create_dir_all(&site).unwrap();
                        fs::create_dir_all(&template).unwrap();

                        // Copy example templates so the compiler can render
                        let src_tpl = Path::new("examples/templates/en");
                        if src_tpl.exists() {
                            for entry in fs::read_dir(src_tpl).unwrap() {
                                let entry = entry.unwrap();
                                let _ = fs::copy(
                                    entry.path(),
                                    template.join(entry.file_name()),
                                )
                                .unwrap();
                            }
                        }

                        generate_corpus(&content, n);
                        (tmp, content, build, site, template)
                    },
                    |(_tmp, content, build, site, template)| {
                        let result = ssg::compile_site(
                            &build, &content, &site, &template,
                        );
                        let _ = black_box(result);
                    },
                );
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = scalability;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(30));
    targets = bench_build_at_scale
}
