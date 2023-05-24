// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::html::generate_html;

pub fn bench_html(c: &mut Criterion) {
    let content = "---\n\
                   title: My Title\n\
                   description: My Description\n\
                   keywords: foo, bar, baz\n\
                   permalink: /my-permalink\n\
                   ---\n\
                   My content";
    c.bench_function("generate HTML", |b| {
        b.iter(|| {
            let result = generate_html(
                black_box(content),
                black_box("My Title"),
                black_box("My Description"),
                black_box(None),
            );
            assert!(result.contains("<h1>My Title</h1>"));
            assert!(result.contains("<h2>My Description</h2>"));
            assert!(result.contains("<p>My content</p>"));
        })
    });
}
