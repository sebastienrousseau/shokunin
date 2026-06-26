// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Per-gate implementations for [`crate::audit`].
//!
//! Each gate is its own module. [`all`] returns the registry order used
//! by [`crate::audit::AuditRunner`] — the order is stable so callers
//! can rely on the JSON / `JUnit` output layout being deterministic.

use super::AuditGate;

pub mod util;
#[allow(unused_imports)]
pub use util::{find_tag_end, hreflang_attr};

pub mod ai_discovery;
pub mod broken_links;
pub mod csp_sri;
pub mod feeds;
pub mod hreflang;
pub mod html5;
pub mod images;
pub mod jsonld;
pub mod markdownlint;
pub mod metadata;
pub mod performance;
pub mod pqc_tls;
pub mod search_index;
pub mod wcag;

/// Returns the 14 built-in gates in registration order.
///
/// Order is part of the public contract: the JSON output, `JUnit` XML
/// output, and CI dashboard tooling all rely on it being stable.
#[must_use]
pub fn all() -> Vec<Box<dyn AuditGate>> {
    vec![
        Box::new(wcag::WcagGate),
        Box::new(jsonld::JsonLdGate),
        Box::new(hreflang::HreflangGate),
        Box::new(csp_sri::CspSriGate),
        Box::new(pqc_tls::PqcTlsGate),
        Box::new(html5::Html5Gate),
        Box::new(broken_links::BrokenLinksGate),
        Box::new(metadata::MetadataGate),
        Box::new(markdownlint::MarkdownlintGate),
        Box::new(performance::PerformanceGate),
        Box::new(ai_discovery::AiDiscoveryGate),
        Box::new(feeds::FeedsGate),
        Box::new(images::ImagesGate),
        Box::new(search_index::SearchIndexGate),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_fourteen_gates_in_stable_order() {
        let gates = all();
        let names: Vec<&str> = gates.iter().map(|g| g.name()).collect();
        assert_eq!(names.len(), 14);
        assert_eq!(
            names,
            vec![
                "wcag",
                "jsonld",
                "hreflang",
                "csp_sri",
                "pqc_tls",
                "html5",
                "links",
                "metadata",
                "markdownlint",
                "performance",
                "ai_discovery",
                "feeds",
                "images",
                "search_index",
            ]
        );
    }

    #[test]
    fn every_gate_has_a_non_empty_explainer() {
        for g in all() {
            assert!(
                !g.explain().trim().is_empty(),
                "gate `{}` has empty explainer",
                g.name()
            );
        }
    }

    #[test]
    fn gate_names_are_unique() {
        let gates = all();
        let mut names: Vec<&str> = gates.iter().map(|g| g.name()).collect();
        names.sort_unstable();
        let original_len = names.len();
        names.dedup();
        assert_eq!(names.len(), original_len, "duplicate gate name detected");
    }

    #[test]
    fn gate_names_are_snake_case_no_whitespace() {
        for g in all() {
            let name = g.name();
            assert!(!name.is_empty(), "empty name");
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || c == '_'),
                "gate `{name}` is not snake_case"
            );
            assert!(!name.contains(' '), "gate `{name}` contains whitespace");
        }
    }

    #[test]
    fn explainers_are_reasonably_long() {
        for g in all() {
            assert!(
                g.explain().len() >= 40,
                "gate `{}` explainer is too short ({} chars)",
                g.name(),
                g.explain().len()
            );
        }
    }

    #[test]
    fn first_gate_is_wcag() {
        let gates = all();
        assert_eq!(gates.first().map(|g| g.name()), Some("wcag"));
    }

    #[test]
    fn last_gate_is_search_index() {
        let gates = all();
        assert_eq!(gates.last().map(|g| g.name()), Some("search_index"));
    }

    #[test]
    fn util_re_exports_compile() {
        let tag = r#"<a href="x">"#;
        assert_eq!(hreflang_attr(tag, "href"), Some("x".to_string()));
        let end = find_tag_end(tag, 0);
        assert_eq!(end, tag.len());
    }

    #[test]
    fn all_returns_fresh_boxes_each_call() {
        let a = all();
        let b = all();
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.name(), y.name());
        }
    }
}
