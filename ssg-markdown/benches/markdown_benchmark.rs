// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use ssg_markdown::{process_markdown, MarkdownOptions};
use comrak::ComrakOptions;

/// Create a valid MarkdownOptions configuration
fn create_valid_options(
    syntax_highlighting: bool,
    custom_blocks: bool,
    enhanced_tables: bool,
    enable_comrak_tables: bool,
) -> MarkdownOptions<'static> {
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.table = enable_comrak_tables;

    MarkdownOptions::new()
        .with_syntax_highlighting(syntax_highlighting)
        .with_custom_blocks(custom_blocks)
        .with_enhanced_tables(enhanced_tables)
        .with_comrak_options(comrak_options)
}

/// Benchmark the Markdown to HTML conversion process with various configurations.
fn markdown_benchmark(c: &mut Criterion) {
    let small_markdown = r#"
# Welcome to SSG Markdown
This is a **bold** statement and this is *italic*.
## Features
- Easy to use
- Extensible
- Fast
Check out [our website](https://example.com) for more information.
    "#;

    let large_markdown = include_str!("../README.md");

    let markdown_sizes = vec![
        ("small", small_markdown),
        ("large", large_markdown),
    ];

    let mut group = c.benchmark_group("Markdown to HTML Conversion");

    for (size, markdown) in markdown_sizes.iter() {
        group.throughput(Throughput::Bytes(markdown.len() as u64));

        // Basic conversion (no enhanced tables)
        let basic_options = create_valid_options(false, false, false, false);
        group.bench_with_input(BenchmarkId::new("basic", size), markdown, |b, markdown| {
            b.iter(|| {
                let _ = process_markdown(black_box(markdown), black_box(&basic_options))
                    .expect("Basic conversion should not fail");
            });
        });

        // Full-featured conversion
        let full_options = create_valid_options(true, true, true, true);
        group.bench_with_input(BenchmarkId::new("full", size), markdown, |b, markdown| {
            b.iter(|| {
                let _ = process_markdown(black_box(markdown), black_box(&full_options))
                    .expect("Full-featured conversion should not fail");
            });
        });

        // Custom configuration
        let custom_options = create_valid_options(true, false, true, true);
        group.bench_with_input(BenchmarkId::new("custom", size), markdown, |b, markdown| {
            b.iter(|| {
                let _ = process_markdown(black_box(markdown), black_box(&custom_options))
                    .expect("Custom conversion should not fail");
            });
        });
    }

    group.finish();
}

criterion_group!(benches, markdown_benchmark);
criterion_main!(benches);
