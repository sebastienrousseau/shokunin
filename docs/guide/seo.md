<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SEO

SSG generates comprehensive SEO metadata automatically from frontmatter and site configuration.

## Open Graph

The `SeoPlugin` injects Open Graph meta tags into every page:

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

For article pages, SSG generates `summary_large_image` Twitter Cards:

```html
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="Page Title" />
<meta name="twitter:description" content="Page description" />
<meta name="twitter:image" content="https://example.com/image.jpg" />
```

## JSON-LD Structured Data

The `JsonLdPlugin` generates JSON-LD for Article and WebPage schema types:

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

SSG also generates `BreadcrumbList` JSON-LD for navigation hierarchy.

## Canonical URLs

The `CanonicalPlugin` injects canonical link tags to prevent duplicate content issues:

```html
<link rel="canonical" href="https://example.com/page" />
```

The canonical URL is constructed from `base_url` in configuration and the page's relative path.

## robots.txt

The `RobotsPlugin` generates a `robots.txt` file in the site root:

```
User-agent: *
Allow: /
Sitemap: https://example.com/sitemap.xml
```

The sitemap URL is derived from the `base_url` configuration.

## Sitemaps

The `SitemapFixPlugin` generates and validates `sitemap.xml` with per-page `<lastmod>` timestamps.

For multi-locale sites, per-locale sitemaps are generated with `xhtml:link` alternates (see [i18n](i18n.md)).

## Google News Sitemaps

The `NewsSitemapFixPlugin` generates a Google News sitemap with keywords for news-oriented content.

## RSS 2.0

The `RssAggregatePlugin` generates an RSS 2.0 feed with:

- Enclosures for media
- Categories from frontmatter tags
- Language code
- `lastBuildDate` and `copyright`

## Atom Feeds

The `AtomFeedPlugin` generates Atom feeds as an alternative to RSS.

## Meta Description

SSG uses the `description` field from frontmatter for the HTML meta description:

```html
<meta name="description" content="Page description from frontmatter" />
```

## Checklist

To maximize SEO with SSG, ensure each page has:

- A unique `title` in frontmatter
- A `description` under 160 characters
- A `date` for date-based content
- An image for social sharing (OG and Twitter Cards)
- A valid `base_url` in configuration

## Next Steps

- [Accessibility](accessibility.md) — WCAG compliance for SEO
- [i18n](i18n.md) — hreflang for international SEO
- [Deployment](deployment.md) — security headers that affect SEO
