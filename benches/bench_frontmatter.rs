// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::frontmatter::{extract, parse_yaml_document};

pub fn bench_extract(c: &mut Criterion) {
    let content = r#"
    ---
    title: "Test"
    ---
    "#;
    c.bench_function("extract", |b| {
        b.iter(|| extract(black_box(content)))
    });
}

pub fn bench_parse_yaml_document(c: &mut Criterion) {
    let content = r#"
    title: "Test"
    "#;
    c.bench_function("parse_yaml_document", |b| {
        b.iter(|| parse_yaml_document(black_box(content)))
    });
}
