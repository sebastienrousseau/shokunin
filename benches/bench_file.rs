// Copyright © 2023-2025 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use staticdatagen::utilities::file::add;
use std::path::PathBuf;

/// Runs a benchmark that measures the performance of the `add` function.
///
/// # Arguments
///
/// * `c` - A reference to a `Criterion` instance.
#[allow(dead_code)]
pub(crate) fn bench_file(c: &mut Criterion) {
    let path = PathBuf::from("content");
    let _ = c.bench_function("add function", |b| {
        b.iter(|| {
            let result = add(&path);
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            } else {
                let _ = black_box(result.unwrap());
            }
        })
    });
}
