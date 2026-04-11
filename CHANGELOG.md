# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  `shokunin.one`); `homepage` → GitHub repository URL.
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

See [release notes](https://github.com/sebastienrousseau/shokunin/releases/tag/v0.0.34).

## [0.0.33] - 2025-02-04

See [release notes](https://github.com/sebastienrousseau/shokunin/releases/tag/v0.0.33).

[0.0.35]: https://github.com/sebastienrousseau/shokunin/compare/v0.0.34...v0.0.35
[0.0.34]: https://github.com/sebastienrousseau/shokunin/compare/v0.0.33...v0.0.34
[0.0.33]: https://github.com/sebastienrousseau/shokunin/releases/tag/v0.0.33
