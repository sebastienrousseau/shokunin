// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use ssg::modules::html::generate_html;

/// Benchmarks the generation of HTML content.
///
/// This function sets up a benchmark for measuring the performance of the `generate_html` function.
/// It uses the `criterion` crate to run the benchmark and measure the time taken to generate HTML.
///
/// # Arguments
///
/// * `c` - A mutable reference to a `Criterion` instance, which is used to set up and run the benchmark.
///
/// # Panics
///
/// This function panics if the `generate_html` function returns an `Err` value, indicating that HTML generation failed.
#[allow(dead_code)]
pub(crate) fn bench_generate_html(c: &mut Criterion) {
    // A sample content string for the HTML generation.
    //
    // This string is used as the content for the HTML page being generated.
    let content = "## Hello, world!\n\nThis is a test.";

    // The title of the HTML page.
    //
    // This string is used as the title of the HTML page being generated.
    let title = "My Page";

    // The description of the HTML page.
    //
    // This string is used as the description of the HTML page being generated.
    let description = "This is a test page";

    // Optional JSON content to be included in the HTML page.
    //
    // This value is used as the optional JSON content for the HTML page being generated.
    let json_content = Some("{\"name\": \"value\"}");

    // Benchmarks the generation of HTML using the provided inputs.
    //
    // This function sets up a benchmark to measure the time taken to generate HTML using the given inputs.
    // It uses the `black_box` function to ensure that the inputs are not optimized away by the compiler.
    //
    // # Arguments
    //
    // * `b` - A builder for the benchmark, which is used to configure the benchmark and specify the code to be benchmarked.
    //
    // # Returns
    //
    // This function does not return a value. It sets up a benchmark and measures the time taken to generate HTML.
    c.bench_function("generate_html", |b| {
        b.iter(|| {
            let html = generate_html(
                black_box(content),
                black_box(title),
                black_box(description),
                black_box(json_content),
            );
            match html {
                Ok(_) => (),
                Err(_) => panic!("HTML generation failed"),
            }
            let _ = black_box(html);
        })
    });
}
