// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::accessibility` public types.

use ssg::accessibility::{
    AccessibilityIssue, AccessibilityReport, CriterionStatus, PageReport,
};

#[test]
fn criterion_status_variants_are_distinguishable() {
    assert!(!matches!(CriterionStatus::Automated, CriterionStatus::Manual));
    assert!(!matches!(CriterionStatus::Runtime, CriterionStatus::NotApplicable));
}

#[test]
fn accessibility_report_constructs_empty() {
    let r = AccessibilityReport {
        pages_scanned: 0,
        total_issues: 0,
        wcag_version: "2.2".to_string(),
        pages: vec![],
    };
    assert_eq!(r.pages_scanned, 0);
}

#[test]
fn page_report_with_issue_records_the_issue() {
    let issue = AccessibilityIssue {
        criterion: "1.1.1".to_string(),
        severity: "error".to_string(),
        message: "missing alt".to_string(),
    };
    let p = PageReport {
        path: "index.html".to_string(),
        issues: vec![issue],
    };
    assert_eq!(p.issues.len(), 1);
}
