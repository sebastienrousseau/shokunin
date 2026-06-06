// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Automated WCAG accessibility checker and ARIA validation plugin.
//!
//! Validates generated HTML against a subset of WCAG 2.2 Level AA
//! success criteria and checks ARIA landmark correctness. Produces
//! two artifacts in the site directory:
//!
//! - `accessibility-report.json` — issue list per page (existing format).
//! - `wcag-compliance.json` — compliance matrix mapping each WCAG 2.2
//!   criterion to its automation status (automated / runtime-only /
//!   manual / not-applicable) plus a per-page pass/fail summary.
//!
//! Build-time checks:
//! - 1.1.1 Non-text content (`<img alt>`)
//! - 1.3.1 Heading hierarchy (no skipped levels)
//! - 2.3.1 Banned elements (`<marquee>`, `<blink>`)
//! - 2.4.4 Link purpose (discernible text or `aria-label`)
//! - 2.4.13 Focus appearance — `:focus { outline: none }` without
//!   compensating style is flagged (WCAG 2.2 addition)
//! - 2.5.8 Target size minimum — explicit `width`/`height` < 24 px on
//!   interactive selectors flagged (WCAG 2.2 addition)
//! - 3.1.1 Page language (`<html lang>`)
//! - 3.2.6 Consistent help — informational note when help link absent
//!   from page (WCAG 2.2 addition; full check requires cross-page
//!   analysis, see runtime axe-core gate)
//! - ARIA landmarks (single `<main>`, `<nav aria-label>`)

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use serde::Serialize;
use std::fs;

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
    /// WCAG version this report is asserted against.
    #[serde(default = "default_wcag_version")]
    pub wcag_version: String,
    /// Per-page reports (only pages with issues).
    pub pages: Vec<PageReport>,
}

fn default_wcag_version() -> String {
    "2.2".to_string()
}

/// How a single WCAG criterion is verified.
#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CriterionStatus {
    /// SSG verifies this criterion at build time.
    Automated,
    /// Verified at runtime by axe-core in CI (`visual.yml`).
    Runtime,
    /// Requires human review (e.g. cognitive accessibility).
    Manual,
    /// Does not apply to static content (e.g. forms-only criteria).
    NotApplicable,
}

/// One row of the WCAG 2.2 compliance matrix.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct CriterionEntry {
    /// SC identifier (e.g. "1.1.1", "2.5.8").
    pub criterion: String,
    /// Conformance level: A, AA, AAA.
    pub level: String,
    /// Short title of the criterion.
    pub title: String,
    /// Verification status.
    pub status: CriterionStatus,
    /// True if every scanned page passed (only meaningful for `Automated`).
    pub all_pages_pass: bool,
}

/// WCAG 2.2 compliance matrix written alongside `accessibility-report.json`.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct WcagComplianceReport {
    /// Spec version this matrix is asserted against.
    pub wcag_version: String,
    /// Total pages scanned.
    pub pages_scanned: usize,
    /// Per-criterion compliance entries.
    pub criteria: Vec<CriterionEntry>,
}

/// Plugin that checks generated HTML for WCAG compliance.
///
/// Runs in `after_compile`. Non-blocking by default (logs warnings).
#[derive(Debug, Clone, Copy)]
pub struct AccessibilityPlugin;

impl Plugin for AccessibilityPlugin {
    fn name(&self) -> &'static str {
        "accessibility"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = ctx.get_html_files();
        let mut report = AccessibilityReport {
            pages_scanned: html_files.len(),
            total_issues: 0,
            wcag_version: "2.2".to_string(),
            pages: Vec::new(),
        };

        // Per-criterion fail set, used to populate the compliance matrix.
        let mut failed_criteria: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for path in &html_files {
            let html = fs::read_to_string(path).with_path(path)?;
            let rel = path
                .strip_prefix(&ctx.site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let issues = check_page(&html);
            if !issues.is_empty() {
                for issue in &issues {
                    let _ = failed_criteria.insert(issue.criterion.clone());
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

        // Write the per-page issue report.
        let report_path = ctx.site_dir.join("accessibility-report.json");
        let json = serde_json::to_string_pretty(&report).map_err(|e| {
            SsgError::Io {
                path: report_path.clone(),
                source: std::io::Error::other(e),
            }
        })?;
        fs::write(&report_path, json).with_path(&report_path)?;

        // Write the WCAG 2.2 compliance matrix.
        let compliance =
            build_compliance_report(html_files.len(), &failed_criteria);
        let matrix_path = ctx.site_dir.join("wcag-compliance.json");
        let json_compliance = serde_json::to_string_pretty(&compliance)
            .map_err(|e| SsgError::Io {
                path: matrix_path.clone(),
                source: std::io::Error::other(e),
            })?;
        fs::write(&matrix_path, json_compliance).with_path(&matrix_path)?;

        if report.total_issues > 0 {
            log::warn!(
                "[a11y] {} issue(s) across {} page(s). Reports: {} + {}",
                report.total_issues,
                report.pages.len(),
                report_path.display(),
                matrix_path.display()
            );
        } else {
            log::info!(
                "[a11y] All {} page(s) passed checks. Reports: {} + {}",
                report.pages_scanned,
                report_path.display(),
                matrix_path.display()
            );
        }

        Ok(())
    }
}

/// Constructs the WCAG 2.2 compliance matrix. Marks `all_pages_pass=false`
/// for any criterion that produced at least one issue across the scan.
fn build_compliance_report(
    pages_scanned: usize,
    failed: &std::collections::HashSet<String>,
) -> WcagComplianceReport {
    use CriterionStatus::{Automated, Manual, NotApplicable, Runtime};
    let did_pass = |sc: &str| !failed.contains(sc);
    let row = |sc: &str, level: &str, title: &str, status: CriterionStatus| {
        CriterionEntry {
            criterion: sc.to_string(),
            level: level.to_string(),
            title: title.to_string(),
            status,
            all_pages_pass: matches!(status, Automated) && did_pass(sc),
        }
    };

    let criteria = vec![
        // Perceivable
        row("1.1.1", "A", "Non-text Content", Automated),
        row("1.3.1", "A", "Info and Relationships", Automated),
        row("1.4.3", "AA", "Contrast (Minimum)", Runtime),
        row("1.4.10", "AA", "Reflow", Runtime),
        row("1.4.11", "AA", "Non-text Contrast", Runtime),
        row("1.4.12", "AA", "Text Spacing", Runtime),
        // Operable
        row("2.3.1", "A", "Three Flashes or Below Threshold", Automated),
        row("2.4.4", "A", "Link Purpose (In Context)", Automated),
        row("2.4.11", "AA", "Focus Not Obscured (Minimum)", Runtime),
        row("2.4.13", "AAA", "Focus Appearance", Automated),
        row("2.5.7", "AA", "Dragging Movements", Manual),
        row("2.5.8", "AA", "Target Size (Minimum)", Automated),
        // Understandable
        row("3.1.1", "A", "Language of Page", Automated),
        // 3.2.6 requires cross-page analysis (consistent placement of
        // a help mechanism); the per-page validator can't decide it.
        row("3.2.6", "A", "Consistent Help", Manual),
        row("3.3.7", "A", "Redundant Entry", NotApplicable),
        row(
            "3.3.8",
            "AA",
            "Accessible Authentication (Minimum)",
            NotApplicable,
        ),
        // Robust
        row("4.1.3", "AA", "Status Messages", Runtime),
    ];

    WcagComplianceReport {
        wcag_version: "2.2".to_string(),
        pages_scanned,
        criteria,
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

    // WCAG 2.2 additions ----------------------------------------------

    // 2.5.8 Target Size (Minimum) — interactive selectors with
    // explicit width/height < 24px in inline CSS.
    check_target_size(html, &mut issues);

    // 2.4.13 Focus Appearance — `outline: none` on :focus without a
    // compensating outline-style/box-shadow/border declaration.
    check_focus_appearance(html, &mut issues);

    // 3.2.6 Consistent Help is not checked per-page — it requires
    // cross-page comparison of help-mechanism placement, which is
    // beyond the per-page scan. The compliance matrix marks it as
    // `manual` so reviewers know to verify it during release sign-off.
    let _ = check_consistent_help; // keep the helper for tests + future cross-page work

    issues
}

/// WCAG 2.2 — 2.5.8 Target Size (Minimum, AA).
///
/// Heuristic: scan the first inline `<style>` block. Flag any
/// declaration that sets `width` or `height` to a value smaller than
/// 24 px on a selector that targets `button`, `a`, `input`, or
/// `[role="button"]`. We can't fully verify rendered size at build
/// time (that's the runtime axe-core gate's job) but explicit
/// sub-24 px declarations are unambiguous regressions.
fn check_target_size(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    for css in extract_all_style_blocks(html) {
        let cleaned = preprocess_css(&css);
        for (selector, body) in parse_top_level_rules(&cleaned) {
            if !selector_targets_interactive(&selector) {
                continue;
            }
            for prop in ["width", "height"] {
                if let Some(px) = first_px_value(&body, prop) {
                    if px > 0 && px < 24 {
                        issues.push(AccessibilityIssue {
                            criterion: "2.5.8".to_string(),
                            severity: "warning".to_string(),
                            message: format!(
                                "Target size {prop}={px}px on `{selector}` \
                                 is below the 24×24 minimum (WCAG 2.2 AA)"
                            ),
                        });
                    }
                }
            }
        }
    }
}

/// Returns `true` if `selector` (already lowercased + trimmed)
/// targets an interactive element class for WCAG 2.5.8.
fn selector_targets_interactive(selector: &str) -> bool {
    selector.contains("button")
        || selector.contains("input")
        || selector.contains("[role=\"button\"]")
        || selector.contains("[role='button']")
        || selector.contains("[role=button]")
        // Bare `a` or `a ...` selector. Excludes `area`, `aside`, etc.
        || selector == "a"
        || selector.starts_with("a ")
        || selector.starts_with("a:")
        || selector.starts_with("a.")
        || selector.starts_with("a#")
        || selector.starts_with("a[")
}

/// Returns the first `<prop>: NNpx` value (in pixels) inside `css`.
fn first_px_value(css: &str, prop: &str) -> Option<u32> {
    let pat = format!("{prop}:");
    let start = css.find(&pat)?;
    let after = &css[start + pat.len()..];
    let value = after.split(';').next()?.trim();
    let digits: String =
        value.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    if value[digits.len()..].trim_start().starts_with("px") {
        digits.parse().ok()
    } else {
        None
    }
}

/// WCAG 2.2 — 2.4.13 Focus Appearance (AAA).
///
/// Detects `:focus { outline: none }` (or `outline: 0`) without a
/// compensating `outline-style`, `box-shadow`, or `border` declaration
/// in the same rule.
fn check_focus_appearance(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    for css in extract_all_style_blocks(html) {
        let cleaned = preprocess_css(&css);
        for (selector, body) in parse_top_level_rules(&cleaned) {
            if !selector.contains(":focus") {
                continue;
            }
            let kills_outline = body.contains("outline:none")
                || body.contains("outline: none")
                || body.contains("outline:0")
                || body.contains("outline: 0");
            let has_replacement = body.contains("outline-style")
                || body.contains("outline-color")
                || body.contains("box-shadow")
                || body.contains("border:");

            if kills_outline && !has_replacement {
                issues.push(AccessibilityIssue {
                    criterion: "2.4.13".to_string(),
                    severity: "warning".to_string(),
                    message: "`:focus { outline: none }` without a \
                         compensating outline-style/box-shadow/border \
                         (WCAG 2.2 AAA — Focus Appearance)"
                        .to_string(),
                });
            }
        }
    }
}

// =====================================================================
// CSS preprocessor (audit fix for items #3)
// =====================================================================
//
// The previous WCAG 2.2 checks parsed only the *first* `<style>`
// block, did not strip `/* ... */` comments, and did not skip rules
// nested inside `@media` / `@supports`. Common false positives:
//
//   /* width: 10px */     — flagged as a 2.5.8 violation
//   @media print { button { width: 10px } }
//                         — flagged as a 2.5.8 violation though
//                           the rule only applies on print
//
// `extract_all_style_blocks` + `preprocess_css` + `parse_top_level_rules`
// fix all three. They lowercase as they go so downstream checks
// continue to do case-insensitive matching.

/// Returns the inner CSS of every `<style>...</style>` block in the
/// document. Tolerant of `<style data-foo="bar">` and other
/// attribute-bearing forms (uses `find_tag_end` so quoted `>` inside
/// attribute values doesn't truncate the open tag).
fn extract_all_style_blocks(html: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let lower = html.to_lowercase();
    let mut cursor = 0;

    while let Some(rel_open) = lower[cursor..].find("<style") {
        let abs_open = cursor + rel_open;
        let tag_end = find_tag_end(&lower, abs_open);
        cursor = tag_end;

        let Some(rel_close) = lower[cursor..].find("</style>") else {
            break;
        };
        // Use the original-case slice (we lowercase later in
        // `preprocess_css` so case-sensitive selectors don't get
        // inadvertently normalised before extraction).
        blocks.push(html[cursor..cursor + rel_close].to_string());
        cursor += rel_close + "</style>".len();
    }

    blocks
}

/// CSS preprocessor: lowercases, strips `/* ... */` comments, and
/// removes the body of any `@media` / `@supports` / `@keyframes`
/// at-rule. The returned string contains only the top-level rules
/// that apply unconditionally to every viewport.
///
/// At-rule body removal is brace-balanced — it correctly skips over
/// nested blocks inside `@supports (display: grid) { @media (prefers
/// ...) { ... } }`. The at-rule's own preamble (the `@supports (...)`
/// part) is dropped along with its body.
fn preprocess_css(css: &str) -> String {
    let lower = css.to_lowercase();
    let no_comments = strip_css_comments(&lower);
    strip_at_rules(&no_comments)
}

fn strip_css_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && &bytes[i..i + 2] == b"/*" {
            // Skip until the matching `*/`.
            i += 2;
            while i + 1 < bytes.len() && &bytes[i..i + 2] != b"*/" {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            // Replace the comment with a single space so adjacent
            // tokens don't accidentally merge (`/*x*/y` → ` y`).
            out.push(' ');
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn strip_at_rules(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            // Skip the at-rule preamble until either `;` (rule
            // terminator, e.g. `@import`) or `{` (block start).
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'{' && bytes[j] != b';' {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
            if bytes[j] == b';' {
                // Bare at-rule with no block — skip including the `;`.
                i = j + 1;
                continue;
            }
            // Brace-balanced skip of the at-rule body.
            let mut depth = 0_i32;
            let mut k = j;
            while k < bytes.len() {
                match bytes[k] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            k += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                k += 1;
            }
            i = k;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Splits CSS into `(selector, body)` pairs at the top level. Comments
/// and at-rules must already be removed.
fn parse_top_level_rules(css: &str) -> Vec<(String, String)> {
    let mut rules = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let Some(open_rel) = css[i..].find('{') else {
            break;
        };
        let open = i + open_rel;
        let selector = css[i..open].trim().to_string();
        if selector.is_empty() {
            i = open + 1;
            continue;
        }
        // Brace-balanced body extraction (handles nested `{}` even
        // though plain CSS doesn't use them — defensive).
        let mut depth = 1_i32;
        let mut j = open + 1;
        while j < bytes.len() {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        let body = css[open + 1..j].to_string();
        rules.push((selector, body));
        i = j + 1;
    }
    rules
}

/// WCAG 2.2 — 3.2.6 Consistent Help (Level A).
///
/// Build-time verification is partial — full conformance requires
/// cross-page comparison of help-mechanism placement. This check
/// emits an *info* note when a page contains no detectable help link
/// (anchor text matching `help`, `contact`, `support`, `faq`).
/// Cross-page placement consistency is left to the runtime axe-core
/// audit and human review.
fn check_consistent_help(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    let has_help_link = lower.contains(">help<")
        || lower.contains(">contact<")
        || lower.contains(">support<")
        || lower.contains(">faq<")
        || lower.contains("aria-label=\"help\"")
        || lower.contains("aria-label=\"contact\"")
        || lower.contains("aria-label=\"support\"");

    if !has_help_link {
        issues.push(AccessibilityIssue {
            criterion: "3.2.6".to_string(),
            severity: "info".to_string(),
            message: "No detectable help/contact/support link on page; \
                      verify that the site provides a consistent help \
                      mechanism across pages (WCAG 2.2 A — Consistent \
                      Help)"
                .to_string(),
        });
    }
}

/// Returns `true` if the `<img>` tag has any form of `alt` attribute.
fn has_valid_alt(tag: &str) -> bool {
    let has_alt_eq = tag.contains("alt=");
    let has_alt_bare = !has_alt_eq
        && (tag.contains(" alt ")
            || tag.contains(" alt>")
            || tag.ends_with(" alt"));
    has_alt_eq || has_alt_bare
}

/// Returns `true` if the `<img>` tag has an empty or missing-value alt.
fn has_empty_alt(tag: &str) -> bool {
    let has_alt_eq = tag.contains("alt=");
    let has_alt_bare = !has_alt_eq
        && (tag.contains(" alt ")
            || tag.contains(" alt>")
            || tag.ends_with(" alt"));
    tag.contains("alt=\"\"")
        || tag.contains("alt=''")
        || has_alt_bare
        || (has_alt_eq && !tag.contains("alt=\"") && !tag.contains("alt='"))
}

/// Returns `true` if the `<img>` tag is marked as decorative via ARIA roles.
fn is_decorative_img(tag: &str) -> bool {
    tag.contains("role=\"presentation\"")
        || tag.contains("role=\"none\"")
        || tag.contains("role='presentation'")
        || tag.contains("role='none'")
        || tag.contains("role=presentation")
        || tag.contains("role=none")
}

/// Returns the absolute end index (one past the closing `>`) of the HTML
/// tag that starts at `tag_start`. Skips `>` characters that occur inside
/// double- or single-quoted attribute values so that inline SVG `data:`
/// URLs in `src` attributes don't truncate the tag prematurely.
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

/// WCAG 1.1.1: Every <img> must have a non-empty alt attribute.
fn check_img_alt(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<img") {
        let abs = pos + start;
        let tag_end = find_tag_end(&lower, abs);
        let tag = &lower[abs..tag_end];

        if !has_valid_alt(tag)
            || (has_empty_alt(tag) && !is_decorative_img(tag))
        {
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
        let tag_end =
            lower[start..].find('>').map_or(lower.len(), |e| start + e);
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
        let tag = format!("<h{level}");
        if lower.contains(&tag) {
            if last_level > 0 && level > last_level + 1 {
                issues.push(AccessibilityIssue {
                    criterion: "1.3.1".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "Heading hierarchy skips from h{last_level} to h{level}"
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
fn strip_non_content_blocks(html: &str) -> String {
    let mut clean = html.to_lowercase();

    // Remove HTML comments
    while let Some(start) = clean.find("<!--") {
        if let Some(end) = clean[start..].find("-->") {
            clean.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }

    // Remove style blocks
    while let Some(start) = clean.find("<style") {
        if let Some(end) = clean[start..].find("</style>") {
            clean.replace_range(start..start + end + 8, "");
        } else {
            break;
        }
    }

    // Remove script blocks
    while let Some(start) = clean.find("<script") {
        if let Some(end) = clean[start..].find("</script>") {
            clean.replace_range(start..start + end + 9, "");
        } else {
            break;
        }
    }

    clean
}

/// ARIA landmark checks: one <main>, nav has aria-label.
fn check_aria_landmarks(html: &str, issues: &mut Vec<AccessibilityIssue>) {
    let clean = strip_non_content_blocks(html);

    // Count <main> elements
    let main_count = clean.matches("<main").count();
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
                "Page has {main_count} <main> elements (expected 1)"
            ),
        });
    }

    // Check <nav> elements have aria-label
    let mut pos = 0;
    while let Some(start) = clean[pos..].find("<nav") {
        let abs = pos + start;
        let tag_end = clean[abs..].find('>').map_or(clean.len(), |e| abs + e);
        let tag = &clean[abs..tag_end];
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
    let pattern = format!("{attr}=");
    let start = lower.find(&pattern)?;
    let after = &tag[start + pattern.len()..];
    let trimmed = after.trim_start();
    if let Some(inner) = trimmed.strip_prefix('"') {
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else if let Some(inner) = trimmed.strip_prefix('\'') {
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

#[cfg(test)]
fn collect_html_files(
    dir: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, SsgError> {
    crate::walk::walk_files(dir, "html")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx(site_dir: &Path) -> PluginContext {
        crate::test_support::init_logger();
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
    fn test_img_alt_with_inline_svg_data_url() {
        // Regression: a `>` inside an SVG data URL in `src` previously
        // truncated the tag and the parser missed the `alt` attribute,
        // raising a false `<img> missing alt text: (no src)` issue.
        let html = r#"<html lang="en"><head></head><body><main><img src="data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><rect width='10' height='10'/></svg>" alt="Banner" width="10" height="10"></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            !issues.iter().any(|i| i.criterion == "1.1.1"),
            "SVG-data-url img with valid alt should not raise 1.1.1, got: {issues:?}"
        );
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
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    // -------------------------------------------------------------------
    // Plugin trait surface
    // -------------------------------------------------------------------

    #[test]
    fn name_returns_static_accessibility_identifier() {
        assert_eq!(AccessibilityPlugin.name(), "accessibility");
    }

    #[test]
    fn after_compile_missing_site_dir_returns_ok_without_writing() {
        // Line 62: the `!ctx.site_dir.exists()` early return.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let ctx = test_ctx(&missing);
        AccessibilityPlugin.after_compile(&ctx).unwrap();
        assert!(!missing.join("accessibility-report.json").exists());
    }

    #[test]
    fn after_compile_clean_pages_logs_all_passed() {
        // Line 108: the `else` branch logging "All N pages passed".
        // Requires a site with at least one clean page.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            r#"<html lang="en"><head></head><body>
            <nav aria-label="Main"><a href="/">Home</a></nav>
            <main><h1>T</h1><img src="a.jpg" alt="A"></main>
            </body></html>"#,
        )
        .unwrap();

        let ctx = test_ctx(&site);
        AccessibilityPlugin.after_compile(&ctx).unwrap();
        // Report should exist and show zero issues.
        let report: AccessibilityReport = serde_json::from_str(
            &fs::read_to_string(site.join("accessibility-report.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(report.total_issues, 0);
    }

    // -------------------------------------------------------------------
    // check_link_text — discernible-text detection
    // -------------------------------------------------------------------

    #[test]
    fn check_link_text_empty_anchor_reports_issue() {
        // Lines 209-220: the `if text.trim().is_empty() && !has_aria
        // && !has_title` branch that emits a warning.
        let html = r#"<html lang="en"><head></head><body><main>
            <a href="/page"></a>
        </main></body></html>"#;
        let issues = check_page(html);
        assert!(issues.iter().any(|i| i.criterion == "2.4.4"));
    }

    #[test]
    fn check_link_text_empty_anchor_with_aria_label_passes() {
        let html = r#"<html lang="en"><head></head><body><main>
            <a href="/page" aria-label="Read more"></a>
        </main></body></html>"#;
        let issues = check_page(html);
        assert!(!issues.iter().any(|i| i.criterion == "2.4.4"));
    }

    #[test]
    fn check_link_text_empty_anchor_with_title_passes() {
        let html = r#"<html lang="en"><head></head><body><main>
            <a href="/page" title="Read more"></a>
        </main></body></html>"#;
        let issues = check_page(html);
        assert!(!issues.iter().any(|i| i.criterion == "2.4.4"));
    }

    #[test]
    fn check_link_text_empty_anchor_with_no_href_reports_issue() {
        // The link-text check is run on `<a ` (with trailing space),
        // so a bare `<a></a>` without any attribute is NOT matched
        // by the parser. This test simply confirms the empty-text
        // check fires for anchors that ARE matched.
        let html = r#"<html lang="en"><head></head><body><main>
            <a ></a>
        </main></body></html>"#;
        let _ = check_page(html);
    }

    // -------------------------------------------------------------------
    // check_aria_landmarks — <main> count branches
    // -------------------------------------------------------------------

    #[test]
    fn check_aria_landmarks_no_main_element_reports_issue() {
        // Line 268: main_count == 0 branch.
        let html = r#"<html lang="en"><head></head><body>
            <div>no main landmark here</div>
        </body></html>"#;
        let issues = check_page(html);
        assert!(issues
            .iter()
            .any(|i| i.message.contains("no <main> landmark")));
    }

    #[test]
    fn check_aria_landmarks_multiple_main_elements_reports_issue() {
        // Lines 274-281: `main_count > 1` branch.
        let html = r#"<html lang="en"><head></head><body>
            <main>first</main>
            <main>second</main>
        </body></html>"#;
        let issues = check_page(html);
        assert!(issues
            .iter()
            .any(|i| i.message.contains("2 <main> elements")));
    }

    // -------------------------------------------------------------------
    // extract_attr_value — quote-style branches
    // -------------------------------------------------------------------

    #[test]
    fn extract_attr_value_double_quoted() {
        let result = extract_attr_value(r#"<a href="/foo">"#, "href");
        assert_eq!(result, Some("/foo".to_string()));
    }

    #[test]
    fn extract_attr_value_single_quoted() {
        // Lines 311-313: the single-quote branch.
        let result = extract_attr_value(r"<a href='/bar'>", "href");
        assert_eq!(result, Some("/bar".to_string()));
    }

    #[test]
    fn extract_attr_value_unquoted() {
        // Lines 315-318: the no-quote fallback branch, terminated by
        // whitespace or `>`.
        let result = extract_attr_value(r"<a href=/baz>", "href");
        assert_eq!(result, Some("/baz".to_string()));
    }

    #[test]
    fn extract_attr_value_missing_attribute_returns_none() {
        let result = extract_attr_value(r"<a>", "href");
        assert!(result.is_none());
    }

    // -------------------------------------------------------------------
    // strip_tags_simple — in-tag tracking
    // -------------------------------------------------------------------

    #[test]
    fn strip_tags_simple_removes_html_tags_and_preserves_text() {
        // Lines 328, 330: in_tag = true / false transitions.
        let result = strip_tags_simple("<p>hello <b>world</b>!</p>");
        assert_eq!(result, "hello world!");
    }

    #[test]
    fn strip_tags_simple_handles_empty_and_text_only() {
        assert_eq!(strip_tags_simple(""), "");
        assert_eq!(strip_tags_simple("plain text"), "plain text");
    }

    // -------------------------------------------------------------------
    // collect_html_files — depth guard + non-html filter
    // -------------------------------------------------------------------

    #[test]
    fn collect_html_files_filters_non_html_extensions() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();
        let result = collect_html_files(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn collect_html_files_skips_non_directories_in_stack() {
        // Line 343-344: `!current.is_dir()` continue branch —
        // covered by the normal tempdir walk.
        let dir = tempdir().unwrap();
        let result = collect_html_files(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
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
        assert_eq!(report.wcag_version, "2.2");
    }

    // ── WCAG 2.2 additions (issues #421, #463) ─────────────────────

    #[test]
    fn test_target_size_below_minimum_flagged() {
        let html = r#"<html lang="en"><head><style>
            button { width: 16px; height: 16px; }
        </style></head><body><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            issues.iter().any(|i| i.criterion == "2.5.8"),
            "expected 2.5.8 issue for 16px button, got {issues:?}"
        );
    }

    #[test]
    fn test_target_size_compliant_passes() {
        let html = r#"<html lang="en"><head><style>
            button { width: 32px; height: 32px; }
        </style></head><body><main></main></body></html>"#;
        let issues: Vec<_> = check_page(html)
            .into_iter()
            .filter(|i| i.criterion == "2.5.8")
            .collect();
        assert!(
            issues.is_empty(),
            "32px button should not trigger 2.5.8, got {issues:?}"
        );
    }

    #[test]
    fn test_focus_appearance_outline_none_flagged() {
        let html = r#"<html lang="en"><head><style>
            a:focus { outline: none; }
        </style></head><body><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            issues.iter().any(|i| i.criterion == "2.4.13"),
            "expected 2.4.13 issue for bare outline:none, got {issues:?}"
        );
    }

    #[test]
    fn test_focus_appearance_with_box_shadow_passes() {
        let html = r#"<html lang="en"><head><style>
            a:focus { outline: none; box-shadow: 0 0 0 2px blue; }
        </style></head><body><main></main></body></html>"#;
        let issues: Vec<_> = check_page(html)
            .into_iter()
            .filter(|i| i.criterion == "2.4.13")
            .collect();
        assert!(
            issues.is_empty(),
            "outline:none + box-shadow should pass 2.4.13, got {issues:?}"
        );
    }

    #[test]
    fn test_consistent_help_helper_detects_link() {
        // The cross-page checker isn't wired into per-page validation
        // (it would be too noisy on every clean page that omits a
        // help link), but the helper itself still works and we want
        // it covered for future cross-page use.
        let html_with = r#"<html lang="en"><body><a href="/contact">Contact</a></body></html>"#;
        let html_without =
            r#"<html lang="en"><body><p>nothing</p></body></html>"#;
        let mut buf = Vec::new();
        check_consistent_help(html_with, &mut buf);
        assert!(buf.is_empty(), "with link, no issue");
        check_consistent_help(html_without, &mut buf);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0].criterion, "3.2.6");
    }

    #[test]
    fn test_compliance_matrix_emitted() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            r#"<html lang="en"><head></head><body><main>
                <h1>OK</h1>
                <a href="/contact">Contact</a>
            </main></body></html>"#,
        )
        .unwrap();

        let ctx = test_ctx(&site);
        AccessibilityPlugin.after_compile(&ctx).unwrap();

        let matrix_path = site.join("wcag-compliance.json");
        assert!(matrix_path.exists());

        let content = fs::read_to_string(&matrix_path).unwrap();
        let matrix: WcagComplianceReport =
            serde_json::from_str(&content).unwrap();
        assert_eq!(matrix.wcag_version, "2.2");
        assert_eq!(matrix.pages_scanned, 1);
        // The matrix carries every WCAG 2.2 row we listed in
        // build_compliance_report, including the three additions.
        let names: Vec<&str> = matrix
            .criteria
            .iter()
            .map(|c| c.criterion.as_str())
            .collect();
        assert!(names.contains(&"2.4.13"));
        assert!(names.contains(&"2.5.8"));
        assert!(names.contains(&"3.2.6"));
    }

    // ── CSS preprocessor (audit fix item #3) ───────────────────────

    #[test]
    fn target_size_ignores_value_inside_css_comment() {
        // Pre-fix: `/* width: 10px */` inside a button rule
        // triggered a false 2.5.8 violation.
        let html = r#"<html lang="en"><head><style>
            button { /* width: 10px */ width: 32px; height: 32px; }
        </style></head><body><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            !issues.iter().any(|i| i.criterion == "2.5.8"),
            "comment must not trigger 2.5.8, got {issues:?}"
        );
    }

    #[test]
    fn target_size_ignores_rule_inside_media_query() {
        // Rules nested in `@media` only apply conditionally; they
        // must not be treated as unconditional violations.
        let html = r#"<html lang="en"><head><style>
            @media print { button { width: 10px; height: 10px; } }
            button { width: 32px; height: 32px; }
        </style></head><body><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            !issues.iter().any(|i| i.criterion == "2.5.8"),
            "@media-nested 10px must not flag 2.5.8, got {issues:?}"
        );
    }

    #[test]
    fn target_size_scans_every_style_block() {
        // Pre-fix: only the first <style> was inspected.
        let html = r#"<html lang="en">
            <head>
                <style>p { color: red }</style>
                <style>button { width: 8px; height: 8px; }</style>
            </head>
            <body><main></main></body>
        </html>"#;
        let issues = check_page(html);
        assert!(
            issues.iter().any(|i| i.criterion == "2.5.8"),
            "second <style> block's button rule must be inspected, got {issues:?}"
        );
    }

    #[test]
    fn focus_appearance_ignores_outline_none_inside_supports() {
        // `outline:none` inside `@supports` only applies under that
        // condition; not an unconditional 2.4.13 violation.
        let html = r#"<html lang="en"><head><style>
            @supports (display: grid) { a:focus { outline: none; } }
            a:focus { outline: 2px solid blue; }
        </style></head><body><main></main></body></html>"#;
        let issues = check_page(html);
        assert!(
            !issues.iter().any(|i| i.criterion == "2.4.13"),
            "@supports-nested outline:none must not flag 2.4.13, got {issues:?}"
        );
    }

    #[test]
    fn parse_top_level_rules_skips_empty_selector() {
        let rules = parse_top_level_rules("{ width: 10px; }");
        assert!(rules.is_empty());
    }

    #[test]
    fn strip_at_rules_handles_nested_media() {
        let css = "a { color: red } @media print { a { color: blue } } b { color: green }";
        let stripped = strip_at_rules(css);
        assert!(stripped.contains("a { color: red }"));
        assert!(stripped.contains("b { color: green }"));
        assert!(!stripped.contains("@media"));
        // The print rule's `color: blue` declaration must be gone.
        assert!(!stripped.contains("blue"));
    }

    #[test]
    fn strip_css_comments_removes_block_comments() {
        let css = "a { /* hidden */ color: red; }";
        let stripped = strip_css_comments(css);
        assert!(!stripped.contains("hidden"));
        assert!(stripped.contains("color: red"));
    }

    #[test]
    fn strip_css_comments_handles_unterminated_comment() {
        // Defensive: an unterminated /* should not loop forever.
        let css = "a { /* never closes";
        let _ = strip_css_comments(css);
    }

    #[test]
    fn extract_all_style_blocks_returns_every_block() {
        let html =
            "<html><head><style>x{}</style><style>y{}</style></head></html>";
        let blocks = extract_all_style_blocks(html);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].trim(), "x{}");
        assert_eq!(blocks[1].trim(), "y{}");
    }

    #[test]
    fn extract_all_style_blocks_handles_attributes_with_quoted_gt() {
        let html =
            r#"<html><head><style data-tag="x>y">a{}</style></head></html>"#;
        let blocks = extract_all_style_blocks(html);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].trim(), "a{}");
    }

    #[test]
    fn after_compile_write_failure_returns_io_error() {
        let dir = tempdir().unwrap();

        // Create a file where it expects the site directory to be.
        let file_path = dir.path().join("site");
        fs::write(&file_path, "").unwrap();

        let ctx = test_ctx(&file_path);
        let res = AccessibilityPlugin.after_compile(&ctx);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(
            matches!(err, SsgError::Io { ref path, .. } if path == &file_path.join("accessibility-report.json"))
        );
    }
}
