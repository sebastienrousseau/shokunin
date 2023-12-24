// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This crate is responsible for benchmarking various components of the application.
#![allow(missing_docs)]
extern crate criterion;
use criterion::Criterion;

/// This is a module for benchmarking file operations.
mod bench_file;
/// This is a module for benchmarking frontmatter operations.
mod bench_frontmatter;
/// This is a module for benchmarking html operations.
mod bench_html;
/// This is a module for benchmarking json operations.
mod bench_json;
/// This is a module for benchmarking markdown operations.
mod bench_metatags;
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
        bench_frontmatter::bench_extract,
        bench_frontmatter::bench_parse_yaml_document,
        bench_html::bench_generate_html,
        bench_json::bench_json,
        bench_metatags::bench_metatags,
        bench_utilities::bench_utilities,
}

// Run benchmarks
criterion::criterion_main!(benches);
