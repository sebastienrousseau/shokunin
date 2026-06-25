// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CSP + SRI hash validation gate.
//!
//! Per page:
//! - Asserts a `Content-Security-Policy` is present (either as a
//!   `_headers` rule or a `<meta http-equiv="Content-Security-Policy">`
//!   tag).
//! - Asserts every cross-origin `<script src>` and
//!   `<link rel="stylesheet" href>` carries an `integrity` SRI hash.
//!
//! The CSP-consistency check across pages is a warning (different
//! pages with different policies are a smell but legitimate when a
//! single section needs a stricter policy).

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use std::collections::HashSet;

const NAME: &str = "csp_sri";

/// CSP + SRI hash validation gate.
#[derive(Debug, Clone, Copy)]
pub struct CspSriGate;

impl AuditGate for CspSriGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Asserts every page declares a Content-Security-Policy (via a \
         <meta http-equiv=\"Content-Security-Policy\"> tag or a site \
         _headers file) and that every cross-origin <script src> / \
         <link rel=\"stylesheet\"> carries a valid `integrity` SRI \
         attribute. Policy drift between pages is reported as a warn."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut policies: HashSet<String> = HashSet::new();

        // Detect a site-level _headers file.
        let site_headers = site.root.join("_headers");
        let has_site_csp = site_headers.exists()
            && std::fs::read_to_string(&site_headers).is_ok_and(|s| {
                s.to_lowercase().contains("content-security-policy")
            });

        for path in &site.html_files {
            let Ok(html) = site.read(path) else { continue };
            let rel = site.rel(path);

            let policy = extract_meta_csp(&html);
            if !has_site_csp && policy.is_none() {
                findings.push(
                    Finding::new(
                        NAME,
                        Severity::Error,
                        "Page has no Content-Security-Policy (no <meta http-equiv> and no _headers)",
                    )
                    .with_code("CSP-MISSING")
                    .with_path(rel.clone()),
                );
            }
            if let Some(p) = policy {
                let _ = policies.insert(p);
            }

            for asset in extract_remote_assets(&html) {
                if asset.integrity.is_none() {
                    findings.push(
                        Finding::new(
                            NAME,
                            Severity::Error,
                            format!(
                                "{} {} missing SRI `integrity` attribute",
                                asset.kind, asset.href
                            ),
                        )
                        .with_code("SRI-MISSING")
                        .with_path(rel.clone()),
                    );
                }
            }
        }

        if policies.len() > 1 {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    format!(
                        "Found {} distinct CSP policies across pages",
                        policies.len()
                    ),
                )
                .with_code("CSP-DRIFT"),
            );
        }

        findings
    }
}

fn extract_meta_csp(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let needle = "<meta";
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find(needle) {
        let abs = cursor + rel;
        let end = lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
        let tag = &html[abs..end];
        if tag
            .to_lowercase()
            .contains("http-equiv=\"content-security-policy\"")
        {
            if let Some(c) = super::hreflang_attr(tag, "content") {
                return Some(c);
            }
        }
        cursor = end;
    }
    None
}

struct RemoteAsset {
    kind: &'static str,
    href: String,
    integrity: Option<String>,
}

fn extract_remote_assets(html: &str) -> Vec<RemoteAsset> {
    let mut out = Vec::new();
    let lower = html.to_lowercase();

    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find("<script") {
        let abs = cursor + rel;
        let end = lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
        let tag = &html[abs..end];
        cursor = end;
        let Some(src) = super::hreflang_attr(tag, "src") else {
            continue;
        };
        if !is_remote(&src) {
            continue;
        }
        out.push(RemoteAsset {
            kind: "<script src=>",
            href: src,
            integrity: super::hreflang_attr(tag, "integrity"),
        });
    }

    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find("<link") {
        let abs = cursor + rel;
        let end = lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
        let tag = &html[abs..end];
        cursor = end;
        let lower_tag = tag.to_lowercase();
        if !lower_tag.contains("rel=\"stylesheet\"")
            && !lower_tag.contains("rel='stylesheet'")
        {
            continue;
        }
        let Some(href) = super::hreflang_attr(tag, "href") else {
            continue;
        };
        if !is_remote(&href) {
            continue;
        }
        out.push(RemoteAsset {
            kind: "<link rel=stylesheet>",
            href,
            integrity: super::hreflang_attr(tag, "integrity"),
        });
    }

    out
}

fn is_remote(href: &str) -> bool {
    href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("//")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn site_with(pages: &[(&str, &str)]) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let mut files = Vec::new();
        for (rel, html) in pages {
            let p = root.join(rel);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&p, html).unwrap();
            files.push(p);
        }
        std::mem::forget(tmp);
        Site {
            root,
            html_files: files,
        }
    }

    #[test]
    fn page_with_csp_and_sri_is_clean() {
        let html = r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'">
            <script src="https://cdn.example/x.js" integrity="sha256-abc"></script>
            <link rel="stylesheet" href="/local.css">
        </head><body></body></html>"#;
        let s = site_with(&[("index.html", html)]);
        let f = CspSriGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn page_missing_csp_and_sri_flagged() {
        let html = r#"<html><head>
            <script src="https://cdn.example/x.js"></script>
        </head><body></body></html>"#;
        let s = site_with(&[("index.html", html)]);
        let f = CspSriGate.run(&s, &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("CSP-MISSING")));
        assert!(f.iter().any(|x| x.code.as_deref() == Some("SRI-MISSING")));
    }
}
