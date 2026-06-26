// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::depgraph::DepGraph`.

use std::path::PathBuf;

use criterion::{criterion_group, Criterion};
use ssg::depgraph::DepGraph;

fn populated(pages: usize, deps_per_page: usize) -> DepGraph {
    let mut g = DepGraph::new();
    for p in 0..pages {
        let page = PathBuf::from(format!("page-{p}.md"));
        for d in 0..deps_per_page {
            g.add_dep(&page, &PathBuf::from(format!("dep-{d}.html")));
        }
    }
    g
}

fn bench_invalidated_pages_1000(c: &mut Criterion) {
    let g = populated(1000, 4);
    let changed = vec![PathBuf::from("dep-0.html")];
    c.bench_function("depgraph::invalidated_pages_1000x4", |b| {
        b.iter(|| g.invalidated(&changed));
    });
}

criterion_group!(benches, bench_invalidated_pages_1000);
