#!/usr/bin/env python3
"""Measure this repository against a fixed rubric and emit a scorecard.

A rating is only worth having if it is *measured*. This crate already lived
through the failure an asserted score produces: `assert_eq!(props["site_title"]
["default"], "My SSG Site")` passed green for thirteen releases while that
placeholder shipped on 7,189 live tag pages. The assertion was true. It
answered the wrong question, and nothing noticed.

So every metric here names the command that produces it, and any metric this
script cannot measure is reported as ``unmeasured`` rather than guessed. An
honest gap scores nothing and says so; it never quietly becomes a 10.

Usage:
    python3 tools/quality_scorecard.py            # human table
    python3 tools/quality_scorecard.py --json     # raw measurements
    python3 tools/quality_scorecard.py --fail-under 8.0

Slow gates (coverage, benchmarks) are skipped unless --deep is passed; they
report ``unmeasured`` instead of a stale cached number, because a figure from
a previous tree is worse than no figure.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parents[1]

# The toolchain CI resolves from `channel = "stable"`. A local
# RUSTUP_TOOLCHAIN export silently pins something older, and lints it cannot
# see are lints this score would miss.
TOOLCHAIN = "+1.98.0"

UNMEASURED = "unmeasured"


@dataclass
class Metric:
    """One measurement, the command behind it, and how it scores."""

    key: str
    label: str
    command: str
    score_fn: Callable[[object], float]
    value: object = UNMEASURED
    detail: str = ""

    @property
    def measured(self) -> bool:
        return self.value != UNMEASURED

    @property
    def score(self) -> float | None:
        return self.score_fn(self.value) if self.measured else None


@dataclass
class Category:
    """A weighted group of metrics."""

    key: str
    label: str
    weight: float
    metrics: list[Metric] = field(default_factory=list)

    @property
    def measured(self) -> list[Metric]:
        return [m for m in self.metrics if m.measured]

    @property
    def score(self) -> float | None:
        got = [m.score for m in self.measured]
        return sum(got) / len(got) if got else None


def band(thresholds: list[tuple[float, float]], *, higher_is_better: bool = True):
    """Score by threshold. First matching band wins."""

    def scorer(value: object) -> float:
        try:
            v = float(value)  # type: ignore[arg-type]
        except (TypeError, ValueError):
            return 0.0
        for edge, points in thresholds:
            if (higher_is_better and v >= edge) or (
                not higher_is_better and v <= edge
            ):
                return points
        return 0.0

    return scorer


def boolean(points_true: float = 10.0, points_false: float = 0.0):
    def scorer(value: object) -> float:
        return points_true if value is True else points_false

    return scorer


# Exit codes that mean "could not measure", never "measured a failure".
# Conflating the two is the bug this whole file exists to avoid: a tool that
# is absent or timed out tells you nothing, and scoring it 0 is a guess
# dressed as a measurement.
RC_MISSING = 127
RC_TIMEOUT = 124


def run(
    cmd: list[str], cwd: Path = ROOT, timeout: int = 1800
) -> tuple[int, str]:
    """Run a command, returning (exit code, combined output)."""
    try:
        p = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
        out = (p.stdout or "") + (p.stderr or "")
        # `cargo foo` for an uninstalled subcommand exits 101, not 127: the
        # binary resolves, the subcommand does not. Without this, "not
        # installed" scores identically to "installed and failing".
        if "no such command" in out.lower() or "is not installed" in out.lower():
            return RC_MISSING, out
        return p.returncode, out
    except FileNotFoundError:
        return RC_MISSING, "binary not found"
    except subprocess.TimeoutExpired:
        return RC_TIMEOUT, "timed out"


def gate(rc: int) -> object:
    """A pass/fail gate result, or ``unmeasured`` if it never really ran."""
    if rc in (RC_MISSING, RC_TIMEOUT):
        return UNMEASURED
    return rc == 0


def _rs_files(*roots: str) -> list[Path]:
    out: list[Path] = []
    for r in roots:
        base = ROOT / r
        if base.is_dir():
            out.extend(p for p in base.rglob("*.rs") if "target" not in p.parts)
    return out


# ── measurement ────────────────────────────────────────────────────────


def measure_code(cat: Category) -> None:
    rc, _ = run(["cargo", TOOLCHAIN, "fmt", "--all", "--", "--check"])
    cat.metrics[0].value = gate(rc)

    rc, _ = run(
        ["cargo", TOOLCHAIN, "clippy", "--lib", "--all-features",
         "--keep-going", "--", "-D", "warnings"]
    )
    cat.metrics[1].value = gate(rc)

    rc, _ = run(
        ["cargo", TOOLCHAIN, "clippy", "--lib", "--tests", "--examples",
         "--all-features", "--keep-going", "--", "-D", "warnings",
         "-A", "clippy::unwrap_used", "-A", "clippy::expect_used"]
    )
    cat.metrics[2].value = gate(rc)

    # `#![forbid(unsafe_code)]` is a compiler-enforced fact, not a promise.
    lib = ROOT / "src" / "lib.rs"
    if lib.is_file():
        cat.metrics[3].value = "#![forbid(unsafe_code)]" in lib.read_text()

    files = _rs_files("src", "crates")
    if files:
        allows = sum(
            len(re.findall(r"#\[allow\(", f.read_text(errors="ignore")))
            for f in files
        )
        cat.metrics[4].value = allows
        cat.metrics[4].detail = f"across {len(files)} .rs files"


def measure_tests(cat: Category, deep: bool) -> None:
    rc, out = run(["cargo", TOOLCHAIN, "test", "--lib", "--all-features"])
    if rc in (0, 101):
        total = sum(
            int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", out)
        )
        failed = sum(
            int(m) for m in re.findall(r"(\d+) failed", out)
        )
        cat.metrics[0].value = total
        cat.metrics[1].value = failed == 0
        cat.metrics[1].detail = f"{failed} failing"

    targets = list((ROOT / "fuzz" / "fuzz_targets").glob("*.rs"))
    cat.metrics[2].value = len(targets)

    if deep:
        rc, out = run(
            ["cargo", TOOLCHAIN, "llvm-cov", "--lib", "--summary-only"],
            timeout=2400,
        )
        if rc == 0:
            m = re.search(
                r"TOTAL\s+\d+\s+\d+\s+([\d.]+)%\s+\d+\s+\d+\s+([\d.]+)%"
                r"\s+\d+\s+\d+\s+([\d.]+)%",
                out,
            )
            if m:
                cat.metrics[3].value = float(m.group(1))
                cat.metrics[4].value = float(m.group(3))
    else:
        for i in (3, 4):
            cat.metrics[i].detail = "run with --deep"


def measure_security(cat: Category) -> None:
    rc, _ = run(["cargo", "deny", "check", "advisories"])
    cat.metrics[0].value = gate(rc)

    rc, _ = run(["cargo", "deny", "check", "licenses"])
    cat.metrics[1].value = gate(rc)

    cat.metrics[2].value = (ROOT / "supply-chain" / "config.toml").is_file()

    # Unsafe in the workspace, counted rather than assumed absent.
    files = _rs_files("src", "crates")
    if files:
        unsafe = sum(
            len(re.findall(r"\bunsafe\s*\{", f.read_text(errors="ignore")))
            for f in files
        )
        cat.metrics[3].value = unsafe


def measure_perf(cat: Category, deep: bool) -> None:
    benches = list((ROOT / "benches").glob("*.rs"))
    cat.metrics[0].value = len(benches)

    baselines = list(
        (ROOT / "benches" / "baselines" / "criterion-json").glob("*.json")
    )
    cat.metrics[1].value = len(baselines)

    cat.metrics[2].value = (ROOT / "tests" / "perf_budgets.rs").is_file()

    if deep:
        rc, _ = run(
            ["cargo", TOOLCHAIN, "test", "--release", "--test", "perf_budgets"],
            timeout=2400,
        )
        cat.metrics[3].value = gate(rc)
    else:
        cat.metrics[3].detail = "run with --deep"


def measure_docs(cat: Category) -> None:
    rc, out = run(["cargo", TOOLCHAIN, "doc", "--no-deps", "--all-features"])
    if rc == 0:
        missing = len(re.findall(r"warning: missing documentation", out))
        cat.metrics[0].value = missing
    cat.metrics[1].value = len(list((ROOT / "docs" / "adrs").glob("*.md")))
    cat.metrics[2].value = (ROOT / "README.md").is_file()

    # A crate published without a README renders bare on crates.io.
    crates = [d for d in (ROOT / "crates").iterdir() if d.is_dir()]
    if crates:
        with_readme = sum(1 for d in crates if (d / "README.md").is_file())
        cat.metrics[3].value = round(100.0 * with_readme / len(crates), 1)
        cat.metrics[3].detail = f"{with_readme}/{len(crates)} crates"


def measure_release(cat: Category) -> None:
    cargo = (ROOT / "Cargo.toml").read_text()
    cat.metrics[0].value = bool(re.search(r'rust-version\s*=', cargo))

    root_v = re.search(r'^version\s*=\s*"([0-9.]+)"', cargo, re.M)
    if root_v:
        v = root_v.group(1)
        members = list((ROOT / "crates").glob("*/Cargo.toml"))
        agree = sum(
            1 for m in members
            if re.search(rf'^version\s*=\s*"{re.escape(v)}"', m.read_text(), re.M)
        )
        cat.metrics[1].value = len(members) - agree
        cat.metrics[1].detail = f"root {v}, {agree}/{len(members)} crates agree"

    rc, _ = run(["cargo", TOOLCHAIN, "semver-checks", "--help"])
    cat.metrics[2].value = gate(rc)
    if cat.metrics[2].value == UNMEASURED:
        cat.metrics[2].detail = "cargo-semver-checks not installed"


def rubric() -> list[Category]:
    return [
        Category("code", "Code quality", 0.20, [
            Metric("fmt", "rustfmt clean", "cargo fmt --all -- --check", boolean()),
            Metric("clippy_lib", "clippy lib strict clean",
                   "cargo clippy --lib --all-features -- -D warnings", boolean()),
            Metric("clippy_tests", "clippy tests+examples clean",
                   "cargo clippy --lib --tests --examples --all-features", boolean()),
            Metric("unsafe_forbid", "#![forbid(unsafe_code)]",
                   "grep src/lib.rs", boolean()),
            Metric("allows", "#[allow(..)] suppressions",
                   "grep -c over src/ and crates/",
                   band([(20, 10), (50, 9), (100, 7), (200, 5), (400, 3)],
                        higher_is_better=False)),
        ]),
        Category("tests", "Tests & verification", 0.22, [
            Metric("count", "unit tests passing",
                   "cargo test --lib --all-features",
                   band([(3000, 10), (1500, 9), (600, 8), (200, 6), (50, 4)])),
            Metric("green", "suite green", "cargo test --lib --all-features",
                   boolean()),
            Metric("fuzz", "fuzz targets", "ls fuzz/fuzz_targets/*.rs",
                   band([(4, 10), (3, 9), (2, 7), (1, 5)])),
            Metric("cov_regions", "region coverage (%)",
                   "cargo llvm-cov --lib --summary-only",
                   band([(98, 10), (95.5, 9), (90, 7), (80, 5), (60, 3)])),
            Metric("cov_lines", "line coverage (%)",
                   "cargo llvm-cov --lib --summary-only",
                   band([(98, 10), (96.5, 9), (90, 7), (80, 5), (60, 3)])),
        ]),
        Category("security", "Security / supply chain", 0.20, [
            Metric("advisories", "cargo-deny advisories clean",
                   "cargo deny check advisories", boolean()),
            Metric("licenses", "cargo-deny licenses clean",
                   "cargo deny check licenses", boolean()),
            Metric("vet", "supply-chain vet configured",
                   "test -f supply-chain/config.toml", boolean()),
            Metric("unsafe_blocks", "unsafe blocks in workspace",
                   "grep -c 'unsafe {' over src/ and crates/",
                   band([(0, 10), (2, 8), (5, 5), (12, 2)],
                        higher_is_better=False)),
        ]),
        Category("perf", "Performance", 0.16, [
            Metric("benches", "benchmark files", "ls benches/*.rs",
                   band([(8, 10), (5, 9), (3, 7), (1, 5)])),
            Metric("baselines", "committed criterion baselines",
                   "ls benches/baselines/criterion-json/*.json",
                   band([(1, 10), (0, 4)])),
            Metric("budget_gate", "perf budget gate present",
                   "test -f tests/perf_budgets.rs", boolean()),
            Metric("budget_pass", "perf budgets within limit",
                   "cargo test --release --test perf_budgets", boolean()),
        ]),
        Category("docs", "Documentation", 0.12, [
            Metric("missing", "missing-docs warnings",
                   "cargo doc --no-deps --all-features",
                   band([(0, 10), (3, 8), (10, 6), (30, 3)],
                        higher_is_better=False)),
            Metric("adrs", "architecture decision records",
                   "ls docs/adrs/*.md",
                   band([(8, 10), (5, 9), (3, 7), (1, 5)])),
            Metric("readme", "root README present", "test -f README.md",
                   boolean()),
            Metric("crate_readmes", "workspace crates with a README (%)",
                   "test -f crates/*/README.md",
                   band([(100, 10), (80, 8), (50, 5), (20, 3)])),
        ]),
        Category("release", "Release & compatibility", 0.10, [
            Metric("msrv", "MSRV pinned", "grep rust-version Cargo.toml",
                   boolean()),
            Metric("version_drift", "crates disagreeing with root version",
                   "compare Cargo.toml versions",
                   band([(0, 10), (1, 6), (3, 3)], higher_is_better=False)),
            Metric("semver", "cargo-semver-checks available",
                   "cargo semver-checks --help", boolean()),
        ]),
    ]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="emit raw measurements")
    ap.add_argument("--deep", action="store_true",
                    help="include slow gates (coverage, perf budgets)")
    ap.add_argument("--fail-under", type=float,
                    help="exit 1 below this overall score")
    args = ap.parse_args(argv)

    cats = rubric()
    measure_code(cats[0])
    measure_tests(cats[1], args.deep)
    measure_security(cats[2])
    measure_perf(cats[3], args.deep)
    measure_docs(cats[4])
    measure_release(cats[5])

    scored = [c for c in cats if c.score is not None]
    total_w = sum(c.weight for c in scored)
    overall = (
        sum(c.score * c.weight for c in scored) / total_w if total_w else 0.0
    )

    n_measured = sum(len(c.measured) for c in cats)
    n_total = sum(len(c.metrics) for c in cats)

    if args.json:
        print(json.dumps({
            "overall": round(overall, 2),
            "measured": n_measured,
            "total": n_total,
            "categories": [
                {
                    "key": c.key,
                    "label": c.label,
                    "weight": c.weight,
                    "score": None if c.score is None else round(c.score, 2),
                    "metrics": [
                        {
                            "key": m.key,
                            "label": m.label,
                            "command": m.command,
                            "value": m.value,
                            "score": m.score,
                            "detail": m.detail,
                        }
                        for m in c.metrics
                    ],
                }
                for c in cats
            ],
        }, indent=2))
    else:
        print("Quality scorecard — every figure below is measured, not asserted.")
        print("'unmeasured' means exactly that; it never scores.\n")
        print(f"{'CATEGORY':<28}{'SCORE':>7}{'COVER':>8}  METRICS")
        print("-" * 74)
        for c in cats:
            s = "  n/a " if c.score is None else f"{c.score:5.2f}"
            print(f"{c.label:<28}{s:>7}{len(c.measured):>5}/{len(c.metrics):<3}"
                  f"weight {int(c.weight * 100)}%")
            for m in c.metrics:
                sc = "    -" if m.score is None else f"{m.score:5.1f}"
                val = m.value if m.measured else UNMEASURED
                extra = f"  ({m.detail})" if m.detail else ""
                print(f"    {sc}  {m.label:<44}{val}{extra}")
            print()
        print("-" * 74)
        print(f"{'WEIGHTED OVERALL':<28}{overall:5.2f}")
        print(f"{n_measured}/{n_total} metrics measured")

    if args.fail_under is not None and overall < args.fail_under:
        print(f"\nFAIL: {overall:.2f} < {args.fail_under}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
