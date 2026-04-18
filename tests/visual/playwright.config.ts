import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  snapshotPathTemplate:
    "{testDir}/../visual-baselines/{testName}/{arg}{ext}",
  timeout: 30_000,
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.001, // 0.1% pixel tolerance
    },
  },
  use: {
    baseURL: "http://localhost:8080",
  },
  projects: [
    {
      name: "mobile",
      use: { ...devices["iPhone 13"] },
    },
    {
      name: "tablet",
      use: { viewport: { width: 768, height: 1024 } },
    },
    {
      name: "desktop",
      use: { viewport: { width: 1440, height: 900 } },
    },
  ],
  webServer: {
    command:
      "python3 -m http.server 8080 -d ../../examples/public 2>/dev/null || npx serve -l 8080 ../../examples/public",
    port: 8080,
    reuseExistingServer: true,
  },
});
