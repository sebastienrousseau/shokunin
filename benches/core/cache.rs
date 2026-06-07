// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::cache::BuildCache`.
//!
//! Drives the public API at realistic content-tree sizes so regressions in
//! fingerprint scanning, persistence, or change-detection surface in CI's
//! `cargo bench` comparison runs.

use std::fs;
use std::path::Path;

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::cache::BuildCache;
use tempfile::TempDir;

fn seed_content(dir: &Path, count: usize) {
    fs::create_dir_all(dir).expect("create_dir_all");
    for i in 0..count {
        fs::write(
            dir.join(format!("post-{i:04}.md")),
            format!("# Post {i}\n\nLorem ipsum body."),
        )
        .expect("write");
    }
}

fn populated_cache_dir(count: usize) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let cache_path = dir.path().join(".ssg-cache.json");
    let content = dir.path().join("content");
    seed_content(&content, count);
    let mut cache = BuildCache::load(&cache_path).expect("load");
    cache.update(&content).expect("update");
    cache.save().expect("save");
    dir
}

fn bench_load_empty(c: &mut Criterion) {
    c.bench_function("cache::load_missing", |b| {
        b.iter_batched(
            tempfile::tempdir,
            |dir| {
                let dir = dir.unwrap();
                BuildCache::load(&dir.path().join(".ssg-cache.json"))
                    .expect("load")
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_changed_files_100_unchanged(c: &mut Criterion) {
    let dir = populated_cache_dir(100);
    let cache_path = dir.path().join(".ssg-cache.json");
    let content = dir.path().join("content");

    c.bench_function("cache::changed_files_100_unchanged", |b| {
        b.iter(|| {
            let cache = BuildCache::load(&cache_path).expect("load");
            cache.changed_files(&content).expect("changed_files")
        });
    });
}

fn bench_update_100_files(c: &mut Criterion) {
    c.bench_function("cache::update_100_files", |b| {
        b.iter_batched(
            || {
                let dir = tempfile::tempdir().unwrap();
                let content = dir.path().join("content");
                seed_content(&content, 100);
                (dir, content)
            },
            |(dir, content)| {
                let cache_path = dir.path().join(".ssg-cache.json");
                let mut cache = BuildCache::load(&cache_path).expect("load");
                cache.update(&content).expect("update");
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_load_empty,
    bench_changed_files_100_unchanged,
    bench_update_100_files
);
