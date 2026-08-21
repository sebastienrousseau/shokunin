# Fuzzing

Four libFuzzer targets covering the parsers that sit on the build
pipeline's untrusted-input boundary. Every one of them takes bytes an
author controls and turns them into structure the rest of the build
trusts, which is exactly where a panic becomes a failed build and a
mis-parse becomes wrong output.

| target | drives | why it is here |
|---|---|---|
| `fuzz_markdown` | `pulldown-cmark` | post bodies |
| `fuzz_frontmatter` | the frontmatter loader | YAML from `_posts/` |
| `fuzz_html_rewrite` | `lol_html` rewriting | every postbuild pass |
| `fuzz_shortcodes` | the shortcode expander | inline author syntax |

## Running

`cargo-fuzz` needs nightly — libFuzzer and the sanitizers are not on
stable. The repository pins `channel = "stable"`, so the toolchain has to
be named explicitly; `cargo fuzz` without `+nightly` fails against the
pinned toolchain rather than falling back.

```bash
cargo install cargo-fuzz --locked
cargo +nightly fuzz list
cargo +nightly fuzz run fuzz_markdown -- -max_total_time=60
```

The first build compiles the whole dependency tree with AddressSanitizer
(~476 crates). Expect several minutes before the first execution, and a
separate `fuzz/target/` from the normal build cache.

## Reproducing a crash

A failure writes the exact input under `fuzz/artifacts/<target>/`. Replay
it against the same target:

```bash
cargo +nightly fuzz run fuzz_markdown fuzz/artifacts/fuzz_markdown/crash-<hash>
```

The reproducer is bytes, not a description: it replays deterministically
and does not need the corpus that found it. Commit it to
`fuzz/corpus/<target>/` so the case is covered from then on — a crash that
is fixed but never seeded can come back silently.

Minimise before filing, so the report carries the smallest input that
still fails:

```bash
cargo +nightly fuzz tmin fuzz_markdown fuzz/artifacts/fuzz_markdown/crash-<hash>
```

## Corpus

`fuzz/corpus/<target>/` holds committed seeds. They are inputs, not
fixtures: libFuzzer mutates from them, so a seed that exercises an unusual
shape is worth more than a large realistic document.

## CI

`.github/workflows/fuzz.yml` runs each target for 300 s on a schedule and
on manual dispatch, in fork mode so one crash does not end the run. It is
deliberately **not** on the PR path: the sanitizer build dominates the
runtime, and paying it on every pull request would cost more than the
regression risk it removes between scheduled runs.
