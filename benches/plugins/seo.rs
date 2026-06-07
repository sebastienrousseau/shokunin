// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

use criterion::{criterion_group, Criterion};
use ssg::seo::helpers::{extract_title, has_meta_tag};
use ssg::seo::validate_jsonld;

const SAMPLE_HTML: &str = r#"
<html>
  <head>
    <title>Sample Page</title>
    <meta name="description" content="Lorem ipsum">
    <script type="application/ld+json">
      {"@context":"https://schema.org","@type":"WebPage","name":"Sample"}
    </script>
  </head>
  <body><h1>Sample</h1></body>
</html>
"#;

fn bench_extract_title(c: &mut Criterion) {
    c.bench_function("seo::helpers::extract_title", |b| {
        b.iter(|| extract_title(SAMPLE_HTML));
    });
}

fn bench_has_meta_tag(c: &mut Criterion) {
    c.bench_function("seo::helpers::has_meta_tag", |b| {
        b.iter(|| has_meta_tag(SAMPLE_HTML, "description"));
    });
}

fn bench_validate_jsonld(c: &mut Criterion) {
    c.bench_function("seo::jsonld::validate_jsonld", |b| {
        b.iter(|| validate_jsonld(SAMPLE_HTML));
    });
}

criterion_group!(
    benches,
    bench_extract_title,
    bench_has_meta_tag,
    bench_validate_jsonld
);
