// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! RSS / Atom feed structural validation gate.
//!
//! Walks the site root for `*.xml` files (typically `rss.xml`,
//! `feed.xml`, `atom.xml`), discriminates RSS 2.0 vs Atom 1.0 by the
//! root element name, and asserts the required-field set for each
//! variant.
//!
//! RSS 2.0 required fields: `<channel>` containing `<title>`,
//! `<link>`, `<description>`. At least one `<item>` is expected when
//! the feed exists.
//!
//! Atom 1.0 required fields: `<feed>` containing `<title>`, `<id>`,
//! `<updated>`. At least one `<entry>` is expected.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use crate::walk::walk_files;

const NAME: &str = "feeds";

/// RSS/Atom feed structural validation gate.
#[derive(Debug, Clone, Copy)]
pub struct FeedsGate;

impl AuditGate for FeedsGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Walks the site root for *.xml files, classifies each as RSS \
         2.0 or Atom 1.0 by root element, and asserts the required \
         fields per variant. RSS 2.0 demands <channel> with <title>, \
         <link>, <description>; Atom 1.0 demands <feed> with <title>, \
         <id>, <updated>. Empty feeds raise a warning."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        let xml_files = walk_files(&site.root, "xml").unwrap_or_default();
        if xml_files.is_empty() {
            return findings;
        }
        for path in &xml_files {
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            let rel = site.rel(path);
            let kind = classify(&text);
            match kind {
                FeedKind::Rss2 => check_rss2(&text, &rel, &mut findings),
                FeedKind::Atom1 => check_atom1(&text, &rel, &mut findings),
                FeedKind::Sitemap => {
                    // Sitemaps are validated by their own pipeline, not this gate.
                }
                FeedKind::Unknown => {
                    // Quietly ignore unknown XML — many SSGs emit
                    // OPML, GraphQL schema, etc.
                }
            }
        }
        findings
    }
}

enum FeedKind {
    Rss2,
    Atom1,
    Sitemap,
    Unknown,
}

fn classify(text: &str) -> FeedKind {
    let lower = text.to_lowercase();
    if lower.contains("<rss") {
        FeedKind::Rss2
    } else if lower.contains("<feed") && lower.contains("atom") {
        FeedKind::Atom1
    } else if lower.contains("<urlset") || lower.contains("<sitemapindex") {
        FeedKind::Sitemap
    } else {
        FeedKind::Unknown
    }
}

fn check_rss2(text: &str, rel: &str, findings: &mut Vec<Finding>) {
    let lower = text.to_lowercase();
    if !lower.contains("<channel") {
        findings.push(
            Finding::new(NAME, Severity::Error, "RSS feed missing <channel>")
                .with_code("RSS-CHANNEL")
                .with_path(rel.to_string()),
        );
        return;
    }
    for required in ["<title", "<link", "<description"] {
        if !lower.contains(required) {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    format!("RSS <channel> missing {required}>"),
                )
                .with_code(format!(
                    "RSS-{}",
                    required.trim_start_matches('<').to_uppercase()
                ))
                .with_path(rel.to_string()),
            );
        }
    }
    if !lower.contains("<item") {
        findings.push(
            Finding::new(
                NAME,
                Severity::Warn,
                "RSS feed contains zero <item> entries",
            )
            .with_code("RSS-EMPTY")
            .with_path(rel.to_string()),
        );
    }
}

fn check_atom1(text: &str, rel: &str, findings: &mut Vec<Finding>) {
    let lower = text.to_lowercase();
    for required in ["<title", "<id", "<updated"] {
        if !lower.contains(required) {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Error,
                    format!("Atom feed missing {required}>"),
                )
                .with_code(format!(
                    "ATOM-{}",
                    required.trim_start_matches('<').to_uppercase()
                ))
                .with_path(rel.to_string()),
            );
        }
    }
    if !lower.contains("<entry") {
        findings.push(
            Finding::new(
                NAME,
                Severity::Warn,
                "Atom feed contains zero <entry> elements",
            )
            .with_code("ATOM-EMPTY")
            .with_path(rel.to_string()),
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn site_with_files(files: &[(&str, &str)]) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        for (name, body) in files {
            let p = root.join(name);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&p, body).unwrap();
        }
        std::mem::forget(tmp);
        Site {
            root,
            html_files: Vec::new(),
        }
    }

    #[test]
    fn valid_rss_passes() {
        let body = r#"<?xml version="1.0"?>
<rss version="2.0"><channel>
  <title>x</title><link>https://x</link><description>d</description>
  <item><title>i</title></item>
</channel></rss>"#;
        let s = site_with_files(&[("rss.xml", body)]);
        let f = FeedsGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn rss_missing_channel_fields_flagged() {
        let body = r#"<?xml version="1.0"?><rss version="2.0"><channel></channel></rss>"#;
        let s = site_with_files(&[("rss.xml", body)]);
        let f = FeedsGate.run(&s, &AuditOptions::default());
        let codes: Vec<_> =
            f.iter().filter_map(|x| x.code.as_deref()).collect();
        assert!(codes.contains(&"RSS-TITLE"));
        assert!(codes.contains(&"RSS-LINK"));
        assert!(codes.contains(&"RSS-DESCRIPTION"));
    }
}
