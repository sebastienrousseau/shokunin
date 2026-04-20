<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
