// Copyright © 2024-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::modules::html::generate_html;

pub fn bench_generate_html(c: &mut Criterion) {
    let content = "## Hello, world!\n\nThis is a test.";
    let title = "My Page";
    let description = "This is a test page";
    let json_content = Some("{\"name\": \"value\"}");

    c.bench_function("generate_html", |b| {
        b.iter(|| {
            let html = generate_html(
                black_box(content),
                black_box(title),
                black_box(description),
                black_box(json_content),
            );
            match html {
                Ok(_) => (),
                Err(_) => panic!("HTML generation failed"),
            }
            let _ = criterion::black_box(html);
        })
    });
}
