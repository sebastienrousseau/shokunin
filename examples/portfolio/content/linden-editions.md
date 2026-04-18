---

# Front Matter (YAML)

author: "hello@mayaokafor.studio (Maya Okafor)"
banner_alt: "Maya Okafor banner"
banner_height: "398"
banner_width: "1440"
banner: ""
cdn: "https://cloudcdn.pro"
changefreq: "weekly"
charset: "utf-8"
cname: "mayaokafor.studio"
copyright: "Copyright © 2024-2026 Maya Okafor. All rights reserved."
date: "April 16, 2026"
description: "Editorial site for a quarterly literary journal — print-to-web typography system, four issues live."
download: ""
format-detection: "telephone=no"
hreflang: "en"
icon: ""
id: "https://mayaokafor.studio"
image_alt: "Maya Okafor logo"
image_height: "630"
image_width: "1200"
image: ""
keywords: "portfolio, ssg, static site generator, rust, example"
language: "en-GB"
layout: "page"
locale: "en_GB"
logo_alt: "Maya Okafor logo"
logo_height: "33"
logo_width: "100"
logo: ""
name: "Maya Okafor"
permalink: "https://mayaokafor.studio/linden-editions/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "Editorial site for a quarterly literary journal (2024)"
theme-color: "26, 58, 138"
tags: "typography, editorial, design-system, case-study"
title: "Linden Editions"
url: "https://mayaokafor.studio"
viewport: "width=device-width, initial-scale=1, shrink-to-fit=no"

# News - The News SiteMap front matter (YAML).
news_genres: "Blog"
news_keywords: "portfolio, ssg, example"
news_language: "en"
news_image_loc: ""
news_loc: "https://mayaokafor.studio"
news_publication_date: "Wed, 16 Apr 2026 00:00:00 GMT"
news_publication_name: "Maya Okafor"
news_title: "Linden Editions"

# RSS - The RSS feed front matter (YAML).
atom_link: https://mayaokafor.studio/rss.xml
category: "Technology"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.38)"
item_description: RSS feed for Maya Okafor
item_guid: "https://mayaokafor.studio/linden-editions/index.html"
item_link: "https://mayaokafor.studio/linden-editions/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "Linden Editions"
last_build_date: "Wed, 16 Apr 2026 00:00:00 GMT"
managing_editor: hello@mayaokafor.studio (Maya Okafor)
pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
ttl: "60"
type: "website"
webmaster: hello@mayaokafor.studio (Maya Okafor)

# Apple - The Apple front matter (YAML).
apple_mobile_web_app_orientations: "portrait"
apple_touch_icon_sizes: "192x192"
apple-mobile-web-app-capable: "yes"
apple-mobile-web-app-status-bar-inset: "black"
apple-mobile-web-app-status-bar-style: "black-translucent"
apple-mobile-web-app-title: "Maya Okafor"
apple-touch-fullscreen: "yes"

# MS Application - The MS Application front matter (YAML).
msapplication-navbutton-color: "rgb(0,102,204)"

# Twitter Card - The Twitter Card front matter (YAML).
twitter_card: "summary"
twitter_creator: "portfolio"
twitter_description: "Editorial site for a quarterly literary journal — print-to-web typography system, four issues live."
twitter_image: ""
twitter_image_alt: "Maya Okafor logo"
twitter_site: "portfolio"
twitter_title: "Linden Editions"
twitter_url: "https://mayaokafor.studio"

# Humans.txt - The Humans.txt front matter (YAML).
author_website: "https://mayaokafor.studio"
author_twitter: "@portfolio"
author_location: "London, UK"
thanks: "Thanks for reading!"
site_last_updated: "2026-04-16"
site_standards: "HTML5, CSS3, RSS, Atom, JSON, XML, YAML, Markdown, TOML"
site_components: "SSG, Maya Okafor Templates"
site_software: "SSG, Rust"

---

## At a glance

- **Client**: Linden Editions (independent literary journal)
- **Year**: 2024
- **Role**: Design lead
- **Duration**: 10 weeks
- **Team**: Me + 1 developer (the journal's editor)

## The problem

Linden had been publishing a print quarterly for six years. Their existing website was a WordPress theme that displayed posts in reverse chronological order — fine for a blog, wrong for a journal that wanted readers to *browse issues*, the way you would in a library.

## What I did

**Audit.** Read all 24 back issues. Catalogued the recurring sections (essay, fiction, poetry, review, interview), the typographic conventions (drop caps, em-dash conversations, full-bleed photography), and the editorial voice (long-form, slow, deliberate).

**Type system.** Built a four-face type system: Newzald for body, Söhne Mono for footnotes, Tiempos for display, Reckless for poetry. All four sit on a shared 24px baseline. The system has 11 tokens — small enough that the editor maintains it herself.

**IA.** Issues, not posts. The home page is a stack of issue covers. Each issue has its own permalink and table of contents. Individual pieces are nested *under* their issue, never floating loose.

**Build.** A static site (no CMS) with a markdown workflow the editor manages from her local machine. Total page weight: 38 KB on the home page, 84 KB on a typical essay. No JavaScript on the reading experience.

## Outcome

- Average session length doubled (1m 12s → 2m 41s) in the four months after launch
- The journal's print-to-digital subscription conversion went from 4% to 11%
- Editor reports the markdown workflow takes "about 20 minutes per issue, where the old WordPress backend took an afternoon"
