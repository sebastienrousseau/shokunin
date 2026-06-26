#!/usr/bin/env node
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Smoke-test fixture for the Cloudflare ISR Worker. Assumes
// `wrangler dev --local --port 8787` is already running.
//
// AC4 coverage: first request hits a cold path → WASM render →
// cached response; second hit is served from cache (no re-render).
// Measures p99-ish wall-clock to spot regressions.

const url = process.argv[2] || "http://127.0.0.1:8787/index.html";
const iterations = Number(process.argv[3] || 20);

async function time(label, fn) {
  const t0 = performance.now();
  const r = await fn();
  const dt = performance.now() - t0;
  console.log(`${label}: ${dt.toFixed(1)}ms (status ${r.status})`);
  return { dt, status: r.status };
}

const cold = await time("cold", () =>
  fetch(url, { headers: { "Cache-Control": "no-cache" } }),
);
const warmTimings = [];
for (let i = 0; i < iterations; i++) {
  const r = await time(`warm[${i}]`, () => fetch(url));
  warmTimings.push(r.dt);
}
warmTimings.sort((a, b) => a - b);
const p99 = warmTimings[Math.floor(warmTimings.length * 0.99)] || warmTimings.at(-1);
console.log(`cold: ${cold.dt.toFixed(1)}ms / warm p99: ${p99.toFixed(1)}ms`);

if (cold.status !== 200 || warmTimings.some((_, i) => warmTimings[i] === undefined)) {
  process.exit(1);
}
