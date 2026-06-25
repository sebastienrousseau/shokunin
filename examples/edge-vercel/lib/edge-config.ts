// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// ContentProvider-equivalent helpers for Vercel Edge Config.
//
// Edge Config is a globally-replicated KV store optimised for
// read-heavy workloads (~15ms p99 reads from the function). We
// expose three calls that mirror the Rust ContentProvider trait so
// the same JS adapter shape can be ported to Upstash, R2-via-fetch,
// or any other KV backing without touching the renderer.

import { get } from "@vercel/edge-config";

export interface ManifestEntry {
  sources: string[];
  hash: string;
  cache?: { s_maxage: number; swr: number };
}

export interface Manifest {
  version: number;
  generated_at: string;
  default_cache: { s_maxage: number; swr: number };
  entries: Record<string, ManifestEntry>;
}

/** Fetches the build manifest (cached in-process per cold start). */
let manifestCache: Manifest | null = null;
export async function getManifest(): Promise<Manifest> {
  if (manifestCache) return manifestCache;
  const m = await get<Manifest>("ssg_manifest");
  if (!m) throw new Error("Edge Config: ssg_manifest missing");
  manifestCache = m;
  return m;
}

/**
 * Fetches a single source by key (`content/posts/foo.md`,
 * `templates/page.html`). Returns null on miss so callers can decide
 * between 404 and 502.
 */
export async function getSource(key: string): Promise<string | null> {
  // Edge Config keys must match `[A-Za-z0-9_-]+`, so slashes are
  // encoded to `__` and `.` to `_`.
  const safeKey = `ssg_content__${key.replace(/[/.]/g, "_")}`;
  const v = await get<string>(safeKey);
  return v ?? null;
}

/**
 * Renders the per-URL Cache-Control value from a manifest entry,
 * falling back to the manifest-wide default.
 */
export function cacheControlFor(
  entry: ManifestEntry,
  manifest: Manifest,
): string {
  const c = entry.cache ?? manifest.default_cache;
  return `s-maxage=${c.s_maxage}, stale-while-revalidate=${c.swr}`;
}
