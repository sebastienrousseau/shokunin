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
description: "Polaris release notes and changelog."
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
layout: "page"
locale: "en_GB"
logo_alt: "Polaris logo"
logo_height: "33"
logo_width: "100"
logo: ""
name: "Polaris"
permalink: "https://docs.polaris.example.com/posts/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "Recent releases of the Polaris CLI and API."
theme-color: "26, 58, 138"
tags: "docs, ssg, example"
title: "Release Notes"
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
news_title: "Release Notes"

# RSS - The RSS feed front matter (YAML).
atom_link: https://docs.polaris.example.com/rss.xml
category: "release-notes"
schema: "doc"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.39)"
item_description: RSS feed for Polaris
item_guid: "https://docs.polaris.example.com/posts/index.html"
item_link: "https://docs.polaris.example.com/posts/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "Release Notes"
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
twitter_description: "Polaris release notes and changelog."
twitter_image: ""
twitter_image_alt: "Polaris logo"
twitter_site: "docs"
twitter_title: "Release Notes"
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

## Release notes

Polaris ships every two weeks. Subscribe to the [RSS feed](/rss.xml) or [Atom feed](/atom.xml) for release announcements.

### v2.4.0 — 16 April 2026

- **Added** support for streaming query results via Server-Sent Events
- **Changed** default output format from `json` to `table` for `query` command
- **Fixed** OAuth token refresh failing on tokens issued before v2.0

### v2.3.0 — 2 April 2026

- **Added** `--format=csv` output mode
- **Added** `polaris datasets describe <id>` command
- **Deprecated** `polaris list` (use `polaris datasets` instead; will be removed in v3.0)

### v2.2.1 — 19 March 2026

- **Fixed** rate-limit handling: client now respects `Retry-After` headers
- **Fixed** crash on Windows when terminal didn't support ANSI colour
