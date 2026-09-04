<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0002: Rayon for build-pipeline orchestration

- **Date:** 2026-06-26
- **Status:** Accepted

## Context

The build pipeline (`src/core/pipeline.rs`) parses N markdown files,
renders N templates, runs M plugin transforms per page, and writes the
result. N grows to 100K+ pages for the enterprise corpora the v1.0
positioning targets. The workload is embarrassingly parallel within a
phase and lightly stateful across phases.

Three orchestration models were on the table:

1. Sequential, one page at a time.
2. `std::thread::spawn` with a manual work queue.
3. `rayon` work-stealing scheduler with `par_iter`.

Option 1 is wrong for the workload shape (10K pages, one CPU, sub-30s
target). Option 2 is correct but reinvents Rayon poorly — the
work-stealing balance under uneven plugin cost is non-trivial.

## Decision

**Rayon is the build pipeline's CPU scheduler.** Every per-page
transform runs as a closure inside `par_iter` / `into_par_iter` /
`par_bridge`. Cross-page state lives behind `Arc<...>` (immutable) or
`Arc<Mutex<...>>` (rare; documented per call site).

This pairs with ADR-0001: no tokio runtime exists, so there is exactly
one thread pool — Rayon's — that owns CPU time.

## Consequences

**Positive.**

- Linear scalability up to the corpus's natural parallelism limit.
  Measured on the v0.0.45 #559 baseline: 10K-page builds scale at
  ~0.92 efficiency on 16-core hardware.
- `par_iter` keeps the call sites declarative; refactoring sequential
  code to parallel costs one identifier.
- Work-stealing absorbs plugin-cost variance (the OG-image plugin is
  10–50x slower than the SEO plugin; Rayon balances automatically).
- Rayon's scope-based borrowing checker integrates with the borrow
  checker — no lifetimes need to escape into `'static`.

**Negative.**

- A Rayon worker that blocks on a syscall (e.g., `fs::write` while a
  shared disk is slow) blocks one worker thread until the syscall
  returns. This is the motivating problem for the v0.0.47 #569
  `IoPool` trait: offload blocking syscalls to a dedicated I/O thread
  so Rayon workers stay CPU-bound.
- `RAYON_NUM_THREADS` env var is the public knob for tuning; we
  document it in `BENCHMARKS.md` but do not surface it as a CLI flag
  (premature config exposure).
- Cross-phase synchronisation requires care: phases run sequentially,
  but within a phase no ordering is guaranteed. Code that depends on
  ordering must explicitly collect → sort → emit.

## Alternatives Considered

- **`tokio::task::spawn_blocking` + `tokio::main`.** Forbidden by
  ADR-0001.
- **Manual `std::thread::spawn` work queue.** Rejected: re-implements
  Rayon's work-stealing without testing, benchmarking, or the
  scope-based lifetime story.
- **`crossbeam_utils::thread::scope`.** Reasonable for one-off
  parallel sections, but worse than Rayon for nested parallelism
  (which the plugin pipeline uses: per-page parallelism nested inside
  per-phase parallelism).
- **GPU offload via `wgpu`.** Out of scope for orchestration. May
  appear later for specific kernels (e.g., the v0.0.49 #575 candle
  embedding pipeline), but the *orchestration* layer stays Rayon.

## Status

Accepted. Encoded by `rayon = "1"` as a direct, unconditional
dependency in `Cargo.toml`.
