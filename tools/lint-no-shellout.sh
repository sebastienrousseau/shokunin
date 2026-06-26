#!/usr/bin/env bash
#
# lint-no-shellout.sh — refuses `Command::new("<shell-binary>")` outside
# the explicitly-exempted shell drivers under src/core/process.rs.
#
# Regression guard for the v0.0.44 LlmPlugin port from curl-shellout to
# ureq (issue #520). Without this lint, a contributor in a hurry could
# reintroduce a shell-injection vector via `std::process::Command::new`
# without anyone noticing in review.
#
# Catches: curl, wget, sh, bash, zsh, fish, dash, ksh, ash, pwsh,
#          PowerShell, cmd, cmd.exe, nc, netcat.
#
# Allowed surface:
#   - src/core/process.rs — clap-derive test helpers use Command::new("test")
#     and Command::new("t"); those binary names are NOT in the regex.
#   - tests/** — integration tests may exec real binaries deliberately.
#
# Exit 0 on clean; 1 on hit (with GitHub Actions ::error:: annotation).

set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)"

# Pattern: Command::new("<exact shell-binary name>")
# - Word boundary on the binary name (the closing quote enforces it).
# - case-insensitive so PowerShell variants are caught.
pattern='Command::new\("(curl|wget|sh|bash|zsh|fish|dash|ksh|ash|pwsh|PowerShell|cmd|cmd\.exe|nc|netcat)"\)'

if ! command -v rg >/dev/null 2>&1; then
    echo "✗ ripgrep (rg) is required for this lint." >&2
    exit 2
fi

# Search the production tree only. tests/ is allowed to exec real
# binaries (visual-regression, fault-injection, etc.).
hits=$(rg --no-heading --line-number --color=never \
        --type rust \
        -P "$pattern" \
        src/ crates/ 2>/dev/null || true)

if [[ -n "$hits" ]]; then
    echo "::error::shellout reintroduced — see SECURITY.md §Security-Relevant Defaults"
    echo "        regression of #520 (curl shellout port to ureq)"
    echo
    echo "$hits"
    exit 1
fi

echo "✓ no shellout patterns in src/ or crates/"
