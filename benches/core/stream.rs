// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::stream`.

use std::fs;

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::stream::{stream_copy, stream_hash};

fn bench_stream_copy_64k(c: &mut Criterion) {
    c.bench_function("stream::copy_64k", |b| {
        b.iter_batched(
            || {
                let dir = tempfile::tempdir().unwrap();
                let src = dir.path().join("src.bin");
                let dst = dir.path().join("dst.bin");
                fs::write(&src, vec![0u8; 64 * 1024]).unwrap();
                (dir, src, dst)
            },
            |(_dir, src, dst)| {
                stream_copy(&src, &dst).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_stream_hash_64k(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("h.bin");
    fs::write(&path, vec![0u8; 64 * 1024]).unwrap();
    c.bench_function("stream::hash_64k", |b| {
        b.iter(|| stream_hash(&path).unwrap());
    });
}

criterion_group!(benches, bench_stream_copy_64k, bench_stream_hash_64k);
