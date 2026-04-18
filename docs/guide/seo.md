<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SEO

SSG creates SEO metadata from your frontmatter and site config. This happens on every build.

## Open Graph

The `SeoPlugin` adds Open Graph meta tags to every page:

```html
<meta property="og:title" content="Page Title" />
<meta property="og:description" content="Page description" />
<meta property="og:type" content="article" />
<meta property="og:url" content="https://example.com/page" />
<meta property="og:image" content="https://example.com/image.jpg" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
<meta property="og:locale" content="en_US" />
```

## Twitter Cards

For article pages, SSG creates `summary_large_image` Twitter Cards:

```html
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="Page Title" />
<meta name="twitter:description" content="Page description" />
<meta name="twitter:image" content="https://example.com/image.jpg" />
```

## JSON-LD Structured Data

The `JsonLdPlugin` creates JSON-LD for Article and WebPage types:

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Article",
  "headline": "Page Title",
  "datePublished": "2026-04-13",
  "dateModified": "2026-04-13",
  "author": {
    "@type": "Person",
    "name": "Author Name"
  },
  "image": {
    "@type": "ImageObject",
    "url": "https://example.com/image.jpg"
  },
  "inLanguage": "en"
}
</script>
```

SSG also creates `BreadcrumbList` JSON-LD for page hierarchy.

## Canonical URLs

The `CanonicalPlugin` adds canonical link tags. These stop duplicate content issues.

```html
<link rel="canonical" href="https://example.com/page" />
```

The URL comes from `base_url` in your config plus the page path.

## robots.txt

The `RobotsPlugin` creates a `robots.txt` file in the site root:

```
User-agent: *
Allow: /
Sitemap: https://example.com/sitemap.xml
```

The sitemap URL comes from the `base_url` config value.

## Sitemaps

The `SitemapFixPlugin` creates and checks `sitemap.xml`. Each page gets a `<lastmod>` timestamp.

For multi-locale sites, SSG creates per-locale sitemaps. These include `xhtml:link` alternates (see [i18n](i18n.md)).

## Google News Sitemaps

The `NewsSitemapFixPlugin` creates a Google News sitemap. It adds keywords for news content.

## RSS 2.0

The `RssAggregatePlugin` creates an RSS 2.0 feed with:

- Enclosures for media
- Categories from frontmatter tags
- Language code
- `lastBuildDate` and `copyright`

## Atom Feeds

The `AtomFeedPlugin` creates Atom feeds as an option besides RSS.

## Content Security Policy

The `CspPlugin` hardens your site's CSP headers. It extracts inline `<style>` and `<script>` blocks to external files. Each file gets an SRI hash.

This removes the need for `'unsafe-inline'` in your CSP. The deploy headers use strict directives:

```
script-src 'self'; style-src 'self'
```

JSON-LD blocks and dev scripts are kept inline (they are safe).

## Meta Description

SSG uses the `description` frontmatter field for the HTML meta tag:

```html
<meta name="description" content="Page description from frontmatter" />
```

## Checklist

For the best SEO results, give each page:

- A unique `title` in frontmatter
- A `description` under 160 characters
- A `date` for date-based content
- An image for social sharing (OG and Twitter Cards)
- A valid `base_url` in your config

## Next Steps

- [Accessibility](accessibility.md) — WCAG compliance for SEO
- [i18n](i18n.md) — hreflang for international SEO
- [Deployment](deployment.md) — security headers that affect SEO
