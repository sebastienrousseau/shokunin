<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Developer Experience

SSG includes dev-time features for fast iteration: live reload with
CSS hot swapping, a browser error overlay, and file change classification.

## Live Reload

The `LiveReloadPlugin` injects a WebSocket client before `</body>`.
On file changes, the browser refreshes automatically.

- **Port**: 35729 (configurable)
- **Reconnection**: exponential backoff (1s → 2s → 4s → 10s max)
- **Idempotent**: already-injected pages are skipped

## CSS Hot Reload

CSS file changes are applied without a full page reload:

1. The file watcher classifies changes by type (CSS, content, template)
2. CSS changes send a `css-reload` WebSocket message
3. The browser cache-busts all `<link rel="stylesheet">` elements
4. No page reload — scroll position and component state are preserved

## Scroll Preservation

When a full reload is required (content or template change):

1. Scroll position is saved to `sessionStorage` before reload
2. After the page loads, position is restored via `setTimeout`

## Browser Error Overlay

Build errors render directly in the browser:

- File path and line number (when extractable)
- Full error message in monospace
- Dark overlay with red accent
- Dismisses automatically on successful rebuild
- Close button for manual dismissal

The overlay uses a JSON WebSocket protocol:

| Message | Action |
|---------|--------|
| `"reload"` | Full page reload |
| `{"type":"error",...}` | Show error overlay |
| `{"type":"clear-error"}` | Dismiss overlay |
| `{"type":"css-reload",...}` | CSS-only refresh |

## Dependency Graph

The `DepGraph` module tracks which pages depend on which templates
and shortcodes. On rebuild, only pages whose dependencies changed
are invalidated.

```sh
# The graph is persisted to .ssg-deps.json
# Plugins populate it during compilation
```

## Incremental Builds

The `BuildCache` fingerprints every content file using FNV-1a hashing.
On subsequent builds, only changed files are reprocessed.

Cache files:
- `.ssg-cache.json` — content fingerprints
- `.ssg-plugin-cache.json` — plugin output fingerprints
- `.ssg-deps.json` — page dependency graph
