// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::collections`.

use std::fs;

use criterion::{criterion_group, BatchSize, Criterion};
use serde::Deserialize;
use ssg::collections::{get_collection, Entry};

#[derive(Debug, Deserialize)]
struct Post {
    #[allow(dead_code)]
    title: String,
}

fn bench_get_collection_100(c: &mut Criterion) {
    c.bench_function("collections::get_collection_100", |b| {
        b.iter_batched(
            || {
                let dir = tempfile::tempdir().unwrap();
                for i in 0..100 {
                    fs::write(
                        dir.path().join(format!("p-{i}.md")),
                        format!("---\ntitle: P{i}\n---\n# P{i}"),
                    )
                    .unwrap();
                }
                dir
            },
            |dir| {
                let _: Vec<Entry<Post>> = get_collection(dir.path()).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_get_collection_100);
