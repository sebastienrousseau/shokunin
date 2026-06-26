// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Vercel invalidation webhook (issue #546 AC8).
//
// POST /__ssg/invalidate
// Authorization: Bearer <SSG_WEBHOOK_TOKEN>
// Body: { "source": "content/posts/foo.md" }
//
// Consults the manifest for every URL that depends on `source`,
// then revalidates each via the Vercel Bypass-Token / on-demand
// ISR API.

import { getManifest } from "../../lib/edge-config";

export const config = {
  runtime: "edge",
};

export default async function handler(request: Request): Promise<Response> {
  if (request.method !== "POST") {
    return new Response("method not allowed", { status: 405 });
  }

  const token = process.env.SSG_WEBHOOK_TOKEN ?? "";
  const provided = (request.headers.get("Authorization") ?? "").replace(
    /^Bearer\s+/i,
    "",
  );
  if (!token || provided !== token) {
    return new Response("unauthorised", { status: 401 });
  }

  let body: { source?: string };
  try {
    body = (await request.json()) as { source?: string };
  } catch {
    return new Response("invalid JSON", { status: 400 });
  }
  if (!body.source) {
    return new Response("missing 'source'", { status: 400 });
  }

  const manifest = await getManifest();
  const affected = Object.entries(manifest.entries)
    .filter(([, e]) => e.sources.includes(body.source!))
    .map(([url]) => url);

  // Trigger Vercel on-demand revalidation for each affected URL.
  // The platform supports POST /api/revalidate?path=...&secret=...
  // when configured with VERCEL_REVALIDATE_SECRET.
  const purge = await Promise.all(
    affected.map(async (path) => {
      const r = await fetch(
        `${process.env.VERCEL_URL ?? "http://localhost:3000"}/api/revalidate?path=${encodeURIComponent(path)}&secret=${process.env.VERCEL_REVALIDATE_SECRET}`,
        { method: "POST" },
      );
      return { path, status: r.status };
    }),
  );

  return new Response(
    JSON.stringify({ ok: true, source: body.source, invalidated: purge }),
    {
      status: 200,
      headers: { "Content-Type": "application/json" },
    },
  );
}
