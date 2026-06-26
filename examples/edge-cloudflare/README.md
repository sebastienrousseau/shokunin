<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">ssg-edge-cloudflare</h1>

<p align="center">
  Reference Cloudflare Workers adapter for SSG's Incremental Static
  Regeneration (ISR) pipeline. Pairs a stateless TypeScript Worker
  with the <code>render_page_isr</code> WASM entry from
  <code>ssg-wasm</code> to serve dynamically-revalidated pages from
  Cloudflare's edge.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/static-site-generator/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=for-the-badge" alt="License" /></a>
  <a href="https://developers.cloudflare.com/workers/"><img src="https://img.shields.io/badge/runtime-Cloudflare%20Workers-f38020?style=for-the-badge&logo=cloudflare&logoColor=white" alt="Cloudflare Workers" /></a>
  <a href="https://www.typescriptlang.org/"><img src="https://img.shields.io/badge/TypeScript-%5E5.3-3178c6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" /></a>
  <a href="https://developers.cloudflare.com/workers/wrangler/"><img src="https://img.shields.io/badge/Wrangler-%5E3.0-orange?style=for-the-badge" alt="Wrangler" /></a>
</p>

---

## Contents

- [Install](#install) — prerequisites and bindings
- [Quick Start](#quick-start) — local dev in 4 commands
- [Architecture](#architecture) — request lifecycle
- [Files](#files) — what each TypeScript module does
- [Configuration](#configuration) — KV namespace, secrets, cache headers
- [Examples](#examples) — ISR fetch, RPC call, invalidation webhook
- [Development](#development) — npm scripts and smoke test
- [Security](#security) — webhook auth and HMAC
- [License](#license)

---

## Install

```bash
# 1. Install JS dependencies.
npm install

# 2. Create the KV namespace and paste its `id` into wrangler.toml.
npx wrangler kv namespace create SSG_CONTENT

# 3. Set the webhook secret (required by the invalidation endpoint).
npx wrangler secret put SSG_WEBHOOK_TOKEN

# 4. (Optional) Cloudflare API credentials for cache purge on invalidate.
npx wrangler secret put CF_API_TOKEN
```

> **Note:** this adapter targets Wrangler 3 and the `nodejs_compat`
> compatibility flag. Node ≥ 20 is required by
> `@cloudflare/workers-types`.

---

## Quick Start

```bash
# Build your site with ISR enabled (from the SSG repo root).
ssg build --isr -c content -t templates -o dist

# Build the WASM renderer + drop it into ./pkg/.
npm run build:wasm

# Upload the manifest + raw sources to KV.
npm run kv:upload

# Deploy.
npm run deploy
```

---

## Architecture

```text
┌────────── Worker (worker.ts) ────────────────────────────────┐
│  fetch(request, env, ctx)                                     │
│   1. cache.match(req) → fresh? return                         │
│   2. stale? serve stale + ctx.waitUntil(revalidate(...))      │
│   3. miss?  render via WASM, cache, return                    │
│                                                                │
│  Mounted endpoints:                                            │
│   /__rpc/<fn>             → rpc.ts          (issue #548)      │
│   /__ssg/invalidate       → webhook.ts      (issue #546 AC8)  │
│   /<anything else>        → ISR render path (issue #546)      │
└───────────────────────────────────────────────────────────────┘
                │
                ▼
       KV Namespace: SSG_CONTENT
       Keys:
         manifest               → dist/.ssg/manifest.json bytes
         content/<path>.md      → raw markdown bytes
         templates/<name>.html  → raw template bytes
```

The Worker is **stateless across invocations** — every cache decision
flows through Cloudflare's Cache API or the KV-store snapshot loaded
at first call (`manifestCache` is per-isolate only).

---

## Files

| File | Purpose |
| :--- | :--- |
| `worker.ts` | Main `fetch` handler. Routes `/__rpc/*` → `handleRpc`, `/__ssg/invalidate` → `handleInvalidate`, everything else through the ISR cache → KV → WASM pipeline. |
| `rpc.ts` | Edge RPC dispatcher (issue #548). `POST /__rpc/<fn_name>` with a JSON body, returns the registered function's JSON output. Non-POST → 405, unknown name → 404, dynamic responses are `Cache-Control: no-store`. |
| `webhook.ts` | `POST /__ssg/invalidate` endpoint. Verifies a bearer token, updates the changed KV entry, walks the manifest's `sources → urls` reverse index, purges affected URLs via the Cloudflare API. |
| `wrangler.toml` | Bindings: `SSG_CONTENT` KV namespace, `CompiledWasm` rule for `pkg/*.wasm`, build hook that auto-rebuilds the WASM. |
| `package.json` | npm scripts (`dev`, `deploy`, `build:wasm`, `kv:upload`, `smoke`) and `wrangler` / `@cloudflare/workers-types` / `typescript` dev deps. |
| `tsconfig.json` | Worker-targeted TypeScript config (`module: "esnext"`, `lib: ["esnext"]`). |
| `scripts/upload-kv.js` | Node script that streams `dist/.ssg/{manifest.json,content/**,templates/**}` into KV with concurrency. |
| `scripts/smoke.js` | Cold-fetch + warm-fetch smoke test used in CI. |

---

## Configuration

### Cache headers

The Worker reads each entry's optional `cache` block from the manifest
and emits:

```text
Cache-Control: s-maxage=<s_maxage>, stale-while-revalidate=<swr>
```

When the entry omits `cache`, the manifest-wide `default_cache` is
used (default `s-maxage=60, stale-while-revalidate=86400`). Per-page
overrides come from markdown frontmatter:

```yaml
---
title: Hot page
isr:
  s_maxage: 600    # CDN may serve fresh for 10 min
  swr: 3600        # then serve stale for up to 1h while revalidating
---
```

### Environment / secrets

| Name | Required | Set via | Purpose |
| :--- | :---: | :--- | :--- |
| `SSG_CONTENT` | yes | `wrangler.toml` (`[[kv_namespaces]]`) | KV namespace ID for manifest + source bytes. |
| `SSG_WEBHOOK_TOKEN` | yes | `wrangler secret put SSG_WEBHOOK_TOKEN` | Bearer token verified by `webhook.ts`. |
| `CF_ACCOUNT_ID` | optional | `wrangler secret put …` | Cloudflare account for cache-purge-by-URL on invalidate. |
| `CF_ZONE_ID` | optional | `wrangler secret put …` | Cloudflare zone for cache-purge-by-URL on invalidate. |
| `CF_API_TOKEN` | optional | `wrangler secret put …` | API token (`Cache Purge` permission) for cache-purge-by-URL. |

---

## Examples

### Cold fetch → cache miss → WASM render

```bash
# Cache miss — Worker pulls from KV, renders via WASM, writes to cache.
curl -s http://127.0.0.1:8787/blog/hello.html

# Inspect the cache-control header.
curl -sI http://127.0.0.1:8787/blog/hello.html | grep -i cache-control
# → Cache-Control: s-maxage=60, stale-while-revalidate=86400

# Second fetch — served from cache, sub-millisecond.
curl -s http://127.0.0.1:8787/blog/hello.html
```

### Call an Edge RPC method (issue #548)

The Rust side registers each `#[ssg_rpc]` function into a static
dispatch table; the Worker mounts the dispatcher at `/__rpc/<fn_name>`.

```bash
# Successful call.
curl -sX POST http://127.0.0.1:8787/__rpc/echo \
  -H 'Content-Type: application/json' \
  -d '{"msg":"hi"}'
# → {"msg":"hi"}

# Non-POST → 405.
curl -sI http://127.0.0.1:8787/__rpc/echo
# → HTTP/1.1 405 Method Not Allowed

# Unknown method name → 404.
curl -sX POST http://127.0.0.1:8787/__rpc/does-not-exist -d 'null'
# → {"error":"unknown rpc"}
```

### Trigger an ISR invalidation (issue #546 AC8)

```bash
# Pull the secret you set during install into your shell.
export SSG_WEBHOOK_TOKEN="<the value you wrangler-secret-put>"

curl -sX POST http://127.0.0.1:8787/__ssg/invalidate \
  -H "Authorization: Bearer $SSG_WEBHOOK_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"source":"content/posts/foo.md"}'
# → {"invalidated":["/posts/foo.html","/tags/news.html"]}
```

The endpoint:

1. Consults the manifest to find every URL that lists `source` as a
   dependency (the post itself plus any tag / archive index).
2. Updates the KV entry for the source with the new bytes.
3. Purges the affected URLs from Cloudflare's cache via the API.

Production deployments should additionally verify HMAC signatures
from the upstream CMS — the bearer-token check is the minimum viable
bar.

---

## Development

```bash
npm run dev      # wrangler dev --local --port 8787
npm run smoke    # cold + warm fetch, asserts Cache-Control
npm run deploy   # wrangler deploy
```

Local edits to `worker.ts` / `rpc.ts` / `webhook.ts` hot-reload via
Wrangler. Changes to `crates/ssg-wasm/src/**` rebuild the WASM via
the `[build]` hook in `wrangler.toml`.

---

## Security

- `SSG_WEBHOOK_TOKEN` enforced on every `POST /__ssg/invalidate`
  request.
- RPC responses are `Cache-Control: no-store` — never cached at any
  edge.
- The Worker carries no `eval` / dynamic-import surface; the WASM
  module is the only non-TypeScript code path.
- Cloudflare KV is the single source of truth for content; the
  Worker's in-isolate `manifestCache` is a per-isolate read-through
  only.
- Edge CPU budget pinned at `cpu_ms = 50` (Free tier p99 ceiling).
  ISR renders typically come in under 5 ms.

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option. See the
repo root `LICENSE-APACHE` / `LICENSE-MIT`.
