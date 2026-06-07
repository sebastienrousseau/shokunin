// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::content` schema parsing.

use criterion::{criterion_group, Criterion};
use ssg::content::parse_schemas;

const POST_SCHEMA: &str = r#"
[[schemas]]
content_type = "post"
[[schemas.fields]]
name = "title"
field_type = "String"
required = true
[[schemas.fields]]
name = "date"
field_type = "DateTime"
required = true
"#;

fn bench_parse_schemas(c: &mut Criterion) {
    c.bench_function("content::parse_schemas", |b| {
        b.iter(|| parse_schemas(POST_SCHEMA).unwrap());
    });
}

criterion_group!(benches, bench_parse_schemas);
