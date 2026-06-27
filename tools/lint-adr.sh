#!/usr/bin/env bash
#
# lint-adr.sh — verifies every `adr: ADR-NNNN` citation in tracked files
# resolves to an existing file in docs/adrs/.
#
# Conventions enforced:
#   - Citation form is `adr: ADR-NNNN`  (or `// adr: ADR-NNNN` in Rust,
#     or `# adr: ADR-NNNN` in shell / TOML, or any host-language comment).
#   - NNNN is four ascii digits, zero-padded.
#   - The matching file is docs/adrs/NNNN-<slug>.md, where <slug> is
#     any non-empty kebab-case identifier.
#
# Scope:
#   - All tracked files (git ls-files), excluding docs/adrs/ itself
#     (the index README cites every ADR — that's not a finding).
#
# Exit 0 on clean; 1 on dangling reference.

set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)"

if ! command -v rg >/dev/null 2>&1; then
    echo "✗ ripgrep (rg) is required for this lint." >&2
    exit 2
fi

# Build the set of known ADR IDs from filenames.
known_ids="$(find docs/adrs -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-*.md' \
    -exec basename {} \; \
    | sed -E 's/^([0-9]{4})-.*/\1/' \
    | sort -u)"

if [[ -z "$known_ids" ]]; then
    echo "✗ no ADRs found under docs/adrs/. Expected files like docs/adrs/0001-*.md" >&2
    exit 2
fi

# Find every adr: ADR-NNNN citation in tracked files (excluding docs/adrs/).
hits="$(git ls-files \
    | grep -v '^docs/adrs/' \
    | xargs rg --no-heading --line-number --color=never \
        -oP 'adr:\s*ADR-\K[0-9]{4}' 2>/dev/null \
    || true)"

if [[ -z "$hits" ]]; then
    echo "✓ no adr: citations in tracked source (yet)"
    exit 0
fi

# Diff cited vs known.
cited_ids="$(echo "$hits" | awk -F: '{print $NF}' | sort -u)"
missing="$(comm -23 <(echo "$cited_ids") <(echo "$known_ids"))"

if [[ -n "$missing" ]]; then
    echo "::error::dangling ADR citation(s):"
    for id in $missing; do
        echo "  ADR-$id is cited but docs/adrs/$id-*.md does not exist"
        echo "$hits" | grep ":$id$" | sed 's/^/    cited at /'
    done
    exit 1
fi

# Report green with stats so CI logs are useful.
n_cited="$(echo "$cited_ids" | wc -l | awk '{print $1}')"
n_known="$(echo "$known_ids" | wc -l | awk '{print $1}')"
echo "✓ $n_cited distinct ADR(s) cited; all $n_cited resolve to docs/adrs/ (of $n_known total)."
