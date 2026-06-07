// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::fs_ops`.

use std::fs;

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::fs_ops::copy_dir_all;

fn bench_copy_dir_all_50_files(c: &mut Criterion) {
    c.bench_function("fs_ops::copy_dir_all_50_files", |b| {
        b.iter_batched(
            || {
                let src = tempfile::tempdir().unwrap();
                let dst = tempfile::tempdir().unwrap();
                for i in 0..50 {
                    fs::write(
                        src.path().join(format!("f-{i}.txt")),
                        "lorem ipsum",
                    )
                    .unwrap();
                }
                (src, dst)
            },
            |(src, dst)| {
                copy_dir_all(src.path(), &dst.path().join("out")).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_copy_dir_all_50_files);
