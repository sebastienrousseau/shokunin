// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::utilities::directory::directory;
use tempfile::TempDir;

pub fn bench_utilities(c: &mut Criterion) {
    let tempdir = TempDir::new().unwrap();
    let dir = tempdir.path().join("test_dir");

    c.bench_function("create directory", |b| {
        b.iter(|| {
            let result = directory(black_box(&dir), black_box("test_dir"));
            assert!(result.is_ok());
        })
    });

    c.bench_function("check if directory exists", |b| {
        b.iter(|| {
            let result = dir.exists();
            assert!(result);
        })
    });

    c.bench_function("check if directory is a directory", |b| {
        b.iter(|| {
            let result = dir.is_dir();
            assert!(result);
        })
    });

    c.bench_function("check if non-existent directory does not exist", |b| {
        let non_existent_dir = tempdir.path().join("non-existent");
        b.iter(|| {
            let result = non_existent_dir.exists();
            assert!(!result);
        })
    });
}
