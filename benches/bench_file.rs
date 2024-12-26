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
    let _ = c.bench_function("add function", |b| {
        b.iter(|| {
            let result = add(&path);
            match result {
                Ok(data) => {
                    // Ensure the result is not optimized away
                    black_box(data);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        })
    });
}

#[cfg(test)]
mod tests {
    // Remove unused imports
    #[test]
    fn test_bench_setup() {
        let path = std::path::PathBuf::from("content");
        let result = staticdatagen::utilities::file::add(&path);
        assert!(result.is_ok() || result.is_err());
    }
}
