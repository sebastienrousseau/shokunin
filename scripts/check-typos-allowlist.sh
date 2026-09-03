#!/usr/bin/env bash
#
# Fails if `_typos.toml` allow-lists a word the spell checker no longer
# flags.
#
# An allow-list is a set of suppressions, and suppressions rot: a word
# gets added for one dictionary version, the dictionary improves, and
# the entry stays forever — silently widening what the gate ignores.
# Two entries were already dead when this check was written (`macos`,
# `retuned`), both added within a day.
#
# Usage: scripts/check-typos-allowlist.sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

command -v typos >/dev/null 2>&1 || {
  echo "note: typos not installed; skipping allow-list check" >&2
  exit 0
}

readonly CONFIG="_typos.toml"
readonly BACKUP="$(mktemp)"
cp "$CONFIG" "$BACKUP"
# shellcheck disable=SC2064
trap "cp '$BACKUP' '$CONFIG'; rm -f '$BACKUP'" EXIT

# Run once with the allow-list stripped to see what is genuinely flagged.
python3 - "$CONFIG" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
s = p.read_text()
marker = "[default.extend-words]"
if marker in s:
    p.write_text(s[: s.index(marker)])
PY

flagged="$(typos --format brief 2>/dev/null |
  sed -E 's/.*`([^`]+)` should be.*/\1/' | sort -u || true)"

cp "$BACKUP" "$CONFIG"

listed="$(sed -n '/\[default\.extend-words\]/,$p' "$CONFIG" |
  sed -nE 's/^([A-Za-z_]+) = ".*/\1/p' | sort -u)"

dead="$(comm -23 <(echo "$listed") <(echo "$flagged"))"

if [ -n "$dead" ]; then
  echo "::error::_typos.toml allow-lists words the checker no longer flags:" >&2
  echo "$dead" | sed 's/^/  /' >&2
  cat >&2 <<'MSG'

Remove them. Every entry that stays is a word the spell checker will
never catch again, and one that is no longer needed buys nothing for
that cost.
MSG
  exit 1
fi

echo "✓ _typos.toml: all $(echo "$listed" | wc -l | tr -d ' ') entries still needed"
