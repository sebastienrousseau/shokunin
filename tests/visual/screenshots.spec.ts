import { test, expect } from "@playwright/test";

/**
 * Visual regression tests for SSG example site.
 *
 * Screenshots are compared against committed baselines in
 * tests/visual-baselines/. Run `make visual-update` to regenerate
 * baselines after intentional design changes.
 */

const pages = [
  { name: "index", path: "/" },
  { name: "contact", path: "/contact/index.html" },
];

for (const { name, path } of pages) {
  test(`${name} page screenshot`, async ({ page }) => {
    await page.goto(path, { waitUntil: "networkidle" });
    await expect(page).toHaveScreenshot(`${name}.png`, {
      fullPage: true,
    });
  });
}
