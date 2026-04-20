#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate is responsible for benchmarking various components of the application.

#![allow(missing_docs)]
use criterion::Criterion;

/// This is a module for benchmarking file operations.
mod bench_file;
/// Scalability benchmarks at 100, 1K, and 10K page counts.
mod bench_scalability;
/// End-to-end site generation benchmarks at varying page counts.
mod bench_site_generation;
/// This is a module for benchmarking yaml operations.
mod bench_utilities;

// Entry point for all benchmarks.
criterion::criterion_group! {
    // Name of the group.
    name = benches;
    // Description of the group.
    config = Criterion::default();
    // Targets of the group.
    targets =
        bench_file::bench_file,
        bench_utilities::bench_utilities,
        bench_site_generation::bench_site_generation,
}

// Run benchmarks
criterion::criterion_main!(benches, bench_scalability::scalability);
