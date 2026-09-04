<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# API Stability Audit — SSG 0.0.39

**Issue:** [#427](https://github.com/sebastienrousseau/static-site-generator/issues/427)
· **Target:** Prepare the public surface for eventual stabilisation
  (no `1.0.0` — or `0.1.0` — is scheduled; see
  [ADR-0009](../adr/0009-versioning-policy-0.0.x-until-0.0.999.md).
  This audit's tiering still stands on its own merits: it documents the
  right stability posture for each item regardless of which `0.0.x`
  release it lands in.)
· **Scope:** ~79 public structs/enums across 30+ modules + the `Plugin` trait

This document is the inventory product of the v0.0.41 milestone "Category
Creation & API Stability". It is *not* the breaking pass — it is the map
that the breaking pass works from. Each item below lists what it is, where
it lives, and which tier of stabilisation it should land in.

The breaking changes themselves (visibility demotions, `#[non_exhaustive]`
additions on stable enums, trait freezes) ship in tagged follow-ups so
each change is reviewable in isolation and the changelog can attribute
the breakage cleanly.

---

## Tier A — Stable, Freeze (Keep `pub`)

User-facing types every consumer depends on. These are the load-bearing
beams of the public API. Any change here is a major-version event.

| Item | Location | Notes |
|---|---|---|
| `compile_site` | `src/pipeline.rs:284` | Direct wrapper over `staticdatagen::compile`. Used in benches and 19 examples. |
| `execute_build_pipeline` | `src/pipeline.rs:200` | Core orchestration entry point. |
| `build_pipeline` | `src/pipeline.rs:160` | Constructs `(PluginManager, PluginContext, ...)` from `SsgConfig` + `RunOptions`. |
| `Paths` | `src/lib.rs:199` | Site-layout configuration; transparent struct. |
| `PathsBuilder` | `src/lib.rs:278` | Fluent builder; will gain methods over time → see Tier B. |
| `Plugin` (trait) | `src/plugin.rs:242` | Plugin extension point. New hooks must remain optional with default impls. |
| `PluginManager` | `src/plugin.rs:328` | Required to construct a build. |
| `PluginContext` | `src/plugin.rs:148` | Passed to every hook → see Tier B for `#[non_exhaustive]`. |
| `MAX_DIR_DEPTH` | `src/lib.rs:195` | Public constant; freeze the value. |
| `run` | `src/lib.rs:425` | The `main()` library entry. |

---

## Tier B — Public, Evolving (Keep `pub`, Add `#[non_exhaustive]`)

Configuration structs and enums that *will* grow. Marking them
`#[non_exhaustive]` now lets future SSG versions add fields/variants
without a major bump. Downstream code already ought to use a wildcard
`_ => …` arm or `..Default::default()` in literals; these annotations
make that requirement load-bearing.

### Configuration structs

| Item | Location | Why |
|---|---|---|
| `SsgConfig` | `src/cmd/config.rs:20` | Will gain new fields (Perplexity API, multi-zone CDN, etc.). |
| `SsgConfigBuilder` | `src/cmd/config.rs:192` | Mirrors `SsgConfig`. |
| `I18nConfig` | `src/i18n.rs:57` | RTL config, collation rules pending. |
| `LlmConfig` | `src/llm.rs:26` | LLM integration is experimental — model names + parameters churn. |
| `JsonLdConfig` | `src/seo/jsonld.rs` | Schema.org `@context` versions evolve. |
| `WatchConfig` | `src/watch.rs:81` | Currently dev-only; may stay or move to Tier C. |
| `TemplateConfig` | `src/template_engine.rs:18` | Custom-filter plumbing will grow. |
| `MemoryBudget` | `src/streaming.rs:30` | Streaming compile heuristics need tuning knobs (issue audit notes a `--batch-size-estimator` flag). |

### Enums (all should carry `#[non_exhaustive]`)

| Item | Location | Why |
|---|---|---|
| `DeployTarget` | `src/deploy.rs:15` | New platforms incoming (AWS, Azure, Cloudflare R2 sites). |
| `FieldType` | `src/content.rs:55` | Will gain `Uuid`, `Slug`, `Email` validators. |
| `UrlPrefixStrategy` | `src/i18n.rs:38` | Custom strategies via plugin. |
| `ProcessError` | `src/process.rs:26` | New error cases on every minor version. |
| `ReadabilityFormula` | `src/llm.rs:750` | More formulas planned (Dale-Chall, Linsear-Write). |
| `ChangeKind` | `src/watch.rs:53` | New change classifications for fast watch-mode rebuilds. |
| `CliError` | `src/cmd/error.rs:55` | **Already annotated** — the precedent for the rest. |

### Reports / results

These are returned to user code from plugins or the build. Adding fields
to a report shouldn't break consumers, but only if the type is
`#[non_exhaustive]`.

| Item | Location |
|---|---|
| `SearchEntry` | `src/search.rs:28` |
| `SearchIndex` | `src/search.rs:48` |
| `SearchLabels` | `src/search.rs:123` |
| `AccessibilityReport` | `src/accessibility.rs:37` |
| `AccessibilityIssue` | `src/accessibility.rs:17` |
| `PageReport` | `src/accessibility.rs:28` |
| `AuditReport` | `src/llm.rs:82` |
| `AiFixReport` | `src/llm.rs:112` |
| `AiFixResult` | `src/llm.rs:97` |
| `FileAuditResult` | `src/llm.rs:67` |
| `ReadabilityAudit` | `src/llm.rs:787` |
| `BuildError` | `src/pipeline.rs:25` |
| `TaxonomyTerm` | `src/taxonomy.rs:22` |
| `FieldDef` | `src/content.rs:115` |

---

## Tier C — Internal (Demote to `pub(crate)`)

Items that are `pub` today but only ever called from inside the crate.
Demoting tightens the surface and prevents future accidental external
dependencies.

| Item | Location | Verified internal? |
|---|---|---|
| `RunOptions` | `src/pipeline.rs:97` | ✅ — no use in `tests/`, `examples/`, `benches/`, `crates/` |
| `LanguageCode` | `src/cmd/error.rs:17` | ✅ — newtype, not exposed in pub fn signatures |
| `PluginCache` | `src/plugin.rs:61` | ⚠️ — exposed via `PluginContext`; demote behind a `pub(crate) fn` accessor |

**Cannot demote** (currently called from integration tests, which use
the crate as an external consumer):

| Item | Location | Blocking caller |
|---|---|---|
| `DepGraph` | `src/depgraph.rs:20` | `tests/perf_regression.rs:219, 421, 431, 436` |
| `BatchResult` | `src/stream.rs:39` | `tests/regression.rs:20, 718` |

These two need a parallel cleanup: either move their tests inside
`src/*/tests.rs` modules, or expose them only behind a `test-util` Cargo
feature. Ticket separately.

---

## Tier D — Official Plugins (Document, Don't Move)

Concrete plugin implementations users will reference and extend. Keep
`pub`; add a stability docstring so reviewers know which patterns are
"officially supported".

`AccessibilityPlugin`, `AiPlugin`, `FingerprintPlugin`, `CspPlugin`,
`DraftPlugin`, `ContentValidationPlugin`, `DeployPlugin`,
`HighlightPlugin`, `I18nPlugin`, `IslandPlugin`, `LiveReloadPlugin`,
`ImageOptimizationPlugin`, `OgImagePlugin`, `PaginationPlugin`,
`SearchPlugin`, `LocalizedSearchPlugin`, `TaxonomyPlugin`,
`CanonicalPlugin`, `JsonLdPlugin`, `RobotsPlugin`, `SeoPlugin`,
`SitemapFixPlugin`, `RssAggregatePlugin`, `AtomFeedPlugin`,
`MinifyPlugin`, `TemplatePlugin`, `MarkdownExtPlugin` — 27 in total.

---

## Auto-Trait & Derive Gaps

| Type | Missing | Action |
|---|---|---|
| `PluginManager` | `Clone` | Document as not-Clone; managers are stateful by design. |
| `FileWatcher` | `Clone` | Document; system resource. |
| `TemplateEngine` | `Clone` | Document; expensive (compiled template state). |
| Builder structs (subset) | `Default` | Add where missing for consistency. |
| All public `Plugin` impls | `Clone` where free | Most are unit structs (`Plugin;`) — already `Copy`-trivial. |

`Send + Sync` is required by the `Plugin` trait bound and is verified
by the trait itself; no separate audit needed.

---

## Recommended Actions for This PR (`v0.0.39`)

The non-breaking, additive subset that lands now:

1. ✅ Commit this audit document at `docs/architecture/api-stability-audit.md`.
2. ✅ Demote `RunOptions` to `pub(crate)` (verified zero external callers).
3. ✅ Add `#[non_exhaustive]` to the seven Tier B enums (verified no
   exhaustive matches in tests/examples).
4. ✅ Add `#[non_exhaustive]` to `DeployPlugin`'s `DeployTarget` so the
   v0.0.39 deploy-target additions don't ship as breaking later.
5. ✅ Document the `Plugin` trait's stability contract: new hook methods
   must default to `Ok(())`; field additions to `PluginContext` are
   non-breaking under `#[non_exhaustive]`.

Each lands as its own commit so blame stays readable.

---

## Defer until warranted (breaking pass — no version-milestone gate)

Per [ADR-0009](../adr/0009-versioning-policy-0.0.x-until-0.0.999.md),
`ssg` does not target `1.0.0-rc.1` on any horizon — every `0.0.x`
release is already free to ship breaking changes under SemVer. These
items are deferred by *readiness*, not by a release milestone; land
each in whichever `0.0.x` release is ready to absorb it, with its own
`CHANGELOG.md` breaking-change entry:

1. Demote `DepGraph` and `BatchResult` to `pub(crate)` after migrating
   `tests/perf_regression.rs` and `tests/regression.rs` to in-crate test
   modules.
2. `#[non_exhaustive]` on Tier B *config* structs (`SsgConfig`,
   `SsgConfigBuilder`, `I18nConfig`, `LlmConfig`, `JsonLdConfig`).
   Construction-site breakage requires `..Default::default()` updates
   in user code.
3. Standardise error handling: collapse `ProcessError` + `CliError` into
   `anyhow::Error` chains; make the granular enums internal.
4. Stabilise the streaming surface: expose `MemoryBudget` and the
   batch-size estimator as top-level configuration.
5. Freeze `Plugin` trait hook signatures; add new hooks only via
   `#[doc(hidden)]` defaults until 2.0.

---

## Snapshot

| Tier | Count | Action |
|---|---|---|
| A — Stable, freeze | 10 | none (already public) |
| B — Public, evolving (`#[non_exhaustive]`) | 28 | additive annotation |
| C — Demote to `pub(crate)` | 3 (this PR) + 2 (deferred) | scope-tightening |
| D — Official plugins | 27 | docstring update |

Total public types audited: **79 structs/enums + 1 trait + 30+
functions**. After this PR's actions land, the breaking-pass set is
pre-staged for whichever future `0.0.x` release is ready to absorb it
(see [ADR-0009](../adr/0009-versioning-policy-0.0.x-until-0.0.999.md)
— no `1.0.0-rc.1` is scheduled).
