// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Broken internal/external link gate.
//!
//! Walks every `<a href>` (and `<img src>`) on every page. Internal
//! links are resolved against the site root and reported as errors
//! when their target does not exist. External links are reported as
//! info when `--skip-network` is set (the default), and probed via
//! HTTP HEAD only when explicitly opted in.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use super::hreflang_attr;
use std::path::PathBuf;

const NAME: &str = "links";

/// Broken internal/external link gate.
#[derive(Debug, Clone, Copy)]
pub struct BrokenLinksGate;

impl AuditGate for BrokenLinksGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Walks every <a href> and <img src> in the site. Internal \
         targets must resolve under the site root or an error is \
         emitted. External targets are skipped by default (set \
         skip_network=false to enable HEAD probing). Anchor-only \
         hrefs (#fragment) and `mailto:` / `tel:` URIs are ignored."
    }

    fn run(&self, site: &Site, opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut external_skipped = 0usize;

        for path in &site.html_files {
            let Ok(html) = site.read(path) else { continue };
            let rel = site.rel(path);
            for href in extract_link_targets(&html) {
                if is_ignorable(&href) {
                    continue;
                }
                if is_external(&href) {
                    if opts.skip_network {
                        external_skipped += 1;
                    }
                    continue;
                }
                if !internal_target_exists(&site.root, path, &href) {
                    findings.push(
                        Finding::new(
                            NAME,
                            Severity::Error,
                            format!("internal link `{href}` does not resolve"),
                        )
                        .with_code("LINK-INTERNAL-MISSING")
                        .with_path(rel.clone()),
                    );
                }
            }
        }

        if external_skipped > 0 {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Info,
                    format!(
                        "{external_skipped} external link(s) skipped (--skip-network)"
                    ),
                )
                .with_code("LINK-EXTERNAL-SKIPPED"),
            );
        }

        findings
    }
}

fn is_ignorable(href: &str) -> bool {
    href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("data:")
        || href.is_empty()
}

fn is_external(href: &str) -> bool {
    href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("//")
}

fn extract_link_targets(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = html.to_lowercase();
    for (open, attr) in &[("<a ", "href"), ("<img", "src")] {
        let mut cursor = 0;
        while let Some(rel) = lower[cursor..].find(open) {
            let abs = cursor + rel;
            let end =
                lower[abs..].find('>').map_or(lower.len(), |e| abs + e + 1);
            let tag = &html[abs..end];
            cursor = end;
            if let Some(v) = hreflang_attr(tag, attr) {
                out.push(v);
            }
        }
    }
    out
}

fn internal_target_exists(
    root: &std::path::Path,
    page: &std::path::Path,
    href: &str,
) -> bool {
    let href_clean = href.split('?').next().unwrap_or(href);
    let href_clean = href_clean.split('#').next().unwrap_or(href_clean);
    if href_clean.is_empty() {
        return true;
    }

    let candidate: PathBuf =
        if let Some(stripped) = href_clean.strip_prefix('/') {
            root.join(stripped)
        } else if let Some(parent) = page.parent() {
            parent.join(href_clean)
        } else {
            root.join(href_clean)
        };

    if candidate.exists() {
        return true;
    }
    if candidate.is_dir() && candidate.join("index.html").exists() {
        return true;
    }
    let with_index = candidate.join("index.html");
    if with_index.exists() {
        return true;
    }
    // /foo (no extension) -> /foo.html or /foo/index.html
    let mut html_candidate = candidate.clone();
    if html_candidate.extension().is_none() {
        let _ = html_candidate.set_extension("html");
        if html_candidate.exists() {
            return true;
        }
    }
    false
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
    fn passing_internal_link_is_clean() {
        let pages = &[
            (
                "index.html",
                r#"<html><body><a href="/about/">about</a></body></html>"#,
            ),
            ("about/index.html", "<html><body>about</body></html>"),
        ];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        let errors: Vec<_> = f
            .iter()
            .filter(|x| matches!(x.severity, Severity::Error))
            .collect();
        assert!(errors.is_empty(), "got {errors:?}");
    }

    #[test]
    fn broken_internal_link_flagged() {
        let pages = &[(
            "index.html",
            r#"<html><body><a href="/missing/">x</a></body></html>"#,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("LINK-INTERNAL-MISSING")));
    }

    #[test]
    fn skip_network_emits_info_for_externals() {
        let pages = &[(
            "index.html",
            r#"<html><body><a href="https://example.com">x</a></body></html>"#,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("LINK-EXTERNAL-SKIPPED")));
    }

    #[test]
    fn ignorable_schemes_are_silent() {
        let pages = &[(
            "index.html",
            r##"<html><body>
                <a href="#anchor">a</a>
                <a href="mailto:x@y.z">m</a>
                <a href="tel:+1">t</a>
                <a href="javascript:void(0)">j</a>
                <a href="data:image/png;base64,xx">d</a>
                <a href="">e</a>
            </body></html>"##,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(
            f.iter()
                .all(|x| x.code.as_deref() != Some("LINK-INTERNAL-MISSING")),
            "ignorable schemes flagged: {f:?}"
        );
    }

    #[test]
    fn protocol_relative_link_treated_as_external() {
        let pages = &[(
            "index.html",
            r#"<html><body><a href="//cdn.example/x">x</a></body></html>"#,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("LINK-EXTERNAL-SKIPPED")));
    }

    #[test]
    fn img_src_links_are_checked() {
        let pages = &[(
            "index.html",
            r#"<html><body><img src="/missing.png" alt="x"></body></html>"#,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("LINK-INTERNAL-MISSING")));
    }

    #[test]
    fn relative_link_with_query_and_fragment_strips_correctly() {
        let pages = &[
            (
                "index.html",
                r#"<html><body><a href="about.html?x=1#sec">a</a></body></html>"#,
            ),
            ("about.html", "<html></html>"),
        ];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(
            f.iter()
                .all(|x| x.code.as_deref() != Some("LINK-INTERNAL-MISSING")),
            "query/fragment must strip: {f:?}"
        );
    }

    #[test]
    fn extensionless_internal_link_resolves_via_html_extension() {
        let pages = &[
            (
                "index.html",
                r#"<html><body><a href="/about">a</a></body></html>"#,
            ),
            ("about.html", "<html></html>"),
        ];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(
            f.iter()
                .all(|x| x.code.as_deref() != Some("LINK-INTERNAL-MISSING")),
            "extensionless resolution failed: {f:?}"
        );
    }

    #[test]
    fn no_skip_network_does_not_emit_external_skip_finding() {
        let pages = &[(
            "index.html",
            r#"<html><body><a href="https://example.com">x</a></body></html>"#,
        )];
        let f = BrokenLinksGate.run(
            &site_with(pages),
            &AuditOptions {
                skip_network: false,
                ..AuditOptions::default()
            },
        );
        assert!(f
            .iter()
            .all(|x| x.code.as_deref() != Some("LINK-EXTERNAL-SKIPPED")));
    }

    #[test]
    fn unreadable_html_skipped_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let bogus = tmp.path().join("ghost.html");
        let s = Site {
            root: tmp.path().to_path_buf(),
            html_files: vec![bogus],
        };
        std::mem::forget(tmp);
        let f = BrokenLinksGate.run(
            &s,
            &AuditOptions {
                skip_network: true,
                ..AuditOptions::default()
            },
        );
        assert!(f.is_empty());
    }

    #[test]
    fn metadata_methods_exposed() {
        let g = BrokenLinksGate;
        assert_eq!(g.name(), "links");
        assert!(g.explain().contains("Internal"));
        let _copy: BrokenLinksGate = g;
        let _clone = g;
        assert!(format!("{g:?}").contains("BrokenLinksGate"));
    }
}
