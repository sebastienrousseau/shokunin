// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::html::generate_html;

pub fn bench_generate_html(c: &mut Criterion) {
    let content = "## Hello, world!\n\nThis is a test.";
    let title = "My Page";
    let description = "This is a test page";
    let json_content = Some("{\"name\": \"value\"}");

    c.bench_function("generate_html", |b| {
        b.iter(|| {
            let html = generate_html(black_box(content), black_box(title), black_box(description), black_box(json_content));
            criterion::black_box(html);
        })
    });
}

