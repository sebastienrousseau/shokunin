<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmark Comparisons

This directory contains scripts and instructions for running reproducible
benchmarks against other static site generators.

## Prerequisites

| SSG | Install |
|-----|---------|
| **SSG (this project)** | `cargo build --release` |
| Hugo | `brew install hugo` or <https://gohugo.io/installation/> |
| Zola | `brew install zola` or <https://www.getzola.org/documentation/getting-started/installation/> |
| Astro | `npm create astro@latest` |

## Running SSG benchmarks

```bash
# Full Criterion suite (10 / 50 / 100 / 1K / 10K pages)
make bench

# Only the site_generation group
cargo bench --bench bench -- site_generation

# Generate HTML report
# Results are written to target/criterion/
```

## Cross-SSG comparison methodology

To produce comparable numbers across SSGs:

1. **Content corpus** -- Generate N identical Markdown pages with the
   `generate_pages()` helper in `benches/bench_site_generation.rs`, then
   copy them into each SSG's content directory.

2. **Measure wall-clock time** -- Use `hyperfine` for fair comparison:

   ```bash
   # Install hyperfine
   cargo install hyperfine

   # Example: 1 000 pages
   hyperfine --warmup 2 --min-runs 5 \
     'cargo run --release -- --content ./corpus --output ./out_ssg' \
     'hugo --source ./hugo_site' \
     'zola --root ./zola_site build' \
     'cd astro_site && npm run build'
   ```

3. **Record results** -- Paste output into `../BENCHMARKS.md`.

## Tips

- Pin CPU frequency (`cpupower frequency-set -g performance` on Linux)
  to reduce variance.
- Close background applications during measurement.
- Run at least 5 iterations; report median and stddev.
- For 50K+ pages, increase Criterion sample size:
  `group.sample_size(10)` (already commented-out in the bench file).
