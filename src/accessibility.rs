// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Automated WCAG accessibility checker and ARIA validation plugin.
//!
//! Validates generated HTML against a subset of WCAG 2.1 Level AA
//! success criteria and checks ARIA landmark correctness. Produces
//! an `accessibility-report.json` in the site directory.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// An individual accessibility issue found in a page.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct AccessibilityIssue {
    /// WCAG success criterion (e.g. "1.1.1").
    pub criterion: String,
    /// Severity: "error" or "warning".
    pub severity: String,
    /// Human-readable description.
    pub message: String,
}

/// Accessibility report for a single page.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PageReport {
    /// Relative path of the HTML file.
    pub path: String,
    /// Issues found.
    pub issues: Vec<AccessibilityIssue>,
}

/// Full accessibility report.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct AccessibilityReport {
    /// Total pages scanned.
    pub pages_scanned: usize,
    /// Total issues found.
    pub total_issues: usize,
    /// Per-page reports (only pages with issues).
    pub pages: Vec<PageReport>,
}

/// Plugin that checks generated HTML for WCAG compliance.
///
/// Runs in `after_compile`. Non-blocking by default (logs warnings).
#[derive(Debug)]
pub struct AccessibilityPlugin;

impl Plugin for AccessibilityPlugin {
    fn name(&self) -> &str {
        "accessibility"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        let mut report = AccessibilityReport {
            pages_scanned: html_files.len(),
            total_issues: 0,
            pages: Vec::new(),
        };

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let rel = path
                .strip_prefix(&ctx.site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let issues = check_page(&html);
            if !issues.is_empty() {
                for issue in &issues {
                    log::warn!(
                        "[a11y] {} — [{}] {}",
                        rel,
                        issue.criterion,
                        issue.message
                    );
                }
                report.total_issues += issues.len();
                report.pages.push(PageReport { path: rel, issues });
            }
        }

        // Write report
        let report_path = ctx.site_dir.join("accessibility-report.json");
        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&report_path, json)?;

        if report.total_issues > 0 {
            log::warn!(
                "[a11y] {} issue(s) across {} page(s). Report: {:?}",
                report.total_issues,
                report.pages.len(),
                report_path
            );
        } else {
            log::info!(
                "[a11y] All {} page(s) passed checks",
                report.pages_scanned
            );
        }

        Ok(())
    }
}

/// Runs all WCAG checks on a single HTML page.
fn check_page(html: &str) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();

    // WCAG 1.1.1: Non-text Content — all <img> must have alt
    check_img_alt(html, &mut issues);

    // WCAG 3.1.1: Language of Page — <html> must have lang
    check_html_lang(html, &mut issues);

    // WCAG 2.4.4: Link Purpose — all <a> must have discernible text
    check_link_text(html, &mut issues);

    // WCAG 1.3.1: Heading hierarchy — no skipped levels
    check_heading_hierarchy(html, &mut issues);

    // WCAG 2.3.1: No flashing — no <marquee> or <blink>
    check_banned_elements(html, &mut issues);

    // ARIA: exactly one <main>, nav elements have aria-label
    check_aria_landmarks(html, &mut issues);

    issues
}

/// WCAG 1.1.1: Every <img> must have a non-empty alt attribute.
fn check_img_alt(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<img") {
        let abs = pos + start;
        let tag_end = lower[abs..]
            .find('>')
            .map(|e| abs + e + 1)
            .unwrap_or(lower.len());
        let tag = &lower[abs..tag_end];

        // Check for alt attribute
        let has_alt = tag.contains("alt=");
        let empty_alt = tag.contains("alt=\"\"") || tag.contains("alt=''");

        if !has_alt || empty_alt {
            // Extract src for the message
            let src = extract_attr_value(&html[abs..tag_end], "src")
                .unwrap_or_default();
            issues.push(AccessibilityIssue {
                criterion: "1.1.1".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "<img> missing alt text: {}",
                    if src.is_empty() { "(no src)" } else { &src }
                ),
            });
        }

        pos = tag_end;
    }
}

/// WCAG 3.1.1: <html> element must have a lang attribute.
fn check_html_lang(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<html") {
        let tag_end = lower[start..]
            .find('>')
            .map(|e| start + e)
            .unwrap_or(lower.len());
        let tag = &lower[start..tag_end];
        if !tag.contains("lang=") {
            issues.push(AccessibilityIssue {
                criterion: "3.1.1".to_string(),
                severity: "error".to_string(),
                message: "<html> missing lang attribute".to_string(),
            });
        }
    }
}

/// WCAG 2.4.4: Links must have discernible text.
fn check_link_text(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<a ") {
        let abs = pos + start;
        let close = lower[abs..].find("</a>").unwrap_or(lower.len() - abs);
        let full = &lower[abs..abs + close];

        // Get inner content (between > and </a>)
        if let Some(gt) = full.find('>') {
            let inner = &full[gt + 1..];
            let text = strip_tags_simple(inner);
            let has_aria = full.contains("aria-label=");
            let has_title = full.contains("title=");

            if text.trim().is_empty() && !has_aria && !has_title {
                let href = extract_attr_value(&html[abs..abs + close], "href")
                    .unwrap_or_default();
                issues.push(AccessibilityIssue {
                    criterion: "2.4.4".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "<a> has no discernible text: href={}",
                        if href.is_empty() { "(none)" } else { &href }
                    ),
                });
            }
        }

        pos = abs + close.max(1);
    }
}

/// WCAG 1.3.1: Heading levels must not skip (e.g. h1 → h3).
fn check_heading_hierarchy(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    let mut last_level: u8 = 0;

    for level in 1..=6u8 {
        let tag = format!("<h{}", level);
        if lower.contains(&tag) {
            if last_level > 0 && level > last_level + 1 {
                issues.push(AccessibilityIssue {
                    criterion: "1.3.1".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "Heading hierarchy skips from h{} to h{}",
                        last_level, level
                    ),
                });
            }
            last_level = level;
        }
    }
}

/// WCAG 2.3.1: No <marquee> or <blink> elements.
fn check_banned_elements(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    for tag in &["<marquee", "<blink"] {
        if lower.contains(tag) {
            issues.push(AccessibilityIssue {
                criterion: "2.3.1".to_string(),
                severity: "error".to_string(),
                message: format!("Banned element {} found", &tag[1..]),
            });
        }
    }
}

/// ARIA landmark checks: one <main>, nav has aria-label.
fn check_aria_landmarks(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();

    // Count <main> elements
    let main_count = lower.matches("<main").count();
    if main_count == 0 {
        issues.push(AccessibilityIssue {
            criterion: "ARIA".to_string(),
            severity: "warning".to_string(),
            message: "Page has no <main> landmark".to_string(),
        });
    } else if main_count > 1 {
        issues.push(AccessibilityIssue {
            criterion: "ARIA".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Page has {} <main> elements (expected 1)",
                main_count
            ),
        });
    }

    // Check <nav> elements have aria-label
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<nav") {
        let abs = pos + start;
        let tag_end = lower[abs..]
            .find('>')
            .map(|e| abs + e)
            .unwrap_or(lower.len());
        let tag = &lower[abs..tag_end];
        if !tag.contains("aria-label") && !tag.contains("aria-labelledby") {
            issues.push(AccessibilityIssue {
                criterion: "ARIA".to_string(),
                severity: "warning".to_string(),
                message: "<nav> missing aria-label".to_string(),
            });
        }
        pos = tag_end;
    }
}

/// Extracts an attribute value from an HTML tag string.
fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pattern = format!("{}=", attr);
    let start = lower.find(&pattern)?;
    let after = &tag[start + pattern.len()..];
    let trimmed = after.trim_start();
    if trimmed.starts_with('"') {
        let inner = &trimmed[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else if trimmed.starts_with('\'') {
        let inner = &trimmed[1..];
        let end = inner.find('\'')?;
        Some(inner[..end].to_string())
    } else {
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(trimmed.len());
        Some(trimmed[..end].to_string())
    }
}

/// Simple tag stripper for checking inner text.
fn strip_tags_simple(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

/// Recursively collects HTML files.
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_ctx(site_dir: &Path) -> PluginContext {
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
        )
    }

    #[test]
    fn test_img_alt_missing() {
        let html = r#"<html lang="en"><head></head><body><main><img src="photo.jpg"></main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.criterion == "1.1.1"));
    }

    #[test]
    fn test_img_alt_present() {
        let html = r#"<html lang="en"><head></head><body><main><img src="photo.jpg" alt="A photo"></main></body></html>"#;
        let issues = check_page(html);
        assert!(!issues.iter().any(|i| i.criterion == "1.1.1"));
    }

    #[test]
    fn test_html_lang_missing() {
        let html = "<html><head></head><body><main></main></body></html>";
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.criterion == "3.1.1"));
    }

    #[test]
    fn test_heading_skip() {
        let html = r#"<html lang="en"><head></head><body><main><h1>Title</h1><h3>Skip</h3></main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.message.contains("skips")));
    }

    #[test]
    fn test_banned_marquee() {
        let html = r#"<html lang="en"><head></head><body><main><marquee>No</marquee></main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.criterion == "2.3.1"));
    }

    #[test]
    fn test_nav_without_label() {
        let html = r#"<html lang="en"><head></head><body><nav></nav><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.message.contains("aria-label")));
    }

    #[test]
    fn test_nav_with_label_passes() {
        let html = r#"<html lang="en"><head></head><body><nav aria-label="Main"></nav><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(!issues.iter().any(|i| i.message.contains("aria-label")));
    }

    #[test]
    fn test_clean_page_no_issues() {
        let html = r#"<html lang="en"><head></head><body>
            <nav aria-label="Main"><a href="/">Home</a></nav>
            <main><h1>Title</h1><h2>Sub</h2>
            <img src="x.jpg" alt="Photo"></main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
    }

    #[test]
    fn test_plugin_writes_report() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            r#"<html><head></head><body><main><img src="x.jpg"></main></body></html>"#,
        )
        .unwrap();

        let ctx = test_ctx(&site);
        AccessibilityPlugin.after_compile(&ctx).unwrap();

        let report_path = site.join("accessibility-report.json");
        assert!(report_path.exists());

        let content = fs::read_to_string(&report_path).unwrap();
        let report: AccessibilityReport =
            serde_json::from_str(&content).unwrap();
        assert_eq!(report.pages_scanned, 1);
        assert!(report.total_issues > 0);
    }
}
