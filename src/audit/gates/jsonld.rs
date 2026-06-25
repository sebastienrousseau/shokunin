// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! JSON-LD (Schema.org) semantic audit gate.
//!
//! Delegates to [`crate::seo::jsonld::validate_jsonld`] so this gate
//! and the build-time `JsonLdPlugin` agree on the same required-field
//! matrix per `@type`.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use crate::seo::validate_jsonld;

const NAME: &str = "jsonld";

/// JSON-LD Schema.org gate.
#[derive(Debug, Clone, Copy)]
pub struct JsonLdGate;

impl AuditGate for JsonLdGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Extracts every <script type=\"application/ld+json\"> block on \
         each page, asserts it parses as JSON, and validates the \
         required fields for its declared @type (Article, WebPage, \
         BreadcrumbList, FAQPage, LocalBusiness, Organization). \
         Unparseable JSON or missing-required-field findings are \
         emitted at error severity; unknown types are pass-through."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        for path in &site.html_files {
            let Ok(html) = site.read(path) else { continue };
            let rel = site.rel(path);
            for err in validate_jsonld(&html) {
                findings.push(
                    Finding::new(
                        NAME,
                        Severity::Error,
                        format!(
                            "[{}] missing/invalid `{}` — {}",
                            err.schema_type, err.field, err.reason
                        ),
                    )
                    .with_code(format!("JSONLD-{}", err.schema_type))
                    .with_path(rel.clone()),
                );
            }
        }
        findings
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn site(html: &str) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("page.html");
        std::fs::write(&path, html).unwrap();
        let root = tmp.path().to_path_buf();
        std::mem::forget(tmp);
        Site {
            root,
            html_files: vec![path],
        }
    }

    #[test]
    fn passing_jsonld_produces_no_findings() {
        let html = r#"<html><head><script type="application/ld+json">
            {"@context":"https://schema.org","@type":"Organization","name":"Acme","url":"https://acme.test"}
        </script></head><body></body></html>"#;
        let f = JsonLdGate.run(&site(html), &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn unparseable_jsonld_is_flagged() {
        let html = r#"<html><head><script type="application/ld+json">{ not json }</script></head><body></body></html>"#;
        let f = JsonLdGate.run(&site(html), &AuditOptions::default());
        assert!(
            f.iter().any(|x| matches!(x.severity, Severity::Error)),
            "expected an error finding, got {f:?}"
        );
    }

    #[test]
    fn empty_site_produces_no_findings() {
        let s = Site {
            root: PathBuf::from("/nonexistent"),
            html_files: Vec::new(),
        };
        let f = JsonLdGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty());
    }
}
