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
description: "REST API reference for Polaris."
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
permalink: "https://docs.polaris.example.com/plugin-api/"
rating: "general"
referrer: "no-referrer"
revisit-after: "7 days"
robots: "index, follow"
short_name: "kaishi"
subtitle: "REST endpoints, request/response shapes, and error codes."
theme-color: "26, 58, 138"
tags: "docs, ssg, example"
title: "API Reference"
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
news_title: "API Reference"

# RSS - The RSS feed front matter (YAML).
atom_link: https://docs.polaris.example.com/rss.xml
category: "api-reference"
schema: "doc"
docs: https://validator.w3.org/feed/docs/rss2.html
generator: "SSG (version 0.0.39)"
item_description: RSS feed for Polaris
item_guid: "https://docs.polaris.example.com/plugin-api/index.html"
item_link: "https://docs.polaris.example.com/plugin-api/index.html"
item_pub_date: "Wed, 16 Apr 2026 00:00:00 GMT"
item_title: "API Reference"
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
twitter_description: "REST API reference for Polaris."
twitter_image: ""
twitter_image_alt: "Polaris logo"
twitter_site: "docs"
twitter_title: "API Reference"
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

## Base URL

```
https://api.polaris.example.com/v1
```

All endpoints accept and return JSON unless otherwise noted. Authentication is via the `Authorization: Bearer <token>` header.

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/datasets` | List datasets you have read access to |
| GET | `/datasets/{id}` | Retrieve a single dataset's metadata |
| POST | `/datasets/{id}/queries` | Submit a query against a dataset |
| GET | `/queries/{id}` | Poll a submitted query for results |
| DELETE | `/queries/{id}` | Cancel a running query |

## Example: submit a query

```sh
curl -X POST \
  -H "Authorization: Bearer $POLARIS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"sql": "select count(*) from events"}' \
  https://api.polaris.example.com/v1/datasets/my-dataset/queries
```

Response:

```json
{
  "query_id": "q_01HZ2K9X8R7Y6V5T4S3R2Q1P0",
  "status": "running",
  "submitted_at": "2026-04-16T09:30:00Z"
}
```

## Error codes

| Code | Meaning |
|------|---------|
| 400 | Malformed JSON, invalid SQL, or missing required field |
| 401 | Missing or expired token |
| 403 | Token valid but lacks permission for this dataset |
| 404 | Dataset or query ID does not exist |
| 429 | Rate limit exceeded — see `Retry-After` header |
| 500 | Server error — please [report it](/contact/) |
