// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use ssg_frontmatter::{extract, parse_yaml_document};

/// Benchmarks the extraction of frontmatter from a content string.
///
/// This function sets up a benchmark for the `extract` function, which extracts
/// the frontmatter from a content string. The content string is hardcoded in
/// this function.
///
/// # Arguments
///
/// * `c` - A mutable reference to a `Criterion` instance, which is used to
///   define and run benchmarks.
///
/// # Example
///
/// ```rust
/// use criterion::{criterion_group, criterion_main, Criterion};
/// use ssg_frontmatter::frontmatter::bench_extract;
///
/// let mut group = criterion_group::CriterionGroup::new();
/// group.bench_function("extract", |b| bench_extract(b));
/// criterion_main(group);
/// ```
///
/// # Panics
///
/// This function does not panic.
#[allow(dead_code)]
pub(crate) fn bench_extract(c: &mut Criterion) {
    let content = r#"
    ---
    title: "Test"
    ---
    "#;

    // Benchmarks the execution of the `extract` function.
    //
    // This function sets up a benchmark for the execution of the `extract`
    // function, which extracts the frontmatter from a content string. The
    // content string is hardcoded in this function.
    //
    // # Arguments
    //
    // * `b` - A mutable reference to a `Bencher` instance, which is used to
    //   define and run benchmarks.
    //
    // # Example
    //
    // ```rust
    // use criterion::{criterion_group, criterion_main, Criterion};
    // use ssg_frontmatter::bench_extract;
    //
    // let mut group = criterion_group::CriterionGroup::new();
    // group.bench_function("extract", |b| bench_extract(b));
    // criterion_main(group);
    // ```
    //
    // # Panics
    //
    // This function does not panic.
    let _ = c.bench_function("extract", |b| {
        b.iter(|| extract(black_box(content)))
    });
}

/// Benchmarks the parsing of YAML document from a content string.
///
/// This function sets up a benchmark for the `parse_yaml_document` function,
/// which parses a YAML document from a content string. The content string is
/// hardcoded in this function.
///
/// # Arguments
///
/// * `c` - A mutable reference to a `Criterion` instance, which is used to
///   define and run benchmarks.
///
/// # Example
///
/// ```rust
/// use criterion::{criterion_group, criterion_main, Criterion};
/// use shokunin_static_site_generator::ssg::modules::frontmatter::bench_parse_yaml_document;
///
/// let mut group = criterion_group::CriterionGroup::new();
/// group.bench_function("parse_yaml_document", |b| bench_parse_yaml_document(b));
/// criterion_main(group);
/// ```
///
/// # Panics
///
/// This function does not panic.
#[allow(dead_code)]
pub(crate) fn bench_parse_yaml_document(c: &mut Criterion) {
    let content = r#"
    title: "Test"
    "#;

    // Benchmarks the execution of the `parse_yaml_document` function.
    //
    // This function sets up a benchmark for the execution of the `parse_yaml_document`
    // function, which parses a YAML document from a content string. The
    // content string is hardcoded in this function.
    //
    // # Arguments
    //
    // * `b` - A mutable reference to a `Bencher` instance, which is used to
    //   define and run benchmarks.
    //
    // # Example
    //
    // ```rust
    // use criterion::{criterion_group, criterion_main, Criterion};
    // use shokunin_static_site_generator::ssg::modules::frontmatter::bench_parse_yaml_document;
    //
    // fn main() {
    //     let mut group = criterion_group::CriterionGroup::new();
    //     group.bench_function("parse_yaml_document", |b| bench_parse_yaml_document(b));
    //     criterion_main(group);
    // }
    // ```
    //
    // # Panics
    //
    // This function does not panic.
    let _ = c.bench_function("parse_yaml_document", |b| {
        b.iter(|| parse_yaml_document(black_box(content)))
    });
}
