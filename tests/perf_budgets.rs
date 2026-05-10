// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hard performance budgets — these tests fail CI when wall-clock
//! exceeds the documented threshold.
//!
//! Where `tests/perf_regression.rs` checks fine-grained per-operation
//! sub-millisecond budgets (slugify, URL parsing, frontmatter walk),
//! this suite checks **end-to-end build budgets** at realistic page
//! counts, against the same `compile_site` entry point a user
//! invokes via `cargo run`.
//!
//! ## Budget table (resolves issue #471 P0 + analysis batch E)
//!
//! | Pages | Local M-arm64 baseline | CI budget (this gate) |
//! |-------|------------------------|------------------------|
//! |   10  | < 5 ms                | **< 100 ms**           |
//! |  100  | 18.7 ms (full build)  | **< 500 ms**           |
//! |  500  | extrapolated < 100 ms | **< 2,000 ms**         |
//!
//! The CI budgets are deliberately ~10× the local baseline to absorb
//! GitHub Actions runner variance (cold-cache compile, shared CPU,
//! macOS/Windows runners ~3× slower than Linux). They are *ceilings*
//! — any genuine algorithmic regression will blow well past the
//! ceiling. The aggressive sub-50ms target tracked in #471 lives in
//! `tests/perf_regression.rs` once the bench corpus template is
//! fixed.
//!
//! ## Skip behaviour
//!
//! These tests run a real `compile_site`, which requires the
//! example template directory. If it's missing (rare, only on
//! corrupted checkouts), the tests skip with an explanatory
//! message rather than failing.
//!
//! ## Determinism
//!
//! Every test creates a fresh `TempDir`, generates a deterministic
//! corpus (`generate_corpus` mirrors the bench harness in
//! `benches/bench_scalability.rs`), runs one warmup iteration to
//! prime caches, then takes the median of 3 measured runs. This
//! filters out one-off scheduler hiccups without hiding genuine
//! regressions.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Generates `n` synthetic Markdown files with realistic frontmatter.
/// Matches the harness in `benches/bench_scalability.rs:generate_corpus`
/// so timings here align with the criterion baseline.
fn generate_corpus(dir: &Path, n: usize) {
    fs::create_dir_all(dir).expect("create content dir");
    for i in 0..n {
        let content = format!(
            "---\n\
             title: \"Page {i}\"\n\
             date: \"2026-01-15T09:00:00+00:00\"\n\
             description: \"Performance budget page {i}\"\n\
             language: \"en-GB\"\n\
             layout: \"page\"\n\
             permalink: \"https://example.com/page-{i}\"\n\
             ---\n\n\
             # Page {i}\n\n\
             Body content for performance budget testing.\n\n\
             ## Section\n\n\
             - Item 1\n\
             - Item 2\n\
             - Item 3\n"
        );
        let filename = if i == 0 {
            "index.md".to_string()
        } else {
            format!("page-{i}.md")
        };
        fs::write(dir.join(filename), content).expect("write page");
    }
}

/// Returns a fresh tempdir + the four directories the build needs.
fn fresh_layout() -> (
    tempfile::TempDir,
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let build = tmp.path().join("build");
    let site = tmp.path().join("site");
    let template = tmp.path().join("templates");
    for d in [&content, &build, &site, &template] {
        fs::create_dir_all(d).unwrap();
    }
    // Copy example templates so the compiler can render. If templates
    // are missing, the test signals a skip via the empty TempDir
    // marker in the fingerprint (see `compile_n_pages`).
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_tpl = workspace.join("examples/templates/en");
    if src_tpl.is_dir() {
        for entry in fs::read_dir(&src_tpl).unwrap().flatten() {
            let _ = fs::copy(
                entry.path(),
                template.join(entry.file_name()),
            );
        }
    }
    (tmp, content, build, site, template)
}

/// Compiles a corpus of `n` pages, returning the wall-clock duration.
/// `None` if templates are missing on this checkout (rare).
fn compile_n_pages(n: usize) -> Option<Duration> {
    let (_tmp, content, build, site, template) = fresh_layout();
    if fs::read_dir(&template)
        .map(|mut d| d.next().is_none())
        .unwrap_or(true)
    {
        return None;
    }
    generate_corpus(&content, n);

    let start = Instant::now();
    let _ = ssg::compile_site(&build, &content, &site, &template);
    Some(start.elapsed())
}

/// Median of 3 measured runs after a single warmup iteration.
fn median_of_3(n: usize) -> Option<Duration> {
    // Warmup primes the file-system cache for the second-and-later runs.
    let _ = compile_n_pages(n)?;
    let mut samples = [
        compile_n_pages(n)?,
        compile_n_pages(n)?,
        compile_n_pages(n)?,
    ];
    samples.sort();
    Some(samples[1])
}

fn assert_under_budget(label: &str, actual: Duration, budget: Duration) {
    assert!(
        actual <= budget,
        "{label}: {actual:.2?} exceeds budget {budget:.2?} \
         — this is a hard CI gate; please profile the regression \
         (see docs/perf/baseline-100p.md for the recipe)"
    );
    eprintln!("[perf_budgets] {label}: {actual:.2?} (budget {budget:.2?})");
}

// =====================================================================
// Budget gates
// =====================================================================

#[test]
fn build_10_pages_within_budget() {
    let Some(t) = median_of_3(10) else {
        eprintln!("[perf_budgets] templates missing — skipping 10-page");
        return;
    };
    assert_under_budget("10-page build", t, Duration::from_millis(100));
}

#[test]
fn build_100_pages_within_budget() {
    let Some(t) = median_of_3(100) else {
        eprintln!("[perf_budgets] templates missing — skipping 100-page");
        return;
    };
    assert_under_budget("100-page build", t, Duration::from_millis(500));
}

#[test]
#[ignore = "500-page build is slow on cold runners; opt-in via \
            `cargo test --test perf_budgets -- --ignored`"]
fn build_500_pages_within_budget() {
    let Some(t) = median_of_3(500) else {
        eprintln!("[perf_budgets] templates missing — skipping 500-page");
        return;
    };
    assert_under_budget("500-page build", t, Duration::from_millis(2000));
}
