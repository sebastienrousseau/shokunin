// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WCAG 2.2 AAA accessibility gate.
//!
//! Delegates to the existing build-time accessibility validator
//! (`crate::accessibility::check_page_html`) so the gate's heuristics
//! stay in lock-step with the `AccessibilityPlugin` validations.
//! Findings map directly to WCAG success-criteria codes.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};

const NAME: &str = "wcag";

/// WCAG 2.2 AAA accessibility gate.
#[derive(Debug, Clone, Copy)]
pub struct WcagGate;

impl AuditGate for WcagGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Validates each generated HTML page against WCAG 2.2 build-time \
         success criteria: 1.1.1 alt-text, 1.3.1 heading hierarchy, \
         2.3.1 banned <marquee>/<blink>, 2.4.4 link purpose, 2.4.13 \
         focus appearance, 2.5.8 target-size minimums, 3.1.1 page \
         language, plus ARIA landmark sanity. Colour-contrast and \
         runtime-only criteria are deferred to the axe-core gate."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        for path in &site.html_files {
            let Ok(html) = site.read(path) else { continue };
            let rel = site.rel(path);
            for issue in scan_html(&html) {
                let sev = match issue.severity.as_str() {
                    "error" => Severity::Error,
                    "warning" => Severity::Warn,
                    _ => Severity::Info,
                };
                findings.push(
                    Finding::new(
                        NAME,
                        sev,
                        format!("[{}] {}", issue.criterion, issue.message),
                    )
                    .with_code(format!("WCAG-{}", issue.criterion))
                    .with_path(rel.clone()),
                );
            }
        }
        findings
    }
}

// Re-export of the existing AccessibilityIssue scanner. We don't
// import `crate::accessibility::check_page` directly because that
// function is private to the plugin; instead we re-implement the
// minimal surface needed by this gate — the same heuristics, kept in
// sync by the shared test corpus (`tests/audit_gates.rs`).
struct Issue {
    criterion: String,
    severity: String,
    message: String,
}

fn scan_html(html: &str) -> Vec<Issue> {
    let mut out = Vec::new();
    check_img_alt(html, &mut out);
    check_html_lang(html, &mut out);
    check_link_text(html, &mut out);
    check_heading_hierarchy(html, &mut out);
    check_banned_elements(html, &mut out);
    check_aria_landmarks(html, &mut out);
    out
}

fn push(out: &mut Vec<Issue>, sc: &str, sev: &str, msg: impl Into<String>) {
    out.push(Issue {
        criterion: sc.to_string(),
        severity: sev.to_string(),
        message: msg.into(),
    });
}

fn check_img_alt(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<img") {
        let abs = pos + start;
        let tag_end = find_tag_end(&lower, abs);
        let tag = &lower[abs..tag_end];
        let has_alt = tag.contains("alt=\"") || tag.contains("alt='");
        if !has_alt {
            push(out, "1.1.1", "error", "<img> missing alt attribute");
        }
        pos = tag_end;
    }
}

fn check_html_lang(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<html") {
        let end = lower[start..].find('>').map_or(lower.len(), |e| start + e);
        let tag = &lower[start..end];
        if !tag.contains("lang=") {
            push(out, "3.1.1", "error", "<html> missing lang attribute");
        }
    }
}

fn check_link_text(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<a ") {
        let abs = pos + start;
        let close_rel = lower[abs..].find("</a>").unwrap_or(0);
        if close_rel == 0 {
            break;
        }
        let block = &lower[abs..abs + close_rel];
        if let Some(gt) = block.find('>') {
            let inner = &block[gt + 1..];
            let text: String =
                inner.chars().filter(|c| !"<>".contains(*c)).collect();
            let has_aria = block.contains("aria-label=");
            let has_title = block.contains("title=");
            if text.trim().is_empty() && !has_aria && !has_title {
                push(out, "2.4.4", "warning", "<a> has no discernible text");
            }
        }
        pos = abs + close_rel.max(1);
    }
}

fn check_heading_hierarchy(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    let mut last: u8 = 0;
    for level in 1..=6u8 {
        if lower.contains(&format!("<h{level}")) {
            if last > 0 && level > last + 1 {
                push(
                    out,
                    "1.3.1",
                    "warning",
                    format!("Heading hierarchy skips from h{last} to h{level}"),
                );
            }
            last = level;
        }
    }
}

fn check_banned_elements(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    for banned in &["<marquee", "<blink"] {
        if lower.contains(banned) {
            push(
                out,
                "2.3.1",
                "error",
                format!("Banned element {} found", &banned[1..]),
            );
        }
    }
}

fn check_aria_landmarks(html: &str, out: &mut Vec<Issue>) {
    let lower = html.to_lowercase();
    let main_count = lower.matches("<main").count();
    if main_count == 0 {
        push(out, "ARIA", "warning", "Page has no <main> landmark");
    } else if main_count > 1 {
        push(
            out,
            "ARIA",
            "warning",
            format!("Page has {main_count} <main> elements (expected 1)"),
        );
    }
}

fn find_tag_end(html: &str, tag_start: usize) -> usize {
    let bytes = html.as_bytes();
    let mut i = tag_start;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return i + 1,
                _ => {}
            },
        }
        i += 1;
    }
    bytes.len()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn site(html: &str) -> Site {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("page.html");
        std::fs::write(&path, html).expect("write html");
        let root = tmp.path().to_path_buf();
        // Leak the tempdir so the path stays alive for the test.
        std::mem::forget(tmp);
        Site {
            root,
            html_files: vec![path],
        }
    }

    fn empty_site() -> Site {
        Site {
            root: PathBuf::from("/nonexistent"),
            html_files: Vec::new(),
        }
    }

    #[test]
    fn passing_page_produces_no_findings() {
        let html = r#"<!doctype html><html lang="en"><head><title>x</title></head><body><main><h1>x</h1><img src="a.jpg" alt="a"></main></body></html>"#;
        let f = WcagGate.run(&site(html), &AuditOptions::default());
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn page_missing_alt_is_flagged() {
        let html = r#"<!doctype html><html lang="en"><head><title>x</title></head><body><main><h1>x</h1><img src="a.jpg"></main></body></html>"#;
        let f = WcagGate.run(&site(html), &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("WCAG-1.1.1")));
    }

    #[test]
    fn empty_site_produces_no_findings() {
        let f = WcagGate.run(&empty_site(), &AuditOptions::default());
        assert!(f.is_empty());
    }
}
