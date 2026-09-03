<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Governance

How decisions get made in this project, so that a contributor can
predict what will happen to their change before they write it.

## Model

SSG is a **BDFL-with-delegation** project. Sébastien Rousseau is the
maintainer and final decision-maker. This is stated plainly rather than
dressed up as a committee, because a small project with an honest
description of its governance is easier to contribute to than one with
aspirational structure nobody exercises.

That model has a known failure mode — a bus factor of one — and the
mitigation is that the decisions live in the repository rather than in
one person's head: architecture in [`docs/adr/`](docs/adr/), the CI
contract in [`DEVELOPMENT.md`](DEVELOPMENT.md), and the release process
in automation rather than a runbook.

## Who can do what

| Role | Granted by | Can |
|---|---|---|
| Contributor | Opening a PR | Propose changes, review, triage |
| Maintainer | Invitation | Merge, release, administer the repo |

There is currently one maintainer. Contributors who land several
substantive changes and show consistent judgement in review will be
invited; there is no application process, and no minimum count, because
judgement is the criterion rather than volume.

## How decisions are made

**Ordinary changes** — bug fixes, tests, documentation, dependency
bumps — need one maintainer approval and green CI. Most changes are
this.

**Architectural decisions** — anything that constrains future work, or
that a future contributor would reasonably question — need an ADR in
[`docs/adr/`](docs/adr/) in Nygard format, merged before or with the
implementation. `tools/lint-adr.sh` enforces the citation graph: an
`adr: ADR-NNNN` reference anywhere in the tree must resolve to a real
file, so a decision cannot be cited after its record is deleted.

Examples of changes that required an ADR: staying tokio-free
([ADR-0001](docs/adr/0001-tokio-free.md)), choosing Rayon for build
orchestration ([ADR-0002](docs/adr/0002-rayon-orchestration.md)), and
the `0.0.x`-until-`0.0.999` versioning policy
([ADR-0009](docs/adr/0009-versioning-policy-0.0.x-until-0.0.999.md)).

**Adding a dependency** is an architectural decision with a hard gate
rather than a judgement call. `cargo vet` runs an exemption ratchet: the
count in `supply-chain/config.toml` may only decrease. A new dependency
must be audited (`cargo vet certify`) or not added. Two were declined
this way while building the packaging artefacts — `roff` and
`clap_complete` — and replaced with narrow in-tree emitters. See
[`supply-chain/README.md`](supply-chain/README.md).

**Breaking changes** follow the versioning policy in ADR-0009. Note the
output-stability rule in the README's Stability section: for a
generator, a change to what the tool *emits* is breaking even when no
API signature moves.

## Disagreement

Technical disagreements are settled by evidence, in this order:

1. A failing test that demonstrates the problem
2. A measurement, with the method stated
3. A written trade-off in the PR or an ADR

"It is faster" or "it is cleaner" without one of the above is an opinion,
and opinions are welcome but do not settle anything. Where evidence is
genuinely balanced, the maintainer decides and records why.

If a decision is made that you believe is wrong, say so in the thread.
Reopening a settled question later requires new evidence, not repetition.

## CI is not negotiable

A red pipeline blocks a merge. There is no "will fix in a follow-up" for
a gate — the same commit, or it did not happen. This is stricter than
many projects and is deliberate: this repository's gates exist because
each one caught something real, and several were added after a silent
failure reached a release.

The corollary is that a gate you believe is wrong should be argued with
and changed, not bypassed. `DEVELOPMENT.md` documents how to run every
one locally.

## Releases

Releases are cut by a maintainer from a signed tag; the pipeline does
the rest. Tags, commits and artefacts are signed. The current support
window is in [`SECURITY.md`](SECURITY.md).

## Changing this document

By pull request, like anything else. Changes to the governance model
itself need maintainer approval and a stated reason.
