<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# AGENTS.md

Invariants for AI-assisted contributions to this repository. Written for
coding agents, and equally applicable to humans in a hurry — every rule
here exists because something silently broke without it.

Read [`DEVELOPMENT.md`](DEVELOPMENT.md) first for how to run the gates,
and [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how the system
fits together. This file covers only what is easy to get wrong.

## The failure mode to watch for

Nearly every defect found in this repository has the same shape:
**something reported success while asserting nothing.**

- A test file that no workflow ran — 29 of them, until
  `tests/ci_test_coverage.rs` was added
- A gate that scanned an empty directory and passed
- A branch that deleted nine ADR files with `git status` clean, because
  `.gitignore` swallowed the renamed directory
- A golden test that skipped on every run and had no golden file
- A `grep … ; echo "ok"` pipeline read as confirmation, when the `echo`
  runs regardless of the grep

So: **a green result is not evidence until you know the test can fail.**
Before claiming a fix works, make it fail on purpose. Before trusting a
command, check its exit status rather than its output.

## Hard rules

**Never add a dependency without running `cargo vet --locked` first.**
The exemption count in `supply-chain/config.toml` is a ratchet that may
only decrease. If vet reports an unvetted crate, the options are to
audit it properly or not add it — never to raise the baseline. Two
crates were declined this way (`roff`, `clap_complete`) and replaced
with narrow in-tree emitters; that is the expected outcome, not a
failure.

**Never fabricate an attestation, a benchmark number, or a test result.**
If you did not run it, say you did not run it.

**Run what CI runs, not something like it.** `cargo clippy --lib` is not
`cargo clippy --lib --tests --examples --all-features`. The table in
`DEVELOPMENT.md` maps each job to its exact command and is CI-checked
against the workflow.

**Commits are signed.** After any rebase, verify with
`git log --format='%G?'` — a rebase rewrites every commit and re-signs
only if configured.

**The same commit, or it did not happen.** A behaviour change ships with
its test. "Follow-up PR" is not a plan.

## Repository-specific traps

**`docs/` is both a build target and a source tree.** `.gitignore` denies
`/docs/*` and re-admits committed subtrees by allowlist. A new file
under `docs/` is invisible to git until its allowlist entry exists, and
`git status` will not tell you — it does not list ignored files. Renaming
a listed directory has the same effect. Verify with
`git add --dry-run <path>` and `git ls-tree -r HEAD -- docs/`, not with
the filesystem.

**`[profile.bench] panic = "unwind"` is load-bearing.** Cargo warns that
it is ignored. Cargo is wrong about the half that matters: it is ignored
for the bench *target*, not for dependencies compiled under the profile,
which otherwise inherit `panic = "abort"` from release and fail to link
as `error[E0463]: can't find crate for ssg`. Do not "fix" the warning.
The evidence is in the comment above it.

**Example-building test suites are gated on `SSG_REQUIRE_EXAMPLES`.**
`tests/example_outputs.rs` and `tests/json_feed_compliance.rs` build and
run shipped examples — about thirteen minutes, serialised on port 3000.
They run in the `examples` job and skip elsewhere. Do not ungate them;
doing so previously pushed all three per-OS `test` jobs past their
timeout.

**Wall-clock performance budgets are Linux-only.** The same 10-page
build measured 131 ms and then 324 ms on consecutive Windows runs. The
machine-independent gate is
`build_cost_per_page_does_not_grow_with_corpus_size`, which compares
per-page cost at two corpus sizes. If a perf gate fails, do not raise
the constant — that was tried three times.

**Documentation counts are derived, never restated.** Plugin and gate
counts live in `README.md` where `tests/readme_sync.rs` checks them
against the registered pipeline. Do not add a count to a document that
nothing gates.

## Before you claim it works

- [ ] The gate was seen to fail before it was seen to pass
- [ ] Exit status checked, not output eyeballed
- [ ] `cargo fmt --all -- --check` and both clippy passes clean
- [ ] `git ls-tree` consulted if anything under `docs/` moved
- [ ] Signatures verified if history was rewritten
- [ ] What you did *not* verify is stated plainly

## Attribution

Include the agent's co-authorship trailer in commits. Do not remove the
`Co-Authored-By` line from an existing commit when amending.
