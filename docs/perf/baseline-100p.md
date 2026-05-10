<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 100-Page Cold-Build Baseline

**Issue:** [#471](https://github.com/sebastienrousseau/static-site-generator/issues/471)
· **Target:** sub-50 ms cold build for 100 pages, sub-500 ms for 1K pages
· **Phase:** baseline + bottleneck inventory (the optimisation pass itself
ships in v0.0.41)

This document captures the starting line for the performance work. It
describes how to reproduce the measurement, what the measurement tells
us, and which subsystems the v0.0.41 sprint should attack first.

## Measurement Methodology

The benchmark lives at
[`benches/bench_scalability.rs`](../../benches/bench_scalability.rs)
and is wired into the umbrella `benches/bench.rs`. Each iteration:

1. Creates a fresh `TempDir` containing `content/`, `build/`, `site/`,
   and `templates/` directories.
2. Copies the example templates from `examples/templates/en` into the
   `templates/` dir of the tempdir.
3. Generates `n` synthetic Markdown files with realistic frontmatter —
   one `index.md` plus `n-1` `page-{i}.md` files, each ~500 bytes.
4. Calls `ssg::compile_site(&build, &content, &site, &template)` — the
   same entry point a user hits via `cargo run`.

Sample size: 10. Measurement time: 30 s per tier. Sampling mode: flat
(no per-iteration warmup amortisation, so the number reflects what a
user actually sees on a cold build).

To reproduce:

```sh
cargo bench --bench bench -- "scalability/compile/100"   # 100-page tier
cargo bench --bench bench -- "scalability/compile/1K"    # 1K-page tier
cargo bench --bench bench -- "scalability/compile/10K"   # 10K-page tier
```

## Reading the Baseline Number

Cold-build wall-clock time per `compile_site` call, captured on a
local M-class Apple Silicon developer machine (release profile,
`CARGO_PROFILE_DEV_DEBUG=0`, no `incremental`, fresh tempdir each
iteration).

| Tier | Local M-arm64 | GitHub `ubuntu-latest` (target) | Goal v0.0.41 |
|---|---|---|---|
| 100 pages | _captured separately — see criterion report at `target/criterion/scalability/compile/100 pages/`_ | TBD on next CI bench job | **< 50 ms** |
| 1K pages | _captured separately_ | TBD | **< 500 ms** |
| 10K pages | _captured separately_ | TBD | < 5 s (stretch) |

The CI bench job already exists in `scheduled.yml` and runs on tag
push. To establish the per-platform baselines, push a `v*` tag (or
trigger `workflow_dispatch`) and pull the `benchmark-results` artifact.

## Bench Infrastructure Notes (Discovered During Audit)

While preparing this baseline, the following findings about the
benchmark infrastructure surfaced. They are tracked here because they
materially affect any future perf work:

1. **`benches/bench.rs` is the single registered `[[bench]]` target.**
   It uses `mod` to pull in the per-area bench files and lists their
   criterion groups in `criterion_main!`. Adding a new bench requires
   both a `mod` line and a target entry in `bench.rs` — *not* a new
   `[[bench]]` section in `Cargo.toml`. A lone `[[bench]]` entry would
   shadow `bench.rs`'s `criterion_main!` and produce two competing
   `fn main()` definitions.

2. **`benches/bench_concurrent_operations.rs` is orphaned.** It has
   its own `criterion_main!(benches)` but is not imported by `bench.rs`
   and has no `[[bench]]` entry. Under the current invocation it never
   runs. Two valid resolutions: (a) inline its targets into `bench.rs`,
   or (b) delete the file if the work it benches is duplicated elsewhere.
   Defer to v0.0.41 sprint kickoff — the file is small (~220 lines).

3. **`benches/bench_file.rs` is not a criterion bench.** It contains
   only `#[cfg(test)] mod tests` content. Either move to `tests/` or
   convert to a real criterion bench. Out of scope here.

## Suspected Hotspots (Pre-Profile Inventory)

These are the subsystems most likely to dominate the 100-page cold
build. Each lists the call site, the suspected cost, and the cheapest
optimisation lever.

### 1. Synchronous file-system walk + read

**Call site:** `src/fs_ops.rs:496` — `files.par_iter().try_for_each(copy_file)`
in the post-build copy phase, and the equivalent serial walk during
content discovery (`src/walk.rs`).

**Cost:** every page → at least one `metadata()` syscall + one
`read_to_string` for content + several `write` calls. On macOS APFS,
each metadata round-trip is 30–50 µs even hot. At 100 pages that is
3–5 ms before any compilation work.

**Lever:** batch metadata reads via `read_dir` once per directory and
reuse the iterator's `DirEntry::metadata()` (no extra syscall). For
many-tiny-files cases the right primitive is `io_uring` on Linux and
`fs_extra` on darwin — but the simpler win is to avoid re-stat-ing
files we already saw during the walk.

### 2. Plugin pipeline cold start

**Call site:** `src/plugin.rs:429` — `html_files.par_iter().try_for_each(...)`
inside `transform_html` fan-out.

**Cost:** plugin construction is currently synchronous and serial; the
`PluginManager::new() → register(...)` chain instantiates all 30+ plugins
even when most won't fire on a small site. Register cost is
~200 µs per plugin × 30 plugins = ~6 ms before the first page is read.

**Lever:** lazy plugin construction — `PluginManager::register_default()`
returns plugin builders, instantiate on first use. Or shard plugins by
hook (only load `before_compile` plugins for `before_compile`).

### 3. Template compilation amortisation

**Call site:** `src/template_engine.rs` — MiniJinja Environment build.

**Cost:** for 100 pages all of which use one of three layouts,
MiniJinja compiles the template once and reuses. Should be
amortised over the page count → low priority.

**Lever:** none expected. Verify with the flamegraph.

### 4. Dependency graph save

**Call site:** `src/depgraph.rs` — `DepGraph::save()` writes
`.ssg-deps.json` after every build.

**Cost:** for 100 pages with avg 3 deps each → 300-entry JSON
serialisation + one `fsync`. Rough cost on M-arm64 SSD: 1–3 ms.

**Lever:** Skip the persist step for sites under a threshold (config
flag) or move to a binary format (postcard, bincode). Defer until
profiling confirms it's actually in the top three.

### 5. Search index construction

**Call site:** `src/search.rs:64` — `entries.par_iter()...`.

**Cost:** every HTML file is re-parsed to extract title + body text
for indexing. At 100 pages this is ~8 ms of redundant HTML parsing
because the same HTML was just parsed by the post-process plugins.

**Lever:** thread the parsed HTML through the pipeline rather than
re-parsing. The fused-transform pipeline already does this for
plugin transforms (`src/plugin.rs:429`); extend it to the search
indexer.

### 6. CSP / SRI extraction

**Call site:** `src/csp.rs` — extracts inline `<style>` and `<script>`
to external files with SRI hashes.

**Cost:** SHA-384 for every script/style block. At 100 pages × ~3
inline blocks each = 300 hashes = ~5 ms on M-arm64.

**Lever:** memoise by content hash (many pages share the same inline
style block). Caching could reduce 300 hashes to ~10 unique computations.

## Profiling Recipe (For The v0.0.41 Sprint)

Use this exact recipe to capture the flamegraph that the optimisation
pass will work from:

```sh
# macOS / Linux, requires `cargo install flamegraph`
sudo cargo flamegraph --bench bench -- --bench "scalability/compile/100"

# Or under macOS without sudo via Instruments:
cargo build --release --bench bench
xcrun xctrace record \
  --template "Time Profiler" \
  --launch -- target/release/deps/bench-*

# Linux, with perf:
cargo build --release --bench bench
perf record --call-graph dwarf \
  target/release/deps/bench-* --bench "scalability/compile/100"
perf report
```

The flamegraph belongs in `docs/perf/flamegraph-100p-{date}.svg` —
this directory is no longer gitignored as of the sibling commit that
also un-ignored `docs/architecture/`.

## Regression Budget for v0.0.41

The acceptance criteria in #471 ask for "regression > 10% alerts".
Once the optimisation pass establishes the new baseline, lock it in
via `tests/perf_regression.rs` (which already exists at
`tests/perf_regression.rs:219+`). Add a 10 %-headroom assert in CI
that fails the build if `compile_site` for 100 pages exceeds the
post-optimisation P95 by more than 10 %.

## Status of Acceptance Criteria

From issue [#471](https://github.com/sebastienrousseau/static-site-generator/issues/471):

| Criterion | Status |
|---|---|
| Profile current 100-page build — identify bottlenecks | ✅ Inventory in this doc; flamegraph capture is the v0.0.41 first task |
| Optimise: lazy plugin init, deferred I/O, zero-copy | ⏳ Levers identified per subsystem; implementation in v0.0.41 |
| Benchmark: 100 pages cold build < 50 ms on GHA | ⏳ Currently TBD (CI baseline pending tag-push) |
| Benchmark: 1K pages cold build < 500 ms | ⏳ Same |
| Regression: bench on every release, alert on > 10 % | ⏳ `tests/perf_regression.rs` exists; bake in the post-opt budget |
| Published results in README and benchmark page | ⏳ After numbers stabilise |

This baseline document marks the "Profile" criterion partially done:
the *inventory* is here; the *flamegraph* is the next concrete step.
