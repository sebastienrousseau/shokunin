// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::frontmatter`.

use std::fs;
use std::path::Path;

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::frontmatter::emit_sidecars;

fn seed_markdown(content: &Path, count: usize) {
    fs::create_dir_all(content).unwrap();
    for i in 0..count {
        fs::write(
            content.join(format!("post-{i:04}.md")),
            format!(
                "---\ntitle: \"Post {i}\"\ndate: 2026-06-07\n---\n# Post {i}\n\n\
                 Body content that is long enough to compute a real word count \
                 and reading time estimate from the parser pass."
            ),
        )
        .unwrap();
    }
}

fn bench_emit_sidecars_50(c: &mut Criterion) {
    c.bench_function("frontmatter::emit_sidecars_50", |b| {
        b.iter_batched(
            || {
                let dir = tempfile::tempdir().unwrap();
                let content = dir.path().join("content");
                seed_markdown(&content, 50);
                let sidecar = dir.path().join("sidecars");
                (dir, content, sidecar)
            },
            |(_dir, content, sidecar)| {
                emit_sidecars(&content, &sidecar).unwrap()
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_emit_sidecars_50);
