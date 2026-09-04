<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0001: Tokio-free architecture

- **Date:** 2026-06-26
- **Status:** Accepted

## Context

`ssg` is a build-time tool. It reads markdown, templates, and config
from disk; it parses and transforms them in parallel; it writes HTML +
assets back to disk. The entire pipeline is CPU-bound bursts separated
by short I/O bursts — exactly the shape Rayon's work-stealing
scheduler is designed for.

In 2026 the conventional reflex for any non-trivial Rust application
is to reach for `tokio`. That reflex is mistaken here for four
concrete reasons:

1. **Two executors are worse than one.** A program that mixes Rayon
   for CPU work and Tokio for I/O introduces a second scheduler, two
   thread pools competing for CPU cores, and a cognitive burden
   ("which executor owns this code?") on every contributor.
2. **Binary size.** `tokio = "1"` with default features adds ~1.5 MB
   to a release binary on x86_64. For a `cargo install ssg` UX, that
   matters.
3. **Compile time.** `tokio` and its transitive `mio`, `parking_lot`,
   `socket2`, `tokio-macros` compile graph adds 6–12 seconds to a cold
   `cargo check`.
4. **`forbid(unsafe_code)` narrative.** `tokio-rt` ships unsafe in the
   runtime (necessarily — it implements `Waker` via raw vtables). The
   project's compliance positioning depends on the `forbid` claim
   being meaningful end-to-end; pulling tokio dilutes it.

We have empirical evidence that the alternative works: v0.0.44 shipped
a feature-complete build pipeline with HMR, watchers, an HTTP client
for local LLMs, and a websocket server, all without tokio. The
patterns are documented in ADR-0002 (Rayon) and below.

## Decision

**We do not use `tokio` in `ssg` core, `crates/ssg-*` library crates,
or any default feature.**

Where async behaviour is required, we use:

- `rayon` for CPU work (see ADR-0002).
- `std::thread::spawn` + `std::sync::mpsc` or `crossbeam-channel` for
  long-lived workers (HMR broadcaster, file watcher).
- `ureq` (sync, blocking, rustls-only) for HTTP (see ADR-0005).
- `notify` (sync, OS-event-driven) for filesystem watching.
- `tungstenite` (sync) for the HMR WebSocket fan-out (see ADR-0004).
- `smol` + `async-tungstenite` for future cases that genuinely require
  a non-blocking executor without dragging in tokio (planned for
  v0.0.48 #571 HMR migration; see future ADR-0008).
- `io-uring` for Linux high-throughput disk I/O via the v0.0.47 #569
  `IoPool` trait, behind the `io-uring` feature flag (see future
  ADR-0007).

Crates outside this list that require tokio are **not eligible** for
inclusion. A new dep that pulls `tokio` requires an ADR superseding
this one.

## Consequences

**Positive.**

- Single executor, single thread pool sizing question. Mental model
  stays small.
- Binary size and compile time stay tight; `cargo install ssg` UX
  preserved.
- `forbid(unsafe_code)` narrative remains end-to-end honest.
- Cross-compilation to musl, Windows, and macOS aarch64 is
  straightforward — no native-libc surprises from the tokio I/O
  driver.
- The architectural commitment is itself a DevRel artefact: see the
  v0.0.45 #559 BENCHMARKS.md rebaseline, which compares the
  Rayon-only pipeline against tokio-based competitors.

**Negative.**

- We re-implement small async patterns by hand (channel-based
  request/response inside the HMR broadcaster; a oneshot
  completion future inside `IoPool`). Each instance is bounded and
  reviewed in code; cumulative complexity is monitored at each
  release.
- Some popular crates (notably anything in the Hyper ecosystem post
  0.14) are off-limits, narrowing the dependency choice set.
- The async-Rust ergonomics improvement of `tokio::main` is forfeit;
  the binary entry point stays synchronous.

## Alternatives Considered

- **`tokio` as the primary executor.** Rejected for the four reasons
  in Context. Reconsider only if a feature genuinely requires an
  async I/O multiplexer that `smol` cannot satisfy.
- **`async-std`.** Rejected: project is in maintenance mode (last
  meaningful release Q1 2024) and offers no advantage over `smol` for
  our use case.
- **`smol` as the *primary* executor.** Rejected: `smol` is excellent
  for ad-hoc async needs but is not designed as a general application
  runtime. The Rayon-first model maps cleaner onto our CPU-bound
  workload.
- **`monoio` / `glommio`.** Both rely on `io_uring` and are Linux-only.
  We get the same benefit through the `IoPool::UringBackend` feature
  flag (planned in v0.0.47 #569) without locking the whole crate to a
  single OS.

## Status

Accepted. Decision encoded in `Cargo.toml` dep selection and enforced
by the `cargo tree | grep tokio` gate in the v0.0.45 `repo-hygiene`
job (#556).
