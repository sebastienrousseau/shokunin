// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Measures peak heap of `ssg::frontmatter::emit_sidecars` on the #578
//! fixture — 10,000 pages from `ssg::bench_corpus::generate_corpus` — and
//! prints one machine-readable line. `tests/heap_frontmatter.rs` in the
//! root crate runs it and asserts against the recorded baseline.
//!
//! A counting global allocator rather than `dhat`: the issue names dhat,
//! but this repository's cargo-vet exemption count is a ratchet and a new
//! dependency would need certifying. Counting bytes live gives peak,
//! total and count directly, with nothing added to the supply chain.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);

struct Counting;

// SAFETY: every call is delegated to `System` unchanged; only atomic
// counters are touched around it, and the returned pointer is never read.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let live = LIVE.fetch_add(size, Ordering::Relaxed) + size;
        let _ = TOTAL.fetch_add(size, Ordering::Relaxed);
        let _ = COUNT.fetch_add(1, Ordering::Relaxed);
        let _ = PEAK.fetch_max(live, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn main() {
    let pages: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(10_000);
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let sidecars = tmp.path().join("meta");
    std::fs::create_dir_all(&content).expect("content dir");
    let spec = ssg::bench_corpus::CorpusSpec {
        pages,
        seed: 42,
        words_per_page: 600,
    };
    let generated =
        ssg::bench_corpus::generate_corpus(&content, &spec).expect("corpus");

    LIVE.store(0, Ordering::Relaxed);
    PEAK.store(0, Ordering::Relaxed);
    TOTAL.store(0, Ordering::Relaxed);
    COUNT.store(0, Ordering::Relaxed);

    let written = ssg::frontmatter::emit_sidecars(&content, &sidecars)
        .expect("emit_sidecars");

    println!(
        "HEAP pages={generated} sidecars={written} peak_kib={} total_kib={} allocs={}",
        PEAK.load(Ordering::Relaxed) / 1024,
        TOTAL.load(Ordering::Relaxed) / 1024,
        COUNT.load(Ordering::Relaxed)
    );
}
