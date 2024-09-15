// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

#![allow(missing_docs)]

//! # Markdown to HTML Benchmark
//!
//! This benchmark test uses the `ssg-markdown` crate to measure the performance of converting
//! Markdown content into HTML. The `criterion` crate is used to handle the benchmarking process.
//!
//! ## Usage
//!
//! Run the benchmark to evaluate the performance of the Markdown conversion process
//! by executing `cargo bench`.

use comrak::ComrakOptions;
use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg_markdown::process_markdown;

/// Benchmark the Markdown to HTML conversion process.
fn markdown_benchmark(c: &mut Criterion) {
    let markdown = r#"
# Welcome to SSG Markdown

This is a **bold** statement and this is *italic*.

## Features

- Easy to use
- Extensible
- Fast

Check out [our website](https://example.com) for more information.
    "#;

    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;

    c.bench_function("Markdown to HTML Conversion", |b| {
        b.iter(|| {
            let _html = process_markdown(
                black_box(markdown),
                black_box(&options),
            )
            .unwrap();
        })
    });
}

// Define the benchmark group and main function for Criterion.
criterion_group!(benches, markdown_benchmark);
criterion_main!(benches);
