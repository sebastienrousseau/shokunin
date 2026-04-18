---

# Front Matter (YAML)

author: "hello@threshold.press (Threshold)"
banner_alt: "Threshold banner"
banner_height: "398"
banner_width: "1440"
banner: ""
cdn: "https://cloudcdn.pro"
changefreq: "weekly"
charset: "utf-8"
cname: "threshold.press"
copyright: "Copyright © 2024-2026 Threshold. All rights reserved."
date: "April 16, 2026"
description: "How type scale, line-height, and contrast affect users with low vision — with practical defaults you can ship today."
download: ""
format-detection: "telephone=no"
hreflang: "en"
icon: ""
id: "https://threshold.press"
image_alt: "Threshold logo"
image_height: "630"
image_width: "1200"
image: ""
keywords: "blog, ssg, static site generator, rust, example"
language: "en-GB"
layout: "page"
locale: "en_GB"
logo_alt: "Threshold logo"
logo_height: "33"
logo_width: "100"
logo: ""
name: "Threshold"
permalink: "https://threshold.press/accessible-typography/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "How type scale, line-height, and contrast affect users with low vision — with practical defaults you can ship today."
theme-color: "143, 250, 113"
tags: "accessibility, typography, WCAG, design"
title: "Designing for low-vision readers"
url: "https://threshold.press"
viewport: "width=device-width, initial-scale=1, shrink-to-fit=no"

# News - The News SiteMap front matter (YAML).
news_genres: "Blog"
news_keywords: "blog, ssg, example"
news_language: "en"
news_image_loc: ""
news_loc: "https://threshold.press"
news_publication_date: "Wed, 16 Apr 2026 00:00:00 GMT"
news_publication_name: "Threshold"
news_title: "Designing for low-vision readers"

# RSS - The RSS feed front matter (YAML).
atom_link: https://threshold.press/rss.xml
category: "Technology"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.40)"
item_description: RSS feed for Threshold
item_guid: "https://threshold.press/accessible-typography/index.html"
item_link: "https://threshold.press/accessible-typography/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "Designing for low-vision readers"
last_build_date: "Wed, 16 Apr 2026 00:00:00 GMT"
managing_editor: hello@threshold.press (Threshold)
pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
ttl: "60"
type: "website"
webmaster: hello@threshold.press (Threshold)

# Apple - The Apple front matter (YAML).
apple_mobile_web_app_orientations: "portrait"
apple_touch_icon_sizes: "192x192"
apple-mobile-web-app-capable: "yes"
apple-mobile-web-app-status-bar-inset: "black"
apple-mobile-web-app-status-bar-style: "black-translucent"
apple-mobile-web-app-title: "Threshold"
apple-touch-fullscreen: "yes"

# MS Application - The MS Application front matter (YAML).
msapplication-navbutton-color: "rgb(0,102,204)"

# Twitter Card - The Twitter Card front matter (YAML).
twitter_card: "summary"
twitter_creator: "thresholdpress"
twitter_description: "How type scale, line-height, and contrast affect users with low vision — with practical defaults you can ship today."
twitter_image: ""
twitter_image_alt: "Threshold logo"
twitter_site: "thresholdpress"
twitter_title: "Designing for low-vision readers"
twitter_url: "https://threshold.press"

# Humans.txt - The Humans.txt front matter (YAML).
author_website: "https://threshold.press"
author_twitter: "@thresholdpress"
author_location: "London, UK"
thanks: "Thanks for reading!"
site_last_updated: "2026-04-16"
site_standards: "HTML5, CSS3, RSS, Atom, JSON, XML, YAML, Markdown, TOML"
site_components: "SSG, Threshold Templates"
site_software: "SSG, Rust"

---

## Designing for low-vision readers

Most accessibility advice about typography stops at "use 16px and good contrast." That's a start, but it leaves several decisions on the table that materially affect users with low vision.

## Three defaults worth changing

- **Line-height of 1.5** — WCAG 1.4.12 calls for at least 1.5x the font size. Most CMS defaults sit around 1.2-1.3.
- **Maximum line length of 80 characters** — beyond that, eye-tracking studies show fatigue rises sharply for users with reduced central vision.
- **Avoid pure white** — `#fff` on `#000` produces glare that low-vision users describe as "stinging." A near-white like `#fafafa` removes the effect without hurting contrast ratios.

## What we ship by default

Our base stylesheet uses `1.7` line-height (leaving room for users to bump to 2 in their reader settings without things breaking), `max-width: 65ch` on body copy, and `oklch(99% 0 0)` for the page background. None of these costs a single conversion.
