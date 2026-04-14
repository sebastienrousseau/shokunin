<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# SSG Documentation Guide

Comprehensive guides for the Static Site Generator (SSG).

## Getting Started

- [Installation](installation.md) — All install methods (curl, Homebrew, Cargo, deb, AUR, Scoop, winget, source)
- [Quick Start](quick-start.md) — Install, scaffold, customize, build, serve, deploy
- [CLI Reference](cli.md) — Full flag reference, examples, env vars, exit codes

## Authoring

- [Configuration](configuration.md) — `ssg.toml` fields, env vars, JSON schema
- [Content](content.md) — Frontmatter, content schemas, GFM, shortcodes
- [Content Schemas](content-schema.md) — TOML schema format, field types, `--validate`
- [Templates](templates.md) — Tera engine, inheritance, blocks, variables

## Features

- [Plugins](plugins.md) — Lifecycle hooks, 22 built-in plugins, custom examples
- [Plugin API](plugin-api.md) — Plugin trait, PluginContext, PluginCache, testing
- [SEO](seo.md) — Open Graph, Twitter Cards, JSON-LD, canonical URLs, sitemaps, feeds
- [Accessibility](accessibility.md) — WCAG 2.1 AA, ARIA, pa11y CI
- [Images](images.md) — Responsive `<picture>`, AVIF/WebP, srcset, lazy loading
- [Search](search.md) — Client-side search index, 28 locales, keyboard shortcut
- [Internationalisation](i18n.md) — Hreflang, x-default, locale sitemaps, lang switcher

## Operations

- [Deployment](deployment.md) — Netlify, Vercel, Cloudflare Pages, GitHub Pages, security headers

## Links

- [API Documentation (docs.rs)](https://docs.rs/ssg)
- [Crates.io](https://crates.io/crates/ssg)
- [GitHub Repository](https://github.com/sebastienrousseau/static-site-generator)
