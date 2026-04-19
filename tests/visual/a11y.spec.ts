// SPDX-License-Identifier: Apache-2.0 OR MIT

import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import * as fs from "fs";
import * as path from "path";

const pages = [
  { name: "index", path: "/" },
  { name: "contact", path: "/contact/index.html" },
];

for (const { name, path: pagePath } of pages) {
  test(`${name} — WCAG 2.1 AA audit`, async ({ page }) => {
    await page.goto(pagePath, { waitUntil: "networkidle" });

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();

    // Write JSON report for CI artifact upload
    const reportDir = path.join(__dirname, "a11y-reports");
    fs.mkdirSync(reportDir, { recursive: true });
    fs.writeFileSync(
      path.join(reportDir, `${name}.json`),
      JSON.stringify(results, null, 2),
    );

    // Log violations for debugging
    if (results.violations.length > 0) {
      console.log(`\n❌ ${name}: ${results.violations.length} violation(s)`);
      for (const v of results.violations) {
        console.log(`  [${v.impact}] ${v.id}: ${v.description}`);
        for (const node of v.nodes) {
          console.log(`    → ${node.html.substring(0, 120)}`);
        }
      }
    } else {
      console.log(`✅ ${name}: 0 violations`);
    }

    expect(
      results.violations,
      `${name} has WCAG 2.1 AA violations`,
    ).toEqual([]);
  });
}
