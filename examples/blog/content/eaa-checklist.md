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
description: "The European Accessibility Act lands in June 2025. Here's the audit list we run on every site before signing it off."
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
permalink: "https://threshold.press/eaa-checklist/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "The European Accessibility Act lands in June 2025. Here's the audit list we run on every site before signing it off."
theme-color: "143, 250, 113"
tags: "EAA, accessibility, WCAG, compliance, legal"
title: "EAA enforcement: a 12-point pre-launch checklist"
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
news_title: "EAA enforcement: a 12-point pre-launch checklist"

# RSS - The RSS feed front matter (YAML).
atom_link: https://threshold.press/rss.xml
category: "Technology"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.36)"
item_description: RSS feed for Threshold
item_guid: "https://threshold.press/eaa-checklist/index.html"
item_link: "https://threshold.press/eaa-checklist/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "EAA enforcement: a 12-point pre-launch checklist"
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
twitter_description: "The European Accessibility Act lands in June 2025. Here's the audit list we run on every site before signing it off."
twitter_image: ""
twitter_image_alt: "Threshold logo"
twitter_site: "thresholdpress"
twitter_title: "EAA enforcement: a 12-point pre-launch checklist"
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

## The 12 checks we run before sign-off

The European Accessibility Act covers any commercial site that serves EU consumers. For most of our clients, the practical scope is **WCAG 2.1 AA** — and that maps to twelve recurring failure points we see across audits.

1. Contrast ratio &ge; 4.5:1 for body text, 3:1 for large text
2. Every `<img>` has a meaningful `alt` (or `alt=""` if decorative, with `role="presentation"`)
3. Heading hierarchy descends without skipping levels
4. Focus order matches DOM order on tab
5. Focus indicator is visible (not the browser default outline removed)
6. Touch targets &ge; 44&times;44 CSS pixels (WCAG 2.5.5)
7. Form fields have associated `<label>` elements
8. Error messages are programmatically linked to the offending field via `aria-describedby`
9. Page is operable without a mouse (test with Tab + Enter only)
10. Page is readable at 200% zoom without horizontal scroll
11. `<html lang="...">` is set
12. Motion respects `prefers-reduced-motion`

## What we don't bother with

ARIA roles you can express with native HTML. We've never had a single audit issue from "should have used `role='button'`" — only from "should have used `<button>` instead of `<div role='button'>`."
