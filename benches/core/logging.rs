// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::logging`.

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::logging::{create_log_file, log_initialization};

fn bench_create_log_file_and_banner(c: &mut Criterion) {
    c.bench_function("logging::create_and_banner", |b| {
        b.iter_batched(
            tempfile::tempdir,
            |dir| {
                let dir = dir.unwrap();
                let path = dir.path().join("ssg.log");
                let mut f =
                    create_log_file(path.to_str().unwrap()).unwrap();
                log_initialization(&mut f, "2026-06-07").unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_create_log_file_and_banner);
