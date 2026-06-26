// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Cloudflare Workers ISR adapter (issue #546).
//
// Stateless fetch handler that:
//   1. Looks the requested URL up in Cloudflare's Cache API.
//      - fresh  → return.
//      - stale  → serve stale + ctx.waitUntil(revalidate(...)).
//      - miss   → block: fetch md+template from KV, render via WASM,
//                 cache, return.
//   2. Honours per-route Cache-Control derived from the manifest.
//
// The WASM module is bundled via Wrangler's `wasm_module` binding
// (see wrangler.toml). The first call to `init()` instantiates it
// against the Worker's V8 isolate.

import init, { render_page_isr } from "./pkg/ssg_wasm.js";
// @ts-expect-error — Wrangler exposes the WASM module via this import shape.
import wasmModule from "./pkg/ssg_wasm_bg.wasm";

import { handleInvalidate } from "./webhook";
import { handleRpc, RPC_PREFIX } from "./rpc";

/** Bindings declared in wrangler.toml. */
export interface Env {
  /** KV namespace with manifest + raw source bytes. */
  SSG_CONTENT: KVNamespace;
  /** HMAC / bearer token verified by the invalidate webhook. */
  SSG_WEBHOOK_TOKEN?: string;
  /** Cloudflare account ID + zone ID for purge-by-URL. */
  CF_ACCOUNT_ID?: string;
  CF_ZONE_ID?: string;
  CF_API_TOKEN?: string;
}

interface CachePolicy {
  s_maxage: number;
  swr: number;
}

interface ManifestEntry {
  sources: string[];
  hash: string;
  cache?: CachePolicy;
}

interface Manifest {
  version: number;
  generated_at: string;
  default_cache: CachePolicy;
  entries: Record<string, ManifestEntry>;
}

let wasmReady: Promise<void> | null = null;
async function ensureWasm(): Promise<void> {
  if (!wasmReady) {
    // wasm-bindgen `init(wasmModule)` takes a compiled WebAssembly.Module
    // directly when bundled via Wrangler.
    wasmReady = init(wasmModule).then(() => undefined);
  }
  return wasmReady;
}

let manifestCache: Manifest | null = null;
async function loadManifest(env: Env): Promise<Manifest> {
  if (manifestCache) return manifestCache;
  const json = await env.SSG_CONTENT.get("manifest");
  if (!json) throw new Error("KV: missing 'manifest' key");
  manifestCache = JSON.parse(json) as Manifest;
  return manifestCache;
}

function cacheControlFor(
  entry: ManifestEntry,
  manifest: Manifest,
): string {
  const c = entry.cache ?? manifest.default_cache;
  return `s-maxage=${c.s_maxage}, stale-while-revalidate=${c.swr}`;
}

/** Translate `/posts/foo/` → `/posts/foo/index.html`. */
function normaliseUrl(pathname: string): string {
  if (pathname === "/") return "/index.html";
  if (pathname.endsWith("/")) return `${pathname}index.html`;
  if (!pathname.includes(".")) return `${pathname}/index.html`;
  return pathname;
}

async function renderForUrl(
  url: string,
  env: Env,
  manifest: Manifest,
): Promise<Response> {
  const entry = manifest.entries[url];
  if (!entry) {
    return new Response("Not found", { status: 404 });
  }

  const markdownKey = entry.sources.find((s) => s.startsWith("content/"));
  if (!markdownKey) {
    return new Response("Manifest entry has no content source", {
      status: 500,
    });
  }

  const templateKey =
    entry.sources.find(
      (s) =>
        s.startsWith("templates/") &&
        (s.endsWith("page.html") || s.endsWith("index.html")),
    ) ?? "templates/page.html";

  const [markdown, layout] = await Promise.all([
    env.SSG_CONTENT.get(markdownKey),
    env.SSG_CONTENT.get(templateKey),
  ]);
  if (markdown === null || layout === null) {
    return new Response("Source missing in KV", { status: 502 });
  }

  await ensureWasm();
  const html = render_page_isr(
    markdown,
    layout,
    JSON.stringify({ url, site_name: "" }),
  );

  return new Response(html, {
    status: 200,
    headers: {
      "Content-Type": "text/html; charset=utf-8",
      "Cache-Control": cacheControlFor(entry, manifest),
      "X-SSG-Hash": entry.hash,
      "X-SSG-Build": manifest.generated_at,
    },
  });
}

async function revalidate(
  url: string,
  request: Request,
  env: Env,
): Promise<void> {
  const manifest = await loadManifest(env);
  const fresh = await renderForUrl(url, env, manifest);
  if (!fresh.ok) return;
  const cache = caches.default;
  await cache.put(request, fresh.clone());
}

const worker: ExportedHandler<Env> = {
  async fetch(request, env, ctx): Promise<Response> {
    const { pathname } = new URL(request.url);

    // Webhook route — separate code path, no caching.
    if (pathname === "/__ssg/invalidate" && request.method === "POST") {
      return handleInvalidate(request, env);
    }

    // RPC route (issue #548) — JSON-over-POST dispatcher. Lives
    // alongside the ISR cache layer but bypasses it entirely;
    // POST responses are never cached at the Edge.
    if (pathname.startsWith(RPC_PREFIX)) {
      await ensureWasm();
      return handleRpc(request, pathname);
    }

    const url = normaliseUrl(pathname);
    const cache = caches.default;

    // 1. Cache lookup.
    const cached = await cache.match(request);
    if (cached) {
      const age = Number(cached.headers.get("Age") ?? "0");
      const cc = cached.headers.get("Cache-Control") ?? "";
      const sMaxAge = Number(/s-maxage=(\d+)/.exec(cc)?.[1] ?? "60");
      if (age < sMaxAge) {
        // Fresh — return as-is.
        return cached;
      }
      // Stale — serve stale + revalidate in background.
      ctx.waitUntil(revalidate(url, request, env));
      return cached;
    }

    // 2. Miss — block-render.
    const manifest = await loadManifest(env);
    const response = await renderForUrl(url, env, manifest);
    if (response.ok) {
      ctx.waitUntil(cache.put(request, response.clone()));
    }
    return response;
  },
};

export default worker;
