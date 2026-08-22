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
///
/// # Examples
///
/// ```
/// use ssg::audit::AuditReport;
/// use ssg::audit::output::json::format;
/// let report = AuditReport { gates: vec![] };
/// let s = format(&report).unwrap();
/// assert!(s.contains("\"gates\""));
/// ```
pub fn format(report: &AuditReport) -> Result<String, SsgError> {
    fail_point!("audit::json-format", |_| {
        Err(SsgError::Io {
            path: std::path::PathBuf::from("<audit-report>"),
            source: std::io::Error::other("injected: audit::json-format"),
        })
    });
    serde_json::to_string_pretty(report).map_err(serialize_error)
}

/// Wraps a `serde_json` serialisation failure in [`SsgError::Io`]
/// against the synthetic `<audit-report>` path.
fn serialize_error(e: serde_json::Error) -> SsgError {
    SsgError::Io {
        path: std::path::PathBuf::from("<audit-report>"),
        source: std::io::Error::other(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{Finding, GateResult, Severity, SeverityCounts};

    // These two tests call `format()`, which contains the
    // `audit::json-format` failpoint. The unkeyed `#[parallel]` joins
    // the same default lock group as `fault_tests`' unkeyed
    // `#[serial]` test below (and `cmd::audit`'s own fault test on the
    // same failpoint), so the fault-injection window can never race
    // with a normal call to `format()` running on another test thread.
    #[test]
    #[serial_test::parallel]
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
    fn serialize_error_maps_to_io_variant() {
        let e = serde_json::from_str::<serde_json::Value>("{")
            .expect_err("truncated JSON must fail");
        match serialize_error(e) {
            SsgError::Io { path, source } => {
                assert_eq!(path, std::path::PathBuf::from("<audit-report>"));
                assert_eq!(source.kind(), std::io::ErrorKind::Other);
            }
            other => panic!("expected SsgError::Io, got {other:?}"),
        }
    }

    #[test]
    #[serial_test::parallel]
    fn json_schema_has_stable_shape() {
        let report = AuditReport { gates: vec![] };
        let s = format(&report).unwrap();
        assert!(s.contains("\"gates\""));
    }
}

#[cfg(all(test, feature = "test-fault-injection"))]
mod fault_tests {
    use super::*;
    use serial_test::serial;

    /// RAII guard that disables a failpoint on drop.
    struct FailGuard(&'static str);

    impl Drop for FailGuard {
        fn drop(&mut self) {
            let _ = fail::cfg(self.0, "off");
        }
    }

    // Unkeyed `#[serial]` (the default lock) pairs with
    // `cmd::audit::fault_tests::json_output_propagates_serialize_error`,
    // which reuses this same `audit::json-format` failpoint on the
    // same unkeyed lock — see that test's doc comment.
    #[test]
    #[serial]
    fn format_propagates_injected_io_error() {
        let _guard = FailGuard("audit::json-format");
        fail::cfg("audit::json-format", "return").expect("activate failpoint");

        let report = AuditReport { gates: vec![] };
        let err = format(&report).unwrap_err();
        match err {
            SsgError::Io { path, source } => {
                assert_eq!(path, std::path::PathBuf::from("<audit-report>"));
                assert_eq!(source.kind(), std::io::ErrorKind::Other);
                assert!(
                    format!("{source}").contains("audit::json-format"),
                    "got: {source}"
                );
            }
            other => panic!("expected SsgError::Io, got {other:?}"),
        }
    }
}
