#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # ISR Manifest Example — Incremental Static Regeneration (v0.0.44, issue #546)
//!
//! Demonstrates the ISR manifest emitter. Without `--isr` the plugin is
//! not registered and the build stays byte-identical to v0.0.43. This
//! example calls [`build_manifest`] directly against a tiny temp site
//! and pretty-prints the resulting [`Manifest`] JSON so you can see the
//! shape the Edge runtime expects to find at `dist/.ssg/manifest.json`.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example isr_example
//! ```

use ssg::isr_manifest::build_manifest;
use std::fs;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let content_dir = tmp.path().join("content");
    let template_dir = tmp.path().join("templates");
    let site_dir = tmp.path().join("public");
    fs::create_dir_all(&content_dir)?;
    fs::create_dir_all(&template_dir)?;
    fs::create_dir_all(&site_dir)?;

    // 1. Two markdown pages: one with an `isr:` cache override, one without.
    fs::write(
        content_dir.join("index.md"),
        "---\ntitle: Home\n---\n# Welcome\n",
    )?;
    fs::write(
        content_dir.join("about.md"),
        "---\ntitle: About\nisr:\n  s_maxage: 600\n  swr: 3600\n---\n# About us\n",
    )?;

    // 2. Minimal layout templates so the source-hash inputs exist.
    fs::write(
        template_dir.join("index.html"),
        "<!doctype html><html><body>{{ content }}</body></html>",
    )?;
    fs::write(
        template_dir.join("page.html"),
        "<!doctype html><html><body>{{ content }}</body></html>",
    )?;

    // 3. Build the manifest — same call the `--isr`-gated plugin makes.
    let manifest = build_manifest(&content_dir, &template_dir, &site_dir)?;

    println!(
        "[isr] manifest version={} entries={} default_cache={}",
        manifest.version,
        manifest.len(),
        manifest.default_cache.to_cache_control(),
    );
    for (url, entry) in &manifest.entries {
        let cache = entry.cache.map_or_else(
            || "(inherits default)".to_string(),
            |c| c.to_cache_control(),
        );
        println!("  {url}  hash={}  cache={cache}", &entry.hash[..16]);
    }

    println!("[isr] full manifest JSON:");
    println!("{}", manifest.to_pretty_json()?);

    Ok(())
}
