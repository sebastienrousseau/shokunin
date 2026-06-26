// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Vercel Edge ISR adapter (issue #546 AC5).
//
// Single catch-all Edge Function that:
//   1. Pulls the manifest from Edge Config.
//   2. Looks up the requested URL.
//   3. Pulls raw md + template, renders via WASM.
//   4. Returns with the per-route Cache-Control so Vercel's CDN
//      caches the response for `s-maxage` and serves stale-while-
//      revalidate for `swr`.
//
// Vercel's CDN handles the SWR semantics natively — no manual
// `cache.match` like the Cloudflare adapter. We just return the
// correct header and let the platform do its job.

import init, { render_page_isr } from "../wasm/ssg_wasm.js";
// @ts-expect-error — Vercel bundles the WASM via this import shape.
import wasmModule from "../wasm/ssg_wasm_bg.wasm?module";

import {
  cacheControlFor,
  getManifest,
  getSource,
} from "../lib/edge-config";

export const config = {
  runtime: "edge",
  // Pin to a regional list close to your readers; "global" is fine
  // for a smoke test.
  regions: ["iad1", "fra1", "sin1"],
};

let wasmReady: Promise<void> | null = null;
async function ensureWasm(): Promise<void> {
  if (!wasmReady) {
    wasmReady = init(wasmModule).then(() => undefined);
  }
  return wasmReady;
}

function normaliseUrl(pathname: string): string {
  if (pathname === "/") return "/index.html";
  if (pathname.endsWith("/")) return `${pathname}index.html`;
  if (!pathname.includes(".")) return `${pathname}/index.html`;
  return pathname;
}

export default async function handler(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);
  const url = normaliseUrl(pathname);

  const manifest = await getManifest();
  const entry = manifest.entries[url];
  if (!entry) {
    return new Response("Not found", { status: 404 });
  }

  const markdownKey = entry.sources.find((s) => s.startsWith("content/"));
  const templateKey =
    entry.sources.find(
      (s) =>
        s.startsWith("templates/") &&
        (s.endsWith("page.html") || s.endsWith("index.html")),
    ) ?? "templates/page.html";
  if (!markdownKey) {
    return new Response("Manifest entry has no content source", {
      status: 500,
    });
  }

  const [markdown, layout] = await Promise.all([
    getSource(markdownKey),
    getSource(templateKey),
  ]);
  if (markdown === null || layout === null) {
    return new Response("Source missing in Edge Config", { status: 502 });
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
