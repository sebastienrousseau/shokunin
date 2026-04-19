<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# WebAssembly

SSG compiles to WebAssembly. The `ssg-core` and `ssg-wasm` crates
run in the browser, Cloudflare Workers, Deno Deploy, and Vercel Edge.

## Architecture

Two crates power the WASM target:

| Crate | Purpose |
|-------|---------|
| `ssg-core` | Pure-logic core — markdown compilation, frontmatter parsing, search index, slugify |
| `ssg-wasm` | `wasm-bindgen` bindings for browser/edge runtimes |

## Available Functions

| Function | Input | Output |
|----------|-------|--------|
| `compile_markdown(md)` | Markdown string | HTML string |
| `compile_page(md)` | Markdown with frontmatter | `{ frontmatter: {...}, html: "..." }` |
| `strip_html(html)` | HTML string | Plain text |

## Browser Usage

```javascript
import init, { compile_markdown, compile_page } from './ssg_wasm.js';

await init();

const html = compile_markdown("# Hello\n\nWorld");
// <h1>Hello</h1>\n<p>World</p>

const page = compile_page("---\ntitle: Test\n---\n# Body");
// { frontmatter: { title: "Test" }, html: "<h1>Body</h1>" }
```

## Edge Deployment

`ssg-wasm` runs on any V8-based edge runtime:

- **Cloudflare Workers** — compile markdown at the edge
- **Deno Deploy** — server-side rendering without Node.js
- **Vercel Edge Functions** — dynamic SSG at CDN nodes

## Build

```sh
wasm-pack build crates/ssg-wasm --target web --out-dir ../../pkg
```

The output in `pkg/` contains the `.wasm` binary and JavaScript bindings.

## Limitations

- No filesystem access (browser sandbox)
- No template rendering (MiniJinja not compiled to WASM)
- No plugin pipeline (WASM exports are pure functions)
