// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

use criterion::{criterion_group, Criterion};
use ssg::shortcodes::expand_shortcodes;

const SAMPLE: &str = "before {{ youtube(id=\"abc\") }} mid \
                      {{ figure(src=\"a.jpg\", caption=\"x\") }} after";

fn bench_expand_shortcodes(c: &mut Criterion) {
    c.bench_function("shortcodes::expand_shortcodes", |b| {
        b.iter(|| expand_shortcodes(SAMPLE));
    });
}

criterion_group!(benches, bench_expand_shortcodes);
