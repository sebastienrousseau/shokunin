---
title: "About"
description: "Translated-slug demonstration — English edition"
permalink: "https://example.invalid/en/about"
layout: page
author: "ssg multilingual_full example"
date: "January 1, 2026"
language: "en"
hreflang: "en"
changefreq: "monthly"
translation_key: "about"
charset: "utf-8"
logo: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHZpZXdCb3g9JzAgMCAzMiAzMic+PHJlY3Qgd2lkdGg9JzMyJyBoZWlnaHQ9JzMyJyByeD0nNicgZmlsbD0nIzI1NjNlYicvPjx0ZXh0IHg9JzE2JyB5PScyMicgZm9udC1mYW1pbHk9J3NhbnMtc2VyaWYnIGZvbnQtc2l6ZT0nMTYnIGZvbnQtd2VpZ2h0PSc3MDAnIGZpbGw9JyNmZmYnIHRleHQtYW5jaG9yPSdtaWRkbGUnPk08L3RleHQ+PC9zdmc+"
logo_alt: "multilingual_full example logo"
logo_width: "32"
logo_height: "32"
name: "About"
---

# About

This page and its four siblings all carry `translation_key: "about"`
but sit at a **different slug in every locale**:

| Locale | Path |
| :--- | :--- |
| en | `/en/about/` |
| fr | `/fr/a-propos/` |
| de | `/de/ueber-uns/` |
| es | `/es/acerca-de/` |
| ja | `/ja/gaiyou/` |

Path matching cannot pair these — the paths differ — so without a
shared key each would be a singleton and none would receive any
`hreflang` at all. The key pairs them, and every one of the five ends
up advertising the other four.

Compare with `post-1` … `post-5`, which use the same slug in all five
locales and so pair by path with no key needed.
