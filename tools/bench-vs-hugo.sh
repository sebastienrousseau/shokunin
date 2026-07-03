#!/usr/bin/env bash
#
# bench-vs-hugo.sh — cross-SSG comparison: ssg vs Hugo (issue #559).
#
# Methodology (BENCHMARKS.md §Cross-SSG Comparison): both tools build
# the same deterministic 100-page corpus (benches/corpus/small), cold
# build every run (output wiped in --prepare), 3 warmup runs, and the
# reported number is the median of 10 measured runs via hyperfine.
#
# Hugo cannot consume the ssg corpus layout directly, so a minimal
# Hugo site (hugo.toml + _default layouts) is scaffolded in a temp dir
# over byte-identical copies of the same 100 markdown pages.
#
# Exits 0 with a notice when hyperfine or hugo is absent so fresh
# clones and CI never hard-fail on a missing competitor binary.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

for tool in hyperfine hugo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "notice: '$tool' not found — skipping the Hugo comparison." >&2
    echo "notice: install $tool and re-run ./tools/bench-vs-hugo.sh" >&2
    exit 0
  fi
done

corpus="benches/corpus/small"
if [ ! -d "$corpus/content" ]; then
  ./tools/seed-bench-corpus.sh small
fi

echo "[bench] building ssg (release, locked)..."
cargo build --release --locked -p ssg >/dev/null

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# ── Scaffold a minimal Hugo site over the same 100 pages ────────────
hugo_site="$work/hugo-site"
mkdir -p "$hugo_site/content" "$hugo_site/layouts/_default"
cp "$corpus"/content/page-*.md "$hugo_site/content/"

cat > "$hugo_site/hugo.toml" <<'EOF'
baseURL = "https://bench.example.com"
title = "bench-small"
disableKinds = ["taxonomy", "term", "rss", "sitemap"]
EOF

cat > "$hugo_site/layouts/_default/single.html" <<'EOF'
<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>{{ .Title }}</title></head>
<body><main><article>{{ .Content }}</article></main></body></html>
EOF

cat > "$hugo_site/layouts/_default/list.html" <<'EOF'
<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>{{ .Title }}</title></head>
<body><main>{{ range .Pages }}<a href="{{ .RelPermalink }}">{{ .Title }}</a>{{ end }}</main></body></html>
EOF

# Corpus frontmatter says `layout: page` — satisfy Hugo's lookup order.
cp "$hugo_site/layouts/_default/single.html" \
   "$hugo_site/layouts/_default/page.html"

# ── Measure: median-of-10, cold build each run ──────────────────────
json="$work/result.json"
md="$work/result.md"

hyperfine --warmup 3 --runs 10 \
  --prepare "rm -rf $work/out" \
  --export-json "$json" \
  --export-markdown "$md" \
  --command-name ssg \
  "./target/release/ssg build --content $corpus/content --template $corpus/templates --output $work/out" \
  --command-name hugo \
  "hugo --quiet -s $hugo_site -d $work/out"

# ── Report: small markdown table (median-of-10) ─────────────────────
echo
echo "## ssg vs Hugo — benches/corpus/small (100 pages, cold build, median of 10 runs)"
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
