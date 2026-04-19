<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Interactive Islands

Islands add client-side interactivity to static pages via Web Components.
Each island loads lazily based on a hydration strategy.

## Shortcode Syntax

```markdown
{{< island component="counter" hydrate="visible" >}}
```

## Hydration Strategies

| Strategy | Trigger | Use Case |
|----------|---------|----------|
| `visible` | IntersectionObserver | Below-the-fold components |
| `idle` | requestIdleCallback | Low-priority widgets |
| `interaction` | click / focus / hover | Interactive elements |

## How It Works

1. The `{{< island >}}` shortcode expands to `<ssg-island>` custom element
2. `IslandPlugin` scans HTML for `<ssg-island>` elements
3. Copies component bundles from `islands/` to `_islands/`
4. Generates `_islands/manifest.json` listing all referenced components
5. Injects the `ssg-island.js` custom element loader

## Creating an Island Component

Place a JavaScript file in `islands/`:

```
islands/
  counter.js
  search-widget.js
```

Each file exports a Web Component class. The loader handles registration.

## Zero JS by Default

Pages without `{{< island >}}` shortcodes ship zero JavaScript.
The island loader script is only injected into pages that use islands.
