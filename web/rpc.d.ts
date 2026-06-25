// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Type declarations for `web/rpc.js` (issue #548).
//
// The per-site `dist/.ssg/rpc.d.ts` file (emitted by the
// `RpcSchemaPlugin` from your registered `#[ssg_rpc]` functions)
// declares the `Rpc` interface. This file describes the *runtime*
// (`createRpc`, `RpcError`) without binding to any particular set
// of RPCs — they're plugged in by the user-supplied generic.

/** Error thrown by an RPC call that returned a non-2xx response. */
export class RpcError<TBody = unknown> extends Error {
  readonly status: number;
  readonly body: TBody | null;
  readonly rpcName: string;
  constructor(status: number, body: TBody | null, name: string);
}

/** Options accepted by `createRpc`. */
export interface CreateRpcOptions {
  /** Base origin. Defaults to "" (same origin). */
  baseUrl?: string;
  /** URL prefix to mount the dispatcher at. Defaults to `/__rpc/`. */
  prefix?: string;
  /** Custom fetch implementation. Defaults to the global. */
  fetch?: typeof fetch;
  /** Extra headers to attach to every request. */
  headers?: HeadersInit;
}

/**
 * Create a typed RPC client. Pass the `Rpc` interface generated
 * into `dist/.ssg/rpc.d.ts` as the type parameter:
 *
 * ```ts
 * import { createRpc } from "/web/rpc.js";
 * import type { Rpc } from "./.ssg/rpc.d.ts";
 * const rpc = createRpc<Rpc>();
 * ```
 */
export function createRpc<TRpc = Record<string, (input: unknown) => Promise<unknown>>>(
  opts?: CreateRpcOptions,
): TRpc;

export default createRpc;
