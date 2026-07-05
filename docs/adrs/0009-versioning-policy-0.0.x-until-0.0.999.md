<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0009: 0.0.x-only versioning until 0.0.999 — no 0.1.0 or 1.0.0 yet

- **Date:** 2026-07-04
- **Status:** Accepted

## Context

`ssg` has been shipping under Cargo's `0.0.x` convention since its
first release, incrementing the patch segment by `0.0.1` per release
(0.0.33 → … → 0.0.47 as of this ADR). Two prior documents in this
repository state a `0.1.0`/`1.0.0` target on a short horizon:

- `ROADMAP.md`'s "Strategic 1.0 Roadmap" lays out Phase 2 (`0.1.0`,
  "2-3 Months") and Phase 3 (`1.0.0`, "6-12 Months") from a June 2026
  baseline.
- `docs/architecture/api-stability-audit.md` (v0.0.39) stages a
  breaking-changes pass explicitly "deferred to `1.0.0-rc.1`".

Neither reflects the project owner's actual intent. The `0.0.x` train
is deliberate, not an oversight to be graduated out of on the first
convenient milestone: per Cargo/SemVer convention, every `0.0.x → 0.0.y`
bump is *already* a breaking-change-eligible release (SemVer places no
compatibility guarantee between any two `0.0.x` versions), which is
exactly the flexibility a project still finding its production API
shape needs. Jumping to `0.1.0` prematurely converts every subsequent
release into a compatibility commitment before the API surface,
enterprise-adoption feedback loop, and audience are mature enough to
justify one.

## Decision

`ssg` (and its workspace crates: `ssg-core`, `ssg-rpc`, `ssg-rpc-macro`,
`ssg-search`, `ssg-wasm`, and any future extraction such as `ssg-a11y`)
stays on `0.0.x` versioning, incrementing by exactly `0.0.1` per
release, through **`0.0.999`** at the earliest. `0.1.0` will not ship
before `0.0.999` is reached. `1.0.0` follows only after a `0.1.0` line
has itself matured — it is not scheduled by this ADR and has no target
date.

This buys ~950 releases of runway (current: 0.0.47) to:

1. Mature the library's public API surface under real usage rather
   than a fixed calendar.
2. Grow an adoption base large enough that a `0.1.0` compatibility
   commitment reflects actual production users, not aspirational ones.
3. Reach full enterprise-readiness (the security/coverage/determinism
   posture this project already invests heavily in — cargo-vet,
   SLSA provenance, WCAG compliance gates, 99%+ region coverage) before
   the version number implies "stable enough to build a business on."

Every `0.0.x` release remains free to ship breaking changes when
warranted (as several already have — see `CHANGELOG.md`'s `0.0.39` and
`0.0.40` breaking-change entries) without violating SemVer, because no
compatibility promise exists below `0.1.0`.

## Consequences

**Positive:**
- Removes calendar pressure from architectural decisions — extractions
  like `ssg-a11y`, the WASI plugin sandbox (#574), or the full i18n
  crate (#588, targeted `0.0.49`) can each land as their own `0.0.x`
  release without forcing a premature API freeze.
- Every release can keep being a real, reviewable, SemVer-legitimate
  unit of change; nothing needs to wait for a "1.0 push."
- Documentation that references `0.1.0`/`1.0.0` as near-term targets
  (this ADR's Context section) needs a one-time correction so
  contributors don't plan against a date that isn't real.

**Negative:**
- External consumers reading `0.0.47` may (incorrectly, per SemVer,
  but predictably, per convention) assume the crate is pre-alpha.
  `README.md`/`CHANGELOG.md` should keep foregrounding the maturity
  signals (test count, coverage, SLSA, supply-chain attestation) that
  the version number alone doesn't convey.
- `docs/architecture/api-stability-audit.md`'s Tier C/D staged actions
  ("Defer to `1.0.0-rc.1`") no longer have a target release to land in;
  they become "defer until warranted," tracked by issue number instead
  of version milestone.

## Alternatives Considered

- **Ship `0.1.0` now, per the existing ROADMAP.** Rejected: locks in a
  compatibility promise before the API has absorbed feedback from
  enterprise adopters, and before extractions like `ssg-a11y` have
  had a chance to reshape the public surface they'll expose.
- **Skip straight to `1.0.0` once "feature complete."** Rejected:
  "feature complete" is not a real signal for a project that keeps
  finding new enterprise requirements (WASI sandboxing, semantic
  search, edge ISR) — there is no natural finish line to gate on.
- **Use pre-release tags (`0.0.47-beta.1`) instead of a long 0.0.x
  train.** Rejected: adds tooling complexity (crates.io pre-release
  semantics, `cargo add` defaults) for no benefit over the existing
  convention this project has already used successfully for 47
  releases.

## Status

Accepted. Supersedes the `0.1.0`/`1.0.0` timelines in `ROADMAP.md` and
`docs/architecture/api-stability-audit.md`, both corrected in the same
change that introduces this ADR.
