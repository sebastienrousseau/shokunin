<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

Reproducible performance measurements for SSG and cross-SSG comparisons.

## Environment

| Key | Value |
|-----|-------|
| CPU | _fill in_ |
| RAM | _fill in_ |
| OS | _fill in_ |
| Rust | _fill in_ |
| Commit | _fill in_ |

## SSG Criterion results

Results from `make bench` (Criterion, median of 100 iterations unless noted).

| Pages | Mean | Std Dev | Throughput (pages/s) |
|------:|-----:|--------:|---------------------:|
| 10 | -- | -- | -- |
| 50 | -- | -- | -- |
| 100 | -- | -- | -- |
| 1 000 | -- | -- | -- |
| 10 000 | -- | -- | -- |
| 50 000* | -- | -- | -- |
| 100 000* | -- | -- | -- |

_* Heavy tiers -- run locally, not in CI._

## Cross-SSG comparison (hyperfine, 1 000 pages)

| SSG | Version | Mean | Min | Max |
|-----|---------|-----:|----:|----:|
| **SSG** | _fill in_ | -- | -- | -- |
| Hugo | _fill in_ | -- | -- | -- |
| Zola | _fill in_ | -- | -- | -- |
| Astro | _fill in_ | -- | -- | -- |

## How to reproduce

```bash
# SSG Criterion suite
make bench

# Cross-SSG comparison
# See benchmarks/README.md for full instructions
```
