#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Audit Example — 14-gate CI audit (v0.0.44, issue #549)
//!
//! Demonstrates the native CI audit runner against a tiny fixture site.
//! Boots up an [`AuditRunner`] with the 14 built-in gates registered,
//! drops a minimal `index.html` into a temp dir, runs the suite, and
//! prints the resulting findings table to stdout.
//!
//! The fixture is intentionally rough so a handful of gates have
//! something to report — you should see warnings or info findings from
//! WCAG, metadata, JSON-LD, CSP/SRI, and similar gates.
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

    // 1. A minimal but realistic index.html so the audit gates have
    //    something to chew on.
    let html = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Audit fixture</title>
  <meta name="description" content="A tiny demo page for the audit example.">
  <link rel="canonical" href="https://example.invalid/">
</head>
<body>
  <header><a href="#main">Skip to main</a></header>
  <main id="main">
    <h1>Audit fixture</h1>
    <p>Hello from the audit example. This page exists only so the
       14 SSG audit gates have a real file to walk.</p>
    <img src="/missing.png">
  </main>
  <footer>&copy; 2026</footer>
</body>
</html>"##;
    fs::write(site_dir.join("index.html"), html)?;

    // 2. Boot a default runner. `skip_network` is on by default so the
    //    broken-link gate won't try to hit the network.
    let cfg = AuditConfig::default();
    let runner = AuditRunner::new(cfg);
    println!(
        "[audit] registered gates ({}): {:?}",
        runner.gate_names().len(),
        runner.gate_names(),
    );

    // 3. Load the site and run the audit.
    let site = Site::load(&site_dir)?;
    println!(
        "[audit] loaded site at {} ({} html files)",
        site.root.display(),
        site.html_files.len(),
    );
    let report = runner.run(&site);

    // 4. Print a compact summary table — one line per gate.
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
