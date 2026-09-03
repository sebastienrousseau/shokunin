#!/usr/bin/env bash
#
# Builds the rendered user manual and asserts it is not a hollow shell.
#
# `mdbook build` exits 0 for a book with no chapters, an empty search
# index, or a table of contents that lost its sections — none of which
# look like failure until someone opens the site. So this checks the
# output rather than the exit code.
#
# CI runs this script; `mdbook serve --open` is the local preview.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Chapters are files that already exist in docs/; SUMMARY.md indexes
# them. Keep this in step with the section headings in docs/SUMMARY.md.
readonly MIN_PAGES=40
readonly SECTIONS=(
  "Getting started"
  "Authoring content"
  "Capabilities"
  "Operating"
  "Reference"
  "Compared with"
  "Decisions"
)

echo "==> mdbook build"
mdbook build

fail=0
bad() {
  printf '  ✗ %s\n' "$*"
  fail=1
}

echo "==> the manual has chapters"
pages=$(find book -name '*.html' | wc -l | tr -d ' ')
if [ "$pages" -lt "$MIN_PAGES" ]; then
  bad "only ${pages} pages rendered, expected at least ${MIN_PAGES}"
else
  echo "  ✓ ${pages} pages"
fi

echo "==> search is built"
if ! find book -name 'searchindex*' | grep -q .; then
  bad "no search index — [output.html.search] is enabled but produced nothing"
else
  echo "  ✓ search index present"
fi

echo "==> every section of the table of contents survived"
for section in "${SECTIONS[@]}"; do
  if ! grep -qF "$section" book/toc.html 2>/dev/null; then
    bad "section missing from the table of contents: ${section}"
  fi
done
[ "$fail" -eq 0 ] && echo "  ✓ ${#SECTIONS[@]} sections"

echo "==> every chapter listed in SUMMARY.md rendered"
missing=0
while read -r chapter; do
  # mdbook renders README.md as index.html, per directory.
  case "$chapter" in
  README.md) html="book/index.html" ;;
  */README.md) html="book/${chapter%/README.md}/index.html" ;;
  *) html="book/${chapter%.md}.html" ;;
  esac
  if [ ! -f "$html" ]; then
    bad "chapter did not render: ${chapter}"
    missing=$((missing + 1))
  fi
done < <(
  grep -oE '\]\(([^)#]+\.md)\)' docs/SUMMARY.md |
    sed -E 's/^\]\(//; s/\)$//' | sort -u
)
[ "$missing" -eq 0 ] && echo "  ✓ all SUMMARY.md chapters present"

echo
if [ "$fail" -ne 0 ]; then
  echo "manual: FAILED"
  exit 1
fi
echo "manual: OK"
