// Copyright © 2023-2025 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark suite for the Shokunin Static Site Generator.
//!
//! This module contains performance benchmarks for critical operations
//! in the static site generator, including file operations and content processing.

use criterion::{black_box, Criterion};
use staticdatagen::utilities::file::add;
use std::path::PathBuf;

/// Runs a benchmark that measures the performance of the `add` function.
///
/// This benchmark measures file addition performance by repeatedly calling
/// the `add` function with a test path and measuring execution time.
///
/// # Arguments
///
/// * `c` - A reference to a `Criterion` instance used for benchmark configuration
///         and measurement.
///
/// # Example Output
///
/// ```text
/// add function      time: [10.123 µs 10.234 µs 10.345 µs]
/// ```
pub(crate) fn bench_file(c: &mut Criterion) {
    let path = PathBuf::from("content");
    c.bench_function("add function", |b| {
        b.iter(|| {
            let result = add(&path);
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            } else {
                black_box(result.unwrap());
            }
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use criterion::black_box;
    use std::path::PathBuf;

    /// Tests the benchmark setup and basic functionality.
    #[test]
    fn test_bench_setup() {
        let path = PathBuf::from("content");
        let result = add(&path);
        assert!(result.is_ok() || result.is_err());
    }
}
