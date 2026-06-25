#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Runtime budget assertions for `ssg audit` (issue #549).
//!
//! The issue body specifies: "full audit on the scaffold site
//! completes in ≤ 5 s". Mirrors the scaffold layout used by the
//! criterion benchmark in `benches/bench_audit.rs` so the budget
//! check is reproducible from `cargo test`.

use ssg::audit::{AuditConfig, AuditRunner, Site};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, body).unwrap();
}

fn scaffold_site() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
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
         </head><body><main><h1>H</h1></main></body></html>",
    );
    for i in 0..9 {
        write(
            root,
            &format!("page-{i}.html"),
            &format!(
                "<!doctype html><html lang=\"en\"><head>\
                 <meta charset=\"utf-8\"><title>P{i}</title>\
                 <meta name=\"description\" content=\"d\">\
                 <meta property=\"og:title\" content=\"P\">\
                 <meta property=\"og:type\" content=\"website\">\
                 <meta property=\"og:image\" content=\"/og.png\">\
                 <meta name=\"twitter:card\" content=\"summary\">\
                 <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'\">\
                 </head><body><main><h1>P{i}</h1></main></body></html>"
            ),
        );
    }
    tmp
}

#[test]
fn full_audit_on_scaffold_completes_under_five_seconds() {
    let tmp = scaffold_site();
    let site = Site::load(tmp.path()).unwrap();
    let runner = AuditRunner::new(AuditConfig::new());
    let start = Instant::now();
    let report = runner.run(&site);
    let elapsed = start.elapsed();
    assert_eq!(report.gates.len(), 14);
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "audit took {elapsed:?}; budget is 5 s"
    );
    eprintln!(
        "[audit_perf] scaffold ({} pages) → {:.2?}",
        site.html_files.len(),
        elapsed
    );
}
