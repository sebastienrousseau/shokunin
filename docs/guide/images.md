<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Images

SSG optimizes images with responsive `<picture>` elements, modern formats, and lazy loading via the `ImageOptimizationPlugin`.

> **Note:** This feature requires the `image-optimization` cargo feature flag.

## How It Works

The plugin runs in `after_compile` and processes all `<img>` tags in generated HTML:

1. Scans `site_dir` for JPEG and PNG images
2. Generates WebP variants at responsive widths
3. Optionally generates AVIF variants (with the `avif` feature)
4. Rewrites `<img>` tags to `<picture>` elements with `srcset`
5. Adds `loading="lazy"`, `decoding="async"`, `width`, and `height`

## Generated Output

For each image, SSG produces:

```html
<picture>
  <source type="image/avif"
    srcset="image-320w.avif 320w,
            image-640w.avif 640w,
            image-1024w.avif 1024w,
            image-1440w.avif 1440w" />
  <source type="image/webp"
    srcset="image-320w.webp 320w,
            image-640w.webp 640w,
            image-1024w.webp 1024w,
            image-1440w.webp 1440w" />
  <img src="/images/image.jpg" alt="Description"
       width="1440" height="960"
       loading="lazy" decoding="async" />
</picture>
```

## Responsive Breakpoints

Default breakpoints (in pixels):

| Breakpoint | Target |
| :--- | :--- |
| 320 | Mobile (small) |
| 640 | Mobile (large) / Tablet |
| 1024 | Tablet / Desktop |
| 1440 | Desktop (large) |

Breakpoints are configurable via the `ImageOptimizationPlugin` struct:

```rust
use ssg::image_plugin::ImageOptimizationPlugin;

let plugin = ImageOptimizationPlugin {
    quality: 80,
    breakpoints: vec![320, 640, 1024, 1440, 1920],
};
```

## Lazy Loading

All images receive `loading="lazy"` and `decoding="async"` by default.

Images with `fetchpriority="high"` receive `loading="eager"` instead, so the browser fetches them immediately. Use this for above-the-fold hero images.

## CLS Prevention

SSG reads `width` and `height` from source image metadata and injects them into the `<img>` element. This allows the browser to reserve the correct space before the image loads, preventing Cumulative Layout Shift (CLS).

## WebP Quality

Default WebP encoding quality is 80 (range: 1-100). Configure via the plugin struct:

```rust
let plugin = ImageOptimizationPlugin {
    quality: 90,
    breakpoints: vec![320, 640, 1024, 1440],
};
```

## Decorative Images

SSG's `AccessibilityPlugin` detects decorative images and applies `role="presentation"` with empty `alt=""`. See [Accessibility](accessibility.md) for details.

## Best Practices

- Provide meaningful `alt` text for all informative images
- Use `fetchpriority="high"` on above-the-fold hero images
- Keep source images at the largest breakpoint size or larger
- Use AVIF where browser support allows for better compression

## Next Steps

- [Accessibility](accessibility.md) — alt text and decorative image handling
- [Templates](templates.md) — using images in templates
- [SEO](seo.md) — OG image metadata
