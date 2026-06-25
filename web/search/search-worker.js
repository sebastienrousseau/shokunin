// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// ssg-search Web Worker bootstrap.
//
// Loads the wasm-pack bundle, fetches the four `<site>/search/*.bin*`
// artifacts in parallel, instantiates a WasmVectorEngine, and listens
// for `search(query)` messages on the worker port.
//
// Wire-protocol (window -> worker):
//   { type: "init",   base: "/search/" }
//   { type: "search", query: "...",  topK: 10 }
//
// Worker -> window:
//   { type: "ready",   count: 1234 }
//   { type: "error",   message: "..." }
//   { type: "results", query: "...", hits: [{ idx, score, url, title, excerpt }] }
//
// The boundary between the worker JS and the WASM module is
// strictly: scalars, strings, Float32Array. No nested JSON crosses
// into WASM. (AC4)

import init, { WasmVectorEngine } from "./ssg_search.js";

let engine = null;
let manifest = null;

async function loadArtifacts(base) {
  // Fetch all four artifacts in parallel.
  const [wasmReady, modelRes, tokRes, embRes, manifestRes] = await Promise.all([
    init(base + "ssg_search_bg.wasm"),
    fetch(base + "model.bin"),
    fetch(base + "tokenizer.bin"),
    fetch(base + "embeddings.bin"),
    fetch(base + "manifest.json"),
  ]);
  void wasmReady;
  if (!modelRes.ok)    throw new Error("failed to fetch model.bin: " + modelRes.status);
  if (!tokRes.ok)      throw new Error("failed to fetch tokenizer.bin: " + tokRes.status);
  if (!embRes.ok)      throw new Error("failed to fetch embeddings.bin: " + embRes.status);
  if (!manifestRes.ok) throw new Error("failed to fetch manifest.json: " + manifestRes.status);

  const [modelBuf, tokBuf, embBuf, manifestJson] = await Promise.all([
    modelRes.arrayBuffer(),
    tokRes.arrayBuffer(),
    embRes.arrayBuffer(),
    manifestRes.json(),
  ]);
  manifest = manifestJson;

  const model = new Uint8Array(modelBuf);
  const tok   = new Uint8Array(tokBuf);
  const emb   = new Uint8Array(embBuf);

  engine = new WasmVectorEngine(model, tok, emb, manifest.count);
  return manifest.count;
}

self.onmessage = async (event) => {
  const msg = event.data;
  try {
    if (msg.type === "init") {
      const count = await loadArtifacts(msg.base || "/search/");
      self.postMessage({ type: "ready", count });
    } else if (msg.type === "search") {
      if (!engine || !manifest) {
        self.postMessage({ type: "error", message: "engine not initialised" });
        return;
      }
      const topK = msg.topK | 0 || 10;
      // The WASM call returns a Float32Array of [idx, score, idx, score, ...]
      const flat = engine.search(String(msg.query || ""), topK);
      const hits = [];
      for (let i = 0; i < flat.length; i += 2) {
        const idx = flat[i] | 0;
        const score = flat[i + 1];
        if (score <= 0) continue;
        const entry = manifest.entries[idx];
        if (!entry) continue;
        hits.push({ idx, score, url: entry.url, title: entry.title, excerpt: entry.excerpt });
      }
      self.postMessage({ type: "results", query: msg.query, hits });
    } else {
      self.postMessage({ type: "error", message: "unknown message type: " + msg.type });
    }
  } catch (e) {
    self.postMessage({ type: "error", message: String((e && e.message) || e) });
  }
};
