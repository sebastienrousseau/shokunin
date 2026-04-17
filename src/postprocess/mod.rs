// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Post-processing plugins that fix staticdatagen output.
//!
//! These plugins run in the `after_compile` phase to sanitise XML feeds,
//! sitemaps, manifests, and HTML output produced by the upstream
//! `staticdatagen` crate.
//!
//! - `SitemapFixPlugin` -- Fixes 1, 2, 10: duplicate XML declarations,
//!   double-slash URLs, and per-page lastmod dates.
//! - `NewsSitemapFixPlugin` -- Fix 3: populates news-sitemap entries
//!   from front-matter metadata.
//! - `RssAggregatePlugin` -- Fix 4: aggregates per-page RSS items into
//!   the root feed.
//! - `ManifestFixPlugin` -- Fix 8: word-boundary-safe description
//!   truncation.
//! - `HtmlFixPlugin` -- Fix 9: repairs broken `.class=` image syntax
//!   and Fix 7: upgrades JSON-LD `@context` from `http` to `https`.
//! - `AtomFeedPlugin` -- Generates an Atom 1.0 `atom.xml` feed from
//!   `.meta.json` sidecars.

pub(crate) mod helpers;

mod atom;
mod html_fix;
mod manifest;
mod news_sitemap;
mod rss;
mod sitemap;

pub use atom::AtomFeedPlugin;
pub use html_fix::HtmlFixPlugin;
pub use manifest::ManifestFixPlugin;
pub use news_sitemap::NewsSitemapFixPlugin;
pub use rss::RssAggregatePlugin;
pub use sitemap::SitemapFixPlugin;
