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
//! Also demonstrates the two v0.0.47 security knobs (issue #586):
//!
//! - **`[security] sri_algorithm`** (plan §3 item 2.3) — picks the
//!   digest (`sha256` / `sha384` / `sha512`) that the fingerprint and
//!   CSP-extraction plugins stamp into `integrity=` attributes on
//!   externalized assets.
//! - **Per-page CSP → edge `_headers`** (spec B4, plan §3 item 2.4) —
//!   pages that keep inline `<script>`/`<style>` blocks (e.g. JSON-LD)
//!   get their own hash-strict per-path `Content-Security-Policy`
//!   entry in the platform files, instead of loosening the global
//!   policy with `'unsafe-inline'`.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example edge_headers_example
//! ```

use ssg::cmd::{EdgeHeadersConfig, SecurityConfig, SriAlgorithm, SsgConfig};
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

    // 1b. `[security] sri_algorithm` (v0.0.47, plan §3 item 2.3).
    //     The equivalent ssg.toml stanza is:
    //
    //         [security]
    //         sri_algorithm = "sha512"
    //
    //     The knob governs the SRI `integrity=` attributes that
    //     `assets::FingerprintPlugin` / `csp::CspPlugin` stamp on
    //     externalized assets. CSP *directive* hashes (the
    //     `'sha256-…'` sources below) always stay SHA-256 for UA
    //     compatibility. Default (section absent) is SHA-384.
    cfg.security = SecurityConfig {
        sri_algorithm: SriAlgorithm::Sha512,
    };

    // 1c. A built page that keeps an inline block: JSON-LD structured
    //     data survives CSP extraction by design, so this page needs
    //     its own hash-strict per-path CSP entry (spec B4).
    let page_rel = Path::new("blog/first-post/index.html");
    let page_path = site_dir.join(page_rel);
    fs::create_dir_all(page_path.parent().expect("page dir"))?;
    let page_html = concat!(
        "<html><head><title>First post</title>",
        r#"<script type="application/ld+json">{"@type":"BlogPosting"}</script>"#,
        "</head><body><p>hello</p></body></html>",
    );
    fs::write(&page_path, page_html)?;

    let ctx = PluginContext::with_config(
        Path::new("/dev/null"),
        Path::new("/dev/null"),
        &site_dir,
        Path::new("/dev/null"),
        cfg.clone(),
    );

    // 2. Run the emitter. It writes:
    //    - <site>/_headers                          (Netlify)
    //    - <site>/.ssg/edge/wrangler-headers.toml   (Cloudflare)
    //    - <site>/.ssg/edge/vercel-headers.json     (Vercel)
    //    `after_compile` emits the global `/*` policy; the fused
    //    transform pass (simulated below by calling `transform_html`
    //    directly) then re-emits with one per-path CSP group per page
    //    that still carries inline blocks.
    let plugin = EdgeHeadersPlugin::new();
    plugin.after_compile(&ctx)?;
    let _unchanged = plugin.transform_html(page_html, &page_path, &ctx)?;

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

    // 3. Show what landed:
    //    - the global `/*` group carries the hash-free baseline CSP;
    //    - `/blog/first-post/` gets a per-path group whose CSP embeds
    //      the exact sha256 of the page's inline JSON-LD block, and
    //      never `'unsafe-inline'`.
    let headers = fs::read_to_string(&outputs[0].1)?;
    assert!(
        headers.contains("/blog/first-post/"),
        "per-path CSP group missing from _headers"
    );
    assert!(
        !headers.contains("unsafe-inline"),
        "per-page CSP must stay hash-strict"
    );
    println!("[edge-headers] per-path CSP group emitted for /blog/first-post/");

    //    And the SRI knob: this is the exact `integrity=` value the
    //    fingerprint plugin stamps for an asset with this content
    //    under `[security] sri_algorithm = "sha512"`.
    let sri = cfg.security.sri_algorithm.integrity(b"body{margin:0}");
    println!("[edge-headers] sample sha512 SRI: {sri}");
    assert!(sri.starts_with("sha512-"));

    println!(
        "[edge-headers] emitted {} files across 3 platforms",
        outputs.len()
    );

    Ok(())
}
