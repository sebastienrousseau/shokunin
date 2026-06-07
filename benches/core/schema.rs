// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::schema`.

use criterion::{criterion_group, Criterion};
use ssg::schema::generate_schema;

fn bench_generate_schema(c: &mut Criterion) {
    c.bench_function("schema::generate_schema", |b| {
        b.iter(generate_schema);
    });
}

criterion_group!(benches, bench_generate_schema);
