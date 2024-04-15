// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use ssg::utilities::file::add;
use std::path::PathBuf;

/// Runs a benchmark that measures the performance of the `add` function.
///
/// # Arguments
///
/// * `c` - A reference to a `Criterion` instance.
#[allow(dead_code)]
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
