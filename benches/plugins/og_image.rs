// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

use criterion::{criterion_group, Criterion};
use ssg::og_image::generate_og_svg;

fn bench_generate_og_svg(c: &mut Criterion) {
    c.bench_function("og_image::generate_og_svg", |b| {
        b.iter(|| {
            generate_og_svg(
                "Hello World",
                "Example Site",
                "#1a1a2e",
                "#e9ecef",
            )
        });
    });
}

criterion_group!(benches, bench_generate_og_svg);
