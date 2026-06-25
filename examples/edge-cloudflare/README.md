# Cloudflare Workers — ISR adapter

Reference adapter for the ISR feature shipped in `ssg` 0.0.44 (issue
#546). Pairs a thin TypeScript Worker with the `render_page_isr` WASM
entry from `ssg-wasm` to serve dynamically-revalidated pages from
Cloudflare's edge.

## Architecture

```
┌───────── Worker (worker.ts) ───────────────────────────────┐
│  fetch(request, env, ctx)                                   │
│   1. cache.match(req) → fresh? return                       │
│   2. stale? serve stale + ctx.waitUntil(revalidate(...))    │
│   3. miss? block: render via WASM, cache, return            │
└─────────────────────────────────────────────────────────────┘
                │
                ▼
       KV Namespace: SSG_CONTENT
       Keys:
         manifest               → dist/.ssg/manifest.json bytes
         content/<path>.md      → raw markdown bytes
         templates/<name>.html  → raw template bytes
```

The Worker is stateless across invocations — every cache decision
flows through Cloudflare's Cache API or the KV store snapshot loaded
at startup.

## Quick start

```bash
# 1. Build your site with ISR enabled.
ssg build --isr -c content -t templates -o dist

# 2. Build + ship the WASM renderer.
cd crates/ssg-wasm
wasm-pack build --target web --release
wasm-opt -Oz -o ../../examples/edge-cloudflare/pkg/ssg_wasm_bg.wasm pkg/ssg_wasm_bg.wasm

# 3. Upload the manifest + raw sources to KV.
cd ../../examples/edge-cloudflare
wrangler kv key put --binding SSG_CONTENT manifest --path ../../dist/.ssg/manifest.json
find ../../dist/.ssg/content -type f | while read p; do
  rel=${p#../../dist/.ssg/content/}
  wrangler kv key put --binding SSG_CONTENT "$rel" --path "$p"
done

# 4. Deploy.
wrangler deploy
```

For a local smoke test (used in CI):

```bash
wrangler dev --local --port 8787
curl -s http://127.0.0.1:8787/index.html       # cache miss → WASM render
curl -sI http://127.0.0.1:8787/index.html | grep -i cache-control
curl -s http://127.0.0.1:8787/index.html       # cache hit → instant
```

## Files

- `worker.ts` — main fetch handler. Cache → KV → WASM → cache write.
- `wrangler.toml` — bindings: `SSG_CONTENT` KV namespace, WASM module.
- `package.json` — wrangler dev dependency, build scripts.
- `webhook.ts` — invalidation endpoint (POST `/__ssg/invalidate`).
- `tsconfig.json` — Worker-targeted TypeScript config.

## Cache headers

The Worker reads each entry's optional `cache` block from the manifest
and emits:

```
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

## Webhook (issue #546 AC8)

`POST /__ssg/invalidate` with body:

```json
{ "source": "content/posts/foo.md" }
```

Steps:

1. Consult the manifest to find every URL that lists `source` as a
   dependency (the post itself plus any tag / archive index).
2. Update the KV entry for the source with the new bytes.
3. Purge the affected URLs from Cloudflare's cache via the API.

The endpoint expects a bearer token in `Authorization` matching the
`SSG_WEBHOOK_TOKEN` secret. Production deployments should additionally
verify HMAC signatures from the upstream CMS.
