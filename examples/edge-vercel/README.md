# Vercel Edge — ISR adapter

Reference adapter for the ISR feature shipped in `ssg` 0.0.44 (issue
#546 AC5). Pairs a thin Vercel Edge Function with the `render_page_isr`
WASM entry from `ssg-wasm` to serve dynamically-revalidated pages.

## Architecture

```
┌───────── Edge Function (api/[...path].ts) ─────────────────┐
│  default async (req: NextRequest)                           │
│   1. Look up the URL in Vercel's data cache (s-maxage).     │
│   2. Pull the manifest + raw md/template from Edge Config.  │
│   3. Render via WASM, return with Cache-Control.            │
└─────────────────────────────────────────────────────────────┘
                │
                ▼
       Edge Config: ssg_manifest, ssg_content/<key>
       (See https://vercel.com/docs/storage/edge-config)
```

Vercel's Edge Config supports up to ~512 KB per project — fine for
the manifest plus a modest content corpus. Larger sites should swap
to a Blob / KV-equivalent (Upstash Redis is a common pick) by
implementing the `ContentProvider`-equivalent helper in
`edge-config.ts`.

## Quick start

```bash
# 1. Build your site with ISR enabled.
ssg build --isr -c content -t templates -o dist

# 2. Build the WASM renderer.
cd crates/ssg-wasm
wasm-pack build --target web --release
wasm-opt -Oz -o ../../examples/edge-vercel/wasm/ssg_wasm_bg.wasm pkg/ssg_wasm_bg.wasm

# 3. Populate Edge Config via the Vercel API.
node ../../examples/edge-vercel/scripts/upload-edge-config.js

# 4. Deploy.
cd ../../examples/edge-vercel && vercel deploy
```

## Files

- `api/[...path].ts` — main Edge Function. Cache-Control + WASM render.
- `api/__ssg/invalidate.ts` — invalidation webhook (AC8).
- `lib/edge-config.ts` — `ContentProvider`-shaped Edge Config helper.
- `vercel.json` — function regions + runtime config.
- `package.json` — dev dependencies.
- `tsconfig.json` — Edge-runtime-targeted TypeScript config.
