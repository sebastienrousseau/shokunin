---
title: "Cupping notes — July"
date: "July 1, 2026"
description: "Three quick impressions from this morning's cupping table."
banner_alt: "Heron Coffee banner"
banner_height: "398"
banner_width: "1440"
banner: ""
charset: "utf-8"
logo_alt: "Heron Coffee logo"
logo_height: "33"
logo_width: "100"
logo: ""
name: "Heron Coffee"
---

# Cupping notes — July

Three quick impressions from this morning's cupping table.

The washed Yirgacheffe is showing bergamot and black tea now that it
has rested a fortnight off roast. The Guji natural is all blueberry —
almost too much at first crack plus twenty seconds, so we are pulling
the drop two degrees earlier next batch. And the new Huila lot lands
somewhere pleasantly in between: red apple, panela, a long clean
finish.

All three will be on the brew bar from Saturday.

<!--
    This post is deliberately minimal (v0.0.47, issue #586, spec
    A4 + A2/B1): front matter carries no `permalink`, no `layout`,
    and none of the per-page SEO/social/feed boilerplate every other
    page in this example repeats (og/twitter meta, news_*, RSS
    item_* fields, etc.). It still carries `charset`, `name`, and
    the `banner_*`/`logo_*` groups — the shared site-chrome fields
    the page template renders unconditionally from front matter with
    no site-wide fallback. Omitting any of those produces broken
    output instead of demonstrating anything about minimal front
    matter: an inaccessible image element with no alt text, a
    missing charset meta tag, or an empty JSON-LD WebPage name.

    - The long-form date ("July 1, 2026") exercises the shared
      flexible date parser (`ssg::dates::parse_flexible_date`) used
      by the RSS/Atom/JSON-feed/sitemap plugins — pre-v0.0.47 this
      format produced "'day' component could not be parsed" warnings.
    - No `permalink:`/`url:` is declared, so the content stager
      derives one from the site's base_url
      (`ssg::content_stager::stage_content_with_site_defaults` →
      `ssg::urls::derive_permalink`), which keeps rss-gen's
      "channel.link is missing" hard-fail unreachable. The build in
      `examples/quickstart_example.rs` verifies the derived link
      appears in rss.xml.
    - `description` is still required: rss-gen (via the pinned
      staticdatagen 0.0.10) hard-validates channel.description. The
      full spec-A4 acceptance ("title+date+body only") completes with
      the staticdatagen 0.0.11 bump (plan §2 item 1.2, upstream leg),
      whose fallback chain skips underivable entries instead of
      aborting.
-->
