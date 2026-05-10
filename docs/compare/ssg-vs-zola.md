<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SSG vs Zola

Both are Rust SSGs. They share the language but differ in architecture.

## At a Glance

| | SSG | Zola |
|---|---|---|
| Language | Rust | Rust |
| GitHub Stars | ~200 | 16.8K |
| Architecture | Plugin-based (25+ plugins) | Monolithic |
| Template Engine | MiniJinja | Tera |
| Build Speed (50 pages) | ~40ms | 36ms |
| `#![forbid(unsafe_code)]` | Yes | Yes |

## Where SSG Leads

- **Plugin system**: 25+ composable plugins vs Zola's monolithic design. Add, remove, or reorder build steps
- **Security**: CSP/SRI extraction, security headers for 4 deploy targets. Zola has none
- **Accessibility**: Build-time WCAG 2.1 AA validation. Zola has no a11y checks
- **AI**: Local LLM integration for content quality. Zola has no AI features
- **WebAssembly**: ssg-core + ssg-wasm for browser/edge. Zola cannot compile to WASM
- **Testing**: 95% coverage floors, proptest, fault injection. Zola has no coverage enforcement
- **Streaming**: 100K+ page support with memory budgets. Zola may OOM on very large sites

## Where Zola Leads

- **Simplicity**: Single binary, zero configuration needed for basic sites
- **Speed**: 36ms for 50 pages — marginally faster at small scale
- **Sass compilation**: Built-in Sass/SCSS support. SSG requires external tooling
- **Community**: 16.8K stars, active ecosystem, good documentation
- **Maturity**: Stable API, production-proven for years

## Choose SSG When

- You need extensibility via plugins
- Security hardening and a11y validation are requirements
- You want AI-powered content quality gates
- Edge/WASM compilation is on your roadmap

## Choose Zola When

- You want the simplest possible Rust SSG
- Built-in Sass compilation matters
- Community size and theme availability are priorities
- Your site is under 10K pages and doesn't need streaming

## SEO Capability Matrix

| SEO criterion | SSG | Zola |
|---|---|---|
| `<html lang>` declared on every page | ✅ Built-in (i18n plugin) | ⚠️ Theme-dependent |
| Non-empty `<title>` per page | ✅ Validated in CI | ⚠️ Theme-dependent |
| `<meta name="description">` non-empty | ✅ Validated | ⚠️ Theme-dependent |
| Canonical URL (`<link rel="canonical">`) | ✅ `CanonicalPlugin` | ⚠️ Theme-dependent |
| Open Graph + Twitter Card chains | ✅ Auto-generated | ⚠️ Theme-dependent |
| JSON-LD structured data + schema.org validation | ✅ Auto + CI gate | ❌ Manual |
| News sitemap | ✅ `NewsSitemapFixPlugin` | ❌ Manual |
| RSS + Atom feeds | ✅ Both, with categories + enclosures | ✅ Atom only by default |
| `robots.txt` per environment | ✅ `RobotsPlugin` | ✅ Built-in |
| Hreflang for multi-locale sites | ✅ `I18nPlugin` + per-locale sitemap | ✅ Built-in |
| Image `<alt>` build-time validation | ✅ `AccessibilityPlugin` | ❌ Not checked |
| `lastmod` per URL | ✅ Auto from frontmatter | ✅ Auto |
| **SBOM discovery** (`<link rel="sbom">`) | ✅ `SbomPlugin` (CycloneDX) | ❌ Not supported |
| Web Vitals + a11y CI gates | ✅ axe-core + perf budgets | ⚠️ External tooling |
| Content-addressable assets + cache headers | ✅ `FingerprintPlugin` + per-platform | ⚠️ Manual |
| CSP without `unsafe-inline` | ✅ Auto-extracted + SRI | ❌ Not built-in |

**Bottom line:** Zola wins on simplicity for small sites without
SEO requirements. SSG wins everywhere SEO conformance is a hard
requirement and you need it as a *gate* rather than a *guideline*.
