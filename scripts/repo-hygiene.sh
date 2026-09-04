#!/usr/bin/env bash
#
# Repository hygiene checks that need no toolchain.
#
# CI runs this script and nothing else, so `./scripts/repo-hygiene.sh`
# reproduces the `repo hygiene` job exactly. When it fails in CI it fails
# here, with the same message.
#
# Usage: scripts/repo-hygiene.sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# GitHub Actions renders `::error::` annotations inline on the diff; the
# same line is just a prefix when run locally, which is fine.
err() { printf '::error::%s\n' "$*" >&2; }

fail=0

# `cargo llvm-cov` writes profraw files wherever LLVM_PROFILE_FILE points.
# `make coverage` pins them under target/coverage/; a run without it
# scatters them through the working tree.
stray=$(
  find . -name '*.profraw' \
    -not -path './target/*' \
    -not -path './.git/*' \
    -not -path './.claude/*' || true
)
if [ -n "$stray" ]; then
  err "stray *.profraw outside target/ — run 'make coverage' (sets LLVM_PROFILE_FILE) and re-run:"
  echo "$stray" >&2
  fail=1
fi

tracked=$(git ls-files '*.profraw' || true)
if [ -n "$tracked" ]; then
  err "*.profraw tracked by git (must be ignored, never committed):"
  echo "$tracked" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "✓ repo hygiene: no stray or tracked *.profraw"
