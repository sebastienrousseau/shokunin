// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `JUnit` XML formatter for audit reports.
//!
//! One `<testsuite>` per gate; one `<testcase>` per finding. A gate
//! with zero findings emits a single passing `<testcase>` so the
//! suite isn't flagged as empty by CI parsers (GitLab, Jenkins).
//!
//! No external XML crate is pulled in — the document shape is too
//! tightly constrained for tag-balancing bugs to slip in, and the
//! existing `quick-xml` dep in the workspace is feature-gated to the
//! optional `minify` build.

use crate::audit::{AuditReport, Severity};
use std::fmt::Write;

/// Renders `report` as `JUnit` XML.
#[must_use]
pub fn format(report: &AuditReport) -> String {
    let mut out = String::with_capacity(2048);
    let _ = writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(out, "<testsuites>");

    for gate in &report.gates {
        let total = gate.findings.len();
        let failures = gate.severity_counts.error;
        let skipped = usize::from(gate.skipped);
        let _ = writeln!(
            out,
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\">",
            escape(&gate.name),
            total.max(1),
            failures,
            skipped
        );
        if gate.skipped {
            let _ = writeln!(
                out,
                "    <testcase name=\"{}\" classname=\"{}\"><skipped message=\"{}\"/></testcase>",
                escape(&gate.name),
                escape(&gate.name),
                escape(gate.skip_reason.as_deref().unwrap_or(""))
            );
        } else if gate.findings.is_empty() {
            let _ = writeln!(
                out,
                "    <testcase name=\"{}\" classname=\"{}\"/>",
                escape(&gate.name),
                escape(&gate.name)
            );
        } else {
            for f in &gate.findings {
                let case_name = f.code.as_deref().unwrap_or(&gate.name);
                let _ = writeln!(
                    out,
                    "    <testcase name=\"{}\" classname=\"{}\">",
                    escape(case_name),
                    escape(&gate.name)
                );
                let tag = match f.severity {
                    Severity::Error => "failure",
                    Severity::Warn => "failure", // warnings show as failures unless --fail-on lifts them
                    Severity::Info => "system-out",
                };
                let path_attr = f
                    .path
                    .as_ref()
                    .map(|p| format!(" file=\"{}\"", escape(p)))
                    .unwrap_or_default();
                if matches!(f.severity, Severity::Info) {
                    let _ = writeln!(
                        out,
                        "      <{tag}>{}</{tag}>",
                        escape(&f.message)
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "      <{tag} type=\"{}\"{}>{}</{tag}>",
                        f.severity,
                        path_attr,
                        escape(&f.message)
                    );
                }
                let _ = writeln!(out, "    </testcase>");
            }
        }
        let _ = writeln!(out, "  </testsuite>");
    }

    let _ = writeln!(out, "</testsuites>");
    out
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::audit::{Finding, GateResult, Severity, SeverityCounts};

    fn one_gate_report() -> AuditReport {
        AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: false,
                skip_reason: None,
                severity_counts: SeverityCounts {
                    info: 0,
                    warn: 0,
                    error: 1,
                },
                findings: vec![Finding::new("g", Severity::Error, "boom")
                    .with_code("X")
                    .with_path("a.html")],
            }],
        }
    }

    #[test]
    fn well_formed_xml_header() {
        let xml = format(&one_gate_report());
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<testsuites>"));
        assert!(xml.contains("</testsuites>"));
    }

    #[test]
    fn one_testcase_per_finding() {
        let xml = format(&one_gate_report());
        assert_eq!(xml.matches("<testcase").count(), 1);
        assert!(xml.contains("<failure"));
    }

    #[test]
    fn skipped_gate_emits_skipped_testcase() {
        let report = AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: true,
                skip_reason: Some("test".to_string()),
                severity_counts: SeverityCounts::default(),
                findings: vec![],
            }],
        };
        let xml = format(&report);
        assert!(xml.contains("<skipped"));
    }

    #[test]
    fn xml_special_chars_escaped() {
        let report = AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: false,
                skip_reason: None,
                severity_counts: SeverityCounts {
                    info: 0,
                    warn: 0,
                    error: 1,
                },
                findings: vec![Finding::new(
                    "g",
                    Severity::Error,
                    "a < b & c > d \"e\"",
                )],
            }],
        };
        let xml = format(&report);
        assert!(xml.contains("a &lt; b &amp; c &gt; d &quot;e&quot;"));
    }
}
