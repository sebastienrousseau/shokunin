#!/usr/bin/env bash
#
# Verifies the GNUmakefile install contract end to end: a staged install
# produces exactly the expected FHS tree, the artefacts in it are valid,
# and uninstall removes precisely what install created.
#
# CI runs this script and nothing else, so `./scripts/install-smoke.sh`
# locally reproduces the CI job exactly. Every red CI round on this branch
# so far came from a local command that differed from the one CI ran.
#
# Usage: scripts/install-smoke.sh [stage-dir]

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

STAGE="${1:-$(mktemp -d)/stage}"
PREFIX_="/usr/local"
fail=0

note() { printf '  %s\n' "$*"; }
bad() {
  printf '  ✗ %s\n' "$*"
  fail=1
}

rm -rf "$STAGE"

echo "==> make DESTDIR=$STAGE install"
make DESTDIR="$STAGE" install >/dev/null

echo "==> the staged tree contains exactly what it should"
# The contract, spelled out. A file appearing here that install stops
# producing is as much a regression as a missing one, so the comparison is
# an equality, not a subset check.
expected=$(
  cat <<EOF
${PREFIX_#/}/bin/ssg
${PREFIX_#/}/share/bash-completion/completions/ssg
${PREFIX_#/}/share/doc/ssg/CHANGELOG.md
${PREFIX_#/}/share/doc/ssg/LICENSE-APACHE
${PREFIX_#/}/share/doc/ssg/LICENSE-MIT
${PREFIX_#/}/share/doc/ssg/README.md
${PREFIX_#/}/share/fish/vendor_completions.d/ssg.fish
${PREFIX_#/}/share/man/man1/ssg.1
${PREFIX_#/}/share/zsh/site-functions/_ssg
EOF
)
actual=$(cd "$STAGE" && find . -type f | sed 's|^\./||' | sort)
if [ "$expected" != "$actual" ]; then
  bad "staged tree does not match the contract"
  diff <(echo "$expected") <(echo "$actual") || true
else
  note "✓ 9 files, all at their FHS paths"
fi

echo "==> the binary is executable and reports its version"
bin="$STAGE$PREFIX_/bin/ssg"
[ -x "$bin" ] || bad "$bin is not executable"
version=$("$bin" --version)
note "✓ $version"
# The man page's .TH line must name the same version the binary reports.
# A stale page in a release archive is exactly the drift this gates.
page="$STAGE$PREFIX_/share/man/man1/ssg.1"
if ! grep -q "\"${version}\"" "$page"; then
  bad "man page does not name '$version'"
  head -1 "$page"
else
  note "✓ man page names $version"
fi

echo "==> the man page is valid roff"
if command -v mandoc >/dev/null 2>&1; then
  # STYLE diagnostics are advisory (long source lines); anything at
  # WARNING or above is a real structural defect.
  if mandoc -T lint "$page" 2>&1 | grep -vE 'STYLE:' | grep -q .; then
    bad "mandoc reported non-style diagnostics"
    mandoc -T lint "$page" 2>&1 | grep -vE 'STYLE:' || true
  else
    note "✓ mandoc: no warnings or errors"
  fi
else
  note "- mandoc not installed; skipping"
fi

echo "==> each completion parses in its own shell"
check_shell() {
  local bin_name="$1" script="$2"
  shift 2
  if ! command -v "$bin_name" >/dev/null 2>&1; then
    note "- $bin_name not installed; skipping"
    return
  fi
  if "$bin_name" "$@" "$script"; then
    note "✓ $bin_name accepts $(basename "$script")"
  else
    bad "$bin_name rejected $script"
  fi
}
check_shell bash "$STAGE$PREFIX_/share/bash-completion/completions/ssg" -n
check_shell zsh "$STAGE$PREFIX_/share/zsh/site-functions/_ssg" -n
check_shell fish "$STAGE$PREFIX_/share/fish/vendor_completions.d/ssg.fish" \
  --no-execute

echo "==> the bash completion actually completes"
if command -v bash >/dev/null 2>&1; then
  got=$(
    bash -c '
      source "$1"
      COMP_WORDS=(ssg ""); COMP_CWORD=1
      _ssg
      echo "${COMPREPLY[*]}"
    ' _ "$STAGE$PREFIX_/share/bash-completion/completions/ssg"
  )
  # Sourcing a syntactically valid script that returns nothing is the
  # failure this catches: `bash -n` alone would call that a pass.
  for sub in build dev check audit deploy; do
    case " $got " in
    *" $sub "*) ;;
    *) bad "completing 'ssg ' did not offer '$sub' (got: $got)" ;;
    esac
  done
  note "✓ 'ssg <TAB>' offers: $got"
fi

echo "==> make uninstall is an exact inverse"
make DESTDIR="$STAGE" uninstall >/dev/null
leftover=$(find "$STAGE" -type f)
if [ -n "$leftover" ]; then
  bad "uninstall left files behind:"
  echo "$leftover"
else
  note "✓ nothing left behind"
fi

echo
if [ "$fail" -ne 0 ]; then
  echo "install smoke: FAILED"
  exit 1
fi
echo "install smoke: OK"
