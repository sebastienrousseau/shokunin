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
description: "Nine new criteria, three of them affect every form on the web. A practical translation from spec language to ship-it advice."
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
permalink: "https://threshold.press/wcag-2-1-vs-2-2/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "Nine new criteria, three of them affect every form on the web. A practical translation from spec language to ship-it advice."
theme-color: "143, 250, 113"
tags: "WCAG, accessibility, web standards"
title: "WCAG 2.2: what actually changed for builders"
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
news_title: "WCAG 2.2: what actually changed for builders"

# RSS - The RSS feed front matter (YAML).
atom_link: https://threshold.press/rss.xml
category: "Technology"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.39)"
item_description: RSS feed for Threshold
item_guid: "https://threshold.press/wcag-2-1-vs-2-2/index.html"
item_link: "https://threshold.press/wcag-2-1-vs-2-2/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "WCAG 2.2: what actually changed for builders"
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
twitter_description: "Nine new criteria, three of them affect every form on the web. A practical translation from spec language to ship-it advice."
twitter_image: ""
twitter_image_alt: "Threshold logo"
twitter_site: "thresholdpress"
twitter_title: "WCAG 2.2: what actually changed for builders"
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

## The nine new criteria, one paragraph each

WCAG 2.2 added nine success criteria. Most are tightenings of existing rules; three are genuinely new constraints.

## The three that change daily work

**2.4.11 Focus Not Obscured (AA)** — sticky headers can no longer cover the keyboard focus indicator. If your nav is sticky and your focus ring is on the second visible item, the spec wants the page to scroll until the focused element is visible.

**2.5.7 Dragging Movements (AA)** — anything you can do by dragging must also be doable with a single tap (or click). Card-sort interfaces, image carousels, drag-to-dismiss banners — all need keyboard equivalents.

**3.3.7 Redundant Entry (A)** — checkout flows can no longer ask the user to re-type information they've already entered (billing address re-entered as shipping, etc.). Auto-fill is acceptable; re-typing is not.

## What stayed the same

Colour contrast ratios. Heading structure rules. Image alt requirements. The fundamentals of WCAG 2.1 AA carry over unchanged — 2.2 is additive, not a rewrite.
