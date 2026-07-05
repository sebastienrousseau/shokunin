#!/usr/bin/env bash
#
# bench-vs-zola.sh — cross-SSG comparison: ssg vs Zola (issue #559).
#
# Methodology (BENCHMARKS.md §Cross-SSG Comparison): both tools build
# the same deterministic 100-page corpus (benches/corpus/small), cold
# build every run (output wiped in --prepare), 3 warmup runs, and the
# reported number is the median of 10 measured runs via hyperfine.
#
# Zola only accepts TOML (+++) frontmatter, so the corpus pages are
# re-emitted into a temp-dir Zola site with equivalent TOML
# frontmatter (same title, byte-identical body) plus minimal
# templates.
#
# Exits 0 with a notice when hyperfine or zola is absent so fresh
# clones and CI never hard-fail on a missing competitor binary.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

for tool in hyperfine zola; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "notice: '$tool' not found — skipping the Zola comparison." >&2
    echo "notice: install $tool and re-run ./tools/bench-vs-zola.sh" >&2
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

# ── Scaffold a minimal Zola site over the same 100 pages ────────────
zola_site="$work/zola-site"
mkdir -p "$zola_site/content" "$zola_site/templates"

cat > "$zola_site/config.toml" <<'EOF'
base_url = "https://bench.example.com"
title = "bench-small"
compile_sass = false
build_search_index = false
EOF

cat > "$zola_site/templates/page.html" <<'EOF'
<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>{{ page.title }}</title></head>
<body><main><article>{{ page.content | safe }}</article></main></body></html>
EOF

cat > "$zola_site/templates/index.html" <<'EOF'
<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>{{ config.title }}</title></head>
<body><main>{% for p in section.pages %}<a href="{{ p.permalink }}">{{ p.title }}</a>{% endfor %}</main></body></html>
EOF

cp "$zola_site/templates/index.html" "$zola_site/templates/section.html"

# Re-emit each page with TOML frontmatter (title only — Zola rejects
# untaxonomied `tags`) and the unchanged markdown body.
echo "[bench] converting ${corpus}/content to Zola TOML frontmatter..."
for f in "$corpus"/content/page-*.md; do
  name="$(basename "$f")"
  title="$(awk -F'"' '/^title:/ {print $2; exit}' "$f")"
  {
    printf '+++\ntitle = "%s"\n+++\n' "$title"
    # Body = everything after the closing `---` of the frontmatter.
    awk '/^---[[:space:]]*$/ {fence++; next} fence >= 2 {print}' "$f"
  } > "$zola_site/content/$name"
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
  --command-name zola \
  "zola --root $zola_site build --output-dir $work/out --force"

# ── Report: small markdown table (median-of-10) ─────────────────────
echo
echo "## ssg vs Zola — benches/corpus/small (100 pages, cold build, median of 10 runs)"
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
