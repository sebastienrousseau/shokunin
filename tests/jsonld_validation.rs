// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration test: walk every generated HTML file under `examples/*/public`
//! that an upstream example build has produced, extract every
//! `<script type="application/ld+json">` block, and assert the
//! [`ssg::seo::validate_jsonld`] invariants hold for the schema.org
//! types we ship.
//!
//! This runs only against pre-built example outputs — it does **not**
//! run a build itself (the heavy `example_outputs.rs` test does that).
//! If no example has been built yet, the test skips with a warning so
//! local `cargo test` runs don't require building the world.
//!
//! Resolves issue #467 acceptance criterion: "build example → extract
//! all JSON-LD → validate each".

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ssg::seo::validate_jsonld;
use std::{fs, path::Path};

fn collect_html(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_html(&p, out);
        } else if p.extension().is_some_and(|e| e == "html") {
            out.push(p);
        }
    }
}

#[test]
fn every_jsonld_block_in_built_examples_is_valid() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"));
    let examples = workspace.join("examples");

    let mut html_files = Vec::new();
    if examples.is_dir() {
        for entry in fs::read_dir(&examples).unwrap().flatten() {
            let public = entry.path().join("public");
            if public.is_dir() {
                collect_html(&public, &mut html_files);
            }
        }
    }

    // Locally a convenience; under CI a failure. A gate that validates
    // zero documents and reports `ok` looks exactly like one that
    // validated everything. `examples/multilingual_full` shipped 37
    // invalid `WebPage` blocks past this very test for months, because
    // nothing in CI populated its `public/` directory (#676).
    if html_files.is_empty() {
        let msg = "[jsonld_validation] no built example output found \
                   under examples/*/public — run `cargo build \
                   --examples && cargo test --test example_outputs` \
                   first to populate.";
        assert!(
            std::env::var_os("CI").is_none(),
            "{msg}\n\nThis is a hard failure under CI: a gate that \
             validates nothing must not report success."
        );
        eprintln!("{msg} Skipping (local run).");
        return;
    }

    let mut all_errors: Vec<(std::path::PathBuf, _)> = Vec::new();
    for path in &html_files {
        let html = fs::read_to_string(path).unwrap();
        let errors = validate_jsonld(&html);
        if !errors.is_empty() {
            for err in errors {
                all_errors.push((path.clone(), err));
            }
        }
    }

    if !all_errors.is_empty() {
        let mut msg = format!(
            "{} JSON-LD validation error(s) across {} file(s):\n",
            all_errors.len(),
            html_files.len()
        );
        for (path, err) in &all_errors {
            msg.push_str(&format!(
                "  {}: {err}\n",
                path.strip_prefix(workspace).unwrap_or(path).display()
            ));
        }
        panic!("{msg}");
    }

    eprintln!(
        "[jsonld_validation] {} HTML file(s) scanned, all JSON-LD blocks \
         pass schema.org required-field checks",
        html_files.len()
    );
}
