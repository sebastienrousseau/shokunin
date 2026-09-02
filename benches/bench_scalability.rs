#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs, dead_code)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Scalability benchmarks: measures build time at 100, 1K, 10K, and 100K pages.

use criterion::{criterion_group, BenchmarkId, Criterion, SamplingMode};
use ssg::bench_corpus::{
    generate_corpus as generate_corpus_seeded, CorpusSpec,
};
use std::fs;
use std::hint::black_box;
use std::path::Path;
use tempfile::TempDir;

/// Writes `n` pages using the shared seeded corpus generator.
///
/// This delegates to `ssg::bench_corpus` rather than emitting its own front
/// matter. Each bench file used to carry its own generator, so timings were
/// only comparable within a single file: `bench_scalability` and
/// `incremental_1000_pages` wrote different front matter and different body
/// lengths while reporting as though they measured the same work.
fn generate_corpus(dir: &Path, n: usize) {
    let spec = CorpusSpec::new(n);
    let _written =
        generate_corpus_seeded(dir, &spec).expect("write benchmark corpus");
}

#[allow(unused_results)]
fn bench_build_at_scale(c: &mut Criterion) {
    // The tiers the elevation plan publishes figures for. 100K is included
    // because the memory budget is asserted at that size; it is the tier the
    // "100K pages under the configured budget" claim rests on.
    let tiers: &[(usize, &str)] = &[
        (1_000, "1K pages"),
        (10_000, "10K pages"),
        (100_000, "100K pages"),
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
