<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR-0003: `lol_html` for streaming HTML rewriting

- **Date:** 2026-06-26
- **Status:** Accepted

## Context

The plugin pipeline mutates generated HTML in multiple passes:

- `CspPlugin` inlines `<style>`/`<script>` to external files and
  injects SHA-384 SRI hashes.
- `ImagePlugin` rewrites `<img>` tags to `<picture>` with WebP/AVIF
  variants and `srcset`.
- `SearchIndexPlugin` extracts text content for the search index.
- `SeoPlugin`, `CanonicalPlugin`, `JsonLdPlugin` inject `<head>`
  elements.
- `MinifyPlugin` collapses whitespace.

Pre-v0.0.44 these passes used `str::find` / `str::rfind` / naive
`.replace("<", ...)`. That approach was fragile:

- An `<img>` tag inside an HTML comment got rewritten anyway.
- A pre-existing `srcset` attribute was clobbered without warning.
- HTML character entities in alt text were re-encoded incorrectly.
- A multiline `<style>` block split across lines was not detected.

Two parser families were on the table for v0.0.44:

1. **Tree-building**: `html5ever`, `kuchikiki`, `scraper`. Build a
   full DOM, mutate, re-serialise.
2. **Streaming SAX-style**: `lol_html` (Cloudflare's Low-Output-Latency
   HTML Rewriter), `quick-xml` (XML-flavoured).

For 100K-page corpora the tree-building cost is the dominant per-page
allocation. `lol_html` is single-pass, zero-copy where possible, and
CSS-selector-driven — the same mental model as jQuery / DOM
`querySelectorAll`.

## Decision

**We use `lol_html` for all HTML inspection and rewriting in plugins.**
String-level manipulation in plugin rewrite paths is forbidden;
v0.0.47 #570 closes out the remaining `str::find` / `str::rfind`
holdouts.

`html5ever` and friends are not pulled into the graph.

## Consequences

**Positive.**

- Zero-copy parse + write means a 100K-page corpus does not heap-spike
  during the post-process pass.
- CSS selectors are a familiar primitive for any plugin author who has
  used jQuery/DOM APIs.
- BSD-3-Clause license, already allow-listed in `deny.toml`.
- No C dependencies (`lol_html` is pure Rust).
- Single-pass design composes with Rayon: each plugin gets its own
  `HtmlRewriter` instance per page; no cross-thread sharing of state.

**Negative.**

- `lol_html` is **streaming**: a rewriter cannot look ahead. Plugins
  that need to know the full document shape (e.g., "extract every
  heading then emit a TOC") must either collect during a first pass
  and inject during a second pass, or use a different tool.
- The `Settings` builder is verbose; we introduced
  `src/util/html_rewriter.rs` and `src/util/head_dom.rs` to amortise
  boilerplate across plugins.
- `lol_html` is on a 3.x major-version cycle; we pin `~3` and review
  on every minor bump.

## Alternatives Considered

- **`html5ever`.** Rejected: tree-building cost dominates for our
  workload; ~3x slower on the v0.0.45 #559 baseline corpus.
- **`scraper`.** Rejected: thin wrapper over `html5ever`, same cost.
- **`kuchikiki`.** Rejected: same family.
- **`quick-xml` adapted to HTML.** Rejected: HTML is not XML;
  contracting an HTML5-compliant parser onto an XML core invites
  pre-existing-content-decoding bugs.
- **Hand-rolled streaming parser.** Rejected: the v0.0.46 #566 fuzz
  corpus would find an indefinite tail of edge cases. We benefit from
  Cloudflare's adversarial corpus against `lol_html`.

## Status

Accepted. `lol_html = "3.0"` is a direct, unconditional dependency.
Migration progress tracked in v0.0.47 #570; until that issue closes,
some legacy `str::find` paths remain and are documented as known
debt.
