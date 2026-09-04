#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# OSS-Fuzz / ClusterFuzzLite build script for the SSG ecosystem.
#
# OSS-Fuzz invokes this from $SRC/static-site-generator with $OUT set to the
# directory the built fuzzers must land in. It is kept in-tree, rather than
# only in google/oss-fuzz, so a change that breaks the fuzz build is caught in
# this repository's own CI instead of days later in an upstream batch job.
#
# The contract OSS-Fuzz expects:
#   * every fuzzer is a static binary in $OUT
#   * each fuzzer may ship a seed corpus as $OUT/<name>_seed_corpus.zip
#   * each fuzzer may ship a dictionary as $OUT/<name>.dict
#   * $CFLAGS / $CXXFLAGS / $RUSTFLAGS carry the sanitizer configuration and
#     must be honoured, not replaced
#
# `set -euo pipefail` is in the body rather than on the shebang line: shebang
# flags apply only when the file is executed directly, and both OSS-Fuzz's
# wrapper and a developer typing `bash fuzz/oss-fuzz-build.sh` invoke it
# through an explicit interpreter, which silently drops them. The first
# version of this script had `#!/bin/bash -eu` and reported "built 4
# fuzzer(s)" after every single target failed to compile.
set -euo pipefail

cd "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

: "${SANITIZER:=address}"
: "${OUT:=$PWD/fuzz/out}"
mkdir -p "$OUT"

# libFuzzer instrumentation needs `-Z sanitizer`, which is nightly-only. The
# OSS-Fuzz base image defaults to nightly, so upstream this is a no-op; locally
# the default is usually stable, and without the override every target fails
# with "the option `Z` is only accepted on the nightly compiler".
TOOLCHAIN=""
if ! rustc -vV | grep -q 'nightly'; then
  if rustup toolchain list 2>/dev/null | grep -q '^nightly'; then
    TOOLCHAIN="+nightly"
  else
    echo "error: fuzzing requires a nightly toolchain (rustup toolchain install nightly)" >&2
    exit 1
  fi
fi

# OSS-Fuzz supplies its own instrumentation flags; appending rather than
# assigning preserves them. `--cfg fuzzing` lets code shrink expensive work
# under fuzzing without affecting production builds.
export RUSTFLAGS="${RUSTFLAGS:-} --cfg fuzzing -Cdebug-assertions -Coverflow-checks"

TRIPLE="$(rustc -vV | sed -n 's|host: ||p')"

build_target() {
  local target="$1"
  echo "==> building fuzz target: ${target} (sanitizer=${SANITIZER})"

  # shellcheck disable=SC2086  # $TOOLCHAIN is a single optional +nightly token
  # The committed `fuzz/Cargo.lock` is what makes this reproducible.
  # cargo-fuzz 0.13.2 has no `--locked` passthrough — `-- --locked` is
  # rejected — so the lockfile alone does the work: cargo uses it as-is
  # and will not reach for a newer dependency on its own.
  cargo ${TOOLCHAIN} fuzz build --sanitizer "${SANITIZER}" -O "${target}"

  local built="fuzz/target/${TRIPLE}/release/${target}"
  # Postcondition, not decoration: `cargo fuzz build` can fail in ways that
  # leave the previous binary in place, and a stale fuzzer reports coverage
  # for code that is no longer there.
  if [[ ! -x "${built}" ]]; then
    echo "error: ${target} did not produce a binary at ${built}" >&2
    return 1
  fi
  cp "${built}" "${OUT}/"

  # A seed corpus turns the first minutes of a run from random flailing into
  # coverage. Optional: a target without seeds still builds.
  if [[ -d "fuzz/corpus/${target}" ]]; then
    ( cd "fuzz/corpus/${target}" && zip -qr "${OUT}/${target}_seed_corpus.zip" . )
    echo "    seeded from fuzz/corpus/${target}"
  fi

  # Dictionaries matter most for the structured inputs here — Markdown,
  # front matter, HTML — where random bytes rarely reach a parser branch.
  if [[ -f "fuzz/dictionaries/${target}.dict" ]]; then
    cp "fuzz/dictionaries/${target}.dict" "${OUT}/${target}.dict"
    echo "    dictionary attached"
  fi
}

# Enumerated from fuzz/Cargo.toml rather than hardcoded, so a target added
# there is fuzzed without a second edit here — the class of drift that leaves
# a fuzzer written but never run.
TARGETS=()
while IFS= read -r line; do
  TARGETS+=("$line")
done < <(sed -n 's/^name = "\(fuzz_[a-z0-9_]*\)"$/\1/p' fuzz/Cargo.toml)

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "error: no fuzz targets found in fuzz/Cargo.toml" >&2
  exit 1
fi

echo "==> ${#TARGETS[@]} fuzz target(s): ${TARGETS[*]}"
for target in "${TARGETS[@]}"; do
  build_target "${target}"
done

# Final postcondition: the count reported must be the count on disk. The
# earlier version printed a total derived from the target list rather than
# from the artefacts, so it claimed success with an empty $OUT.
BUILT=0
for target in "${TARGETS[@]}"; do
  [[ -x "${OUT}/${target}" ]] && BUILT=$((BUILT + 1))
done

if [[ "${BUILT}" -ne "${#TARGETS[@]}" ]]; then
  echo "error: expected ${#TARGETS[@]} fuzzer(s) in ${OUT}, found ${BUILT}" >&2
  exit 1
fi

echo "==> built ${BUILT} fuzzer(s) into ${OUT}"
