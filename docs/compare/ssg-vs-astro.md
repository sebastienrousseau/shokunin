<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SSG vs Astro

Different tools for different architectures. SSG is a Rust binary;
Astro is a JavaScript framework acquired by Cloudflare in January 2026.

## At a Glance

| | SSG | Astro 6 |
|---|---|---|
| Language | Rust | JavaScript/TypeScript |
| GitHub Stars | ~200 | 50K+ |
| Owner | Independent (open source) | Cloudflare |
| Architecture | Plugin pipeline | Islands architecture |
| Build Speed (50 pages) | ~40ms | ~2s |
| Runtime | Binary | Node.js / Deno |

## Where SSG Leads

- **Build speed**: 50× faster builds (Rust vs JavaScript)
- **Zero runtime**: No Node.js required. Single binary
- **Accessibility**: Built-in WCAG 2.1 AA validation. Astro requires community plugins
- **AI**: Local LLM integration with readability auditing. Astro has Review Loop (annotation only)
- **Memory safety**: `#![forbid(unsafe_code)]`, no GC pauses, no memory leaks
- **Test rigour**: 95% coverage floors, proptest, fault injection. Unmatched in SSG space
- **Supply chain**: `cargo deny` with licence validation. Smaller dependency tree than npm

## Where Astro Leads

- **Community**: 50K+ stars, Cloudflare backing, massive ecosystem
- **Islands architecture**: Pioneered partial hydration — the most mature islands implementation
- **Framework agnostic**: Use React, Vue, Svelte, or Solid in the same project
- **Live content collections**: Real-time data without rebuilds
- **Edge native**: First-class Cloudflare Workers integration (same company)
- **CSP support**: Built-in since Astro 6 (matches SSG's capability)
- **Developer experience**: Vite-powered HMR, error overlay, instant feedback

## Choose SSG When

- Build speed and binary size matter (CI pipelines, embedded systems)
- You need build-time WCAG validation for regulatory compliance (EAA, ADA, Section 508)
- Local AI content augmentation without cloud APIs is a requirement
- Your deployment target doesn't include Node.js
- WebAssembly compilation for edge/browser is on your roadmap

## Choose Astro When

- You need React/Vue/Svelte component support
- Cloudflare Workers is your deployment target
- Real-time content collections are required
- Your team is JavaScript-first
- Community ecosystem size and theme availability are priorities

## SEO Capability Matrix

| SEO criterion | SSG | Astro |
|---|---|---|
| `<html lang>` declared | ✅ Built-in (i18n) | ✅ via `BaseLayout` |
| `<title>`, `<meta description>` | ✅ Built-in + CI-validated | ✅ Astro slot pattern |
| Canonical URL | ✅ `CanonicalPlugin` | ✅ `<link rel="canonical">` slot |
| Open Graph + Twitter Card | ✅ Auto-generated, including OG image SVG | ✅ via `astro-seo` integration (optional) |
| JSON-LD structured data + schema.org validation | ✅ Auto + CI gate (`validate_jsonld`) | ⚠️ Manual JSON-LD (no validator) |
| Sitemap (`sitemap.xml` + per-locale) | ✅ Built-in | ✅ `@astrojs/sitemap` integration |
| News sitemap | ✅ `NewsSitemapFixPlugin` | ❌ Manual |
| RSS + Atom feeds | ✅ Both, with categories + enclosures | ✅ `@astrojs/rss` integration |
| `robots.txt` | ✅ Per-environment via `RobotsPlugin` | ✅ via `astro-robots-txt` |
| Hreflang for multi-locale sites | ✅ `I18nPlugin` + per-locale sitemap | ✅ `astro:i18n` (Astro 4+) |
| Image `<alt>` build-time validation | ✅ `AccessibilityPlugin` flags missing alt | ❌ Manual |
| `lastmod` per URL | ✅ Auto from frontmatter | ✅ via integration config |
| **SBOM discovery** (`<link rel="sbom">`) | ✅ `SbomPlugin` (CycloneDX 1.5) | ❌ Not supported |
| Web Vitals + a11y CI gates | ✅ axe-core + perf budgets (built-in) | ⚠️ Lighthouse-CI (external) |
| Content-addressable assets + cache headers | ✅ `FingerprintPlugin` + per-platform | ✅ Vite-managed |
| CSP without `unsafe-inline` | ✅ `CspPlugin` extracts inline → external + SRI | ✅ since Astro 6 |
| WCAG 2.2 build-time checks | ✅ 8 criteria automated, matrix in `wcag-compliance.json` | ❌ Not built-in |

**Bottom line:** Astro and SSG are the two SSGs with the most
overlapping SEO+a11y feature sets. The differentiation is the
*gate-vs-guideline* axis — SSG fails the build on
schema.org-invalid JSON-LD, missing alt text, sub-24px target
sizes, and `outline:none` without a focus replacement; Astro
treats these as guidance you can opt into via integrations.
