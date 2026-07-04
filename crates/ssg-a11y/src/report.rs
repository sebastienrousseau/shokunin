// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Construction of the WCAG 2.2 compliance matrix.

use std::collections::HashSet;

use crate::types::{
    CriterionEntry,
    CriterionStatus::{self, Automated, Manual, NotApplicable, Runtime},
    WcagComplianceReport,
};

/// Constructs the WCAG 2.2 compliance matrix. Marks `all_pages_pass=false`
/// for any criterion that produced at least one issue across the scan.
///
/// `failed` should contain the [`AccessibilityIssue::criterion`](crate::AccessibilityIssue::criterion)
/// of every issue raised by [`crate::check_page`] across all scanned pages.
pub fn build_compliance_report(
    pages_scanned: usize,
    failed: &HashSet<String>,
) -> WcagComplianceReport {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn build_compliance_report_marks_failed_criterion_as_not_passing() {
        let mut failed = HashSet::new();
        let _ = failed.insert("1.1.1".to_string());
        let report = build_compliance_report(3, &failed);
        assert_eq!(report.wcag_version, "2.2");
        assert_eq!(report.pages_scanned, 3);

        let img_alt = report
            .criteria
            .iter()
            .find(|c| c.criterion == "1.1.1")
            .expect("1.1.1 row must be present");
        assert!(!img_alt.all_pages_pass);

        // A different automated criterion with no failures still passes.
        let lang = report
            .criteria
            .iter()
            .find(|c| c.criterion == "3.1.1")
            .expect("3.1.1 row must be present");
        assert!(lang.all_pages_pass);

        // Runtime/manual/not-applicable rows never report all_pages_pass.
        let contrast = report
            .criteria
            .iter()
            .find(|c| c.criterion == "1.4.3")
            .expect("1.4.3 row must be present");
        assert!(!contrast.all_pages_pass);
    }
}
