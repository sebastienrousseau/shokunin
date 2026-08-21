// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmarks for the HTML paths that moved from byte scans to a parser.
//!
//! ssg#539, #540 and #570 replaced `str::find` scans with `lol_html` (or,
//! where the caller reassembles the document by string surgery, with
//! comment masking). Those scans matched their target bytes inside HTML
//! comments, so a commented-out `<script>` was hashed into CSP and hoisted
//! into a real external file.
//!
//! Correctness cost throughput, and this exists so the cost is a tracked
//! number rather than something rediscovered by bisecting a slow build. A
//! parser builds a state machine; `memchr` does not. The regression is the
//! price of the fix, not a defect in it.
//!
//! Each input comes in two shapes: `clean`, and `decoy` — the same document
//! with a commented-out block ahead of the real one. The decoy is the case
//! the old code got wrong, so it is the case worth measuring.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

/// A page with a realistic head: several meta tags, a stylesheet, a script.
fn page(decoy: bool) -> String {
    let mut s = String::from("<!DOCTYPE html><html lang=\"en\"><head>");
    if decoy {
        s.push_str("<!-- <meta name=\"description\" content=\"OLD\"> -->");
        s.push_str("<!-- <script>legacy()</script> -->");
        s.push_str("<!-- </head> -->");
    }
    s.push_str("<title>Benchmark Page</title>");
    for i in 0..12 {
        s.push_str(&format!(
            "<meta name=\"m{i}\" content=\"value {i} with some length to it\">"
        ));
    }
    s.push_str("<meta name=\"description\" content=\"REAL\">");
    s.push_str("<link rel=\"canonical\" href=\"https://example.com/p\">");
    s.push_str("<style>.a{color:red}.b{color:blue}</style>");
    s.push_str("<script>window.x=1;window.y=2;</script>");
    s.push_str("</head><body><main><p>Body copy.</p></main></body></html>");
    s
}

fn bench_head_injection(c: &mut Criterion) {
    let clean = page(false);
    let decoy = page(true);
    let payload = "<meta name=\"injected\" content=\"1\">";

    let mut g = c.benchmark_group("html_scanning::inject_before_head_close");
    let _ = g.bench_function("clean", |b| {
        b.iter(|| {
            ssg::util::head_dom::inject_before_head_close(
                black_box(&clean),
                black_box(payload),
            )
        });
    });
    // The shape the byte splice got wrong: it injected inside the comment.
    let _ = g.bench_function("decoy", |b| {
        b.iter(|| {
            ssg::util::head_dom::inject_before_head_close(
                black_box(&decoy),
                black_box(payload),
            )
        });
    });
    g.finish();
}

fn bench_head_meta(c: &mut Criterion) {
    let clean = page(false);
    let decoy = page(true);

    let mut g = c.benchmark_group("html_scanning::extract_head_meta");
    let _ = g.bench_function("clean", |b| {
        b.iter(|| ssg::util::head_dom::extract_head_meta(black_box(&clean)));
    });
    let _ = g.bench_function("decoy", |b| {
        b.iter(|| ssg::util::head_dom::extract_head_meta(black_box(&decoy)));
    });
    g.finish();
}

fn bench_tag_end(c: &mut Criterion) {
    // The scanner #711 collapsed from four copies to one. Two shapes: a
    // plain tag, and one whose attribute value contains `>` — the case the
    // skip logic exists for.
    let plain = "<img src=\"a.png\" alt=\"plain\">rest";
    let quoted =
        "<img src=\"data:image/svg+xml,<svg><path d='M0 0'/></svg>\" alt=\"x\">rest";

    let mut g = c.benchmark_group("html_scanning::find_tag_end");
    let _ = g.bench_function("plain", |b| {
        b.iter(|| {
            ssg::audit::gates::find_tag_end(black_box(plain), black_box(0))
        });
    });
    let _ = g.bench_function("quoted_gt", |b| {
        b.iter(|| {
            ssg::audit::gates::find_tag_end(black_box(quoted), black_box(0))
        });
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_head_injection,
    bench_head_meta,
    bench_tag_end
);
criterion_main!(benches);
