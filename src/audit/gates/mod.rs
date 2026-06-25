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
}
