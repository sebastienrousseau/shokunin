// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Edge RPC dispatcher (issue #548).
//
// Wire contract:
//   POST /__rpc/<fn_name>   body: JSON   → 200 { ...output } | 4xx/5xx { error }
//   <any other method>      → 405 { "error": "method not allowed" }
//   POST /__rpc/<unknown>   → 404 { "error": "unknown rpc" }
//
// The Rust side (ssg-rpc + ssg-rpc-macro) registers each
// `#[ssg_rpc]` function into a static dispatch table; the WASM
// module re-exports a single `rpc_dispatch(name, payload)` entry
// point that returns an envelope `{ status, body }`.
//
// This file is the HTTP-framing layer that:
//   1. Validates the method (POST only — AC6).
//   2. Strips the `/__rpc/` prefix.
//   3. Calls into WASM.
//   4. Hands back the response with correct Content-Type +
//      no-cache headers (RPC responses must not be cached at
//      any edge — they are dynamic by definition).

// @ts-expect-error — bundled by Wrangler alongside worker.ts.
import { rpc_dispatch } from "./pkg/ssg_wasm.js";

/** URL prefix that mounts the RPC dispatcher. */
export const RPC_PREFIX = "/__rpc/";

interface RpcEnvelope {
  status: number;
  body: string;
}

const JSON_HEADERS: HeadersInit = {
  "Content-Type": "application/json; charset=utf-8",
  // RPC responses are dynamic — caching them at the edge would
  // create stale-result hazards.
  "Cache-Control": "no-store",
};

/**
 * Dispatch an incoming request against the registered RPCs.
 *
 * @param request the incoming `Request` (must be POST — AC6).
 * @param pathname the URL pathname (callers pre-extract this so we
 *                 don't re-parse the URL).
 */
export async function handleRpc(
  request: Request,
  pathname: string,
): Promise<Response> {
  // AC6 — POST only. GET / PUT / DELETE all map to 405.
  if (request.method !== "POST") {
    return new Response(
      JSON.stringify({ error: "method not allowed" }),
      { status: 405, headers: JSON_HEADERS },
    );
  }

  // AC3 — Unknown / empty name → 404. We do NOT enumerate the
  // registered names; the wire body is the terse string the JS
  // client expects.
  const name = pathname.slice(RPC_PREFIX.length);
  if (name.length === 0) {
    return new Response(
      JSON.stringify({ error: "unknown rpc" }),
      { status: 404, headers: JSON_HEADERS },
    );
  }

  // AC2 — Single dispatcher. Body must be JSON (or empty).
  let payload = "";
  if (request.headers.get("content-length") !== "0") {
    try {
      payload = await request.text();
      if (payload.length === 0) {
        payload = "null";
      }
    } catch (e) {
      return new Response(
        JSON.stringify({ error: `bad request: ${(e as Error).message}` }),
        { status: 400, headers: JSON_HEADERS },
      );
    }
  } else {
    payload = "null";
  }

  // Cross into WASM. The envelope is JSON-as-a-string in `body`
  // (the Rust side splices it in verbatim, so it's already a
  // well-formed JSON document).
  const envelopeJson = rpc_dispatch(name, payload) as string;
  let envelope: RpcEnvelope;
  try {
    envelope = JSON.parse(envelopeJson) as RpcEnvelope;
  } catch (e) {
    // This would imply the Rust dispatcher returned a malformed
    // envelope, which is a bug — surface it as a 500 rather than
    // silently dropping.
    return new Response(
      JSON.stringify({
        error: `internal error: malformed dispatcher envelope: ${(e as Error).message}`,
      }),
      { status: 500, headers: JSON_HEADERS },
    );
  }

  // AC3 — Map the Rust-side 404 ("not found") to the wire string
  // the client expects when the name isn't registered.
  if (envelope.status === 404) {
    return new Response(
      JSON.stringify({ error: "unknown rpc" }),
      { status: 404, headers: JSON_HEADERS },
    );
  }

  return new Response(envelope.body, {
    status: envelope.status,
    headers: JSON_HEADERS,
  });
}
