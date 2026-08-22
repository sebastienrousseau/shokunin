// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Rich-text formatter for audit reports.
//!
//! The default output mode for `ssg audit` on TTY. Uses ANSI escape
//! codes for severity colour, but the codes are kept short so the
//! output stays readable when redirected to a file.

use crate::audit::{AuditReport, Severity};
use std::fmt::Write;

/// Renders `report` into `out` using a grouped-by-gate layout.
///
/// # Examples
///
/// ```
/// use ssg::audit::AuditReport;
/// use ssg::audit::output::text::format;
/// let report = AuditReport { gates: vec![] };
/// let mut out = String::new();
/// format(&report, &mut out);
/// assert!(out.is_empty());
/// ```
pub fn format(report: &AuditReport, out: &mut String) {
    for gate in &report.gates {
        if gate.skipped {
            let _ = writeln!(
                out,
                "[{}] skipped — {}",
                gate.name,
                gate.skip_reason.as_deref().unwrap_or("no reason given")
            );
            continue;
        }

        let counts = &gate.severity_counts;
        if counts.total() == 0 {
            let _ = writeln!(out, "[{}] OK", gate.name);
            continue;
        }
        let _ = writeln!(
            out,
            "[{}] {} finding(s) — {} error / {} warn / {} info",
            gate.name,
            counts.total(),
            counts.error,
            counts.warn,
            counts.info
        );
        for f in &gate.findings {
            let sigil = severity_sigil(f.severity);
            let path = f.path.as_deref().unwrap_or("");
            let code = f.code.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "  {sigil} {} {} {}",
                code,
                if path.is_empty() {
                    String::new()
                } else {
                    format!("({path})")
                },
                f.message
            );
        }
    }
}

const fn severity_sigil(s: Severity) -> &'static str {
    match s {
        Severity::Info => "i",
        Severity::Warn => "!",
        Severity::Error => "x",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{Finding, GateResult, Severity, SeverityCounts};

    #[test]
    fn skipped_gate_renders_skipped_line() {
        let report = AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: true,
                skip_reason: Some("test".to_string()),
                severity_counts: SeverityCounts::default(),
                findings: vec![],
            }],
        };
        let mut s = String::new();
        format(&report, &mut s);
        assert!(s.contains("[g] skipped"));
        assert!(s.contains("test"));
    }

    #[test]
    fn ok_gate_renders_ok_line() {
        let report = AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: false,
                skip_reason: None,
                severity_counts: SeverityCounts::default(),
                findings: vec![],
            }],
        };
        let mut s = String::new();
        format(&report, &mut s);
        assert!(s.contains("[g] OK"));
    }

    #[test]
    fn findings_are_rendered_with_severity_sigil() {
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
                findings: vec![Finding::new("g", Severity::Error, "boom")
                    .with_code("C")
                    .with_path("x.html")],
            }],
        };
        let mut s = String::new();
        format(&report, &mut s);
        assert!(s.contains("x C"));
        assert!(s.contains("(x.html)"));
        assert!(s.contains("boom"));
    }
}
