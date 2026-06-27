#!/usr/bin/env bash
#
# seed-bench-corpus.sh — generate deterministic synthetic content
# corpora for the bench suite (issue #494 / #559).
#
# Each page body and frontmatter block is derived from SHA-256 of the
# page index so a fresh run produces a byte-identical tree across
# machines, OSes, and clones. That makes Criterion baselines and
# hyperfine numbers cross-machine-meaningful.
#
# Usage:
#   ./tools/seed-bench-corpus.sh [size]
#
# Sizes:
#   tiny    10 pages    smoke
#   small   100 pages   perf-budget gate corpus (default)
#   medium  1000 pages  scalability bench
#   large   10000 pages streaming-mode trigger
#
# Output: benches/corpus/<size>/{config.toml,templates/,content/page-*.md}
#
# Re-running with the same size is idempotent: the corpus is wiped
# and re-emitted before each run.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

size="${1:-small}"
case "$size" in
    tiny)   pages=10 ;;
    small)  pages=100 ;;
    medium) pages=1000 ;;
    large)  pages=10000 ;;
    *)
        echo "✗ unknown size '$size'. Use: tiny | small | medium | large" >&2
        exit 1
        ;;
esac

root="benches/corpus/$size"
echo "[seed] target: $root  ($pages pages)"

rm -rf "$root"
mkdir -p "$root/content" "$root/templates" "$root/build" "$root/public"

cat > "$root/config.toml" <<EOF
site_name        = "SSG bench-corpus-${size}"
base_url         = "https://bench.example.com"
content_dir      = "content"
output_dir       = "public"
template_dir     = "templates"
language         = "en"
site_description = "Synthetic deterministic corpus, ${pages} pages"
site_title       = "bench-${size}"
EOF

# Minimal templates so compile_site has a base/page to extend.
cat > "$root/templates/base.html" <<'EOF'
<!DOCTYPE html>
<html lang="{{ language | default(value='en') }}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% block title %}{{ title }}{% endblock %}</title>
  <meta name="description" content="{{ description | default(value='') }}">
</head>
<body>
  <main>{% block content %}{% endblock %}</main>
</body>
</html>
EOF

cat > "$root/templates/page.html" <<'EOF'
{% extends "base.html" %}
{% block content %}<article>{{ content | safe }}</article>{% endblock %}
EOF

cat > "$root/templates/index.html" <<'EOF'
{% extends "base.html" %}
{% block content %}<section>{{ content | safe }}</section>{% endblock %}
EOF

# Pure Bash + sha256sum: produces a per-page deterministic 16-char
# hex seed. On macOS we'd need shasum; portable wrapper:
sha256_hex() {
    if command -v sha256sum >/dev/null 2>&1; then
        printf '%s' "$1" | sha256sum | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
    else
        # Last-resort openssl
        printf '%s' "$1" | openssl dgst -sha256 -hex \
            | awk '{print $NF}'
    fi
}

# Per-page body template: title + 3 short paragraphs whose first
# 4 chars rotate through the seed so byte-deterministic, but
# the body length stays roughly stable across pages.
echo "[seed] writing $pages markdown pages..."
i=0
while [ $i -lt $pages ]; do
    seed="$(sha256_hex "page-${i}")"
    short="${seed:0:8}"
    cat > "$root/content/page-${i}.md" <<EOF
---
title: "Bench page ${i} — ${short}"
description: "Deterministic synthetic page for the ${size} corpus (seed ${short})."
layout: page
date: 2026-01-01
tags:
  - bench
  - corpus-${size}
---

# Bench page ${i}

Deterministic synthetic body anchored on the SHA-256 prefix ${short}.

This page is one of ${pages} written by \`tools/seed-bench-corpus.sh\`.
The body length stays roughly stable across pages so the Criterion
benchmarks measure compile cost, not corpus variance.

\`\`\`text
seed   = ${seed}
short  = ${short}
index  = ${i}
\`\`\`
EOF
    i=$((i + 1))
    # Lightweight progress beep every 1000 pages on large.
    if [ $((i % 1000)) -eq 0 ] && [ "$size" = "large" ]; then
        echo "[seed]   ... ${i}/${pages}"
    fi
done

echo "[seed] done. ${pages} pages under $root/content/"
echo
echo "next: cargo run --release -- build -c $root/content -o $root/public -t $root/templates"
