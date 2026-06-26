#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmarks for the `ssg audit` runtime (issue #549).
//!
//! Targets a scaffold-sized site (≈ 10 HTML pages) to verify the
//! "full audit completes in ≤ 5 s" acceptance criterion from the
//! issue. Real-world sites scale roughly linearly in page count;
//! `bench_audit_scaffold` is the single canonical measurement.

use criterion::Criterion;
use ssg::audit::{AuditConfig, AuditRunner, Site};
use std::fs;
use std::hint::black_box;
use std::path::Path;

/// Materialises a 10-page scaffold site in a tempdir and returns the
/// path so callers can pass it to [`Site::load`].
fn scaffold_site() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    // Index page.
    write(
        root,
        "index.html",
        "<!doctype html><html lang=\"en\"><head>\
         <meta charset=\"utf-8\"><title>Home</title>\
         <meta name=\"description\" content=\"home\">\
         <meta property=\"og:title\" content=\"H\">\
         <meta property=\"og:type\" content=\"website\">\
         <meta property=\"og:image\" content=\"/og.png\">\
         <meta name=\"twitter:card\" content=\"summary\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
         </head><body><main><h1>H</h1><a href=\"/about/\">about</a>\
         <img src=\"/a.jpg\" alt=\"a\" width=\"10\" height=\"10\"></main></body></html>",
    );
    write(
        root,
        "about/index.html",
        "<!doctype html><html lang=\"en\"><head>\
         <meta charset=\"utf-8\"><title>About</title>\
         <meta name=\"description\" content=\"about\">\
         <meta property=\"og:title\" content=\"A\">\
         <meta property=\"og:type\" content=\"website\">\
         <meta property=\"og:image\" content=\"/og.png\">\
         <meta name=\"twitter:card\" content=\"summary\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
         </head><body><main><h1>About</h1></main></body></html>",
    );
    for i in 0..8 {
        write(
            root,
            &format!("blog/{i}.html"),
            &format!(
                "<!doctype html><html lang=\"en\"><head>\
                 <meta charset=\"utf-8\"><title>Post {i}</title>\
                 <meta name=\"description\" content=\"d\">\
                 <meta property=\"og:title\" content=\"P\">\
                 <meta property=\"og:type\" content=\"article\">\
                 <meta property=\"og:image\" content=\"/og.png\">\
                 <meta name=\"twitter:card\" content=\"summary_large_image\">\
                 <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
                 </head><body><main><h1>Post {i}</h1></main></body></html>"
            ),
        );
    }
    // Companion assets so the link + image gates have real targets.
    fs::write(root.join("a.jpg"), vec![0u8; 100]).unwrap();
    fs::write(root.join("a.webp"), vec![0u8; 100]).unwrap();
    tmp
}

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, body).unwrap();
}

/// Benchmarks a full 14-gate audit pass over the scaffold site.
#[allow(dead_code, unreachable_pub)]
pub fn bench_audit_scaffold(c: &mut Criterion) {
    let tmp = scaffold_site();
    let site = Site::load(tmp.path()).expect("load scaffold");
    let runner = AuditRunner::new(AuditConfig::new());

    let _ = c.bench_function("audit_full_scaffold", |b| {
        b.iter(|| {
            let report = runner.run(&site);
            let _ = black_box(report);
        });
    });
}

// The 5-second-budget assertion lives in `tests/audit_perf.rs` —
// criterion's harness owns the bench-binary entry point so unit-tests
// inside bench files aren't picked up by `cargo test --bench`.
