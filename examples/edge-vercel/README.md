<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">ssg-edge-vercel</h1>

<p align="center">
  Reference Vercel Edge adapter for SSG's Incremental Static
  Regeneration (ISR) pipeline. Pairs a single catch-all Edge Function
  with the <code>render_page_isr</code> WASM entry from
  <code>ssg-wasm</code> to serve dynamically-revalidated pages from
  Vercel's CDN.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/static-site-generator/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=for-the-badge" alt="License" /></a>
  <a href="https://vercel.com/docs/functions/edge-functions"><img src="https://img.shields.io/badge/runtime-Vercel%20Edge-000000?style=for-the-badge&logo=vercel&logoColor=white" alt="Vercel Edge" /></a>
  <a href="https://www.typescriptlang.org/"><img src="https://img.shields.io/badge/TypeScript-%5E5.3-3178c6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" /></a>
  <a href="https://vercel.com/docs/cli"><img src="https://img.shields.io/badge/Vercel%20CLI-%5E33.0-black?style=for-the-badge" alt="Vercel CLI" /></a>
</p>

---

## Contents

- [Install](#install) — prerequisites and Edge Config setup
- [Quick Start](#quick-start) — local dev in 4 commands
- [Architecture](#architecture) — request lifecycle and routing
- [Files](#files) — what each TypeScript module does
- [Configuration](#configuration) — Edge Config, regions, cache headers
- [Examples](#examples) — ISR fetch and invalidation webhook
- [Development](#development) — npm scripts and local dev
- [Security](#security) — webhook auth and Edge Config tokens
- [Sizing](#sizing) — when to swap Edge Config for Blob / Upstash
- [License](#license)

---

## Install

```bash
# 1. Install JS dependencies.
npm install

# 2. Authenticate with Vercel and link the project.
npx vercel login
npx vercel link

# 3. Create an Edge Config store and connect it to the project
#    (Dashboard: Storage → Edge Config → Create). The integration
#    auto-injects the EDGE_CONFIG env var at runtime.

# 4. Set the webhook secret used by api/__ssg/invalidate.ts.
npx vercel env add SSG_WEBHOOK_TOKEN
```

> **Note:** this adapter targets the Vercel Edge runtime (V8 isolate,
> no Node APIs). The `@vercel/edge-config` SDK reads Edge Config at
> sub-millisecond latency from every region.

---

## Quick Start

```bash
# Build your site with ISR enabled (from the SSG repo root).
ssg build --isr -c content -t templates -o dist

# Build the WASM renderer + drop it into ./wasm/.
npm run build:wasm

# Populate Edge Config with the manifest + sources.
npm run config:upload

# Deploy.
npm run deploy
```

---

## Architecture

```text
┌─────── Edge Function (api/[...path].ts) ─────────────────────┐
│  default async handler(request)                                │
│   1. Normalise the URL (/ → /index.html, /foo/ → /foo/index)   │
│   2. Pull the manifest from Edge Config (sub-ms).              │
│   3. Pull raw md + template (Edge Config or Upstash fallback). │
│   4. Render via WASM (`render_page_isr`).                      │
│   5. Return 200 + Cache-Control + X-SSG-Hash + X-SSG-Build.    │
│  Vercel's CDN handles the SWR semantics natively — no manual   │
│  `cache.match()` needed.                                       │
└────────────────────────────────────────────────────────────────┘
                │
                ▼
       Edge Config: ssg_manifest, ssg_content/<key>
       (See https://vercel.com/docs/storage/edge-config)
```

### Routing (`vercel.json`)

```json
{
  "rewrites": [
    { "source": "/__ssg/invalidate", "destination": "/api/__ssg/invalidate" },
    { "source": "/(.*)",              "destination": "/api/$1" }
  ]
}
```

Without these rewrites, requests to `/blog/hello.html` would 404 —
Vercel only routes `/api/**` to functions by default. The catch-all
rewrite sends every other path through the ISR handler.

---

## Files

| File | Purpose |
| :--- | :--- |
| `api/[...path].ts` | Main Edge Function. Manifest lookup → Edge Config fetch → WASM render → response with `Cache-Control`. Pinned to `iad1, fra1, sin1` regions by default. |
| `api/__ssg/invalidate.ts` | Invalidation webhook (issue #546 AC8). Verifies a bearer token, patches the changed Edge Config entry, returns the affected URL list. |
| `lib/edge-config.ts` | `ContentProvider`-shaped helper around the `@vercel/edge-config` SDK. Exports `getManifest`, `getSource`, `cacheControlFor`. |
| `vercel.json` | Function regions, runtime (`edge`), memory (128 MB), and the catch-all rewrite. |
| `package.json` | npm scripts (`dev`, `deploy`, `build:wasm`, `config:upload`) and dev dependencies. |
| `tsconfig.json` | Edge-runtime-targeted TypeScript (`module: "esnext"`, no Node lib). |
| `scripts/upload-edge-config.js` | Node script that streams `dist/.ssg/{manifest.json,content/**,templates/**}` into Edge Config via the Vercel API. |

---

## Configuration

### Cache headers

Same shape as the Cloudflare adapter — per-entry `cache` block from
the manifest with a `default_cache` fallback. Vercel's CDN honours
`stale-while-revalidate` natively.

```yaml
---
title: Hot page
isr:
  s_maxage: 600    # CDN may serve fresh for 10 min
  swr: 3600        # then serve stale for up to 1h while revalidating
---
```

### Regions

`api/[...path].ts` pins to three regions by default:

```ts
export const config = {
  runtime: "edge",
  regions: ["iad1", "fra1", "sin1"], // US East, Frankfurt, Singapore
};
```

Replace with `"all"` for full global coverage or a tighter list for
cost control. See [Vercel's region list](https://vercel.com/docs/edge-network/regions).

### Environment / secrets

| Name | Required | Set via | Purpose |
| :--- | :---: | :--- | :--- |
| `EDGE_CONFIG` | yes | Vercel integration (auto-injected) | Connection string for the `@vercel/edge-config` SDK. |
| `SSG_WEBHOOK_TOKEN` | yes | `vercel env add SSG_WEBHOOK_TOKEN` | Bearer token verified by `api/__ssg/invalidate.ts`. |
| `VERCEL_API_TOKEN` | optional | `vercel env add VERCEL_API_TOKEN` | API token used by `scripts/upload-edge-config.js` to write Edge Config from CI. |

---

## Examples

### ISR fetch

```bash
# After `vercel deploy`, requests hit the catch-all rewrite → Edge
# Function → WASM render. The first hit is a cold cache miss; the
# next request to the same URL is served from Vercel's CDN.

curl -s https://<your-project>.vercel.app/blog/hello.html
curl -sI https://<your-project>.vercel.app/blog/hello.html | \
  grep -iE 'cache-control|x-ssg'
# → Cache-Control: s-maxage=60, stale-while-revalidate=86400
# → X-SSG-Hash:    9f3c1a7d…
# → X-SSG-Build:   2026-06-25T22:34:11Z
```

### Trigger an invalidation (issue #546 AC8)

```bash
# Pull the secret you set during install into your shell.
export SSG_WEBHOOK_TOKEN="<the value you vercel-env-add>"

curl -sX POST https://<your-project>.vercel.app/__ssg/invalidate \
  -H "Authorization: Bearer $SSG_WEBHOOK_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"source":"content/posts/foo.md"}'
# → {"invalidated":["/posts/foo.html","/tags/news.html"]}
```

The handler:

1. Resolves the manifest's `sources → urls` reverse index.
2. Patches the Edge Config entry for the changed source.
3. Returns the list of URLs Vercel's CDN should revalidate on next
   hit.

---

## Development

```bash
npm run dev              # vercel dev --listen 3000
npm run config:upload    # node ./scripts/upload-edge-config.js
npm run deploy           # vercel deploy --prod
```

`vercel dev` runs the Edge Function locally against the production
Edge Config store (or a `.env.local` override). Hot reload is on by
default.

---

## Security

- `SSG_WEBHOOK_TOKEN` enforced on every `POST /__ssg/invalidate`
  request.
- Edge Config tokens are read-only from the runtime; writes (from
  `scripts/upload-edge-config.js`) go through a separate write-scoped
  `VERCEL_API_TOKEN`.
- The Edge Function carries no `eval` / dynamic-import surface; the
  WASM module is the only non-TypeScript code path.
- Memory pinned at `128` MB (`vercel.json`) — Vercel's edge runtime
  hard-caps single-request memory.

---

## Sizing

| Corpus size | Recommended backing store | Reason |
| :--- | :--- | :--- |
| ≤ 512 KB total (Hobby) | Edge Config | Sub-ms reads from every region; zero extra setup. |
| ≤ 1 MB total (Pro)     | Edge Config | Same as above with the paid-plan ceiling. |
| > 1 MB total           | Upstash Redis or Vercel Blob (TODO) | Edge Config max key/value sizes will start to bite. Swap by reimplementing `getManifest` / `getSource` in `lib/edge-config.ts` — the rest of `api/[...path].ts` is provider-agnostic. |

The Upstash Redis path is a planned follow-up; today only the Edge
Config backend ships.

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option. See the
repo root `LICENSE-APACHE` / `LICENSE-MIT`.
