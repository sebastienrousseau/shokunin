#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Audit Example — 14-gate CI audit (v0.0.44, issue #549)
//!
//! Demonstrates the native CI audit runner against a small fixture
//! site that is **deliberately well-formed**: a passing demo, not a
//! gallery of failures.
//!
//! The fixture ships:
//!
//! - `index.html` — proper `<title>`, `<meta description>`, canonical,
//!   Open Graph + Twitter cards, JSON-LD Article, lang attr, skip-link,
//!   alt text on every image, no broken links.
//! - `about.html` — same shape, second page so `BreadcrumbList` /
//!   sitemap gates have material to work with.
//! - `robots.txt` + `sitemap.xml` + `llms.txt` — satisfies the AI
//!   discovery + crawler gates.
//!
//! With this fixture, every gate either passes outright or fires only
//! `info`-level findings. You should see `[audit] highest severity:
//! info` (or "no findings") at the bottom.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example audit_example
//! ```

use ssg::audit::{AuditConfig, AuditRunner, Site};
use std::fs;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let site_dir = tmp.path().join("site");
    fs::create_dir_all(&site_dir)?;

    // -------------------------------------------------------------
    // 1. Build a deliberately-clean fixture site.
    // -------------------------------------------------------------
    fs::write(site_dir.join("index.html"), page_html("Home", "/"))?;
    fs::write(
        site_dir.join("about.html"),
        page_html("About", "/about.html"),
    )?;
    fs::write(site_dir.join("robots.txt"), ROBOTS_TXT)?;
    fs::write(site_dir.join("sitemap.xml"), SITEMAP_XML)?;
    fs::write(site_dir.join("llms.txt"), LLMS_TXT)?;
    // The pages reference /og.png — emit a tiny placeholder so the
    // broken-links gate doesn't fire, plus sibling .webp / .avif so
    // the images gate's "no modern format" check is satisfied.
    fs::write(site_dir.join("og.png"), PLACEHOLDER_PNG)?;
    fs::write(site_dir.join("og.webp"), PLACEHOLDER_PNG)?;
    fs::write(site_dir.join("og.avif"), PLACEHOLDER_PNG)?;
    // _headers satisfies the CSP/SRI gate (each page can omit the
    // <meta http-equiv> tag when a site-level header exists) AND
    // the PQC TLS gate (1-year HSTS + TLS 1.3 mention).
    fs::write(site_dir.join("_headers"), HEADERS_FILE)?;

    // -------------------------------------------------------------
    // 2. Boot a default runner. `skip_network` is on by default so the
    //    broken-link gate won't try to hit the network.
    // -------------------------------------------------------------
    let cfg = AuditConfig::default();
    let runner = AuditRunner::new(cfg);
    println!(
        "[audit] registered gates ({}): {:?}",
        runner.gate_names().len(),
        runner.gate_names(),
    );

    // -------------------------------------------------------------
    // 3. Load the site and run the audit.
    // -------------------------------------------------------------
    let site = Site::load(&site_dir)?;
    println!(
        "[audit] loaded site at {} ({} html files)",
        site.root.display(),
        site.html_files.len(),
    );
    let report = runner.run(&site);

    // -------------------------------------------------------------
    // 4. Print a compact summary table — one line per gate.
    // -------------------------------------------------------------
    println!("[audit] results:");
    println!(
        "{:<20}  {:>5}  {:>5}  {:>5}",
        "gate", "info", "warn", "error"
    );
    for gate in &report.gates {
        println!(
            "{:<20}  {:>5}  {:>5}  {:>5}",
            gate.name,
            gate.severity_counts.info,
            gate.severity_counts.warn,
            gate.severity_counts.error,
        );
    }

    if let Some(max) = report.max_severity() {
        println!("[audit] highest severity: {max}");
    } else {
        println!("[audit] no findings — site passed every gate cleanly");
    }

    Ok(())
}

fn page_html(title: &str, path: &str) -> String {
    let url = format!("https://example.invalid{path}");
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — Audit fixture</title>
  <meta name="description" content="A clean two-page demo for the SSG audit example.">
  <link rel="canonical" href="{url}">
  <meta property="og:title" content="{title} — Audit fixture">
  <meta property="og:description" content="A clean two-page demo for the SSG audit example.">
  <meta property="og:type" content="article">
  <meta property="og:url" content="{url}">
  <meta property="og:image" content="https://example.invalid/og.png">
  <meta name="twitter:card" content="summary_large_image">
  <script type="application/ld+json">
  {{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "{title} — Audit fixture",
    "datePublished": "2026-06-26",
    "author": {{ "@type": "Person", "name": "Audit demo" }},
    "image": "https://example.invalid/og.png",
    "url": "{url}"
  }}
  </script>
</head>
<body>
  <a href="#main" class="visually-hidden">Skip to main content</a>
  <header>
    <nav aria-label="Primary">
      <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/about.html">About</a></li>
      </ul>
    </nav>
  </header>
  <main id="main">
    <h1>{title}</h1>
    <p>This page exists so every SSG audit gate has a well-formed
       sample to walk. It satisfies WCAG 2.2 AAA where automatable,
       carries JSON-LD, Open Graph, Twitter Card metadata, declares a
       canonical URL, and never links to anything that doesn't exist.</p>
    <figure>
      <img src="/og.png" alt="Decorative gradient used as the Open Graph card."
           width="1200" height="630">
      <figcaption>The shared OG card.</figcaption>
    </figure>
  </main>
  <footer>
    <p>&copy; 2026 Audit fixture.</p>
  </footer>
</body>
</html>"##
    )
}

const ROBOTS_TXT: &str =
    "User-agent: *\nAllow: /\nSitemap: https://example.invalid/sitemap.xml\n";

const SITEMAP_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.invalid/</loc>
    <lastmod>2026-06-26</lastmod>
  </url>
  <url>
    <loc>https://example.invalid/about.html</loc>
    <lastmod>2026-06-26</lastmod>
  </url>
</urlset>
"#;

const LLMS_TXT: &str = r#"# Audit fixture

A clean two-page demo for the SSG audit example. See
https://example.invalid/ for the live site.

## Pages

- [Home](https://example.invalid/)
- [About](https://example.invalid/about.html)
"#;

// `_headers` declaring a strict CSP, a 1-year HSTS, and TLS 1.3 — the
// csp_sri gate accepts the CSP in lieu of a per-page meta http-equiv
// tag; the pqc_tls gate checks HSTS max-age >= 31536000 and that
// TLS 1.3 is declared somewhere.
const HEADERS_FILE: &str = "/*\n  \
    Content-Security-Policy: default-src 'self'\n  \
    Strict-Transport-Security: max-age=31536000; includeSubDomains; preload\n  \
    # TLS-1.3 baseline enforced at the edge (Cloudflare auto-negotiates X25519+ML-KEM-768).\n";

// 1×1 transparent PNG — the smallest valid PNG that satisfies the
// broken-links gate's existence check for `/og.png`.
const PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
    0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];
