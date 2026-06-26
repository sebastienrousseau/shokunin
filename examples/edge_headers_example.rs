#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Edge Headers Example — PQC-aware platform configs (v0.0.44, issue #550)
//!
//! Demonstrates the `EdgeHeadersPlugin` by running it against a temp
//! `site_dir` with all three platforms enabled (Cloudflare, Netlify,
//! Vercel) and dumping each emitted file to stdout. Useful for seeing
//! the exact `_headers` / `wrangler-headers.toml` /
//! `vercel-headers.json` payload your deploy step would upload.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example edge_headers_example
//! ```

use ssg::cmd::{EdgeHeadersConfig, SsgConfig};
use ssg::plugin::{Plugin, PluginContext};
use ssg::postprocess::EdgeHeadersPlugin;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let site_dir = tmp.path().join("site");
    fs::create_dir_all(&site_dir)?;

    // 1. Wire a config with all three platforms turned on, plus a
    //    sample override so the override path is exercised.
    let mut overrides = BTreeMap::new();
    let _ = overrides.insert(
        "Permissions-Policy".to_string(),
        "camera=(), geolocation=(self), microphone=()".to_string(),
    );
    let mut cfg = SsgConfig::default();
    cfg.edge_headers = EdgeHeadersConfig {
        targets: vec![
            "cloudflare".to_string(),
            "netlify".to_string(),
            "vercel".to_string(),
        ],
        overrides,
    };

    let ctx = PluginContext::with_config(
        Path::new("/dev/null"),
        Path::new("/dev/null"),
        &site_dir,
        Path::new("/dev/null"),
        cfg,
    );

    // 2. Run the emitter. It writes:
    //    - <site>/_headers                          (Netlify)
    //    - <site>/.ssg/edge/wrangler-headers.toml   (Cloudflare)
    //    - <site>/.ssg/edge/vercel-headers.json     (Vercel)
    EdgeHeadersPlugin::new().after_compile(&ctx)?;

    let edge_dir = site_dir.join(".ssg").join("edge");
    let outputs = [
        ("netlify _headers", site_dir.join("_headers")),
        (
            "cloudflare wrangler-headers.toml",
            edge_dir.join("wrangler-headers.toml"),
        ),
        (
            "vercel vercel-headers.json",
            edge_dir.join("vercel-headers.json"),
        ),
    ];

    for (label, path) in &outputs {
        let body = fs::read_to_string(path)?;
        println!("──── {label} ({} bytes) ────", body.len());
        println!("{body}");
    }

    println!(
        "[edge-headers] emitted {} files across 3 platforms",
        outputs.len()
    );

    Ok(())
}
