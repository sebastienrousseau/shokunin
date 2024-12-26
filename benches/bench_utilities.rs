// Copyright © 2023-2025 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use staticdatagen::utilities::directory::directory;
use tempfile::TempDir;

/**
 * This function is used to benchmark the utilities of the Shokunin Static Site Generator.
 * It creates a temporary directory, performs various operations on it, and measures their performance.
 *
 * # Arguments
 *
 * * `c` - A reference to a `Criterion` object, which is used to define and run benchmarks.
 */
#[allow(dead_code)]
pub(crate) fn bench_utilities(c: &mut Criterion) {
    // Creates a temporary directory and gets its path.
    let tempdir = TempDir::new().unwrap();
    let dir = tempdir.path().join("test_dir");

    // Benchmarks the creation of a directory.
    let _ = c.bench_function("create directory", |b| {
        b.iter(|| {
            // Creates a directory with the given path and name.
            let result =
                directory(black_box(&dir), black_box("test_dir"));
            // Asserts that the result is Ok, indicating that the directory was created successfully.
            assert!(result.is_ok());
        })
    });

    // Benchmarks checking if a directory exists.
    let _ = c.bench_function("check if directory exists", |b| {
        b.iter(|| {
            // Checks if the directory exists.
            let result = dir.exists();
            // Asserts that the result is true, indicating that the directory exists.
            assert!(result);
        })
    });

    // Benchmarks checking if a directory is a directory.
    let _ =
        c.bench_function("check if directory is a directory", |b| {
            b.iter(|| {
                // Checks if the directory is a directory.
                let result = dir.is_dir();
                // Asserts that the result is true, indicating that the directory is a directory.
                assert!(result);
            })
        });

    // Benchmarks checking if a non-existent directory does not exist.
    let _ = c.bench_function(
        "check if non-existent directory does not exist",
        |b| {
            let non_existent_dir = tempdir.path().join("non-existent");
            b.iter(|| {
                // Checks if the non-existent directory exists.
                let result = non_existent_dir.exists();
                // Asserts that the result is false, indicating that the non-existent directory does not exist.
                assert!(!result);
            })
        },
    );
}
