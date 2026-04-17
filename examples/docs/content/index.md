---

# Front Matter (YAML)

author: "support@polaris.example.com (Polaris)"
banner_alt: "Polaris banner"
banner_height: "398"
banner_width: "1440"
banner: ""
cdn: "https://cloudcdn.pro"
changefreq: "weekly"
charset: "utf-8"
cname: "docs.polaris.example.com"
copyright: "Copyright © 2024-2026 Polaris. All rights reserved."
date: "April 16, 2026"
description: "Documentation template for any developer tool, library, or API. Replace the content, keep the structure."
download: ""
format-detection: "telephone=no"
hreflang: "en"
icon: ""
id: "https://docs.polaris.example.com"
image_alt: "Polaris logo"
image_height: "630"
image_width: "1200"
image: ""
keywords: "docs, ssg, static site generator, rust, example"
language: "en-GB"
layout: "index"
locale: "en_GB"
logo_alt: "Polaris logo"
logo_height: "33"
logo_width: "100"
logo: ""
name: "Polaris"
permalink: "https://docs.polaris.example.com/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "Polaris — a documentation template you can adapt."
theme-color: "26, 58, 138"
tags: "docs, ssg, example"
title: "Polaris Documentation"
url: "https://docs.polaris.example.com"
viewport: "width=device-width, initial-scale=1, shrink-to-fit=no"

# News - The News SiteMap front matter (YAML).
news_genres: "Blog"
news_keywords: "docs, ssg, example"
news_language: "en"
news_image_loc: ""
news_loc: "https://docs.polaris.example.com"
news_publication_date: "Wed, 16 Apr 2026 00:00:00 GMT"
news_publication_name: "Polaris"
news_title: "Polaris Documentation"

# RSS - The RSS feed front matter (YAML).
atom_link: https://docs.polaris.example.com/rss.xml
category: "welcome"
schema: "doc"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.36)"
item_description: RSS feed for Polaris
item_guid: "https://docs.polaris.example.com/index.html"
item_link: "https://docs.polaris.example.com/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "Polaris Documentation"
last_build_date: "Wed, 16 Apr 2026 00:00:00 GMT"
managing_editor: support@polaris.example.com (Polaris)
pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
ttl: "60"
type: "website"
webmaster: support@polaris.example.com (Polaris)

# Apple - The Apple front matter (YAML).
apple_mobile_web_app_orientations: "portrait"
apple_touch_icon_sizes: "192x192"
apple-mobile-web-app-capable: "yes"
apple-mobile-web-app-status-bar-inset: "black"
apple-mobile-web-app-status-bar-style: "black-translucent"
apple-mobile-web-app-title: "Polaris"
apple-touch-fullscreen: "yes"

# MS Application - The MS Application front matter (YAML).
msapplication-navbutton-color: "rgb(0,102,204)"

# Twitter Card - The Twitter Card front matter (YAML).
twitter_card: "summary"
twitter_creator: "docs"
twitter_description: "Documentation template for any developer tool, library, or API. Replace the content, keep the structure."
twitter_image: ""
twitter_image_alt: "Polaris logo"
twitter_site: "docs"
twitter_title: "Polaris Documentation"
twitter_url: "https://docs.polaris.example.com"

# Humans.txt - The Humans.txt front matter (YAML).
author_website: "https://docs.polaris.example.com"
author_twitter: "@docs"
author_location: "London, UK"
thanks: "Thanks for reading!"
site_last_updated: "2026-04-16"
site_standards: "HTML5, CSS3, RSS, Atom, JSON, XML, YAML, Markdown, TOML"
site_components: "SSG, Polaris Templates"
site_software: "SSG, Rust"

---

## Welcome

**Polaris** is a documentation template you can clone, edit, and ship as the docs site for any developer tool, library, or API. The structure is opinionated; the content is yours to replace.

## What's in this template

- **Getting started** — install + first run, in under a page
- **Configuration reference** — every option, in a single scannable table
- **API reference** — endpoint + method tables with examples
- **Release notes** — chronological changelog, surfaced via the *Posts* page
- **Browse by topic** — automatic tag aggregation across every page
- **Contact** — support routing for users who can't find an answer

## How to use it

1. Clone this directory
2. Replace every page in `content/` with your own copy
3. Update brand colour, logo URL, and footer copyright in the frontmatter
4. Build with `cargo run --example docs`
5. Deploy the contents of `examples/docs/public/` to any static host

That's it. The plugins, sitemap, search index, and accessibility report all work without further configuration.
