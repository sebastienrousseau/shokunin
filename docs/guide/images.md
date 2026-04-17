<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Images

SSG makes images responsive and fast. It uses `<picture>` elements, modern formats, and lazy loading.

> **Note:** This feature requires the `image-optimization` cargo feature flag.

## How It Works

The plugin runs in `after_compile`. It finds all `<img>` tags in your HTML.

1. Scans `site_dir` for JPEG and PNG images
2. Creates WebP versions at each breakpoint width
3. Can also create AVIF versions (with the `avif` feature)
4. Rewrites `<img>` tags as `<picture>` elements with `srcset`
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

You can change breakpoints via the `ImageOptimizationPlugin` struct:

```rust
use ssg::image_plugin::ImageOptimizationPlugin;

let plugin = ImageOptimizationPlugin {
    quality: 80,
    breakpoints: vec![320, 640, 1024, 1440, 1920],
};
```

## Lazy Loading

All images get `loading="lazy"` and `decoding="async"` by default.

Images with `fetchpriority="high"` get `loading="eager"` instead. The browser then fetches them right away. Use this for hero images above the fold.

## CLS Prevention

SSG reads `width` and `height` from the source image. It adds them to the `<img>` tag. This lets the browser reserve space before the image loads. It stops Cumulative Layout Shift (CLS).

## WebP Quality

The default WebP quality is 80 (range: 1-100). Set it via the plugin struct:

```rust
let plugin = ImageOptimizationPlugin {
    quality: 90,
    breakpoints: vec![320, 640, 1024, 1440],
};
```

## Decorative Images

The `AccessibilityPlugin` finds decorative images. It adds `role="presentation"` and empty `alt=""`. See [Accessibility](accessibility.md) for details.

## Best Practices

- Provide clear `alt` text for all informative images
- Use `fetchpriority="high"` on above-the-fold hero images
- Keep source images at the largest breakpoint size or bigger
- Use AVIF where browsers support it for better compression

## Next Steps

- [Accessibility](accessibility.md) — alt text and decorative image handling
- [Templates](templates.md) — using images in templates
- [SEO](seo.md) — OG image metadata
