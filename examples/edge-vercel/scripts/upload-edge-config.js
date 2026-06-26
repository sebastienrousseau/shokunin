#!/usr/bin/env node
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Uploads dist/.ssg/manifest.json + every file under dist/.ssg/content/
// into Vercel Edge Config via the REST API.
//
// Env vars required:
//   VERCEL_TEAM_ID, VERCEL_TOKEN, EDGE_CONFIG_ID
//
// Usage:
//   node ./scripts/upload-edge-config.js [dist_dir]

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";
import process from "node:process";

const distDir = process.argv[2] || "../../dist";
const manifestPath = join(distDir, ".ssg", "manifest.json");
const contentRoot = join(distDir, ".ssg", "content");

const { VERCEL_TEAM_ID, VERCEL_TOKEN, EDGE_CONFIG_ID } = process.env;
for (const k of ["VERCEL_TEAM_ID", "VERCEL_TOKEN", "EDGE_CONFIG_ID"]) {
  if (!process.env[k]) {
    console.error(`missing env var: ${k}`);
    process.exit(1);
  }
}

const endpoint = `https://api.vercel.com/v1/edge-config/${EDGE_CONFIG_ID}/items?teamId=${VERCEL_TEAM_ID}`;

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const s = statSync(p);
    if (s.isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

function safeKey(rel) {
  return `ssg_content__${rel.replace(/[/.]/g, "_")}`;
}

const items = [
  {
    operation: "upsert",
    key: "ssg_manifest",
    value: JSON.parse(readFileSync(manifestPath, "utf8")),
  },
];

for (const f of walk(contentRoot)) {
  const rel = relative(contentRoot, f).split(sep).join("/");
  items.push({
    operation: "upsert",
    key: safeKey(rel),
    value: readFileSync(f, "utf8"),
  });
}

console.log(`upserting ${items.length} items to Edge Config ${EDGE_CONFIG_ID}…`);
const r = await fetch(endpoint, {
  method: "PATCH",
  headers: {
    Authorization: `Bearer ${VERCEL_TOKEN}`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({ items }),
});
if (!r.ok) {
  console.error(`upload failed: ${r.status}`, await r.text());
  process.exit(1);
}
console.log("done");
