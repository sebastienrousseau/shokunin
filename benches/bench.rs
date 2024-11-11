// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate is responsible for benchmarking various components of the application.

#![allow(missing_docs)]
use criterion::Criterion;

/// This is a module for benchmarking file operations.
mod bench_file;
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
}

// Run benchmarks
criterion::criterion_main!(benches);
