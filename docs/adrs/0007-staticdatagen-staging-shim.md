<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0007: Content-staging shim for `staticdatagen` upstream regressions

- **Date:** 2026-06-27
- **Status:** Accepted (temporary — superseded once upstream issues land)

## Context

`ssg 0.0.45` discovered that `staticdatagen 0.0.9` (the markdown → HTML
compiler we delegate to) and its downstream dependencies
(`staticweaver 0.0.2`, `metadata-gen 0.0.4`, `rss-gen 0.0.5`) contain
five brittleness points that cause real-world user sites to fail
completely at build time:

1. **Empty `layout:` key** → `MiniJinja` errors with
   `invalid template or partial name: ""`.
2. **Templates missing `main.js` / `sw.js`** →
   `copy_auxiliary_files` aborts with opaque `os error 2`.
3. **No `tags.md` or `tags/index.md`** →
   `write_tags_html_to_file` aborts after every other artefact lands.
4. **Template references `{{ var }}` content omits** →
   `staticweaver` errors with `Unresolved template tag: <var>`.
5. **YAML-spec-valid multi-line quoted scalar** (e.g. `key: "\nvalue"`)
   → `noyalib` parser inside `metadata-gen` reports
   `No valid front matter found`.

The triggering case was `sebastienrousseau/sebastienrousseau.github.io`
(2,371 markdown files), which `ssg 0.0.44` and an early `ssg 0.0.45`
both crashed on. Each crash was an isolated edge case — the user's
content is YAML-spec-conformant; the upstream parsers / templating
engine are over-strict.

We cannot ship patched versions of `staticdatagen` (or its deps) from
this PR — those crates are separately versioned and release-gated.
But every minute without a shim is a minute the SSG demo "you don't
even need to think about layouts" pitch is broken in practice.

## Decision

**We ship a content-staging shim in `src/core/content_stager.rs`** that
pre-processes content + templates into a parallel directory tree under
`std::env::temp_dir()` before handing them off to
`staticdatagen::compile`. Each of the five brittleness points has a
corresponding shim function:

| Upstream brittleness | Shim function |
|---|---|
| Empty layout key | `inject_default_layout_if_missing` |
| Missing aux template files | `stage_templates_with_required_stubs` |
| Missing tags page | `ensure_tags_stub` |
| Template var not in content | `collect_template_vars` + `inject_missing_keys` |
| Multi-line quoted YAML scalar | `collapse_multiline_quoted_scalars` |

The shim runs unconditionally on every build. The user's source
directories are never written to. The staged tree lives under a
per-process / per-build-dir hashed path in `std::env::temp_dir()` so
concurrent builds (including the parallel test runner) cannot collide.

## Consequences

**Positive.**

- The 2,371-file real-world site builds successfully on the first try.
  Every flat-tree user site authored to v0.0.44 expectations now
  works on v0.0.45.
- The shim is a clean perimeter — every brittleness point has a single
  named function that can be removed when the corresponding upstream
  issue lands. The diff stays auditable.
- Idempotent: a build re-run produces byte-identical staging output,
  so there's no drift between consecutive runs.
- Parallelised via Rayon (matches ADR-0002) so the staging cost on a
  100-page corpus stays inside the perf-budget gate.
- The 6 regression tests in `tests/regression_user_site.rs` give us a
  named, runnable repro for each upstream issue.

**Negative.**

- Adds 1,300 LOC to the library — non-trivial maintenance cost.
- Each user build pays an extra I/O pass:
  read every `.md`, transform, write to staging, then let
  `staticdatagen::compile` re-read. Measurable but bounded (≤ 200 ms
  on a 100-page corpus after Rayon parallelisation).
- Couples our build pipeline to specific upstream bug shapes —
  if `staticdatagen` 0.0.10 changes the bug shape (e.g. the layout
  field name moves), the shim could become wrong shape.
- Tags-stub synthesises a "fake" page at
  `https://example.invalid/tags/`. Sites without a real tags page now
  ship that placeholder in their RSS feed. Acceptable for a v0.0.45
  hot-fix; ugly long-term.

## Alternatives Considered

- **Patch `staticdatagen` directly from this PR.** Rejected — it's a
  separately-released crate. The cycle time and SLSA-attestation
  blast radius are too large for a hot-fix.
- **Block the build at the first bad file with a clear error.**
  Rejected — the user expects v0.0.44 behaviour (silent layout
  defaulting). A loud error every time would be worse DX than a
  silent shim.
- **Ship a YAML rewriter that normalises every file in-place.**
  Rejected — the user's checkout is sacred. Source files must not
  change as a side effect of `ssg build`.
- **Ship a `--no-staging` flag that bypasses the shim.** Rejected —
  the shim runs unconditionally because every user site benefits.
  When upstream lands the equivalent fix, we delete the shim
  function, not toggle it.
- **Inject defaults into the in-memory metadata `HashMap` between
  `staticdatagen`'s parse and render passes.** Rejected — no public
  hook in `staticdatagen 0.0.9`. Adding one is itself an upstream PR.

## Status

Accepted as **a temporary measure**. The path to removal is documented
in `sebastienrousseau/static-site-generator#585` (v0.0.46 tracker),
which links each shim function to its upstream issue:

| Shim | Upstream issue |
|---|---|
| `inject_default_layout_if_missing` | `sebastienrousseau/staticdatagen#67` |
| `stage_templates_with_required_stubs` | `sebastienrousseau/staticdatagen#68` |
| `ensure_tags_stub` | `sebastienrousseau/staticdatagen#69` |
| `collect_template_vars` + `inject_missing_keys` | `sebastienrousseau/staticweaver#28` |
| `collapse_multiline_quoted_scalars` | `sebastienrousseau/metadata-gen#20` |

When each upstream lands, the corresponding shim function is removed,
its tests stay (re-purposed to assert upstream behaviour), and the
ADR's status flips to `Superseded by upstream`. Definition of done
for v0.0.46: `content_stager.rs` ≤ 200 LOC.

The shim's existence is itself a feature flag: as long as
`content_stager.rs` is non-trivial, we have unfinished upstream debt.
