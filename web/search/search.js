// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// ssg-search drop-in shim (AC7).
//
// Drop a <script type="module" src="/search/search.js" defer></script>
// onto any page that contains an <input data-ssg-search>. The shim:
//
//   1. Spawns the WebWorker that owns the WASM vector engine.
//   2. Loads the four artifact files lazily on the first keystroke.
//   3. Renders a <ul data-ssg-search-results> below the input
//      (creating it if absent) with the top-K hits.
//
// Hard size budget: keep this file under 2 KB minified. No deps.

(() => {
  const inputs = document.querySelectorAll("input[data-ssg-search]");
  if (!inputs.length) return;
  const base = inputs[0].getAttribute("data-ssg-search-base") || "/search/";
  const worker = new Worker(base + "search-worker.js", { type: "module" });

  let inited = false;
  const queue = [];

  function ensureResultsEl(input) {
    let el = input.nextElementSibling;
    if (!el || !el.matches("[data-ssg-search-results]")) {
      el = document.createElement("ul");
      el.setAttribute("data-ssg-search-results", "");
      input.insertAdjacentElement("afterend", el);
    }
    return el;
  }

  function render(input, hits) {
    const el = ensureResultsEl(input);
    el.innerHTML = "";
    for (const h of hits) {
      const li = document.createElement("li");
      const a = document.createElement("a");
      a.href = h.url;
      a.textContent = h.title || h.url;
      li.appendChild(a);
      if (h.excerpt) {
        const span = document.createElement("span");
        span.textContent = " — " + h.excerpt;
        li.appendChild(span);
      }
      el.appendChild(li);
    }
  }

  let lastInput = null;

  worker.onmessage = (e) => {
    const m = e.data;
    if (m.type === "ready") {
      inited = true;
      // Drain queued queries that arrived before the worker was ready.
      while (queue.length) {
        const q = queue.shift();
        worker.postMessage({ type: "search", query: q.query, topK: q.topK });
        lastInput = q.input;
      }
    } else if (m.type === "results" && lastInput) {
      render(lastInput, m.hits);
    } else if (m.type === "error") {
      // eslint-disable-next-line no-console
      console.error("[ssg-search]", m.message);
    }
  };

  function dispatch(input) {
    const query = input.value;
    if (!query) {
      const el = ensureResultsEl(input);
      el.innerHTML = "";
      return;
    }
    const topK = parseInt(input.getAttribute("data-ssg-search-top-k") || "10", 10);
    if (!inited) {
      queue.push({ query, topK, input });
      // Kick the worker on the first keystroke (lazy boot keeps cold-load fast).
      if (queue.length === 1) worker.postMessage({ type: "init", base });
      return;
    }
    lastInput = input;
    worker.postMessage({ type: "search", query, topK });
  }

  function debounce(fn, ms) {
    let t = 0;
    return function debounced(...args) {
      clearTimeout(t);
      t = setTimeout(() => fn.apply(this, args), ms);
    };
  }

  for (const input of inputs) {
    const handler = debounce(() => dispatch(input), 80);
    input.addEventListener("input", handler);
  }
})();
