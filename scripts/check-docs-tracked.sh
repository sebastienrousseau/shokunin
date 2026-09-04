#!/usr/bin/env bash
#
# Fails if a file under docs/ exists on disk but git cannot see it.
#
# `docs/` is both a build target and a source tree: .gitignore denies
# `/docs/*` and re-admits the committed subtrees by allowlist. A new file
# or a renamed directory that is not in that allowlist is invisible —
# `git status` does not list ignored files, so the working tree looks
# clean while the content is in no commit.
#
# That is not hypothetical. Renaming docs/adrs to docs/adr invalidated
# its allowlist entry and produced a commit that deleted nine ADRs and
# added none back, with `git status` clean and two lint gates green.
#
# Usage: scripts/check-docs-tracked.sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Everything under docs/ that git is ignoring.
ignored="$(git status --porcelain --ignored -- docs/ 2>/dev/null |
  awk '$1 == "!!" { $1 = ""; sub(/^ /, ""); print }' || true)"

# Paths that are legitimately not source. Keep this list short: each
# entry is a place the guard stops looking, so a wrong entry here
# recreates exactly the blind spot this script exists to close.
#   - generated output directories
#   - OS and editor droppings
allow_generated='^docs/(api|book|_site|target)/|(^|/)[.]DS_Store$|(^|/)Thumbs[.]db$|~$'

unexpected="$(printf '%s\n' "$ignored" |
  grep -v '^$' |
  grep -Ev "$allow_generated" || true)"

if [ -n "$unexpected" ]; then
  cat >&2 <<MSG
::error::these paths under docs/ exist but git is ignoring them:

$unexpected

If they are source, add an allowlist entry to .gitignore next to the
other '!/docs/...' lines, then confirm with:

    git add --dry-run <path>      # must print "add '<path>'"
    git ls-tree -r HEAD -- docs/  # must list it after committing

If they are generated output, add them to allow_generated in
scripts/check-docs-tracked.sh so this guard stays meaningful.
MSG
  exit 1
fi

echo "✓ docs/: no source files hidden by .gitignore"
