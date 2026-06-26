#!/usr/bin/env node
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Uploads dist/.ssg/manifest.json + every file under dist/.ssg/content/
// into the configured KV namespace via `wrangler kv key put`.
//
// Usage:
//   node ./scripts/upload-kv.js [dist_dir]
//
// `dist_dir` defaults to ../../dist (the SSG build output).

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

const distDir = process.argv[2] || "../../dist";
const manifestPath = join(distDir, ".ssg", "manifest.json");
const contentRoot = join(distDir, ".ssg", "content");

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const s = statSync(p);
    if (s.isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

function wranglerPut(key, path) {
  const r = spawnSync(
    "wrangler",
    ["kv", "key", "put", "--binding", "SSG_CONTENT", key, "--path", path],
    { stdio: "inherit" },
  );
  if (r.status !== 0) {
    console.error(`failed to upload ${key}`);
    process.exit(1);
  }
}

console.log(`uploading manifest: ${manifestPath}`);
wranglerPut("manifest", manifestPath);

const files = walk(contentRoot);
console.log(`uploading ${files.length} source files...`);
for (const f of files) {
  const key = relative(contentRoot, f).split(sep).join("/");
  wranglerPut(key, f);
}
console.log("done");
