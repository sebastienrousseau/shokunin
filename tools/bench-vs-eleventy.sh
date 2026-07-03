#!/usr/bin/env bash
#
# bench-vs-eleventy.sh — cross-SSG comparison: ssg vs Eleventy
# (issue #559).
#
# Methodology (BENCHMARKS.md §Cross-SSG Comparison): both tools build
# the same deterministic 100-page corpus (benches/corpus/small), cold
# build every run (output wiped in --prepare), 3 warmup runs, and the
# reported number is the median of 10 measured runs via hyperfine.
#
# Eleventy reads the corpus's YAML frontmatter natively; only the
# `layout: page` key is dropped (it references an ssg template that
# Eleventy would fail to resolve). Pages are copied into a temp dir so
# the tracked corpus is never mutated.
#
# Exits 0 with a notice when hyperfine, npx, or a runnable
# @11ty/eleventy is absent so fresh clones and CI never hard-fail on
# a missing competitor binary.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

for tool in hyperfine npx; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "notice: '$tool' not found — skipping the Eleventy comparison." >&2
    echo "notice: install $tool and re-run ./tools/bench-vs-eleventy.sh" >&2
    exit 0
  fi
done

# Prime the npx cache (and verify Eleventy is actually runnable) once,
# outside the timed runs, so no download noise leaks into the numbers.
if ! npx --yes @11ty/eleventy --version >/dev/null 2>&1; then
  echo "notice: 'npx @11ty/eleventy' is not runnable (offline?) — skipping the Eleventy comparison." >&2
  exit 0
fi

corpus="benches/corpus/small"
if [ ! -d "$corpus/content" ]; then
  ./tools/seed-bench-corpus.sh small
fi

echo "[bench] building ssg (release, locked)..."
cargo build --release --locked -p ssg >/dev/null

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# ── Stage the same 100 pages for Eleventy ───────────────────────────
e11ty_in="$work/11ty-content"
mkdir -p "$e11ty_in"
for f in "$corpus"/content/page-*.md; do
  sed '/^layout: page$/d' "$f" > "$e11ty_in/$(basename "$f")"
done

# ── Measure: median-of-10, cold build each run ──────────────────────
json="$work/result.json"
md="$work/result.md"

hyperfine --warmup 3 --runs 10 \
  --prepare "rm -rf $work/out" \
  --export-json "$json" \
  --export-markdown "$md" \
  --command-name ssg \
  "./target/release/ssg build --content $corpus/content --template $corpus/templates --output $work/out" \
  --command-name eleventy \
  "npx --yes @11ty/eleventy --input=$e11ty_in --output=$work/out --quiet"

# ── Report: small markdown table (median-of-10) ─────────────────────
echo
echo "## ssg vs Eleventy — benches/corpus/small (100 pages, cold build, median of 10 runs)"
echo
if command -v python3 >/dev/null 2>&1; then
  python3 - "$json" <<'EOF'
import json, sys
results = json.load(open(sys.argv[1]))["results"]
print("| Tool | Median | Min | Max |")
print("|------|-------:|----:|----:|")
for r in results:
    print("| {} | {:.0f} ms | {:.0f} ms | {:.0f} ms |".format(
        r["command"], r["median"] * 1000, r["min"] * 1000, r["max"] * 1000))
EOF
else
  # Fallback: hyperfine's own markdown export (mean ± σ).
  cat "$md"
fi
