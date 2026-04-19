<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

Reproducible performance measurements for SSG v0.0.37.

## CI Performance Gates

These budgets are enforced on every PR via `tests/perf_regression.rs`.
A regression beyond the budget blocks the merge.

| Operation | Budget | Typical | Test |
|-----------|--------|---------|------|
| 100-page compilation | < 5s | ~200ms | `compile_100_pages_under_5s` |
| 50-page search index | < 500ms | ~50ms | `search_index_50_pages_under_50ms` |
| 50-page SEO injection | < 500ms | ~30ms | `seo_plugin_50_pages_under_50ms` |
| 50-page a11y audit | < 500ms | ~20ms | `accessibility_check_50_pages_under_50ms` |
| 1000-file cache fingerprint | < 500ms | ~40ms | `cache_fingerprint_1000_files_under_50ms` |
| 1000-file stream hash | < 500ms | ~50ms | `stream_hash_1000_files_under_50ms` |
| 10K depgraph invalidation | < 200ms | ~10ms | `depgraph_10k_entries_under_50ms` |
| 100K budget calculations | < 10ms | ~1ms | `memory_budget_calculation_instant` |

## Streaming Compilation

For sites exceeding the memory budget, SSG compiles in batches:

| Pages | Mode | Memory Budget |
|------:|------|---------------|
| < 8,000 | In-memory | Default |
| 8,000+ | Streaming | 512 MB (default) |
| 100,000+ | Streaming | Configurable via `--max-memory` |

Streaming adds ~10% overhead but enables sites that would otherwise OOM.

## Criterion Benchmarks

Run locally with `cargo bench` or `make bench`.

| Suite | File | Measures |
|-------|------|----------|
| Site generation | `bench_site_generation.rs` | 10 → 10,000 pages |
| Concurrent operations | `bench_concurrent_operations.rs` | Parallel file copy, directory traversal |
| File I/O | `bench_file.rs` | Read/write patterns |
| Utilities | `bench_utilities.rs` | Hash, slug, string operations |

## Cross-SSG Comparison

| Capability | SSG | Hugo | Zola | Astro 6 |
|---|---|---|---|---|
| Language | Rust | Go | Rust | JS/TS |
| 50-page build | ~40ms | 178ms | **36ms** | ~2s |
| Streaming (100K+) | **512 MB budget** | **Million Pages** | OOM risk | N/A |
| Built-in WCAG | **Yes** | No | No | No |
| CSP/SRI extraction | **Yes** | No | No | Yes |
| Local LLM | **Yes** | No | No | No |
| WASM target | **Yes** | No | No | N/A (JS) |
| CI coverage floor | **95%** | None | None | None |

## How to Reproduce

```bash
# Run Criterion benchmarks
cargo bench

# Run CI performance gates locally
cargo test --test perf_regression

# Run all tests (1,366 unit + 27 enterprise)
cargo test --workspace
```

## Environment

Benchmark results vary by hardware. CI gates use generous budgets
to accommodate GitHub Actions runners. Local results are typically
2-5x faster than CI.

| Key | CI Runner | Recommended Local |
|-----|-----------|-------------------|
| CPU | 2 vCPU (GitHub Actions) | 4+ cores |
| RAM | 7 GB | 8+ GB |
| OS | Ubuntu latest | macOS / Linux |
| Rust | Stable (latest) | Stable (latest) |
