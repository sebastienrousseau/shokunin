# ssg-wasm

WebAssembly bindings for `ssg-core` — run SSG in the browser or at the edge.

A thin `wasm-bindgen` surface over the compilation pipeline. The work happens
in [`ssg-core`](../ssg-core); this crate exists to expose it to JavaScript
without dragging in anything that needs a system toolchain.

## Not published

`publish = false`. It is built as a WASM artefact rather than consumed as a
Rust dependency, so it is absent from crates.io by design — not an oversight
in the release workflow.

## API

```js
import init, { compile_markdown, compile_page, strip_html } from "./ssg_wasm.js";

await init();
const html = compile_markdown("# Title\n\nBody.");
```

| binding | returns |
|---|---|
| `compile_markdown(input)` | HTML string |
| `compile_page(input)` | structured page object, or a JS error |
| `strip_html(input)` | plain text |

## Building

```bash
wasm-pack build crates/ssg-wasm --target web
```

## Licence

MIT or Apache-2.0, at your option.
