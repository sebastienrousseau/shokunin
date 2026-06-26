// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON formatter for audit reports.
//!
//! Schema is stable: `{ "gates": [ { name, skipped, skip_reason?,
//! severity_counts: { info, warn, error }, findings: [...] } ] }`.
//! Golden-file tested in `tests/audit_gates.rs` so any breaking
//! re-shape requires a deliberate commit.
//!
//! See also the sibling `JUnit` formatter for CI dashboards that
//! prefer the JUnit/Surefire schema over JSON.

use crate::audit::AuditReport;
use crate::error::SsgError;

/// Serialises `report` to a pretty-printed JSON string.
///
/// # Errors
/// Returns [`SsgError::Io`] when `serde_json` cannot serialise the
/// report — only possible if a finding's strings contain invalid
/// UTF-8, which the type-system prevents in safe Rust.
pub fn format(report: &AuditReport) -> Result<String, SsgError> {
    serde_json::to_string_pretty(report).map_err(|e| SsgError::Io {
        path: std::path::PathBuf::from("<audit-report>"),
        source: std::io::Error::other(e),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::audit::{Finding, GateResult, Severity, SeverityCounts};

    #[test]
    fn json_round_trips() {
        let report = AuditReport {
            gates: vec![GateResult {
                name: "g".to_string(),
                skipped: false,
                skip_reason: None,
                severity_counts: SeverityCounts {
                    info: 0,
                    warn: 1,
                    error: 0,
                },
                findings: vec![Finding::new("g", Severity::Warn, "msg")
                    .with_code("X")
                    .with_path("a.html")],
            }],
        };
        let s = format(&report).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["gates"][0]["name"], "g");
        assert_eq!(v["gates"][0]["severity_counts"]["warn"], 1);
        assert_eq!(v["gates"][0]["findings"][0]["code"], "X");
        assert_eq!(v["gates"][0]["findings"][0]["path"], "a.html");
        assert_eq!(v["gates"][0]["findings"][0]["severity"], "warn");
    }

    #[test]
    fn json_schema_has_stable_shape() {
        let report = AuditReport { gates: vec![] };
        let s = format(&report).unwrap();
        assert!(s.contains("\"gates\""));
    }
}
