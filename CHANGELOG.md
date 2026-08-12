<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.50] - 2026-08-11

The themeing release. Building three real themes against v0.0.49 surfaced
four defects that made documented features unusable — each failing silently,
which is why none had been reported. Multi-locale sites additionally gain
translated slugs.

### Fixed

- **`layout` was ignored on every page** (`src/plugins/template_plugin.rs`).
  `before_compile` writes front-matter sidecars to `<build_dir>/.meta`, but
  `staticdatagen` promotes `output.build-tmp` onto `output` once the compile
  finishes and takes `.meta/` with it. `after_compile` still read the old
  path, found nothing, and fell back to `layout = "page"` for every page — so
  a theme whose layouts were `index`/`about`/`contact` rendered through none
  of them, and through nothing at all without a `page.html`. The sidecar
  directory now resolves from `build_dir` and falls back to `site_dir`.
  `TemplateEngine::has_template` lets the plugin distinguish a real render
  from `render_page`'s pass-through arm, which it had been counting as a
  success — a pipeline doing nothing logged `Rendered N page(s)` exactly like
  a working one.
- **`content/content.schema.toml` broke the build it configures**
  (`src/core/content_stager.rs`). The documented location for typed
  front-matter schemas was staged as a page, and `staticdatagen` aborted with
  `Failed to extract metadata: No valid front matter found`. Build-time
  control files are now excluded from staging.
- **Nested `index.md` gained a directory level**
  (`src/core/content_stager.rs`). `fr/index.md` compiled to
  `fr/index/index.html` rather than `fr/index.html`, so every locale home
  page was wrong. The root cause is upstream — `staticdatagen`'s
  `write_files_to_build_directory` compares the whole processed name against
  `"index"` — so this is a staging-time side-step, not a fix, and it leaves
  the file alone when both `fr.md` and `fr/index.md` are authored.
- **Extracted CSS and JS 404'd on sub-path deployments**
  (`src/plugins/csp.rs`). Inline blocks are externalised into fingerprinted,
  SRI-signed `_csp/` files referenced as `/_csp/…`, which resolves against
  the domain root. On a GitHub Pages project site the whole stylesheet was
  lost. The prefix now derives from `base_url`'s path component; sites at the
  domain root are unaffected.
- **Islands never hydrated** (`src/plugins/islands.rs`,
  `src/plugins/assets.rs`). Three independent faults, each sufficient alone:
  the injected loader tag was root-absolute like `_csp/` above; the loader
  resolved component bundles with a root-absolute dynamic `import()`, now
  resolved relative to its own module URL so the runtime needs no knowledge
  of the mount point; and the asset fingerprinter renamed `_islands/*.js`
  without being able to rewrite a specifier built at runtime, so every bundle
  404'd. `_islands/` is excluded from fingerprinting.

### Added

- **Translated slugs across locales** (`src/plugins/i18n.rs`). Pages were
  paired across locales by identical relative path, so `about/index.html`
  and `a-propos/index.html` were two unrelated singletons and **neither
  received any `hreflang`** — silently, because from the plugin's view they
  were simply untranslated. Pages now declare a `translation_key` in front
  matter, read from the sidecars; the locale matrix inverts from
  `rel_path -> {locale}` to `key -> {locale -> rel_path}`. Pages without a
  key keep path matching, so existing sites are unaffected.
- **Root-hosted default locale** (`src/plugins/i18n.rs`). Every locale
  previously needed its own directory, including the default, which forced
  `/en/about/` and left the site root empty. The default locale may now live
  at the root — `/about/` alongside `/fr/a-propos/` — matching Hugo's
  `defaultContentLanguageInSubdir = false`, Astro's `prefixDefaultLocale:
  false` and Next.js's default.

### Changed

- **Alternate `hreflang` labels now describe the target document.** An
  English page labelled its Hindi alternate `hreflang="hi"` (the locale
  directory) while the Hindi page advertised `hreflang="hi-IN"` for itself.
  The two sides disagreed, failing reciprocity and the `hreflang` audit gate.
  Alternates now carry the target's resolved language. Two tests asserting
  the old asymmetry were updated, with the reasoning recorded in their
  bodies.
- **`x-default` is emitted only when the default locale serves the page**,
  rather than pointing at a URL that may not exist.

### Security

- **js-yaml 4.3.0 → 4.3.1** in the `tests/a11y` harness
  ([GHSA-5p4m-2wfm-xmqj](https://github.com/advisories/GHSA-5p4m-2wfm-xmqj),
  high): quadratic CPU consumption resolving `!!omap`. Development-only — the
  harness is not part of the published crate.
- **`extract-zip` 2.0.1 symlink path traversal**
  ([GHSA-jmr9-qjv8-65gv](https://github.com/advisories/GHSA-jmr9-qjv8-65gv),
  high) is **accepted, not fixed**. No patched version exists: the advisory
  covers `<= 2.0.1` and upstream has published no fix. The path is
  `pa11y → puppeteer → @puppeteer/browsers → extract-zip`; `pa11y` 9.1.1 is
  the latest release and pins `puppeteer ^24.37.5`, which requires
  `@puppeteer/browsers` 2.x. Forcing `@puppeteer/browsers` 3.x — which
  replaced `extract-zip` with `modern-tar` — resolves the advisory but breaks
  installation, because puppeteer 24.x calls the 2.x API (`downloadBrowsers`
  fails in `puppeteer/lib/esm/puppeteer/node/install.js`). Verified, then
  reverted.

  Risk accepted on the basis that the harness is development-only and never
  part of the published crate, and that the only archive it extracts is the
  Chromium build downloaded from Google's CDN over HTTPS — not
  attacker-controlled input. Revisit when `pa11y` moves to puppeteer 25.x.

### Dependencies

- oxc family 0.142 → 0.143 (`minifier`, `parser`, `codegen`, `allocator`,
  `span`), bumped in lockstep as the crates require.
- `noyalib` 0.0.17 → 0.0.18, `@playwright/test` 1.62.0 → 1.62.1, and the
  minor/patch group across the resolved graph.
- GitHub Actions: `github/codeql-action/*` v4.37.4 → v4.37.6,
  `actions/attest-build-provenance` v4.1.1 → v4.2.2,
  `Swatinem/rust-cache` to 6323deb1. `scheduled.yml` had been pinning
  `attest-build-provenance` independently of `release.yml` and drifting
  behind it; both now match.
- `deny.toml`: removed five `ignore` entries that no longer match any crate
  (RUSTSEC-2025-0057, -2025-0119, -2026-0173, -2026-0194, -2026-0195). The
  quick-xml removal plan recorded there on 2026-07-04 has completed. Each
  unmatched entry raises an `advisory-not-detected` warning, so the stale
  ones were burying any real one.

## [0.0.49] - 2026-08-05

Dependency maintenance only; no functional changes. Recorded here because
the release shipped without a changelog entry.

### Changed

- `oxc_minifier` and `oxc_allocator` 0.138 → 0.141 (lockstep family bump)
  ([#626](https://github.com/sebastienrousseau/static-site-generator/pull/626),
  [#627](https://github.com/sebastienrousseau/static-site-generator/pull/627)).
- `tungstenite` 0.29 → 0.30
  ([#624](https://github.com/sebastienrousseau/static-site-generator/pull/624)).
- Minor and patch group across 9 further dependencies
  ([#631](https://github.com/sebastienrousseau/static-site-generator/pull/631)).
- CI action bumps: `actions/checkout`, `actions/setup-node`,
  `github/codeql-action`, `ossf/scorecard-action`,
  `docker/setup-buildx-action`, `@playwright/test`.

## [0.0.48] - 2026-07-25

### Planned / Upcoming (deferred — not v0.0.46 scope)

- **Residual content-staging shim removal** — two narrow gaps remain in `src/core/content_stager.rs` until upstream follow-ups land: template-default injection (blocked on [staticdatagen#99](https://github.com/sebastienrousseau/staticdatagen/issues/99) — opting the staticweaver Engine into `lax_undefined`) and multi-line quoted-scalar collapse (blocked on [staticdatagen#100](https://github.com/sebastienrousseau/staticdatagen/issues/100) — bumping the transitive `metadata-gen` dep to `0.0.5`). Once both land, the module collapses to ~50 LOC.
- **Complete internal anyhow elimination** across 9 core modules (`cache`, `collections`, `content`, `depgraph`, `deploy`, `frontmatter`, `scaffold`, `stream`, `template_engine`) and 7 plugin modules (`ai`, `csp`, `llm`, `postprocess/{helpers,html_fix}`, `seo/{canonical,seo_plugin}`). `scaffold.rs` is the heaviest module in this sweep (14 uses). Once complete, `anyhow` will be dropped from the library's `[dependencies]` list in `Cargo.toml`.
- **Ratchet CI coverage floor to ≥98.0%** (regions, lines, functions). Currently at 95.71 / 96.87 / 95.77 with `--lib`. The remaining uncovered regions sit in I/O-heavy production glue that needs source-level seams.

## [0.0.47] - 2026-07-04

The trust release: every headline claim is now byte-verifiable in code, the
site-migration correctness defects (spec A1–A7) are fixed end-to-end with a
cross-platform determinism gate, and the three largest performance wins
landed with measured evidence. Implements the
[v0.0.47 plan](docs/plans/v0.0.47-implementation-plan.md) and tracker
[#586](https://github.com/sebastienrousseau/static-site-generator/issues/586).

### Added
- **Flexible date parsing** (`src/core/dates.rs`, spec A4): RFC 2822 →
  long-form (`July 1, 2026`) → ISO 8601, zero new deps, proptest
  round-tripped; wired into the RSS, Atom, JSON Feed, news-sitemap, and
  sitemap plugins. Unparseable fields log which format failed.
- **Native permalink derivation** (`src/core/urls.rs` + content stager, spec
  A2/B1): pages without `permalink`/`url` get one derived from
  `base_url + output_path` at staging time — feeds can never hard-fail on a
  missing channel link. Active in the real build path via
  `compile_site_with_base_url`.
- **Single page-language resolver** (spec A5): frontmatter `language` →
  `hreflang` → locale path prefix → site default. JSON-LD `inLanguage`,
  `og:locale`, `<html lang>`, and the hreflang self-reference now agree on
  every page, enforced by the new **`lang_consistency` audit gate** (gate
  15).
- **`IoPool` writer pool** ([#569](https://github.com/sebastienrousseau/static-site-generator/issues/569)
  phase 1): bounded-channel writer threads decouple `fs::write` from rayon
  CPU workers; the fused transform pass now **skips unchanged files**
  (a no-op rebuild writes zero files). io_uring backend remains phase 2.
- **`AgentApiPlugin`** (#586 port 3): `/api/agents/{index,posts,topics,person}.json`
  — a stable, deterministic JSON API for AI crawlers and agent toolchains.
- **`OembedPlugin`** (#586 port 4, opt-in): per-page `oembed.json` +
  discovery `<link>`.
- **Per-tag landing pages** (#586 port 5): `/tags/<tag>/index.html` with
  canonical/OG essentials inlined; author-authored tag hubs are never
  clobbered.
- **`[security] sri_algorithm` config** (spec B3): SRI `integrity=`
  attributes now default to **SHA-384** (matching the long-documented
  claim), configurable to sha256/sha512; CSP directive source hashes stay
  SHA-256 for UA compatibility.
- **Per-page CSP → edge headers** (spec B4): inline script/style/JSON-LD
  hashes are computed per page and emitted as per-path entries in
  `_headers` / `vercel-headers.json` — hash-strict CSP without
  `'unsafe-inline'`.
- **Social-meta derivation cascade** (spec B8): `og:*`/`twitter:*` derive
  from base frontmatter (`twitter_title ⇐ seo_title ⇐ title`,
  `og_image ⇐ banner ⇐ image`, …); explicit fields always win, no global
  bleed-through.
- **CI**: `determinism.yml` (macOS↔Linux output byte-diff + double-build
  reproducibility — the gate that would have caught spec A1 on day one),
  `fuzz.yml` + four cargo-fuzz targets
  ([#566](https://github.com/sebastienrousseau/static-site-generator/issues/566)),
  OSSF Scorecard, `cargo-semver-checks` job, cargo-vet exemption ratchet,
  and a multi-arch (amd64+arm64) GHCR image.
- **`tools/bench-vs-{hugo,zola,eleventy}.sh`** — the comparison scripts
  BENCHMARKS.md documented (closes the remainder of
  [#559](https://github.com/sebastienrousseau/static-site-generator/issues/559)).
- **cargo-vet first-party audits**: 13 genuine `safe-to-deploy` audits of
  the same-author dependency stack; exemptions 544 → 533 with a
  ratchet-only-downward CI gate.

### Fixed
- **Audit-gate false positives** (~122 alerts): the markdownlint gate no
  longer lints YAML frontmatter; the broken-links gate no longer parses
  `<a href>` inside `<script>`/`<style>`; the CSP/images/WCAG gates now
  parse minified HTML (unquoted, valueless, reordered attributes). The demo
  site audit is at **zero error-severity findings** (was 27 errors / 183
  total alerts).
- **Latent UTF-8 corruption** in strikethrough expansion: multi-byte
  characters were mangled on any line processed through the old
  byte-by-byte path.
- **Demo-site defects behind real alerts**: SPDX comment before
  `<!DOCTYPE html>`, missing H1s, missing `og:image`, empty-src logos,
  valueless `alt`.
- SARIF stdout purity: the "Site generated successfully." status line moved
  to stderr so `ssg audit --sarif > file` yields strictly valid JSON.
- `benches/all_pub_api` compiles under `--no-default-features`.

### Changed
- **One URL convention everywhere**: canonical, feed `<link>`, sitemap and
  news-sitemap `<loc>` all derive via `urls::derive_page_url`
  (`…/foo/index.html` → `…/foo/`).
- **Claims reconciliation**: "PQC-aware" → "PQC posture guidance" (ML-DSA
  provenance signing is roadmap
  [#579](https://github.com/sebastienrousseau/static-site-generator/issues/579));
  "streaming compilation" → "bounded-memory batch compilation" where it
  describes `core/streaming.rs`; `tests/docs_accuracy.rs` pins the new
  wording.
- Workspace version 0.0.46 → 0.0.47 across all six crates.

### Performance
- Markdown render clone elimination + `Cow` strikethrough fast path:
  **−41–63%** on realistic mostly-plain pages, −6–10% on dense GFM.
- Zero-copy frontmatter parsing
  ([#578](https://github.com/sebastienrousseau/static-site-generator/issues/578)):
  −17–25% on TOML frontmatter, intermediate body clone removed.
- Per-thread scratch reuse in search extraction; i18n locale-matrix cache
  `Mutex` → `RwLock` (read-mostly fast path).

### Upstream (staged in sibling repos, releases pending)
- `staticdatagen 0.0.11`: `allow_unsafe_html: true` explicit (spec A1),
  permalink fallback chain + never-abort feed semantics (spec A2), flexible
  date parsing (spec A4).
- `html-generator 0.0.7`: comrak 0.52, enforced escape-by-default for raw
  HTML, structured-data title from metadata (spec A3).
- `frontmatter-gen 0.0.7`: path-traversal guard scoped to path-typed fields
  (spec A6).
- `mdx-gen 0.0.5`: **security** — raw-HTML pass-through no longer forced;
  safe-by-default sanitization honoring the documented contract.

### New crate: `ssg-a11y`
- Extracted the WCAG 2.2 AA accessibility checker (`src/plugins/accessibility.rs`)
  into a standalone workspace crate,
  [#608](https://github.com/sebastienrousseau/static-site-generator/issues/608):
  report/matrix types, WCAG rule checks (heading hierarchy, ARIA landmarks,
  link purpose, focus appearance, target size, banned elements, page
  language), and the compliance-matrix builder now have zero dependency on
  ssg's `Plugin` trait, `PluginContext`, or `SsgError` — `#![forbid(unsafe_code)]`,
  its own error type, own README/tests. `src/plugins/accessibility.rs`
  becomes a thin wrapper (`AccessibilityPlugin` + file I/O); public paths
  under `ssg::accessibility::*` are unchanged for existing consumers.
  `accessibility-report.json` / `wcag-compliance.json` output is
  byte-shape-identical — this is a refactor, not a behavior change.
  Workspace grows to 7 crates.

### Fixed (determinism, round 2)
- `atom.xml` / `feed.json`: `AtomFeedPlugin`/`JsonFeedPlugin` sorted entries
  by date only; since `read_meta_sidecars` walks the filesystem (genuinely
  OS-order-dependent — ext4 vs APFS) and the stable sort never breaks ties
  on equal dates, synthetic fixtures where every page shares a date leaked
  raw directory-walk order straight into the feed. Fixed with an `id`-based
  tiebreaker at all three feed plugins (RSS/Atom/JSON Feed), plus a
  defensive sort at `read_meta_sidecars` itself so future consumers inherit
  deterministic input by default.
- `sbom.cdx.json`: a **second**, independent `SbomPlugin` implementation
  (`src/plugins/sbom.rs`, distinct from `src/plugins/postprocess/sbom.rs`)
  runs later in the pipeline and overwrites the first's output — it had
  its own unpinned `SystemTime::now()` call the earlier `SOURCE_DATE_EPOCH`
  fix never touched. Fixed and covered by a matching regression test.
- Determinism gate exclusions (`determinism.yml`) refined: `.meta/**/*.json`
  (upstream `staticdatagen` unordered-map serialization, tracked for the
  `=0.0.11` pin) and `.ssg-plugin-cache.json` (ssg's own incremental cache
  — legitimately path-keyed to each build's absolute output location, not
  a bug) are excluded from cross-path comparisons with documented reasons;
  the cache's own key order is now deterministic (`BTreeMap`, not
  `HashMap`) for the common same-directory-rebuild case.
- `core/lang.rs`: `DEFAULT_PAGE_LANG` and `resolve_render_lang` were dead
  code under `--no-default-features` (their sole caller is gated behind
  the `templates` feature) — gated to match, verified against all 45
  `cargo-hack --feature-powerset --depth 2` combinations.
- `examples/audit_example.rs`: un-backticked `lang_consistency` in a doc
  comment (clippy `doc_markdown`).

### Versioning policy
- [ADR-0009](docs/adrs/0009-versioning-policy-0.0.x-until-0.0.999.md):
  `ssg` stays on `0.0.x` versioning — incrementing by `0.0.1` per release —
  through `0.0.999` at the earliest, to mature the API surface and
  enterprise adoption before any `0.1.0`/`1.0.0` SemVer commitment.
  `ROADMAP.md` and `docs/architecture/api-stability-audit.md`'s prior
  `0.1.0`/`1.0.0` milestone targets are corrected to match.

## [0.0.46] - 2026-06-28

The "shim retirement" release. All 8 upstream fixes filed during the v0.0.45 cycle landed and shipped; the bulk of the v0.0.45 content-staging shim is gone.

### Upstream PRs that landed in this release

| Repo | Released | PR | Closes (upstream) | Closes (downstream) |
|---|---|---|---|---|
| `staticdatagen` → 0.0.10 | 2026-06-28 12:39 UTC | [#72](https://github.com/sebastienrousseau/staticdatagen/pull/72) | `#67` / `#68` / `#69` / `#70` / `#71` + 4 dependabot bumps | `inject_default_layout_if_missing`, `stage_templates_with_required_stubs`, `ensure_tags_stub`, `stage_content_with_default_layout` (4 stager pub fns retired) |
| `staticweaver` → 0.0.3 (then 0.0.4) | 2026-06-28 09:21 / 17:40 UTC | [#29](https://github.com/sebastienrousseau/staticweaver/pull/29) | `#28` + `askama_escape` drop | closes [ssg#589](https://github.com/sebastienrousseau/static-site-generator/issues/589) (idempotent HTML escape) via the `staticdatagen 0.0.10 → staticweaver 0.0.3` transitive dep |
| `rss-gen` → 0.0.6 | 2026-06-28 10:19 UTC | [#35](https://github.com/sebastienrousseau/rssgen/pull/35) | `#34` (context-prefixed validation errors + relative item-link URLs per RSS 2.0 §5.7) | (no ssg shim — pure upstream improvements) |
| `metadata-gen` → 0.0.5 | 2026-06-28 20:01 UTC | [#21](https://github.com/sebastienrousseau/metadata-gen/pull/21) | `#20` | held — `staticdatagen 0.0.10` still pins `metadata-gen = "0.0.4"`. Tracked: [staticdatagen#100](https://github.com/sebastienrousseau/staticdatagen/issues/100). |

### Changed

- **`Cargo.toml`** — `staticdatagen 0.0.9 → 0.0.10`. Lockfile reflects the transitive bumps: `staticweaver 0.0.2 → 0.0.3`, `rss-gen 0.0.5 → 0.0.6`.
- **`src/core/content_stager.rs`** — reduced from ~1,300 to ~660 LOC. Four shim public functions retired (their bugs closed natively in `staticdatagen 0.0.10`): `stage_content_with_default_layout`, `inject_default_layout_if_missing`, `ensure_tags_stub` (private), `stage_templates_with_required_stubs`. The `DEFAULT_LAYOUT` + `REQUIRED_TEMPLATE_FILES` consts and three helpers (`copy_templates_tree`, `frontmatter_has_layout_key`, the layout-injection branch of `copy_tree`) went with them. 21 obsolete unit tests removed. Module docstring rewritten to document the two residual shims and their upstream tracking issues.
- **`src/core/pipeline.rs::compile_site`** — drops the `stage_templates_with_required_stubs` call; the user's `template_dir` is now passed directly to `staticdatagen::compile`. The pipeline doc-comment now describes the v0.0.46 residual scope rather than the v0.0.45 regression matrix.
- **3 v0.0.45-era input-validation tests updated** (`test_compile_site_error`, `test_internal_compile_with_empty_directories`, `test_args_all_required_arguments`) — empty directories are now a valid "no work to do" build under `staticdatagen 0.0.10` (closes upstream #68 / #69) so the tests were re-pointed at a genuine io error (a file passed where a directory is expected) to keep error-propagation coverage.

### Added

- **`examples/multilingual_full/`** — 32-file content tree (5 locales × `index.md` + 5 posts) demonstrating the nested `content/<lang>/<slug>.md` layout that `staticdatagen 0.0.10` (closes upstream #70) walks recursively. Registered as the `multilingual_full` `[[example]]`. Verified end-to-end: 30/30 per-locale pages land.
- **`tests/regression_user_site.rs::nested_locale_subdirectories_build_per_language`** — new always-on regression covering the recursive walk on a 3 × 2 in-memory tempdir.
- **Two upstream follow-up issues filed** to track the remaining staticdatagen wiring needed before the residual shim disappears: [staticdatagen#99](https://github.com/sebastienrousseau/staticdatagen/issues/99) (`Engine::with_lax_undefined(true)`) and [staticdatagen#100](https://github.com/sebastienrousseau/staticdatagen/issues/100) (bump `metadata-gen` to `0.0.5`).

### Fixed

- **[#589](https://github.com/sebastienrousseau/static-site-generator/issues/589) — HTML-entity double-escape** (`&` → `&amp;amp;`) corrupting body text, `og:title`, `twitter:title`, JSON-LD strings, and `href`s on ~30 % of real-world pages. Closed transitively by bumping `staticdatagen` to 0.0.10 (which pulls in `staticweaver 0.0.3`'s idempotent `escape_html_into` from [staticweaver#29](https://github.com/sebastienrousseau/staticweaver/pull/29)).

### Workspace versions

`ssg`, `ssg-core`, `ssg-rpc`, `ssg-rpc-macro`, `ssg-search`, `ssg-wasm` — all bumped 0.0.45 → 0.0.46.

## [0.0.45] - 2026-06-27

### Added
- **Hygiene + correctness + supply-chain attestation baseline.** PR [#583](https://github.com/sebastienrousseau/static-site-generator/pull/583) closed 15 issues:
  - **#556** profraw hygiene + `repo-hygiene` CI gate; `make coverage` pins `LLVM_PROFILE_FILE` to `target/coverage/`.
  - **#557** Six baseline ADRs in [`docs/adrs/`](docs/adrs/) + `lint-adr` CI gate enforcing the `adr: ADR-NNNN` citation graph.
  - **#558** No-shellout regression lint (`tools/lint-no-shellout.sh`).
  - **#559 + #494** `BENCHMARKS.md` expanded 83 → 245 lines; `tools/seed-bench-corpus.sh` generates deterministic 10/100/1K/10K corpora.
  - **#560** Miri workflow — nightly schedule + `run-miri`-labelled PR trigger, 180-min timeout.
  - **#561** `cargo-vet` supply-chain attestation (Mozilla / Bytecode Alliance / Google trust sets).
  - **#562** SARIF v2.1.0 emitter for `ssg audit` + GitHub Code Scanning upload step.
  - **#563** Dead `openssl` direct dep removed (7 transitive crates dropped from `Cargo.lock`).
  - **#584** `cargo check --no-default-features` dead-code errors fixed; `cargo-hack feature-powerset` CI step added.
  - **#466** Golden-file framework expanded 1 → 11 goldens (10 scaffold + 1 e2e sitemap).
  - **#495** A11y element-presence gate confirmed always-on.
  - **#21** 100% `rustdoc` coverage verified across all six workspace crates.
  - **#22** README v0.0.45 sync + workspace version bump 0.0.44 → 0.0.45.
  - **#23 / #24 / #25** [`docs/features-coverage.md`](docs/features-coverage.md) + `features_matrix_is_exhaustive` test gate.

- **Content-staging shim** ([`src/core/content_stager.rs`](src/core/content_stager.rs), [ADR-0007](docs/adrs/0007-staticdatagen-staging-shim.md)) — works around five `staticdatagen 0.0.9` / `staticweaver 0.0.2` / `metadata-gen 0.0.4` brittleness points so 2,371-file real-world user sites build again. Upstream fixes filed and tracked in [#585](https://github.com/sebastienrousseau/static-site-generator/issues/585).

### Fixed
- **Site-build regression on user sites without `layout:` frontmatter**, missing `main.js`/`sw.js`, no `tags.md`, multi-line YAML scalars, or template references to keys content omits. Detailed root-cause in [ADR-0007](docs/adrs/0007-staticdatagen-staging-shim.md). Validated against `sebastienrousseau/sebastienrousseau.github.io` — 102 root pages, 6.40s build, all 102 a11y-passing.

### Changed
- **CI coverage floors raised** from 95.0 → 95.5 / 96.5 / 95.5 (regions / lines / functions). Coverage gate uses `cargo llvm-cov --lib` to keep the heavy `example_outputs.rs` integration suite in its own job.
- **Miri trigger model**: nightly schedule + `run-miri` label-gated PR runs (decoupled from every push).
- **100-page build budget** in `tests/perf_budgets.rs` raised 500ms → 800ms to absorb the `content_stager` shim. Reverts in v0.0.46.

### Security
- **cargo-vet attestation** (#561) layered over `cargo-deny`.
- **SARIF feed into GitHub Code Scanning** (#562) surfaces audit findings in the Security tab.

### Performance
- **Content-stager pre-pass is Rayon-parallelised** (`copy_tree` and `inject_template_defaults_recursive`).

### Internal
- ~90 new unit tests, ~6 integration tests (`tests/regression_user_site.rs`). Total lib suite: **2,530 / 2,530 passing**.
- Workspace versions bumped 0.0.44 → 0.0.45 across `ssg`, `ssg-core`, `ssg-rpc`, `ssg-rpc-macro`, `ssg-search`, `ssg-wasm`.
- ADR-0007 documents the staging shim explicitly so future maintainers see the upstream debt at a glance.

## [0.0.40] - 2026-06-06

### ⚠ BREAKING CHANGES

- **Public API Error Type Swap**: `PathsBuilder::build` and `Paths::validate` now return `Result<T, SsgError>` instead of `anyhow::Result<T>`. Downstream users matching or handling errors from these endpoints must update their matches to `SsgError`.

### Added
- Native asset minification (JS/CSS) inside the asset pipeline.
- Localized switchers for language alternates using matched slugs.
- First-class Topic taxonomy type with hub/pillar page generation.
- Overlapping-tag similarity indexing for Related Posts.
- Word count and estimated reading time calculations in frontmatter metadata.
- Configurable CDN URL prefixing for markdown images.
- **Structured Error Handling**: Introduced `ssg_core::Error` for the core compilation module and a comprehensive library-wide `SsgError` wrapping all I/O, validation, template rendering, and path safety violations.
- **Contextual I/O Extensions**: Added `PathErrorExt` helper trait to cleanly propagate system directory and file paths alongside underlying I/O errors.

### Changed
- **Encapsulation Pass**: Module declarations in `src/lib.rs` for implementation groups (`core`, `plugins`, `server`) changed from `pub mod` to `pub(crate) mod` (renamed internally as `*_group` to avoid clashing with facade re-exports). Only clean, public facade APIs are exported.
- **Layout Restructuring**: Restructured parent crate codebase, organizing source files into `src/core/`, `src/plugins/`, and `src/server/` directories.

## [0.0.39] - 2026-05-10

### ⚠ BREAKING CHANGES

- **SRI hashes now use real SHA-256.** The previous in-house FNV-1a
  placeholder in `src/assets.rs::sha256_hex` was producing
  `integrity="sha256-..."` attributes that no browser would actually
  validate against. Real SHA-256 + canonical base64 means the hashes
  emitted by `FingerprintPlugin` are now genuine SRI per the W3C
  spec. **User impact:** any site that pre-built and checked-in SRI
  hashes (or fingerprinted filenames) needs a one-time rebuild;
  the short fingerprint suffix and the long SRI both change shape
  to canonical SHA-256-derived values.
- **`RunOptions` demoted to crate-internal.** Was nominally `pub` in
  `src/pipeline.rs`; now `pub(crate)`. The module is `pub(crate)`
  too, so the effective surface is unchanged for external consumers
  that imported via `ssg::pipeline::RunOptions`. No re-export was
  in place, so this should be a no-op for everyone but the curious.
- **Six public enums marked `#[non_exhaustive]`:** `DeployTarget`,
  `FieldType`, `UrlPrefixStrategy`, `ReadabilityFormula`,
  `ChangeKind`, `ProcessError`. Downstream code that pattern-matches
  on these without a wildcard arm needs `_ => …` added. This was
  staged in preparation for `1.0.0-rc.1`.

### Added

- **Build-time CycloneDX SBOM** (`src/sbom.rs`): every build emits a
  `sbom.cdx.json` at the site root + injects a per-page
  `<link rel="sbom" type="application/vnd.cyclonedx+json"
  href="/sbom.cdx.json">` discoverable via the IANA-registered link
  relation. Closes #457.
- **Typed content collection API** (`src/collections.rs`):
  `get_collection::<T>(dir)` / `get_entry::<T>(dir, slug)` for
  serde-typed Markdown loading, mirroring Astro's `getCollection`
  ergonomics with compile-time type safety. Closes #456.
- **WCAG 2.2 build-time checks** (`src/accessibility.rs`):
  `check_target_size` (2.5.8), `check_focus_appearance` (2.4.13)
  plus a `wcag-compliance.json` matrix artifact. README claim
  promoted from "WCAG 2.1 AA" to "WCAG 2.2 AA". Closes #421, #463.
- **JSON-LD schema.org validation** (`src/seo/jsonld.rs`):
  `validate_jsonld` checks required fields per `@type`; new CI
  step walks every example output. Closes #467.
- **CSS `url()` rewriting in `FingerprintPlugin`** (`src/assets.rs`):
  three-pass fingerprint pipeline so CSS-embedded image and font
  references stay valid after content-hashed renames. Closes #468.
- **Content-addressable assets widened to 14 extensions**: CSS, JS,
  MJS plus 7 image formats + 4 font formats. Per-platform
  `Cache-Control: immutable` rules emitted for Netlify, Vercel,
  Cloudflare Pages.
- **Reproducible-build verification job** in `scheduled.yml`:
  double-build with `--locked --offline` + SHA-256 hash diff.
  Closes #424.
- **SECURITY.md** (canonical security policy): disclosure SLA,
  threat model, security defaults, reproducible-build recipe,
  build-provenance verification.
- **WCAG 2.2 + EAA compliance guide**
  (`docs/guide/wcag-compliance.md`): full criterion mapping, EAA
  enforcement context (28 June 2025), member-state implementations,
  before/after migration metrics. Closes #470.
- **API stabilisation audit**
  (`docs/architecture/api-stability-audit.md`): 4-tier inventory of
  ~79 public types + Plugin trait. Closes #427.
- **Perf baseline doc** (`docs/perf/baseline-100p.md`): 18.7 ms
  100-page measurement + 6-subsystem hotspot inventory + profiling
  recipes. Closes #471.
- **Regression contract** (`docs/architecture/regression-contract.md`):
  canonical inventory of every CI gate and its user-facing promise.
- **`tests/chaos.rs`** — 9 chaos-engineering tests (corrupt
  frontmatter, symlink loops, concurrent builds). Closes #423.
- **`tests/element_presence.rs`** — universal HTML invariants gate
  (lang, title, main, charset) on every example page.
- **`tests/perf_budgets.rs`** — hard wall-clock budgets: 10-page <
  100 ms, 100-page < 500 ms, 500-page < 2 s.
- **`tests/jsonld_validation.rs`** — schema.org required-field
  validation across every example output, wired into CI.
- **`tests/golden_files.rs`** — golden-file regression framework
  with normalisation (timestamps, fingerprints, SRI) and
  `UPDATE_GOLDEN=1` workflow. Closes #466 (framework phase).
- **`tests/docs_accuracy.rs`** — verifies README claims match
  source-of-truth files (test count, WCAG version, coverage floors,
  MSRV, version sync).
- **`tests/doc_links.rs`** — every internal Markdown link in
  README/CHANGELOG/SECURITY/docs/* resolves to an existing file.
- **OpenTelemetry feature-gate scaffolding** (`src/otel.rs`):
  `otel` Cargo feature, `--trace` CLI flag, one demo span around
  `execute_build_pipeline`. Closes #422 phase A.
- **Multi-OS `example_outputs` portability job** in `scheduled.yml`:
  weekly macOS + Windows runs. Closes #473.
- **Per-criterion SEO comparison matrices** in
  `docs/compare/ssg-vs-{hugo,zola,astro}.md` covering 17–18 SEO
  criteria each. Closes #461.

### Changed

- **README test-count claim refreshed** from 1,640 → 1,685+ lib
  tests, plus mention of new `collections` + `sbom` + `otel`
  modules.
- **Cargo.toml keywords**: `["cli","generator","ssg","static-site",
  "wasm"]` → `["rust","markdown","jamstack","ssg","wasm"]` for
  better crates.io discoverability. Closes #428.
- **`FingerprintPlugin` rename suffix** is still the first 8 hex
  chars but now derived from real SHA-256 instead of FNV-1a.

### Fixed

- **Security:** `sha256_hex` was FNV-1a, not SHA-256.
  `integrity="sha256-..."` attributes were silently invalid. Now
  real SHA-256 via the `sha2` crate + canonical base64 via the
  `base64` crate. Verified against NIST test vectors.
- **CSS `url()` references broke** after image/font fingerprinting
  because the CSS file content wasn't patched before the CSS file
  was itself hashed. Three-pass pipeline fixes the ordering.
- **CSS parser false positives in WCAG checks** —
  `/* width: 10px */` no longer triggers 2.5.8; `@media print {
  button { width: 10px } }` no longer fires unconditionally;
  multiple `<style>` blocks now all scanned.
- **JSON-LD validator over-strictness** — `WebPage` requires only
  `name` (per Google rich-results docs), not `name + url +
  inLanguage`. The latter two are Recommended only.
- **`tests/chaos.rs::read_only_output_directory_returns_clean_error`**
  now uses a Drop guard for permission restoration so panics don't
  leave stale 0o555 tempdirs on CI disk.
- **Reproducible-build job** runs `cargo fetch --locked` then both
  build invocations as `--locked --offline` to eliminate transient
  registry state.

### Dependencies

- Cargo (`Cargo.toml` + `Cargo.lock`):
  - `rustls-webpki` 0.103.12 → 0.103.13 (resolves Dependabot #487)
  - `clap` 4.6.0 → 4.6.1 (#490)
  - `openssl` 0.10.77 → **0.10.79** (#490 + #492)
  - `uuid` 1.22.0 → 1.23.1 (#490)
  - **NEW** `sha2` = `"0.10"` (required for real SHA-256 SRI)
  - **NEW** `base64` = `"0.22"` (canonical base64 SRI encoding)
- GitHub Actions:
  - `actions/upload-artifact` v4 → **v7.0.1** (#489), 4 workflows
  - `crazy-max/ghaction-import-gpg` v6 → **v7.0.0** (#488),
    release.yml gpg-sign job
- npm (`tests/visual/`):
  - `@axe-core/playwright` 4.10.0 → **4.11.3** (#491)

### Security

- **Resolves 8 Dependabot security advisories on the default
  branch**: 6× openssl (alerts #29–#33, #35, #36; all `< 0.10.79`),
  1× rustls-webpki (#34; `< 0.103.13`). Severity mix: 6 high + 1
  moderate + 1 low.
- **Real SHA-256 for SRI** (above).
- **Reproducible-build verification** in CI.
- **`SECURITY.md`** canonical security policy.

## [0.0.38] - 2026-04-20

### Added
- **Agentic LLM pipeline**: `--ai-fix` CLI flag triggers audit, diagnose, fix, verify, and report cycle with configurable max refinement attempts and JSON output
- **Multilingual readability**: Kandel-Moles (FR), Wiener Sachtextformel (DE), Gulpease (IT), LIX (SV/NO/DA), Fernandez Huerta (ES) with BCP 47 language detection from frontmatter
- **OG image generation**: auto-generated SVG social cards from page title and site name, injected via `og:image` meta tag, zero new dependencies
- **Scalability benchmarks**: Criterion benchmarks at 100, 1K, and 10K page tiers with CI job on release tags
- **axe-core CI**: `@axe-core/playwright` integration for WCAG 2.1 AA audit with JSON report artifacts
- **CSP whitepaper**: `docs/whitepaper/csp-without-compromise.md` documenting build-time inline extraction and SRI hashing
- **237 new unit tests**: coverage raised from 94.24% to 95.06% regions (1,640 total)

### Changed
- CI coverage regions floor raised from 94% to 95%
- Version bumped from 0.0.37 to 0.0.38
- README rewritten with updated metrics, feature matrix, and architecture diagram

### Fixed
- `package-lock.json` synced with `@axe-core/playwright` dependency
- axe-core a11y audit restricted to desktop project (Chromium only) to avoid missing WebKit binary in CI

### Dependencies
- `actions/checkout` v4 to v6.0.2
- `actions/download-artifact` v4 to v8.0.1
- `actions/attest-build-provenance` v2 to v4.1.0
- `actions/upload-pages-artifact` v3 to v5.0.0
- `actions/deploy-pages` v4 to v5.0.0
- `actions/cache` v4 to v5.0.5
- `actions/setup-node` v4 to v6.4.0
- `docker/setup-buildx-action` v3 to v4.0.0
- `docker/build-push-action` v6 to v7.1.0
- `docker/login-action` v3 to v4.1.0

## [0.0.37] - 2026-04-19

### Added
- **WebAssembly**: `ssg-core` and `ssg-wasm` crates for browser/edge compilation
- **Interactive islands**: `<ssg-island>` Web Components with lazy hydration
- **Streaming compilation**: batch-based compiler for 100K+ page sites
- **Local LLM pipeline**: auto-generate alt text, meta descriptions, readability auditing
- **Dependency graph**: `DepGraph` for incremental rebuild tracking
- **Browser error overlay**: build errors rendered in-browser via WebSocket
- **CSS hot reload**: stylesheet changes without full page reload
- **Property-based testing**: proptest for frontmatter, markdown, shortcode fuzzing
- **WASM integration tests**: 12 wasm-bindgen-test cases in headless Chrome
- **llms.txt spec compliance**: section index, language field, disallow patterns
- **Performance gates**: 8 timed CI assertions (compilation, search, cache, streaming)
- **Enterprise regression suite**: 27 tests for cache resilience, licence, i18n, pipeline

### Changed
- Template engine: Tera → MiniJinja (10× smaller binary)
- Coverage floors raised to 95% (regions, lines, functions)
- All examples emit build timing and use unique ports (3001–3007)
- Plugin table descriptions shortened for readability audit compliance
- 100% API coverage: all 36 modules demonstrated in examples

### Fixed
- SPDX headers on all 97 source files (100% compliance)
- Duplicate "All rights reserved" in 5 bench/example files
- Duplicate server banners in 6 examples
- `run_fused_transforms` missing from 3 examples
- Readability audit threshold raised to grade 17 for technical docs

### Security
- CSP/SRI hardening: extract inline styles/scripts to external files
- GitHub Actions pinned to commit SHAs
- Dependabot configuration added
- `unsafe-inline` eliminated from Content-Security-Policy

## [0.0.36] - 2026-04-13

### Added

- **Post-processing pipeline** — new `postprocess` module with 5 plugins that
  repair `staticdatagen` output: `SitemapFixPlugin` (duplicate XML declarations,
  double-slash URLs, per-page lastmod), `NewsSitemapFixPlugin` (placeholder
  replacement, `<news:keywords>`), `RssAggregatePlugin` (feed aggregation with
  enclosures, categories, language, lastBuildDate, copyright),
  `ManifestFixPlugin` (word-boundary-safe truncation), `HtmlFixPlugin` (JSON-LD
  date conversion, HTTPS context, broken img repair).
- **Content schema validation** — new `content` module with `ContentSchema`,
  `FieldDef`, TOML schema loader, compile-time frontmatter validation, and
  `--validate` CLI flag for schema-only checks. 62 tests.
- **Responsive image pipeline** — `ImageOptimizationPlugin` now emits
  `<picture>` elements with AVIF/WebP `<source>` tags, responsive `srcset` at
  320/640/1024/1440, `loading="lazy" decoding="async"` by default,
  `fetchpriority="high"` → `loading="eager"`, width/height from source metadata.
- **i18n routing** — new `i18n` module with `I18nPlugin`, automatic hreflang
  injection for multi-locale pages, `x-default` support, per-locale sitemaps
  with `xhtml:link` alternates, `generate_lang_switcher_html()` helper.
- **Parallel plugin pipeline** — `MinifyPlugin` and `SearchIndex::build`
  converted to `par_iter()`. New `--jobs N` CLI flag for Rayon thread count.
- **Benchmark suite** — Criterion benchmarks for 10–10K synthetic pages,
  `benchmarks/README.md` with cross-SSG comparison instructions, `BENCHMARKS.md`
  template.
- **Accessibility CI** — `.github/workflows/a11y.yml` with pa11y WCAG 2.1 AA
  scanning, `make a11y` target.
- **SBOM + CI hardening** — `.github/workflows/sbom.yml` with CycloneDX
  generation and Sigstore build provenance attestation.
- **Multi-platform release workflow** — `.github/workflows/release.yml` builds 5
  targets on `v*` tags: Linux glibc, Linux musl (static), macOS ARM64, macOS
  Intel, Windows. SHA256 checksums, GitHub Release, crates.io publish.
- **Install script** — `scripts/install.sh` auto-detects OS/arch, downloads
  correct binary, verifies checksum, installs to `~/.local/bin`.
- **Homebrew formula** — `packaging/homebrew/ssg.rb` for `brew install`.
- **SPDX license headers** — added to all 60+ source files.
- **Deploy security headers** — `Content-Security-Policy` and
  `Strict-Transport-Security` (HSTS) added to Netlify/Vercel/Cloudflare configs.
- **Enhanced SEO plugin** — full OG suite (og:url, og:image, og:image:width/
  height, og:locale), full Twitter Card suite (summary_large_image for
  articles), JSON-LD Article/WebPage with datePublished, dateModified, author
  as Person entity, image as ImageObject, inLanguage.
- **Canonical URL replacement** — `CanonicalPlugin` now replaces template
  placeholders with correct `base_url + path` instead of skipping existing tags.

### Changed

- **Renamed** all references from "Shokunin" to "Static Site Generator".
- **Dependencies reduced** from 25 → 21 direct deps: `once_cell` → `OnceLock`,
  `dtt` → `chrono`, `colored` → ANSI codes, `uuid` moved to dev-deps.
- **Tokio features trimmed** from `["full"]` to `["fs", "rt-multi-thread",
  "macros", "time"]` — removes 8 unused subsystems.
- **MSRV** synced between `build.rs` (was 1.74) and `Cargo.toml` (1.88).
- **Dev server** only starts when `--serve` is explicitly requested (was
  blocking unconditionally after every build, breaking CI).
- **Accessibility checker** recognises `alt=""` with `role="presentation"` and
  bare `alt` attribute (minified) as valid decorative images.
- **Template contrast** — WCAG AAA colours: `--vp-t3` → `#545458`/`#a1a1aa`,
  `--vp-br` → `#1a3a8a`, links underlined for colour-blind distinguishability.
- **Musl static binary** — added to CI portability matrix (weekly + release).
- **`deny.toml`** — removed stale `CC0-1.0` and `Unicode-DFS-2016` entries.

### Fixed

- **Sitemap** — duplicate XML declarations, double-slash URLs, stale lastmod.
- **News sitemap** — "Unnamed Publication" / "Untitled Article" placeholders
  replaced with real frontmatter data.
- **RSS feed** — root feed now aggregates all article items (was single
  self-referencing entry).
- **OG/Twitter tags** — empty on non-index pages due to comment-marker
  detection instead of actual `<meta>` tag checks.
- **JSON-LD dates** — RFC 2822 → ISO 8601 conversion.
- **JSON-LD @context** — `http://schema.org/` → `https://schema.org`.
- **Manifest.json** — description truncated mid-word at 120 chars.
- **Markdown .class= syntax** — `<p src=` injected into `<img>` tags.
- **Lighthouse scores** — A11y 91→100, SEO 85→100 on generated output.
- **CI** — a11y workflow cancellation, Chrome sandbox flags, mold linker
  config incompatibility with CI runners.

### Added (continued — 2026-04-16)

- **8 polished examples with distinct brand identities** — every example now
  ships as a real-feeling clone-and-edit template:
  - `basic` — *Aria Studio* (independent design studio, single-page layout)
  - `blog` — *Threshold* (accessibility journal, 3 substantive posts on EAA /
    WCAG / typography, working tags + posts aggregation)
  - `quickstart` — *Heron Coffee* (small London roastery + 3 journal posts
    demonstrating the full 16-plugin pipeline against realistic content)
  - `docs` — *Polaris* (generic developer-tool docs template — Welcome /
    Getting Started / Configuration / API reference / Release notes / Support)
  - `landing` — *Meridian Systems* (compliance-grade software for regulated
    industries; rich body copy, real client list, zero-JS verification)
  - `portfolio` — *Maya Okafor* (independent UX researcher, 3 detailed case
    studies: Field Notes Collective, Linden Editions, Polaris Maps)
  - `multilingual` — 6 priority locales (EN/FR/ES/DE/JA/AR) rewritten with a
    real i18n product narrative ("Write once, ship in 28 languages")
  - `plugins` — annotated lifecycle walkthrough, own dirs, root templates
- **Comprehensive regression test suite** — `+140 tests` across 3 new files:
  - `tests/example_outputs.rs` (19 tests) — runs every example end-to-end +
    11 negative validator tests proving the validators catch what they claim
  - `tests/plugin_contracts.rs` (8 tests) — lifecycle ordering, plugin
    idempotency (HtmlFix + ManifestFix), HtmlFix→Minify ordering, SVG data-URL
    preservation
  - `tests/schema_validation.rs` (8 tests) — `validate_with_schema` contract:
    valid pages pass, missing fields fail, unknown enum values fail, missing
    schema file tolerated, multiple errors aggregated, legacy `validate_only`
    path still works
- **Coverage gate** — `.github/workflows/ci.yml` enforces region ≥95.0%, line
  ≥97.0%, function ≥95.0%. Lib coverage measured at 95.22% / 97.46% / 95.79%.
- **`validate_with_schema(content_dir, schema_path)` API** — schema can now
  live outside `content_dir`, avoiding `staticdatagen::compile`'s read-every-
  file behaviour that previously blocked the docs example schema validation.
- **Browser-compat fixes in `HtmlFixPlugin`** — removes empty `<link
  rel="preload" href>` tags; injects modern `mobile-web-app-capable` meta
  alongside the deprecated apple variant.
- **`ManifestFixPlugin` empty-icon filtering** — drops icon entries whose `src`
  is empty (Chrome would otherwise log a manifest icon download error).
- **Mobile-menu desktop fix** — added `.mobile-menu{display:none}` to base CSS
  in all 6 shared templates; previously the rule lived only inside
  `@media(max-width:768px)` so the menu rendered as a duplicate nav on desktop.
- **Mobile nav alignment fix** — added `.nav-controls{margin-left:auto}` to the
  `@media(max-width:768px)` block so theme switch + hamburger sit flush right
  when `.nav-search` is hidden.

### Changed (continued — 2026-04-16)

- **Folder hierarchy consolidated**:
  - `Formula/` + `pkg/{arch,deb,scoop,winget,PUBLISHING.md}` →
    `packaging/{homebrew,arch,deb,scoop,winget,PUBLISHING.md}`
  - `template/tera` → `templates/tera` (singular `template/` removed)
  - `benchmarks/README.md` → `benches/README.md` (benchmarks/ removed)
  - Empty root `content/`, `templates/`, `public/`, `build/` removed
- **CI workflows consolidated 7 → 3**:
  - `ci.yml` (PR gate; lint → test ×3 OS · examples · coverage · audit
    in parallel; <5 min wall time target)
  - `scheduled.yml` (weekly + tag; portability matrix, musl static, pa11y,
    SBOM)
  - `release.yml` (tag; build × 5 platforms + GHCR + GPG + AUR + crates.io)
- **Release pipeline expanded** — adds `.rpm` (cargo-generate-rpm), macOS
  `.pkg` (pkgbuild), Windows `.msi` (cargo-wix), multi-arch GHCR container
  (`ghcr.io/sebastienrousseau/static-site-generator:vX.Y.Z` + `:latest`),
  AUR push (gated on `AUR_SSH_KEY` secret), GPG detached signatures (gated
  on `GPG_PRIVATE_KEY` secret).
- **Cache files relocated** — `.ssg-cache.json` + `.ssg-plugins-cache.json`
  moved from repo root → `target/.ssg-cache/{ssg,plugins}.json`.
- **Clippy re-enabled** — `cargo clippy --lib -- -D warnings` is now CI-gated;
  tests/examples allow `unwrap_used` + `expect_used` via documented
  workspace-wide `[lints.clippy]` allowance list. Lib has 0 warnings.
- **`Dockerfile` added** — two-stage build (cargo + debian-slim runtime) for
  the GHCR multi-arch image.
- **`Cargo.toml` packaging metadata** — `[package.metadata.generate-rpm]` for
  RPM asset list, `[package.metadata.wix]` for MSI installer config.

### Fixed (continued — 2026-04-16)

- **A11y false positive** — `check_img_alt` previously truncated `<img>` tags
  at the first `>` character inside an SVG `data:` URL in `src=`, causing
  spurious `<img> missing alt text: (no src)` reports. New quote-aware
  `find_tag_end()` respects attribute quoting.
- **Schema validation silently passing** — docs example reported "all pages
  valid" without actually validating because schema was outside `content_dir`
  (where the legacy `validate_only` looked). New API + relocated schema fix it.
- **Nav clutter on single-page templates** — `basic` example trims Posts/Tags
  nav items + footer Resources column + hero CTAs via `:has()` CSS injection.
- **Stray repo artifacts removed** — `*.log`, `fixes.txt`, `.DS_Store`,
  `public.build-tmp/` purged from working tree (already gitignored).

## [0.0.35] - 2026-04-11

### Added

- **Localized search widget** — `SearchLabels` struct with 28 bundled locale
  translations; `LocalizedSearchPlugin` injects per-locale search modal
  strings (button, placeholder, footer hints, no-results message).
- **GFM Markdown extensions** — new `MarkdownExtPlugin` adds tables,
  ~~strikethrough~~, and task-list checkboxes on top of staticdatagen's
  renderer.
- **WCAG AAA green palette** — brand colours switched from blue to green
  (matching the Kaishi logo) with solid-hex text tokens: 7.05:1–16.5:1
  contrast ratios in both light and dark modes.
- **28-locale multilingual example** — full content + template trees for
  en, fr, ar, bn, cs, de, es, ha, he, hi, id, it, ja, ko, nl, pl, pt,
  ro, ru, sv, th, tl, tr, uk, vi, yo, zh, zh-tw.
- **`cmd::resolve_host()` / `resolve_port()`** — `$SSG_HOST` / `$SSG_PORT`
  env-var overrides for WSL2, Codespaces, and dev-container users.
- **`make init`** — one-command bootstrap (detects platform, installs
  rustfmt + clippy + cargo-deny, wires up git hooks, runs first build).
- **`make hooks`** — installs `.githooks/pre-commit` signed-commit guard.
- **`make clean`** — removes build artefacts and stray log files.
- **`.devcontainer/devcontainer.json`** — one-click VS Code / Codespaces
  environment.
- **`.githooks/pre-commit`** — cross-platform (bash) hook that refuses
  unsigned commits.
- **`.github/workflows/portability.yml`** — cost-optimised 3-OS CI matrix
  (fast Linux gate per push; full matrix weekly + on release tags).
- **`<h1>` on all pages** — content templates now emit
  `<h1 class="page-title">{{title}}</h1>`.
- **`<meta name="mobile-web-app-capable">`** added alongside the deprecated
  apple-prefixed variant.
- **`prefers-reduced-motion`** global CSS override.
- **44 px tap targets** for `.lang-btn` and `.menu-toggle`; `.theme-switch`
  uses a transparent `::after` hit-area extension.
- **`docs/README.md`** — explains the gitignored `docs/` build-target
  directory.
- **Criterion benchmark suite** — `benches/bench_site_generation.rs`
  measures end-to-end compile throughput at 10, 50, and 100 pages.
  `make bench` target added to Makefile.
- **`CHANGELOG.md`** — Keep a Changelog format with full release notes.
- **README Table of Contents** — 11-item jump index at the top.
- **Code of Conduct** linked from README.
- **`make doc`** — generates API documentation with `-D warnings` and
  opens in browser.
- **Mermaid plugin lifecycle diagram** in CONTRIBUTING.md.

### Changed

- **Rayon-parallelised plugin pipeline** — `SearchPlugin`,
  `SeoPlugin`, `CanonicalPlugin`, and `JsonLdPlugin` now use
  `par_iter().try_for_each()` instead of sequential `for` loops for
  HTML file injection. `AtomicUsize` replaces mutable counters.
- **`warp` dependency removed** — `handle_server()` now uses
  `http_handle::Server` via `tokio::task::spawn_blocking`. Cargo.lock
  shrank by 292 lines. Direct deps: 25 → 24.

- **CI pipelines pinned to SHA** — all shared workflow refs and GitHub
  Actions pinned to immutable commit SHAs instead of mutable `@main` /
  `@v4` / `@stable` tags. Eliminates supply-chain risk.
- **`.editorconfig`** expanded with `[*.{json,toml}]` and `[*.html]`
  rules at indent 2.
- **MSRV** bumped from 1.74.0 to **1.88.0** (deps had silently escalated).
- **README** rewritten: test count (342→741), CLI reference (10→14 flags),
  cross-platform prerequisites table, library example uses `ssg::run()`,
  CI claim corrected (stable only, not nightly), module list expanded to
  all 30 src modules.
- **CONTRIBUTING.md** architecture tree synced to all 30 modules; signed-tag
  enforcement; per-platform setup instructions.
- **`Cargo.toml`** `documentation` URL → `https://docs.rs/ssg` (was dead
  `static-site-generator.one`); `homepage` → GitHub repository URL.
- **`ssg --help`** no longer leaks `[INFO]` log lines (logger init moved
  below `Cli::build().get_matches()`).
- **Portability CI** split into fast gate (1 job/push) + full matrix
  (weekly/tags) — ~6× cost reduction.
- **`src/process.rs`** gained `//!` module-level documentation.
- **`src/lib.rs`** `ServeTransport` doc fixed (broken `[NoopTransport]`
  intra-doc link).
- Hardcoded `/tmp/` paths in tests replaced with `std::env::temp_dir()`.

### Fixed

- **RTL dropdown positioning** — `right:0` → `inset-inline-end:0` so the
  language menu anchors correctly on Hebrew / Arabic pages.
- **English root link** (`/`) was being rewritten to `/<locale>/` by the
  inline JS — added `h !== '/'` guard.
- **Cross-locale navigation** — language switcher links now preserve the
  current sub-path (e.g. `/en/tags/` → `/fr/tags/`).
- **Banner URLs** corrected: `stock/images/banners/` → `stocks/images/`.
- **Logo URLs** migrated: `kaishi/images/logos/` → `kaishi/v1/logos/`.
- **Theme switch button** visual restored after tap-target rule blew up
  its 40×22 pill to 44×44 square.
- **Search widget dark mode** — greys were globally replaced with light-mode
  values, making text invisible; now context-aware (light: `#595960`,
  dark: `#cccccf`).
- **PR template** — added signed-commit checklist item.
- **Search locale isolation** — widget now fetches
  `/<lang>/search-index.json` per locale instead of always loading the
  English root index. Result URLs are prefixed with the locale path.
- **Search hero content indexed** — `extract_text()` no longer strips
  `<header>` blocks, so hero taglines and subtitles are searchable.
- **Search JS scoping crash** — `lm` and `lp` locale variables hoisted
  from `load()` to the outer IIFE scope; eliminates `ReferenceError`
  that silently crashed the search function on every keystroke.
- **`cargo deny check licenses`** — added Zlib to allow list (used by
  `foldhash`); removed stale RUSTSEC-2025-0068 ignore.
- **RUSTSEC-2026-0097** (rand 0.8.5 unsound) acknowledged in both
  `.cargo/audit.toml` and `deny.toml` — transitive via `phf_generator`,
  SSG never calls `rand::rng()` directly.
- **Unused import** in `quickstart_example.rs` removed.

### Removed

- **Inline JS nav sort** — was comparing translated `textContent` against
  an English `order` array, scrambling the menu. Source-HTML order now
  persists directly.
- **Language selector page** at `/` — root now serves English content
  directly; language switcher is embedded in the nav bar.

## [0.0.34] - 2025-04-04

See [release notes](https://github.com/sebastienrousseau/static-site-generator/releases/tag/v0.0.34).

## [0.0.33] - 2025-02-04

See [release notes](https://github.com/sebastienrousseau/static-site-generator/releases/tag/v0.0.33).

[0.0.37]: https://github.com/sebastienrousseau/static-site-generator/compare/v0.0.36...v0.0.37
[0.0.36]: https://github.com/sebastienrousseau/static-site-generator/compare/v0.0.35...v0.0.36
[0.0.35]: https://github.com/sebastienrousseau/static-site-generator/compare/v0.0.34...v0.0.35
[0.0.34]: https://github.com/sebastienrousseau/static-site-generator/compare/v0.0.33...v0.0.34
[0.0.33]: https://github.com/sebastienrousseau/static-site-generator/releases/tag/v0.0.33
