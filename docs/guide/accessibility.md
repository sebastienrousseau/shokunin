<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Accessibility

SSG validates generated HTML against WCAG 2.1 Level AA on every build via the `AccessibilityPlugin`.

## Automatic Checks

The plugin runs in `after_compile` and checks every HTML file for:

### Images (WCAG 1.1.1)
- Missing `alt` attributes on `<img>` elements
- Decorative images automatically receive `role="presentation"` and empty `alt=""`
- Informative images are flagged if `alt` is missing

### Heading Hierarchy (WCAG 1.3.1)
- Skipped heading levels (e.g. `<h1>` followed by `<h3>`)
- Multiple `<h1>` elements on a single page
- Missing `<h1>` on pages

### Link Text (WCAG 2.4.4)
- Links with generic text ("click here", "read more")
- Empty link text
- Links that are not descriptive

### ARIA Landmarks (WCAG 1.3.1, 4.1.2)
- Presence of main landmark (`<main>` or `role="main"`)
- Proper use of ARIA roles
- Duplicate landmark roles without labels

## Accessibility Report

After each build, the plugin writes `accessibility-report.json` to the site directory:

```json
{
  "pages_scanned": 42,
  "total_issues": 3,
  "pages": [
    {
      "path": "about/index.html",
      "issues": [
        {
          "criterion": "1.1.1",
          "severity": "error",
          "message": "Image missing alt attribute: /images/photo.jpg"
        }
      ]
    }
  ]
}
```

Issues include:
- **criterion** — the WCAG success criterion ID
- **severity** — `"error"` or `"warning"`
- **message** — human-readable description

## pa11y CI Integration

SSG includes a `make a11y` target that runs [pa11y](https://pa11y.org/) against the generated site for more comprehensive accessibility auditing.

The CI workflow (`.github/workflows/`) runs pa11y on every push:

1. Builds the example site with `cargo run`
2. Starts the dev server
3. Runs pa11y against all pages
4. Fails the build if any WCAG 2.1 AA violations are found

### Running Locally

```sh
# Build the example site
ssg -c content -o public -t templates

# Run pa11y (requires Node.js)
npx pa11y-ci --config .pa11yci.json
```

Or use the Makefile:

```sh
make a11y
```

## Best Practices

### Images
- Always provide meaningful `alt` text for informative images
- Use empty `alt=""` for decorative images (SSG detects these automatically)
- Include `width` and `height` to prevent layout shift

### Headings
- Start each page with a single `<h1>`
- Follow the heading hierarchy without skipping levels
- Use headings for structure, not for styling

### Links
- Write descriptive link text that makes sense out of context
- Avoid generic text like "click here" or "learn more"

### Landmarks
- Use semantic HTML elements: `<header>`, `<nav>`, `<main>`, `<footer>`
- Each page should have exactly one `<main>` element

## Non-blocking by Default

The `AccessibilityPlugin` logs warnings but does not fail the build. The report is always generated so you can integrate it into your own CI checks.

## Next Steps

- [Images](images.md) — responsive images with proper alt text
- [Templates](templates.md) — ensure templates use semantic HTML
- [SEO](seo.md) — accessibility and SEO overlap
