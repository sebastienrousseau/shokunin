// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::walk`.

use std::fs;
use std::path::Path;

use criterion::{criterion_group, Criterion};
use ssg::walk::{walk_files, walk_files_bounded_count};
use tempfile::TempDir;

fn seed_flat_tree(dir: &Path, count: usize) {
    fs::create_dir_all(dir).unwrap();
    for i in 0..count {
        fs::write(dir.join(format!("post-{i:04}.md")), "").unwrap();
    }
}

fn seed_deep_tree(dir: &Path, depth: usize, files_per_dir: usize) {
    let mut current = dir.to_path_buf();
    for d in 0..depth {
        current = current.join(format!("d{d}"));
        fs::create_dir_all(&current).unwrap();
        for i in 0..files_per_dir {
            fs::write(current.join(format!("f{i}.md")), "").unwrap();
        }
    }
}

fn populated_flat(count: usize) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    seed_flat_tree(dir.path(), count);
    dir
}

fn populated_deep(depth: usize, files_per_dir: usize) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    seed_deep_tree(dir.path(), depth, files_per_dir);
    dir
}

fn bench_walk_flat_500(c: &mut Criterion) {
    let dir = populated_flat(500);
    c.bench_function("walk::flat_500", |b| {
        b.iter(|| walk_files(dir.path(), "md").unwrap());
    });
}

fn bench_walk_deep_10x10(c: &mut Criterion) {
    let dir = populated_deep(10, 10);
    c.bench_function("walk::deep_10x10", |b| {
        b.iter(|| walk_files(dir.path(), "md").unwrap());
    });
}

fn bench_walk_bounded_count_100(c: &mut Criterion) {
    let dir = populated_flat(1000);
    c.bench_function("walk::bounded_count_100_of_1000", |b| {
        b.iter(|| walk_files_bounded_count(dir.path(), "md", 100).unwrap());
    });
}

criterion_group!(
    benches,
    bench_walk_flat_500,
    bench_walk_deep_10x10,
    bench_walk_bounded_count_100
);
