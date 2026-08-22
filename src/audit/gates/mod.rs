// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Per-gate implementations for [`crate::audit`].
//!
//! Each gate is its own module. [`all`] returns the registry order used
//! by [`crate::audit::AuditRunner`] — the order is stable so callers
//! can rely on the JSON / `JUnit` output layout being deterministic.

use super::AuditGate;

pub mod util;
// Re-exported for downstream crates and the audit binary; the library
// build alone does not reference every name, which is not the same as
// them being unused.
#[allow(unused_imports)]
pub use util::{find_tag_end, hreflang_attr, strip_script_and_style};

pub mod ai_discovery;
pub mod broken_links;
pub mod csp_sri;
pub mod feeds;
pub mod hreflang;
pub mod html5;
pub mod images;
pub mod jsonld;
pub mod lang_consistency;
pub mod markdownlint;
pub mod metadata;
pub mod performance;
pub mod pqc_tls;
pub mod search_index;
pub mod wcag;

/// Returns the 15 built-in gates in registration order.
///
/// Order is part of the public contract: the JSON output, `JUnit` XML
/// output, and CI dashboard tooling all rely on it being stable — new
/// gates append at the end so existing positions never shift.
///
/// # Examples
///
/// ```
/// use ssg::audit::gates::all;
/// let gates = all();
/// assert_eq!(gates.len(), 15);
/// assert_eq!(gates[0].name(), "wcag");
/// ```
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
        Box::new(lang_consistency::LangConsistencyGate),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_fifteen_gates_in_stable_order() {
        let gates = all();
        let names: Vec<&str> = gates.iter().map(|g| g.name()).collect();
        assert_eq!(names.len(), 15);
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
                "lang_consistency",
            ]
        );
    }

    #[test]
    fn every_gate_has_a_non_empty_explainer() {
        for g in all() {
            let name = g.name();
            let explainer = g.explain();
            assert!(
                !explainer.trim().is_empty(),
                "gate `{name}` has empty explainer"
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
            let name = g.name();
            let len = g.explain().len();
            assert!(
                len >= 40,
                "gate `{name}` explainer is too short ({len} chars)"
            );
        }
    }

    #[test]
    fn first_gate_is_wcag() {
        let gates = all();
        assert_eq!(gates.first().map(|g| g.name()), Some("wcag"));
    }

    #[test]
    fn last_gate_is_lang_consistency() {
        // New gates append at the end so existing positions are stable.
        let gates = all();
        assert_eq!(gates.last().map(|g| g.name()), Some("lang_consistency"));
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
