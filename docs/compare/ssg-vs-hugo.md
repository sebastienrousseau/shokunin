<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SSG vs Hugo

An honest feature comparison. Both are excellent tools for different needs.

## At a Glance

| | SSG | Hugo |
|---|---|---|
| Language | Rust | Go |
| GitHub Stars | ~200 | 87K+ |
| Template Engine | MiniJinja | Go templates |
| Plugin System | Trait-based (25+ plugins) | Template hooks |
| Build Speed (50 pages) | ~40ms | 178ms |
| Streaming (100K+ pages) | 512 MB budget | Million Pages |

## Where SSG Leads

- **Security by default**: CSP extraction, SRI hashes, `unsafe-inline` elimination — all automatic
- **Accessibility validation**: WCAG 2.1 AA checks run on every build. Hugo has no built-in a11y
- **AI integration**: Local LLM for alt text, meta descriptions, readability auditing. No cloud API keys
- **WebAssembly**: Compiles to WASM for browser/edge environments
- **Test infrastructure**: 95% coverage floors, property-based testing, fault injection, performance gates

## Where Hugo Leads

- **Community**: 87K+ stars, thousands of themes, massive documentation
- **Raw speed**: Hugo is the fastest SSG at any scale, especially 10K+ pages
- **Content adapters**: Pull content from remote APIs at build time
- **Maturity**: Stable, battle-tested at enterprise scale for years
- **Ecosystem**: Hugo Modules for sharing content, templates, and configuration

## Choose SSG When

- Security and accessibility are hard requirements (government, finance, healthcare)
- You need local AI content augmentation without cloud dependencies
- You want build-time WCAG validation as a CI gate
- Edge/WASM compilation is on your roadmap

## Choose Hugo When

- You need the fastest possible builds at massive scale
- Theme ecosystem and community support matter most
- You're building documentation sites with content from remote APIs
- Your team already knows Go templates

## SEO Capability Matrix

The criteria below are the ones search-engine teams actually check
when ranking a site. SSG ships every row as a default; Hugo ships
some via `params`, some via theme conventions, some not at all.

| SEO criterion | SSG | Hugo |
|---|---|---|
| `<html lang>` declared on every page | ✅ Built-in (i18n plugin) | ⚠️ Theme-dependent |
| Non-empty `<title>` per page | ✅ Validated by `tests/element_presence.rs` | ⚠️ Theme-dependent |
| `<meta name="description">` non-empty | ✅ Validated | ⚠️ Theme-dependent |
| Canonical URL (`<link rel="canonical">`) | ✅ `CanonicalPlugin` (built-in) | ✅ via `_default/baseof.html` |
| Open Graph chain (`og:title`/`og:description`/`og:type`/`og:image`) | ✅ Auto-generated; OG image SVG built from page metadata | ⚠️ Theme-dependent |
| Twitter Card meta | ✅ Auto-generated `summary_large_image` for articles | ⚠️ Theme-dependent |
| JSON-LD structured data | ✅ `Article`/`WebPage`/`BreadcrumbList` auto-generated; **schema.org required-field validation in CI** (`tests/jsonld_validation.rs`) | ⚠️ Manual (theme/templates) |
| Sitemap (`sitemap.xml` + per-locale) | ✅ Built-in | ✅ Built-in |
| News sitemap (`<news:keywords>`) | ✅ `NewsSitemapFixPlugin` | ❌ Manual |
| RSS + Atom feeds | ✅ Both auto-generated, with categories + enclosures | ✅ RSS only by default |
| `robots.txt` | ✅ `RobotsPlugin` (per-environment) | ✅ Built-in |
| Hreflang for multi-locale sites | ✅ `I18nPlugin` injects + per-locale sitemap with `xhtml:link` | ⚠️ Theme-dependent |
| Image `<alt>` validation | ✅ `AccessibilityPlugin` flags missing alt at build | ❌ Not checked |
| `lastmod` per URL in sitemap | ✅ `SitemapFixPlugin` derives from frontmatter date | ✅ Auto |
| **SBOM discovery** (`<link rel="sbom">`) | ✅ `SbomPlugin` (CycloneDX 1.5) | ❌ Not supported |
| Web Vitals optimisation gates | ✅ axe-core CI + perf budgets (`tests/perf_budgets.rs`) | ⚠️ External tooling |
| Content-addressable assets (Cache-Control: immutable) | ✅ `FingerprintPlugin` + per-platform cache headers | ⚠️ Hugo Pipes (manual config) |
| CSP without `unsafe-inline` | ✅ `CspPlugin` extracts inline → external + SRI | ❌ Not built-in |

**Bottom line for SEO teams:** SSG bundles every Lighthouse-SEO and
Lighthouse-Accessibility check as a **build-time gate**. Hugo
delegates most of this to the chosen theme, which means your SEO
posture varies by template choice.

## Page-Weight Comparison (50-page blog)

Measured against the example in `examples/blog`:

| Metric | SSG `examples/blog` | Hugo `gohugoio/hugoBasicExample` |
|---|---|---|
| HTML page (gzipped) | ~6 KB | ~9 KB |
| CSS (after CSP extraction + minify + SRI) | ~12 KB (one file, immutable) | ~14 KB (theme-dependent) |
| Total first-contentful-paint payload | ~18 KB | ~23 KB |
| Lighthouse SEO | 100 | 92 (default theme) |
| Lighthouse Accessibility | 100 | 89 (default theme) |
