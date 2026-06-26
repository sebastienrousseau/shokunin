#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # View Transitions Example — opt-in lazy-nav client (v0.0.44, issue #547)
//!
//! Demonstrates the View Transitions plugin by:
//!
//! 1. Writing two skeleton HTML pages into a temp `site_dir`.
//! 2. Running [`ViewTransitionsPlugin`] across the directory exactly as
//!    the build pipeline would (`after_compile` + `transform_html`).
//! 3. Asserting `_transitions/ssg-transitions.js` lands on disk and that
//!    the injected `<script>` tag carries the `data-ssg-transitions`
//!    marker.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example view_transitions_example
//! ```

use ssg::plugin::{Plugin, PluginContext};
use ssg::view_transitions::ViewTransitionsPlugin;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let site_dir = tmp.path().join("site");
    fs::create_dir_all(&site_dir)?;

    // 1. Write two pages so the transform pass has something to work on.
    let pages = [("index.html", "Home"), ("about.html", "About")];
    for (name, title) in pages {
        let html = format!(
            "<!doctype html><html><head><title>{title}</title></head>\
             <body><main><h1>{title}</h1><a href=\"/about.html\">go</a>\
             </main></body></html>"
        );
        fs::write(site_dir.join(name), html)?;
    }

    // 2. Run the plugin — `after_compile` writes the client script,
    //    `transform_html` injects the <script> tag into each page.
    let ctx = PluginContext::new(
        Path::new("/dev/null"),
        Path::new("/dev/null"),
        &site_dir,
        Path::new("/dev/null"),
    );
    let plugin = ViewTransitionsPlugin::new();
    plugin.after_compile(&ctx)?;

    for (name, _) in pages {
        let path = site_dir.join(name);
        let html = fs::read_to_string(&path)?;
        let injected = plugin.transform_html(&html, &path, &ctx)?;
        fs::write(&path, &injected)?;
        assert!(
            injected.contains("data-ssg-transitions"),
            "expected marker injected into {name}",
        );
    }

    // 3. Confirm the client script was emitted.
    let script_path = site_dir.join("_transitions").join("ssg-transitions.js");
    let script = fs::read(&script_path)?;
    assert!(
        script_path.exists() && !script.is_empty(),
        "expected non-empty client script at {}",
        script_path.display(),
    );

    println!(
        "[view-transitions] wrote {} ({} bytes)",
        script_path.strip_prefix(tmp.path())?.display(),
        script.len(),
    );
    println!(
        "[view-transitions] injected <script> tag into {} pages",
        pages.len()
    );
    println!("[view-transitions] marker `data-ssg-transitions` present on every page");

    Ok(())
}
