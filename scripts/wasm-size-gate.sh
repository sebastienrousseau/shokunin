#!/usr/bin/env bash
# Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Enforces issue #546 AC10: the optimised Edge WASM payload must
# stay ≤ 2 MB gzipped. Mirrors what `.github/workflows/wasm.yml` does
# in CI, so local runs catch regressions before push.
#
# Usage:
#   scripts/wasm-size-gate.sh                  # build + measure + gate
#   scripts/wasm-size-gate.sh --no-build       # measure existing pkg/
#
# Exits 0 on success, 1 if the budget is exceeded.

set -euo pipefail

BUDGET=2097152  # 2 MB in bytes
DO_BUILD=1
for arg in "$@"; do
  case "$arg" in
    --no-build) DO_BUILD=0 ;;
    -h|--help)
      sed -n '1,/^$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown flag: $arg" >&2
      exit 2
      ;;
  esac
done

cd "$(git rev-parse --show-toplevel)"

if [ "$DO_BUILD" -eq 1 ]; then
  if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack not installed — install with:"
    echo "  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
  fi
  echo "building ssg-wasm via wasm-pack..."
  wasm-pack build crates/ssg-wasm --target web --out-dir ../../pkg
fi

WASM=pkg/ssg_wasm_bg.wasm
if [ ! -f "$WASM" ]; then
  echo "ERROR: $WASM not found" >&2
  exit 1
fi

# Optional wasm-opt pass. Must allow bulk-memory + reference-types
# because LLVM's wasm32 backend emits memory.copy ops for slice
# operations by default since 2024, and modern Edge runtimes
# (Cloudflare, Vercel, Deno Deploy) all support both proposals.
if command -v wasm-opt >/dev/null 2>&1; then
  echo "running wasm-opt -Oz..."
  wasm-opt -Oz --strip-debug --vacuum \
    --enable-bulk-memory --enable-bulk-memory-opt \
    --enable-reference-types --enable-mutable-globals \
    -o "$WASM.opt" "$WASM"
  mv "$WASM.opt" "$WASM"
else
  echo "wasm-opt not installed — skipping optimiser pass"
  echo "  install via: brew install binaryen   (macOS)"
fi

RAW=$(wc -c < "$WASM" | tr -d ' ')
GZIP=$(gzip --best -c "$WASM" | wc -c | tr -d ' ')

printf "raw:     %s bytes\n" "$RAW"
printf "gzipped: %s bytes (budget %s)\n" "$GZIP" "$BUDGET"

if [ "$GZIP" -gt "$BUDGET" ]; then
  echo "FAIL: gzipped wasm exceeds 2 MB budget (#546 AC10)"
  exit 1
fi
echo "OK: under budget by $((BUDGET - GZIP)) bytes"
