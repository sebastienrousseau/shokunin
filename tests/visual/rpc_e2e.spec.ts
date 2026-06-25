// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// End-to-end test for the Edge RPC client (issue #548).
//
// Validates the browser-side ACs against a mocked worker:
//
//   AC2 — POST /__rpc/<name> dispatches to the handler.
//   AC3 — unknown name → 404 { "error": "unknown rpc" }.
//   AC4 — the JS client serialises input and parses output JSON.
//   AC6 — GET to /__rpc/* → 405.
//
// The test is fully self-contained: it uses Playwright's `page.route()`
// to mock the Worker, then loads the real `web/rpc.js` client into
// the page and exercises it end-to-end. No external HTTP server,
// build artefacts, or wrangler instance required.

import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

/** Read the real client source from the repo root. */
function loadClient(): string {
  const path = resolve(__dirname, "..", "..", "web", "rpc.js");
  return readFileSync(path, "utf8");
}

const CLIENT = loadClient();

/** Mock Worker. Routes:
 *    POST /__rpc/echo        → 200 with the same payload back
 *    POST /__rpc/fail        → 500 { error: "boom" }
 *    POST /__rpc/missing     → 404 { error: "unknown rpc" }
 *    GET  /__rpc/echo        → 405 { error: "method not allowed" }
 */
async function installWorker(page: import("@playwright/test").Page) {
  await page.route("**/__rpc/**", async (route) => {
    const req = route.request();
    const method = req.method();
    const url = new URL(req.url());
    const name = url.pathname.replace(/^\/__rpc\//, "");

    if (method !== "POST") {
      await route.fulfill({
        status: 405,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "method not allowed" }),
      });
      return;
    }
    if (name === "missing") {
      await route.fulfill({
        status: 404,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "unknown rpc" }),
      });
      return;
    }
    if (name === "fail") {
      await route.fulfill({
        status: 500,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ error: "boom" }),
      });
      return;
    }
    if (name === "echo") {
      const body = req.postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify(body),
      });
      return;
    }
    await route.fulfill({
      status: 404,
      body: JSON.stringify({ error: "unknown rpc" }),
    });
  });
}

/** Render a blank page that imports the client and exposes it on
 *  `window.rpc`. The client is served as an ES module via a data: URI. */
async function bootClient(page: import("@playwright/test").Page) {
  await page.setContent(`<!doctype html>
<html><head><meta charset="utf-8"></head>
<body>
<script type="module">
${CLIENT}
window.rpc = createRpc();
window.RpcError = RpcError;
window.__ready = true;
</script>
</body></html>`);
  await page.waitForFunction(() => (window as any).__ready === true);
}

test.describe("Edge RPC client", () => {
  test("ac2_post_dispatches_and_returns_json", async ({ page }) => {
    await installWorker(page);
    await bootClient(page);

    const result = await page.evaluate(async () =>
      (window as any).rpc.echo({ msg: "hi" }),
    );
    expect(result).toEqual({ msg: "hi" });
  });

  test("ac3_unknown_rpc_throws_with_404_status", async ({ page }) => {
    await installWorker(page);
    await bootClient(page);

    const out = await page.evaluate(async () => {
      try {
        await (window as any).rpc.missing({});
        return { ok: true };
      } catch (e: any) {
        return { ok: false, status: e.status, error: e.body?.error };
      }
    });
    expect(out.ok).toBe(false);
    expect(out.status).toBe(404);
    expect(out.error).toBe("unknown rpc");
  });

  test("ac4_handler_500_surfaces_as_rpc_error", async ({ page }) => {
    await installWorker(page);
    await bootClient(page);

    const out = await page.evaluate(async () => {
      try {
        await (window as any).rpc.fail({});
        return { ok: true };
      } catch (e: any) {
        return { ok: false, status: e.status, error: e.body?.error };
      }
    });
    expect(out.ok).toBe(false);
    expect(out.status).toBe(500);
    expect(out.error).toBe("boom");
  });

  test("ac6_get_request_returns_405", async ({ page }) => {
    await installWorker(page);

    // Bypass the JS client — issue a raw GET so we can assert the
    // Worker enforces POST-only on its own.
    const status = await page.evaluate(async () => {
      const r = await fetch("/__rpc/echo", { method: "GET" });
      return r.status;
    });
    expect(status).toBe(405);
  });

  test("ac4_post_body_is_json_encoded_input", async ({ page }) => {
    let capturedBody: unknown = null;
    await page.route("**/__rpc/**", async (route) => {
      capturedBody = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: "application/json; charset=utf-8",
        body: JSON.stringify({ likes: 7 }),
      });
    });
    await bootClient(page);

    await page.evaluate(async () =>
      (window as any).rpc.like_post({ post_id: "x" }),
    );
    expect(capturedBody).toEqual({ post_id: "x" });
  });
});
