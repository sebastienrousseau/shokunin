#!/usr/bin/env node
/**
 * WCAG 2.1 AA audit of a generated site, via axe-core in Playwright.
 *
 * Replaces the pa11y toolchain that used to live in tests/a11y. That
 * toolchain pulled in puppeteer, which pulls in @puppeteer/browsers, which
 * pulls in extract-zip — the source of five open high-severity Dependabot
 * alerts, one of which (GHSA-jmr9-qjv8-65gv, unvalidated symlink path
 * traversal) has no patched release at all. The only fix npm could offer
 * was downgrading pa11y across a major version, trading one unmaintained
 * browser stack for an older one.
 *
 * This runs on @axe-core/playwright, which tests/visual already depended
 * on for a11y.spec.ts, so the audit gained no new dependencies and the
 * vulnerable subtree left the repository entirely.
 *
 * Two deliberate differences from the job this replaces:
 *
 *   - Colour contrast is checked. The pa11y config ignored both 1.4.3
 *     rules (G18 and G145), so the gate could not see a contrast failure.
 *   - A crashed run fails. The old loop matched 'Error:' in pa11y's output
 *     and printed "skipped", so a browser that would not start was
 *     indistinguishable from a clean pass.
 *
 * Usage:  node audit-site.mjs --base http://localhost:8787 [--dir <site>]
 *
 * Pages are discovered the same way the old job discovered them: the root
 * index.html plus every immediate subdirectory's index.html.
 */
import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const arg = (name, fallback) =>
  process.argv.includes(name)
    ? process.argv[process.argv.indexOf(name) + 1]
    : fallback;

const BASE = arg('--base', 'http://localhost:8787');
const DIR = arg('--dir', null);
const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

/** Root index.html plus each immediate subdirectory's index.html. */
function discover(dir) {
  const out = [];
  if (existsSync(join(dir, 'index.html'))) out.push('/index.html');
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    if (existsSync(join(dir, entry.name, 'index.html')))
      out.push(`/${entry.name}/index.html`);
  }
  return out.sort();
}

const paths = DIR ? discover(DIR) : ['/index.html'];
if (paths.length === 0) {
  console.error(`audit-site: no index.html found under ${DIR}`);
  process.exit(1);
}

const browser = await chromium.launch({
  args: ['--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage'],
});
// AxeBuilder requires a page from an explicit context, not browser.newPage().
const context = await browser.newContext();
const page = await context.newPage();

const findings = [];
for (const path of paths) {
  const res = await page.goto(`${BASE}${path}`, { waitUntil: 'networkidle' });
  if (!res || res.status() >= 400) {
    findings.push({ path, id: 'http', impact: 'error', help: `status ${res?.status()}`, nodes: [] });
    continue;
  }
  const { violations } = await new AxeBuilder({ page }).withTags(TAGS).analyze();
  for (const v of violations) {
    findings.push({
      path, id: v.id, impact: v.impact, help: v.help,
      nodes: v.nodes.map((n) => n.target.join(' ')),
    });
  }
}
await browser.close();

if (findings.length === 0) {
  console.log(`a11y: ${paths.length} page(s), WCAG 2.1 AA — 0 violations`);
  process.exit(0);
}

console.log(`a11y: ${findings.length} violation(s) across ${paths.length} page(s)\n`);
for (const f of findings) {
  console.log(`::error::[${f.impact}] ${f.id} in ${f.path} — ${f.help}`);
  for (const n of f.nodes.slice(0, 4)) console.log(`    ${n}`);
  if (f.nodes.length > 4) console.log(`    … and ${f.nodes.length - 4} more`);
}
process.exit(1);
