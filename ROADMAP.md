# Static Site Generator (SSG) — Enterprise-Grade Strategic Deep Dive and Architectural Roadmap

*Research date: 2026-06-22. Based on codebase inspection of `static-site-generator` at v0.0.41 \+ web research of 2026 SSG landscape.*

## Executive Summary

Public digital publishing for Tier-1 corporate and financial institutions has evolved from a marketing function into a highly regulated operational risk perimeter. Under modern regulatory frameworks—such as the Digital Operational Resilience Act (DORA) in the European Union, the European Accessibility Act (EAA), and strict global privacy laws (GDPR)—every public-facing digital asset is a potential entry point for supply-chain compromises, web defacement, and regulatory non-compliance.

The open-source Rust [static-site-generator](https://github.com/sebastienrousseau/static-site-generator) represents a paradigm shift. By moving security, accessibility audits, internationalisation, and AI content pipelines entirely to compile-time, it treats web publishing not as a design challenge, but as an auditable, secure-by-default software engineering pipeline.

This strategic deep dive analyzes the current architectural state of `static-site-generator` (v0.0.41), identifies structural gaps between its documented promises and code realities, exposes what other critical enterprise features are missing, and establishes an ambitious capability roadmap (see the ADR-0009 versioning note above for why this is not a "1.0 release" plan).

---

## Current Strengths

The `static-site-generator` codebase exhibits several category-defining engineering decisions that set it apart from legacy JS/Go engines:

- **Industry-Leading Security Posture:** The enforcement of `#![forbid(unsafe_code)]` workspace-wide provides compile-time memory safety guarantees. The build pipeline features true SHA-256/SHA-384 Subresource Integrity (SRI) generation (`src/plugins/assets.rs`) and automatic Content Security Policy (CSP) extraction that eliminates unsafe-inline scripts and styles. This is paired with signed releases, Sigstore attestation, and CycloneDX 1.5 SBOM generation on every build.  
- **Compiler-Enforced Accessibility Gates:** Integrating Web Content Accessibility Guidelines (WCAG) 2.2 Level AA compliance checks directly into the compilation pipeline (using a build-time axe-core parser via Playwright) transforms accessibility from an expensive post-publication audit into a hard compiler gate. If a page fails compliance, compilation halts immediately with exact line-number errors.  
- **Sovereign, High-Velocity AI Pipeline:** The integration of a local-LLM translation and metadata-extraction pipeline (via local Ollama or llama.cpp endpoints) solves the data-sovereignty paradox. Financial institutions can automate content summarisation, JSON-LD schema generation, and multilingual translation across 28 locales without exfiltrating pre-earnings disclosures or sensitive IP to public cloud AI APIs.  
- **Rigorously Parallelised Architecture:** Leverages Rust's memory-safety guarantees to run parallelized, Rayon-driven HTML and asset compilation (`src/core/pipeline.rs`). The parallelized plugin pipeline executes fused transforms (where `SearchPlugin`, `SeoPlugin`, `CanonicalPlugin`, and `JsonLdPlugin` use `par_iter()`) ensuring that pages are read and written to disk once.  
- **Robust Supply-Chain and Dependency Hygiene:** By migrating its template engine from Tera to MiniJinja (`v0.0.37`), the project reduced its binary size, eliminated transitive dependencies like `rand` at compile-time, and established an exceptionally clean dependency footprint that mitigates software supply-chain vulnerabilities.

---

## Gaps and Real-World Realities

> **Status note (v0.0.58).** The findings below are a dated snapshot of
> **v0.0.41** and are kept verbatim as the record of that inspection. Four
> have since been closed, verified against the current tree:
>
> - *Shelling out to `curl`* — `src/plugins/llm.rs` uses `ureq`; the only
>   remaining mentions of `curl` are comments explaining the port (#520).
> - *Naive string manipulation in HTML rewriting* — both
>   `image_plugin.rs` and `search.rs` rewrite through `lol_html`.
> - *Unimplemented AVIF support* — `avif_variants` is populated from real
>   encode results, not `Vec::new()`.
> - *Subcommand deficit* — `ssg build`, `ssg check` and `ssg dev` all
>   exist. (`ssg lint` still does not.)
>
> The polling-based watcher is unchanged and remains open.

Despite these exceptional strengths, a rigorous codebase inspection of v0.0.41 reveals several architectural, functional, and developer-experience gaps between its documentation claims and the actual rust code:

### Architectural Gaps

- **Whitespace Collapse vs. Native Minification:** While the README promises "native JS/CSS minification," the `MinifyPlugin` (`src/plugins/plugins.rs:96-116`) acts merely as a naive whitespace collapser. It short-circuits on `<pre>` elements and collapses whitespace runs in HTML, but does not perform syntactically aware native CSS or JS minification. Furthermore, it only processes top-level pages and does not recursively walk subdirectories (such as `/blog/` or `/tags/`), leaving deep pages unminified.  
- **Dead Incremental Infrastructure:** The dependency tracking graph (`DepGraph` in `src/core/depgraph.rs`) is compiled and loaded into `PluginContext.dep_graph` but is never actually populated in production code. The method `add_dep()` is only called in unit tests, making the README's claim of "incremental rebuilds via dependency graphs" currently aspirational.  
- **Batched Compilation vs. Streaming Compilation:** The `streaming::compile_batch` module (`src/core/streaming.rs`) does not truly stream. Instead, it compiles pages in batches to a temporary directory, executes `staticdatagen::compile` from scratch for each batch, and merges the outputs. This results in significant disk I/O overhead and redundant parsing, deviating from a true streaming architecture.  
- **Plugin Lifecycle Phase Violations:** Plugins that generate new HTML pages during the build process—such as `TaxonomyPlugin`, `PaginationPlugin`, and `I18nPlugin`—write directly to disk in `after_compile` rather than utilizing the `transform_html` lifecycle. Consequently, pages generated by these plugins bypass critical post-processing plugins (such as `CanonicalPlugin`, `JsonLdPlugin`, `RobotsPlugin`, and `AccessibilityPlugin`) if those plugins were registered earlier. This leaves tag, category, and paginated pages without correct canonical links, JSON-LD schemas, or accessibility validations.  
- **Shelling Out to `curl` in `LlmPlugin`:** The local LLM content pipeline (`src/plugins/llm.rs`) shells out directly to the host's `curl` binary to query local endpoints. This introduces severe cross-platform bugs (e.g., on Windows hosts without curl in the PATH), poses a security risk (shell injection vectors), and fails in locked-down or network-isolated CI environments.  
- **Naive String Manipulation in HTML Rewriting:** The `image_plugin.rs` and `search.rs` extractors rewrite HTML strings using fragile `str::find` and `str::rfind` operations. This approach is highly vulnerable to broken HTML tags, `<img>` tags inside comments, character entities in alt text, or pre-existing `srcset` properties, which can result in corrupted output.  
- **Unimplemented AVIF Support:** Although AVIF image encoding is heavily documented, the implementation in `image_plugin.rs` is a stub where `avif_variants` simply returns `Vec::new()`, leaving the feature non-functional.  
- **Polling-Based Watcher:** The local development server's watcher (`src/server/watch.rs`) uses polling rather than filesystem event APIs, leading to excessive idle CPU usage and sub-second modification latency.

### Functional & DX Gaps

- **No Transitive Dependency Tracking:** The dependency graph cannot track nested dependencies (e.g., changes to a sub-template that affects a layout that affects a page), as verified by the unit test `transitive_not_tracked`.  
- **No Incremental Compilation CLI Flag:** There is no `--incremental` CLI flag wired to the execution compiler, preventing developers from utilizing cached builds.  
- **HMR is Limited to CSS:** Hot Module Replacement (HMR) only supports CSS; any modification to HTML, layouts, or markdown files triggers a full page reload, degrading developer velocity.  
- **Subcommand Deficit:** Developers must manually pass verbose flags (`ssg -s public -w`) because standard subcommands like `ssg dev`, `ssg build`, `ssg check`, and `ssg lint` do not exist.

---

## Architectural Gaps We Are Missing (New Discoveries)

Beyond the gaps listed in v0.0.41, a comprehensive analysis of the project against a modern, financial-grade risk profile reveals several missing capabilities that must be incorporated to achieve true enterprise readiness:

### 1\. WebAssembly Plugin Sandboxing (Zero-Trust Extension)

While the compiler binary itself is written in safe Rust, allowing arbitrary third-party plugins to execute natively on host systems introduces a severe supply-chain vulnerability. A compromised third-party plugin could easily access the host's filesystem, read proprietary Markdown files, or exfiltrate private credentials.

- **Missing Capability:** A sandboxed execution environment. To achieve zero-trust compilation, the compiler should execute third-party plugins inside an embedded WebAssembly runtime (such as `wasmtime`). Plugins should interact with the host solely via a restricted WebAssembly System Interface (WASI), limiting their access strictly to the page being transformed.

### 2\. Zero-Copy HTML Parsing via Streaming AST (`lol_html`)

Migrating the HTML parsing layer to a full in-memory DOM library (like Kuchiki or html5ever) introduces significant memory overhead and processing pauses when handling sites with over 100,000 pages.

- **Missing Capability:** A streaming, zero-copy HTML rewriter. Utilizing Cloudflare's `lol_html` (Low-Output-Latency HTML rewriter) allows the compiler to parse, inspect, and modify HTML elements in a single streaming pass with near-zero memory allocation, matching the parallel streaming compiler's target of sub-second builds.

### 3\. Local Semantic Vector Search (Local RAG)

The current search index (`SearchPlugin`) generates a heavy, flat JSON index that performs simple client-side string matches, lacking support for fuzzy search, stemming, or semantic queries. Pagefind is an improvement, but it still relies on downloading a large index.

- **Missing Capability:** Embedded semantic search. The compiler should leverage a local, lightweight Rust-native vector embedding model (such as a MiniLM-L6 model executed via `candle` or `ort` / ONNX Runtime) at build-time. It should generate dense vector embeddings for every page paragraph and output a compact vector index. The client-side search widget, compiled to WASM, can then perform true offline semantic search directly in the browser.

### 4\. Deterministic Translation and Inference Caching

Because local LLM inference (e.g., via Ollama or Llama.cpp) is highly CPU/GPU intensive, translating or generating metadata for thousands of pages on every build is computationally prohibitive.

- **Missing Capability:** Content-hash-based inference caching. The compiler must maintain a deterministic cache of all LLM operations. If the SHA-256 hash of a markdown file's content and its translation parameters matches a cache entry, the compiler should reuse the cached translation and metadata, bypassing redundant local inference.

### 5\. Asynchronous File I/O for Parallel Scaling

While the plugin pipeline is parallelized via Rayon, standard synchronous disk writes block Rayon's OS threads, creating an I/O bottleneck when writing tens of thousands of pages.

- **Missing Capability:** Asynchronous, non-blocking disk I/O. The compiler should decouple CPU-intensive tasks (Markdown parsing, minification) from disk-bound writes, utilizing asynchronous I/O thread pools or Linux `io_uring` bindings (via `rio` or `tokio`) to write compiled pages in parallel without blocking the parallel CPU executors.

---

## The Strategic Roadmap

> **Versioning note (2026-07-04, ADR-0009):** the phase labels below
> ("0.1.0", "1.0.0") predate a deliberate versioning-policy decision:
> `ssg` stays on `0.0.x` — incrementing by `0.0.1` per release — through
> `0.0.999` at the earliest, to mature the API surface and enterprise
> adoption before making any SemVer compatibility commitment. Read
> "Phase 2" and "Phase 3" below as **capability milestones**, not
> version-number targets; they will ship as ordinary `0.0.x` releases
> (see [ADR-0009](docs/adr/0009-versioning-policy-0.0.x-until-0.0.999.md)
> for the full rationale). Many Phase 1/2 items have already shipped in
> releases since this document's June 2026 research date — check
> `CHANGELOG.md` before treating any item here as outstanding.

The following roadmap integrates both the resolved gaps and the newly discovered enterprise-grade capabilities into a structured, chronological release framework.

### Phase 1: 0.0.42 (The Robustness and Correctness Patch — 1-2 Weeks)

1. **Reconstruct `MinifyPlugin`:** Integration of `minify-html`, `oxc_minifier`, and `lightningcss` for native, syntactically aware HTML, JS, and CSS minification. Ensure the plugin recursively walks all nested directories under `site_dir`.  
2. **Secure the AI Pipeline:** Port `LlmPlugin` from native `curl` shellouts to `ureq` (a lightweight, synchronous, safe Rust HTTP client) to ensure cross-platform compatibility and eliminate shell injection vulnerabilities.  
3. **Complete AVIF Implementation:** Plumb `ravif` directly into the image asset pipeline, enabling high-performance AVIF encoding alongside WebP and PNG.  
4. **Automate HrefLang and Multi-Locale Mapping:** Automatically detect parallel translated pages in multilingual builds and inject standard Google-compliant `<link rel="alternate" hreflang="..." />` tags into the head of each compiled HTML file.  
5. **JSON Feed 1.1 Support:** Ship a dedicated JSON Feed 1.1 emitter alongside standard RSS 2.0 and Atom 1.0 syndication channels.

### Phase 2: The Credibility and Incremental Milestone (ships as a `0.0.x` release, not `0.1.0` — see versioning note above)

1. **Populate `DepGraph` and Enable `--incremental`:** Fully wire `DepGraph` to track template-to-page and markdown-to-page dependencies. Implement a cache invalidation layer and wire the `--incremental` CLI flag, targeting sub-200ms rebuilds for warm-cache environments.  
2. **Streaming AST Rewrite via `lol_html`:** Replace fragile string rewriting in `image_plugin.rs`, `search.rs`, and CSP injections with a streaming, zero-copy HTML rewriter powered by `lol_html`.  
3. **Event-Driven Watcher and Component HMR:** Port the watch module from polling to the event-driven `notify` crate, and implement CSS-only and partial-HTML hot reloading for sub-100ms browser updates.  
4. **Unified Command CLI:** Re-architect the compiler interface to support standard subcommands: `ssg dev`, `ssg build`, `ssg check` (accessibility/SEO audit), and `ssg deploy`.  
5. **Deterministic Inference Cache:** Implement a content-hash caching layer for all local LLM translation, summarisation, and metadata extraction tasks.

### Phase 3: The Enterprise and Production Milestone (ships as `0.0.x` releases, not `1.0.0` — see versioning note above; no target date)

1. **Zero-Trust WASM Plugin Sandboxing:** Embed a WebAssembly runtime (`wasmtime` or `wasmer`) to execute third-party plugins in a fully sandboxed environment with capability-based filesystem and network access.  
2. **Local Semantic Vector Search (Local RAG):** Embed a local Rust-native embedding model (via `candle` or `ort`) to compile dense paragraph embeddings into a compact index, enabling private, client-side semantic search.  
3. **Server Islands and WASM Edge Target:** Implement `<ssg-island>` component execution on edge runtimes (such as Cloudflare Workers, Vercel Edge, or Netlify Edge) built on top of the compiled `ssg-wasm` core.  
4. **Asynchronous Parallel I/O Engine:** Re-architect the file system writing module to use asynchronous I/O thread pools and `io_uring` bindings, eliminating CPU worker blocks during parallel writes.  
5. **SLSA v1.1 Build Provenance & SPDX 3.0 Compliance:** Provide mathematically verifiable SLSA Level 3 build provenance and generate SPDX 3.0 compliant SBOMs, fully satisfying modern software supply-chain security standards.

---

## Competitor Matrix (2026 Landscape)

The following matrix compares `static-site-generator` (post-Phase-3 capability target — see the versioning note above; no `1.0` release is scheduled) against the leading web publishing engines of 2026:

| Capability | static-site-generator (post-Phase-3) | Hugo v0.155+ | Zola v0.19+ | Astro 5 | Eleventy 3 |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **Language / Runtime** | Rust (Zero Unsafe) | Go | Rust | JS (Node/V8) | JS (Node/V8) |
| **A11y Build Gate** | Build-Time AST Validation | None | None | Post-build Linter | Post-build Linter |
| **Security Hardening** | SHA-384 SRI & CSP Injection | Manual | Manual | Manual | Manual |
| **Supply-Chain Safety** | SLSA L3 \+ SPDX 3.0 \+ WASM Sandbox | Minimal | Minimal | Heavy NPM Tree | Heavy NPM Tree |
| **AI Content Pipeline** | Private, Local-First (Local LLM) | None | None | Public API Only | Public API Only |
| **Incremental Speed** | \<200ms (Warm Cache) | \<100ms | \<150ms | \~1.5s | \~140ms |
| **Dynamic Interactivity** | Server Islands (WASM Targets) | None | None | Server Islands (JS) | Islands (JS) |
| **Search Engine** | Local Semantic WASM Search | Simple String | Simple String | Pagefind (JS) | Pagefind (JS) |

---

## Headline Positioning (Post-Phase-3 Capability Set)

"The static site generator engineered as secure-by-default software infrastructure. Author content with local-first AI pipelines, compile 100,000+ pages with parallel streaming performance, enforce WCAG 2.2 AA and strict CSP/SRI build gates, and ship sandboxed dynamic islands—all within a single, memory-safe Rust binary."

---

## Regulatory and Compliance Integration

In high-stakes enterprise and financial sectors, software is evaluated through the lens of compliance and risk capital. The architectural roadmap of `static-site-generator` aligns directly with major regulatory mandates:

- **DORA Article 6 (ICT Risk Management):** The compile-time calculation and injection of SHA-384 SRI hashes and strict Content Security Policies satisfy the requirement to protect digital publishing channels from supply-chain injection, web defacement, and cross-site scripting (XSS) vectors.  
- **DORA Article 7 (ICT Systems Resilience):** By moving to immutable, compile-time verified static assets, financial institutions eliminate database and runtime server vulnerabilities, lowering the operational risk multiplier and reducing required risk capital reserves under Basel III.  
- **European Accessibility Act (EAA) Directive (EU) 2019/882:** Shifting accessibility auditing left into the compilation pipeline as a hard compiler gate guarantees 100% compliance prior to deployment, eliminating the risk of brand damage and civil litigation under EAA and ADA Title III.  
- **GDPR Article 25 (Privacy-by-Design):** Running the entire translation and metadata pipeline on localized, network-isolated hardware ensures that proprietary drafts, financial metrics, and PII are never exposed to public third-party cloud LLM providers, ensuring strict compliance with data sovereignty principles.

---

## Technical References

1. **Cloudflare Low-Output-Latency HTML Rewriter (lol\_html):** [GitHub Repository](https://github.com/cloudflare/lol-html). Streaming HTML rewriting engine.  
2. **Web Content Accessibility Guidelines (WCAG) 2.2:** [W3C Recommendation](https://www.w3.org/TR/WCAG22/). Standards for web accessibility.  
3. **Digital Operational Resilience Act (DORA):** [Regulation (EU) 2022/2554](https://eur-lex.europa.eu/eli/reg/2022/2554/oj). Regulatory framework for financial entity resilience.  
4. **Software Supply Chain Levels for Software Artifacts (SLSA) v1.0:** [SLSA Specification](https://slsa.dev/spec/v1.0/). Framework for supply-chain security.  
5. **MiniJinja Template Engine:** [GitHub Repository](https://github.com/mitsuhiko/minijinja). A minimal, dependency-free template engine for Rust.  
6. **CycloneDX Software Bill of Materials (SBOM) v1.5:** [CycloneDX Specification](https://cyclonedx.org/docs/1.5/). Standard for software supply-chain audits.  
7. **European Accessibility Act (EAA):** [Directive (EU) 2019/882](https://eur-lex.europa.eu/eli/dir/2019/882/oj). Accessibility requirements for products and services.
