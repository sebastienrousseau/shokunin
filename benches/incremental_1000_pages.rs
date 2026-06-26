// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Issue #524 acceptance bench: warm-cache rebuild ≤ 200ms on a
//! 1000-page fixture after a single-content edit.
//!
//! Run with:
//!
//! ```bash
//! cargo bench --bench incremental_1000_pages
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, BatchSize, Criterion, SamplingMode,
};
use ssg::depgraph::{self, DepGraph};

fn write(p: &Path, body: &str) {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, body).unwrap();
}

/// Builds a self-contained 1000-page fixture under a tempdir and
/// returns its layout.
struct Fixture {
    _tmp: tempfile::TempDir,
    content: PathBuf,
    template: PathBuf,
    build: PathBuf,
    cache: PathBuf,
}

fn generate_corpus(n: usize) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    let template = tmp.path().join("templates");
    let build = tmp.path().join("public");
    let cache = tmp.path().join(".ssg-cache");
    fs::create_dir_all(&content).unwrap();
    fs::create_dir_all(&template).unwrap();
    fs::create_dir_all(&build).unwrap();
    write(&template.join("post.html"), "<html>{{title}}</html>");
    for i in 0..n {
        let body = format!(
            "---\nlayout: \"post\"\ntitle: \"Page {i}\"\n---\n# Page {i}\n"
        );
        write(&content.join(format!("p-{i}.md")), &body);
    }
    Fixture {
        _tmp: tmp,
        content,
        template,
        build,
        cache,
    }
}

/// Measures the warm-cache hot path: load graph → hash all sources →
/// diff. This is the dominant cost of a no-op incremental rebuild;
/// the actual file-system writes only fire when the diff is
/// non-empty.
fn bench_warm_cache_1000_pages(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");
    group.sample_size(20);
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(8));

    let f = generate_corpus(1000);
    let mut graph = DepGraph::new();
    depgraph::populate(&mut graph, &f.content, &f.template, &f.build).unwrap();
    graph.save(&f.cache).unwrap();

    group.bench_function("warm_cache_1000_pages_noop", |b| {
        b.iter(|| {
            let warm = DepGraph::load(&f.cache);
            let current =
                depgraph::current_hashes(&f.content, &f.template).unwrap();
            let diff = warm.diff(&current);
            assert!(diff.is_empty());
        });
    });

    group.finish();
}

/// Measures the single-edit warm path: one markdown file's body is
/// mutated, the graph is loaded, sources are re-hashed, and the
/// invalidated output set is computed. Mirrors the inner loop
/// performed by `ssg build --incremental` before any actual rendering
/// fires.
fn bench_single_edit_1000_pages(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");
    group.sample_size(20);
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(8));

    let f = generate_corpus(1000);
    let mut graph = DepGraph::new();
    depgraph::populate(&mut graph, &f.content, &f.template, &f.build).unwrap();
    graph.save(&f.cache).unwrap();
    let target = f.content.join("p-500.md");

    group.bench_function("single_edit_1000_pages_invalidation", |b| {
        b.iter_batched(
            || {
                // Touch the file with a new body so the SHA changes.
                let body = format!(
                    "---\nlayout: \"post\"\ntitle: \"Page 500 v{}\"\n---\n",
                    rand_nonce()
                );
                fs::write(&target, body).unwrap();
            },
            |()| {
                let warm = DepGraph::load(&f.cache);
                let current =
                    depgraph::current_hashes(&f.content, &f.template).unwrap();
                let diff = warm.diff(&current);
                let invalidated = warm.invalidated_outputs(&diff.changed);
                assert_eq!(
                    invalidated.len(),
                    1,
                    "exactly one output should be invalidated"
                );
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Cheap LCG-driven nonce so each benchmark iteration writes a
/// distinct body without pulling `rand` into dev-deps.
fn rand_nonce() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0x1234_5678_9abc_def0);
    let prev = STATE.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
    prev.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

criterion_group! {
    name = incremental;
    config = Criterion::default().measurement_time(Duration::from_secs(8));
    targets = bench_warm_cache_1000_pages, bench_single_edit_1000_pages
}
criterion_main!(incremental);
