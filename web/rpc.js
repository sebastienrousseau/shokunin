// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Edge RPC client (issue #548).
//
// Wraps `fetch` with JSON ser/deser + 405/404/500 surfacing. The
// generated TypeScript declarations in `dist/.ssg/rpc.d.ts` give
// `createRpc<Rpc>()` end-to-end type safety; this file is the
// runtime that backs them.
//
// Usage (JS):
//
//   import { createRpc } from "/web/rpc.js";
//   const rpc = createRpc();
//   const out = await rpc.like_post({ post_id: "x" });
//
// Usage (TS):
//
//   import { createRpc } from "/web/rpc.js";
//   import type { Rpc } from "./.ssg/rpc.d.ts";
//   const rpc: Rpc = createRpc();
//
// The bundle stays < 1 KB minified — there is no JSON Schema
// validation on the wire, no class hierarchies, no transports
// other than `fetch`.

/**
 * Error thrown when an RPC call fails. The `status` field mirrors
 * the HTTP status from the Edge Worker; `body` is the parsed
 * response body (typically `{ error: "..." }`).
 */
export class RpcError extends Error {
  constructor(status, body, name) {
    super(`rpc ${name} failed: ${status} ${body && body.error ? body.error : ""}`.trim());
    this.status = status;
    this.body = body;
    this.rpcName = name;
  }
}

/**
 * Create a Proxy-backed RPC client.
 *
 * @param {object} [opts]
 * @param {string} [opts.baseUrl] base origin (default: same origin).
 * @param {string} [opts.prefix]  URL prefix (default: "/__rpc/").
 * @param {typeof fetch} [opts.fetch] custom fetch (default: global).
 * @param {HeadersInit} [opts.headers] extra request headers.
 */
export function createRpc(opts) {
  const o = opts || {};
  const baseUrl = o.baseUrl || "";
  const prefix = o.prefix || "/__rpc/";
  const f = o.fetch || (typeof fetch !== "undefined" ? fetch : null);
  if (!f) throw new Error("createRpc: no fetch available");
  const extra = o.headers || {};

  return new Proxy({}, {
    get(_target, name) {
      if (typeof name !== "string") return undefined;
      return async (input) => {
        const url = baseUrl + prefix + name;
        const body = JSON.stringify(input === undefined ? null : input);
        const res = await f(url, {
          method: "POST",
          headers: Object.assign(
            { "Content-Type": "application/json" },
            extra,
          ),
          body,
        });
        const text = await res.text();
        let parsed = null;
        if (text.length > 0) {
          try { parsed = JSON.parse(text); }
          catch (_) { parsed = text; }
        }
        if (!res.ok) throw new RpcError(res.status, parsed, name);
        return parsed;
      };
    },
  });
}

export default createRpc;
