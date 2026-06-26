<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ssg-search

Browser-native vector semantic search for [SSG](https://crates.io/crates/ssg) —
WASM-compiled, int8-quantised, `Float32Array` boundary (issue
[#545](https://github.com/sebastienrousseau/static-site-generator/issues/545)).

This crate is part of the SSG workspace. Documentation lives on
[docs.rs](https://docs.rs/ssg-search/) and the canonical README is the
[repository root](https://github.com/sebastienrousseau/static-site-generator#readme).

## Architectural invariants

1. **JS/WASM boundary is `Float32Array` only.** No nested objects, no
   JSON across the wire.
2. **All vectors are pre-normalised** at build time so the runtime
   similarity reduces to a pure dot product (no division, no `sqrt`).
   Negative cosine scores are clipped to 0 in the engine.
3. **Builds are reproducible.** Given identical input bytes and
   encoder weights, `embeddings.bin` is byte-identical across
   platforms.
4. **Single-threaded by design.** Browser WASM is single-threaded;
   the engine carries no `Arc` / `Mutex` / Rayon overhead.

## Default vs `model2vec` features

The default build ships a deterministic hashed-n-gram projection
embedder — model-free, zero heavy dependencies, byte-reproducible.
Opt in to the real `model2vec-rs` encoder with `--features model2vec`
when you want the larger model.

## Quick start

```rust,no_run
use ssg_search::artifacts::{Artifacts, InputDoc};
use ssg_search::VectorEngine;

let docs = vec![InputDoc {
    url: "/post-1".into(),
    title: "Rust WebAssembly".into(),
    body: "rust compiles to wasm".into(),
    excerpt: "rust wasm".into(),
}];
let arts = Artifacts::from_docs(&docs);
let engine = VectorEngine::new(
    &arts.model, &arts.tokenizer, &arts.embeddings, arts.count(),
).unwrap();
let top = engine.search("rust webassembly", 3);
println!("{top:?}");
```

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
