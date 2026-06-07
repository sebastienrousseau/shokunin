// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

use criterion::{criterion_group, Criterion};
use ssg::markdown_ext::expand_gfm;

const SAMPLE: &str = "# Header\n\n\
| col | val |\n| --- | --- |\n| 1 | a |\n| 2 | b |\n\n\
- [x] done\n- [ ] todo\n\n\
~~struck~~ regular **bold** _italic_.\n";

fn bench_expand_gfm(c: &mut Criterion) {
    c.bench_function("markdown_ext::expand_gfm", |b| {
        b.iter(|| expand_gfm(SAMPLE, None));
    });
}

criterion_group!(benches, bench_expand_gfm);
