<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ssg-a11y

Standalone WCAG 2.2 AA accessibility checker for Rust web tooling —
build-time HTML validation, framework-agnostic.

This crate is part of the [SSG](https://crates.io/crates/ssg) workspace
but has **zero dependency** on SSG itself (no `Plugin` trait, no
`SsgError`, no file I/O). It operates purely on `&str` HTML in,
structured issue data out, so it drops into the build pipeline of any
Rust site or app generator — [Leptos](https://leptos.dev),
[Dioxus](https://dioxuslabs.com), [Yew](https://yew.rs), a hand-rolled
SSG, or a CI check that scans rendered HTML fixtures.

Documentation lives on [docs.rs](https://docs.rs/ssg-a11y) and the
canonical README for the wider workspace is the [repository
root](https://github.com/sebastienrousseau/static-site-generator#readme).

## Why

Automated accessibility linting catches a meaningful subset of WCAG
issues at zero runtime cost, before a page ever reaches a browser or a
runtime tool such as axe-core. Running it at build time means a CI
failure blocks the regression, rather than a user (or a later manual
audit) discovering it.

## Installation

```toml
[dependencies]
ssg-a11y = "0.0.47"
```

## Quick start

```rust
use ssg_a11y::check_page;

let html = r#"<html lang="en"><body>
    <nav><a href="/">Home</a></nav>
    <main><h1>Welcome</h1><img src="hero.jpg"></main>
</body></html>"#;

let issues = check_page(html);
for issue in &issues {
    println!("[{}] {}: {}", issue.severity, issue.criterion, issue.message);
}
// -> [warning] ARIA: <nav> missing aria-label
// -> [error] 1.1.1: <img> missing alt text: hero.jpg
```

`check_page` performs no I/O — reading the HTML (from disk, from a
rendered template, from an in-memory buffer) is entirely up to the
caller, which is what makes the crate embeddable anywhere.

## Aggregating a full-site report

```rust
use ssg_a11y::{check_page, AccessibilityIssue, AccessibilityReport, PageReport};
use std::collections::HashSet;

fn scan_site(pages: &[(&str, &str)]) -> AccessibilityReport {
    let mut report = AccessibilityReport {
        pages_scanned: pages.len(),
        total_issues: 0,
        wcag_version: "2.2".to_string(),
        pages: Vec::new(),
    };
    let mut failed_criteria: HashSet<String> = HashSet::new();

    for (path, html) in pages {
        let issues: Vec<AccessibilityIssue> = check_page(html);
        if !issues.is_empty() {
            for issue in &issues {
                let _ = failed_criteria.insert(issue.criterion.clone());
            }
            report.total_issues += issues.len();
            report.pages.push(PageReport {
                path: (*path).to_string(),
                issues,
            });
        }
    }

    report
}
```

`AccessibilityReport` derives `serde::Serialize`/`Deserialize`, so
`serde_json::to_string_pretty(&report)` produces a build artifact
(e.g. `accessibility-report.json`) that CI or a dashboard can consume.

## WCAG 2.2 compliance matrix

[`build_compliance_report`] produces a full matrix mapping every WCAG
2.2 success criterion this crate is aware of to its verification
status ([`CriterionStatus::Automated`], `Runtime`, `Manual`, or
`NotApplicable`), plus whether every scanned page passed it:

```rust
use ssg_a11y::build_compliance_report;
use std::collections::HashSet;

let failed: HashSet<String> = HashSet::new();
let matrix = build_compliance_report(42, &failed);
assert_eq!(matrix.wcag_version, "2.2");
```

## Checks performed

Build-time, per-page (`check_page`):

| Criterion | Level | Title                         |
|-----------|-------|--------------------------------|
| 1.1.1     | A     | Non-text Content (`<img alt>`) |
| 1.3.1     | A     | Heading hierarchy (no skipped levels) |
| 2.3.1     | A     | Banned elements (`<marquee>`, `<blink>`) |
| 2.4.4     | A     | Link Purpose (discernible text or `aria-label`) |
| 2.4.13    | AAA   | Focus Appearance (`:focus { outline: none }` without a compensating style) |
| 2.5.8     | AA    | Target Size Minimum (interactive elements < 24×24px) |
| 3.1.1     | A     | Language of Page (`<html lang>`) |
| —         | —     | ARIA landmarks (single `<main>`, `<nav aria-label>`) |

A standalone `check_consistent_help` helper (3.2.6, Consistent Help) is
also available for callers that want to run cross-page analysis
themselves; it is not wired into `check_page` because a single page has
no way to know whether a help mechanism is placed consistently
site-wide.

Everything else in the WCAG 2.2 AA matrix (contrast, reflow, text
spacing, dragging movements, redundant entry, ...) either requires a
real renderer (best verified with a runtime tool such as axe-core) or
manual review — `build_compliance_report` marks each accordingly so
consumers can track full-spec conformance rather than only what this
crate can check.

## Design notes

- **No parsing dependency.** Tag/attribute scanning and the inline-CSS
  preprocessor are hand-rolled char/byte iteration — no
  `html5ever`/`scraper`/`cssparser` in the dependency graph, which keeps
  this crate embeddable in constrained build pipelines (including WASM
  build tooling) without pulling a heavy parser.
- **Pure functions.** Every check takes `&str` and returns/pushes data;
  there is no global state, no file I/O, no framework coupling.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
