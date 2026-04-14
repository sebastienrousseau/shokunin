<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Search

SSG generates a client-side full-text search index with a modal UI, requiring no server or external service.

## How It Works

1. At build time, `SearchIndex` scans all HTML files in the site directory
2. Extracts the page title, URL, headings, and body text from each page
3. Writes a `search-index.json` file to the site root
4. The `SearchPlugin` injects a `<script>` tag and search UI into every HTML page

The search runs entirely in the browser using the pre-built JSON index.

## Search Index

The index is written to `search-index.json` with this structure:

```json
{
  "entries": [
    {
      "title": "Getting Started",
      "url": "/getting-started/index.html",
      "content": "Truncated plain text...",
      "headings": ["Installation", "Quick Start"]
    }
  ]
}
```

### Limits

| Limit | Value |
| :--- | :--- |
| Max content per page | 5,000 characters |
| Max indexed pages | 50,000 |

Content is truncated to keep the index compact for fast client-side loading.

## Search UI

The injected search UI is a modal overlay with:

- A search input field
- Real-time fuzzy matching as you type
- Result list with title, URL, and content preview
- Keyboard navigation (arrow keys + Enter)

### Keyboard Shortcut

Open the search modal with:

- **Ctrl+K** (Windows / Linux)
- **Cmd+K** (macOS)

Press **Escape** to close.

## 28 Locale Translations

The search UI is translated into 28 locales via `LocalizedSearchPlugin`. The placeholder text, "No results" message, and other UI strings adapt to the site's language setting.

## Parallel Indexing

The search index is built using Rayon `par_iter` for parallel processing of HTML files, making indexing fast even for large sites.

## Extracted Data

For each HTML page, the indexer extracts:

- **Title** — from `<title>` tag or first `<h1>`
- **URL** — relative path from site root
- **Headings** — all `<h2>`-`<h6>` headings for section search
- **Content** — plain text with HTML tags stripped

## Disabling Search

Search is part of the built-in plugin pipeline. To exclude specific pages from the index, you can remove them from the output directory before the search plugin runs.

## Next Steps

- [Templates](templates.md) — the search UI is injected into your templates
- [Deployment](deployment.md) — the search index is a static JSON file
- [i18n](i18n.md) — localized search UI strings
