// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Report and compliance-matrix types shared by every WCAG check.

use serde::{Deserialize, Serialize};

/// An individual accessibility issue found in a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityIssue {
    /// WCAG success criterion (e.g. "1.1.1").
    pub criterion: String,
    /// Severity: "error" or "warning".
    pub severity: String,
    /// Human-readable description.
    pub message: String,
}

/// Accessibility report for a single page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageReport {
    /// Relative path of the HTML file.
    pub path: String,
    /// Issues found.
    pub issues: Vec<AccessibilityIssue>,
}

/// Full accessibility report.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Default value for [`AccessibilityReport::wcag_version`] when the field
/// is absent from a serialised report (used by `#[serde(default = ...)]`).
pub(crate) fn default_wcag_version() -> String {
    "2.2".to_string()
}

/// How a single WCAG criterion is verified.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CriterionStatus {
    /// Verified at build time by this crate.
    Automated,
    /// Verified at runtime by a tool such as axe-core.
    Runtime,
    /// Requires human review (e.g. cognitive accessibility).
    Manual,
    /// Does not apply to static content (e.g. forms-only criteria).
    NotApplicable,
}

/// One row of the WCAG 2.2 compliance matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// WCAG 2.2 compliance matrix describing which criteria this crate can
/// verify automatically, and whether every scanned page passed them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WcagComplianceReport {
    /// Spec version this matrix is asserted against.
    pub wcag_version: String,
    /// Total pages scanned.
    pub pages_scanned: usize,
    /// Per-criterion compliance entries.
    pub criteria: Vec<CriterionEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wcag_version_is_22() {
        // Covers default_wcag_version fn body. Used by serde when the
        // wcag_version field is absent during deserialise.
        assert_eq!(default_wcag_version(), "2.2");
    }

    #[test]
    fn accessibility_report_deserialises_without_wcag_version() {
        // Confirms the serde default integration: a JSON blob without
        // wcag_version still parses and yields "2.2".
        let json = r#"{"pages_scanned":0,"total_issues":0,"pages":[]}"#;
        let r: AccessibilityReport = serde_json::from_str(json).unwrap();
        assert_eq!(r.wcag_version, "2.2");
    }
}
