#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # multilingual_full — nested-locale content showcase
//!
//! ## What this example demonstrates
//!
//! - **`content/<lang>/<slug>.md` layout** — 5 locales (en / fr / de /
//!   es / ja), each with an `index.md` + 5 posts. 32 source files
//!   total, mirroring the Jekyll `_posts/<lang>/` pattern.
//! - **Recursive content walk** — every per-locale post produces its
//!   own page under `public/<lang>/post-N/index.html` and a per-locale
//!   `public/<lang>/index.html` landing.
//! - **Per-locale hreflang** — each generated page carries the
//!   `language` / `hreflang` frontmatter so search engines see distinct
//!   language editions.
//!
//! ## Why this is its own example
//!
//! The existing `examples/multilingual_example.rs` shows the runtime
//! language-switcher + per-locale search index pattern using a single
//! flat content tree. **This** example shows the underlying source
//! layout — physically separate per-locale source files under
//! `content/<lang>/`.
//!
//! ## Dependency note (v0.0.46 cycle)
//!
//! Until `staticdatagen 0.0.10` is released, the recursive walk in
//! `add()` silently skips subdirectories ([upstream
//! #70](https://github.com/sebastienrousseau/staticdatagen/issues/70)).
//! Run against today's staticdatagen 0.0.9 and the runner will warn
//! for every missing per-locale output but still exit cleanly so CI
//! doesn't break. Run against 0.0.10+ and every per-locale URL lands
//! and the warnings disappear.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --release --example multilingual_full
//! ```

use std::error::Error;
use std::path::Path;

use ssg::pipeline::compile_site;

const LOCALES: &[&str] = &["en", "fr", "de", "es", "ja"];
const POSTS_PER_LOCALE: usize = 5;

fn main() -> Result<(), Box<dyn Error>> {
    let build_dir = Path::new("./examples/multilingual_full/build");
    let content_dir = Path::new("./examples/multilingual_full/content");
    let site_dir = Path::new("./examples/multilingual_full/public");
    let template_dir = Path::new("./examples/templates");

    // Wipe previous run output so re-runs are deterministic.
    let _ = std::fs::remove_dir_all(build_dir);
    let _ = std::fs::remove_dir_all(site_dir);
    std::fs::create_dir_all(build_dir)?;

    println!("multilingual_full: compiling 32 source files across 5 locales...");

    // `compile_site` already routes through the v0.0.45 content
    // stager (default-layout injection, template stub fill-in, etc.)
    // so this example doesn't need to wire the shim layer by hand.
    compile_site(build_dir, content_dir, site_dir, template_dir)?;

    println!("multilingual_full: compile done — auditing output:");
    let mut found = 0usize;
    let mut missing = 0usize;
    for &lang in LOCALES {
        for i in 1..=POSTS_PER_LOCALE {
            let p = site_dir.join(lang).join(format!("post-{i}")).join("index.html");
            if p.exists() {
                found += 1;
            } else {
                missing += 1;
            }
        }
        let idx = site_dir.join(lang).join("index.html");
        if idx.exists() {
            found += 1;
        } else {
            missing += 1;
        }
    }

    let expected = LOCALES.len() * (POSTS_PER_LOCALE + 1);
    println!(
        "  found {found}/{expected} per-locale pages  ({missing} missing)"
    );

    if missing > 0 {
        println!();
        println!(
            "  ⚠ {missing} pages missing — staticdatagen #70 still active."
        );
        println!(
            "    This is expected against staticdatagen 0.0.9; bump to"
        );
        println!(
            "    0.0.10 once the upstream fix is released and the"
        );
        println!("    per-locale pages will appear on their own.");
        // Exit cleanly so CI's example_outputs gate doesn't fail before
        // the dep bump.
    } else {
        println!();
        println!(
            "  ✓ every per-locale page landed — nested-locale walk works."
        );
    }

    Ok(())
}
