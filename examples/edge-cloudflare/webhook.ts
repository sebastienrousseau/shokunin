// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Cloudflare Workers invalidation webhook (issue #546 AC8).
//
// POST /__ssg/invalidate
// Authorization: Bearer <SSG_WEBHOOK_TOKEN>
// Content-Type: application/json
// Body: { "source": "content/posts/foo.md", "bytes": "<base64 optional>" }
//
// 1. Verifies bearer token.
// 2. If `bytes` present: updates KV under the `source` key.
// 3. Consults the manifest for every URL that depends on `source`.
// 4. Purges each URL from Cloudflare's cache via the API.

import type { Env } from "./worker";

interface InvalidatePayload {
  source: string;
  /** Optional new content as base64. Caller may also re-deploy KV out-of-band. */
  bytes?: string;
}

interface ManifestEntry {
  sources: string[];
  hash: string;
}

interface Manifest {
  entries: Record<string, ManifestEntry>;
}

function unauthorised(): Response {
  return new Response("unauthorised", { status: 401 });
}

function badRequest(detail: string): Response {
  return new Response(`bad request: ${detail}`, { status: 400 });
}

/**
 * Routes `POST /__ssg/invalidate`. The verb check is performed by the
 * caller so this handler can also be invoked programmatically.
 */
export async function handleInvalidate(
  request: Request,
  env: Env,
): Promise<Response> {
  // 1. Auth.
  const token = env.SSG_WEBHOOK_TOKEN ?? "";
  const provided = (request.headers.get("Authorization") ?? "").replace(
    /^Bearer\s+/i,
    "",
  );
  if (!token || provided !== token) {
    return unauthorised();
  }

  // 2. Parse.
  let body: InvalidatePayload;
  try {
    body = (await request.json()) as InvalidatePayload;
  } catch {
    return badRequest("invalid JSON");
  }
  if (!body.source || typeof body.source !== "string") {
    return badRequest("missing 'source'");
  }

  // 3. Update KV (if caller supplied fresh bytes).
  if (body.bytes) {
    try {
      const decoded = atob(body.bytes);
      await env.SSG_CONTENT.put(body.source, decoded);
    } catch (e) {
      return badRequest(`bytes decode: ${(e as Error).message}`);
    }
  }

  // 4. Find affected URLs via the manifest.
  const manifestJson = await env.SSG_CONTENT.get("manifest");
  if (!manifestJson) return new Response("manifest missing", { status: 502 });
  const manifest = JSON.parse(manifestJson) as Manifest;
  const affected = Object.entries(manifest.entries)
    .filter(([, e]) => e.sources.includes(body.source))
    .map(([url]) => url);

  // 5. Purge from CDN.
  const purgeResult = await purgeUrls(affected, env);

  return new Response(
    JSON.stringify({
      ok: true,
      source: body.source,
      invalidated: affected,
      purge: purgeResult,
    }),
    {
      status: 200,
      headers: { "Content-Type": "application/json" },
    },
  );
}

interface PurgeResult {
  attempted: number;
  purged: number;
  errors: string[];
}

/**
 * Issues a Cloudflare cache purge for the supplied URL list. Returns
 * a structured summary so callers (and test fixtures) can assert on
 * the partial success path without re-implementing the API contract.
 */
export async function purgeUrls(
  urls: string[],
  env: Env,
): Promise<PurgeResult> {
  const result: PurgeResult = {
    attempted: urls.length,
    purged: 0,
    errors: [],
  };

  if (urls.length === 0) return result;
  if (!env.CF_ZONE_ID || !env.CF_API_TOKEN) {
    result.errors.push(
      "CF_ZONE_ID or CF_API_TOKEN unset — purge skipped (dev mode)",
    );
    return result;
  }

  const endpoint = `https://api.cloudflare.com/client/v4/zones/${env.CF_ZONE_ID}/purge_cache`;
  // Cloudflare's purge API accepts up to 30 URLs per call.
  const batches: string[][] = [];
  for (let i = 0; i < urls.length; i += 30) {
    batches.push(urls.slice(i, i + 30));
  }

  for (const batch of batches) {
    const r = await fetch(endpoint, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${env.CF_API_TOKEN}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ files: batch }),
    });
    if (r.ok) {
      result.purged += batch.length;
    } else {
      result.errors.push(`purge batch failed: ${r.status} ${r.statusText}`);
    }
  }

  return result;
}
