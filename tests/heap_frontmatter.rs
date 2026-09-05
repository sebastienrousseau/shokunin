// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Peak-heap gate for the frontmatter path (#578).
//!
//! Runs the `ssg-heap-probe` workspace member — a counting allocator
//! around `emit_sidecars` on the 10,000-page corpus — and asserts the peak
//! against the baseline recorded before the fix. The probe is a separate
//! crate because a `GlobalAlloc` impl is `unsafe` by trait definition and
//! this crate is `#![forbid(unsafe_code)]`.
//!
//! Baseline, measured on v0.0.58 with the same probe: **1,927 KiB**. The
//! whole of it was the sorted `Vec<PathBuf>` that `emit_sidecars` collected
//! and then held for the entire pass — no single document's parse ever
//! exceeded it. Streaming the walk with per-directory sorting took it to
//! 941 KiB (−51%). #578 asks for ≤60% of baseline; that is the bound here.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;

/// Peak heap of `emit_sidecars` on the fixture before the fix, in KiB.
const BASELINE_PEAK_KIB: u64 = 1_927;

/// #578's acceptance criterion: at most 60% of the baseline.
const BOUND_PEAK_KIB: u64 = BASELINE_PEAK_KIB * 60 / 100;

#[test]
fn frontmatter_peak_heap_is_within_the_578_bound() {
    let out = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--release",
            "-p",
            "ssg-heap-probe",
            "--",
            "10000",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run ssg-heap-probe");
    assert!(
        out.status.success(),
        "probe failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout
        .lines()
        .find(|l| l.starts_with("HEAP "))
        .expect("probe printed a HEAP line");
    let peak_kib: u64 = line
        .split_whitespace()
        .find_map(|kv| kv.strip_prefix("peak_kib="))
        .and_then(|v| v.parse().ok())
        .expect("peak_kib field");

    eprintln!("[heap] {line}");
    assert!(
        peak_kib <= BOUND_PEAK_KIB,
        "emit_sidecars peaked at {peak_kib} KiB on the 10,000-page fixture; \
         #578 requires <= {BOUND_PEAK_KIB} KiB (60% of the {BASELINE_PEAK_KIB} KiB \
         baseline). Something is holding per-page state across the pass again."
    );
}
