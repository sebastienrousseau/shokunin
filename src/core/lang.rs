// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render-time page-language precedence (spec A5, plan §2 1.5).
//!
//! The template engine emits `<html lang="{{ site.language }}">` for
//! every page (scaffold `templates/tera/base.html`, user templates).
//! Spec A5 requires the *page's* resolved language to win there, so
//! that `<html lang>` agrees with the other three language sinks
//! (JSON-LD `inLanguage`, `og:locale`, hreflang self-reference), all
//! of which resolve through
//! `crate::seo::lang::resolve_page_lang`.
//!
//! `resolve_page_lang` needs a *built* page (its on-disk path, its
//! rendered HTML, a [`PluginContext`](crate::plugin::PluginContext)),
//! none of which exist at template-render time — the render context
//! only has the page's front matter and the site globals. This module
//! is therefore the emitter-side subset of the same precedence chain:
//! steps 1 (front-matter `language`), 2 (front-matter `hreflang`),
//! 5 (site default) and 6 (the `"en"` constant). Steps 3 and 4
//! (locale path prefix, existing `<html lang>`) only apply to built
//! pages and remain in `seo::lang`, which the SEO sinks and the
//! language-mismatch audit gate keep using.
//!
//! ## Coordination note (Wave 2)
//!
//! [`normalize_bcp47`] is the single normalisation implementation:
//! the SEO-side `seo::lang::normalize_bcp47` is a thin wrapper over
//! this function so the render-time engine and the SEO sinks can
//! never disagree on canonical BCP-47 form.

// Crate-internal on purpose: this helper backs the template engine
// and must not become public API before the resolver work settles.
// rustc's `unreachable_pub` wants `pub(crate)` here while clippy's
// nursery `redundant_pub_crate` wants plain `pub` — keep the honest
// `pub(crate)` spelling and silence the nursery lint (same pattern as
// `seo::lang`).
#![allow(clippy::redundant_pub_crate)]

#[cfg(feature = "templates")]
use std::collections::HashMap;

/// Final constant fallback when no other source resolves (spec A5).
///
/// Mirrors `seo::lang::DEFAULT_PAGE_LANG`. Gated on `templates`: its
/// only consumer, [`resolve_render_lang`], is only reachable from
/// `template_engine::render_page`, itself feature-gated — without
/// this, `--no-default-features` sees it as dead code.
#[cfg(feature = "templates")]
pub(crate) const DEFAULT_PAGE_LANG: &str = "en";

/// Resolves the language a page should be *rendered* with, from the
/// information available inside a template render context.
///
/// Precedence, first match wins (the emitter-side subset of
/// `seo::lang::resolve_page_lang` — see the module docs):
///
/// 1. Front-matter `language`.
/// 2. Front-matter `hreflang`.
/// 3. The site default language (`site.language` global).
/// 4. [`DEFAULT_PAGE_LANG`] (`"en"`).
///
/// Every returned value is normalised to BCP-47 hyphen form
/// (`EN_gb` → `en-GB`), matching what the SEO sinks publish.
///
/// Gated on `templates`: its sole caller is
/// `template_engine::render_page`, itself feature-gated. Note
/// `normalize_bcp47` below is NOT gated — `seo::lang::normalize_bcp47`
/// delegates to it unconditionally.
#[cfg(feature = "templates")]
pub(crate) fn resolve_render_lang(
    frontmatter: &HashMap<String, serde_json::Value>,
    site_language: Option<&str>,
) -> String {
    for key in ["language", "hreflang"] {
        if let Some(lang) = frontmatter
            .get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(normalize_bcp47)
        {
            return lang;
        }
    }
    site_language
        .and_then(normalize_bcp47)
        .unwrap_or_else(|| DEFAULT_PAGE_LANG.to_string())
}

/// Normalises a raw language tag into canonical BCP-47 hyphen form.
///
/// Accepts underscore-separated input (`en_GB`), lowercases the
/// primary subtag, uppercases two-letter regions and title-cases
/// four-letter scripts (`zh-hans` → `zh-Hans`). Returns `None` when
/// the value is empty or not shaped like a language tag, so callers
/// fall through to the next resolver source.
///
/// Byte-identical to `seo::lang::normalize_bcp47` — see the module
/// docs' coordination note.
pub(crate) fn normalize_bcp47(raw: &str) -> Option<String> {
    let cleaned = raw.trim().replace('_', "-");
    let mut parts = cleaned.split('-');

    // `split` always yields at least one item, so the fallback is
    // purely defensive: an empty primary fails the length check below.
    let primary = parts.next().unwrap_or_default();
    if !(2..=3).contains(&primary.len())
        || !primary.bytes().all(|b| b.is_ascii_alphabetic())
    {
        return None;
    }

    let mut out = primary.to_ascii_lowercase();
    for sub in parts {
        if sub.is_empty() || !sub.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return None;
        }
        out.push('-');
        match sub.len() {
            // Two-letter region: en-GB.
            2 if sub.bytes().all(|b| b.is_ascii_alphabetic()) => {
                out.push_str(&sub.to_ascii_uppercase());
            }
            // Four-letter script: zh-Hans.
            4 if sub.bytes().all(|b| b.is_ascii_alphabetic()) => {
                let (head, tail) = sub.split_at(1);
                out.push_str(&head.to_ascii_uppercase());
                out.push_str(&tail.to_ascii_lowercase());
            }
            // Anything else (numeric regions, variants): keep lowercase.
            _ => out.push_str(&sub.to_ascii_lowercase()),
        }
    }
    Some(out)
}

// Gated on `templates` to match the items under test: `HashMap` is
// imported behind that feature (line 41) and `resolve_render_lang` is
// defined behind it, so without the gate this module does not compile
// with default features off. It never had one, and nothing noticed —
// the feature-powerset job runs `cargo check`, which does not build
// test code, so `cargo test --lib --no-default-features` had simply
// never been compiled.
#[cfg(all(test, feature = "templates"))]
mod tests {
    use super::*;

    fn fm(pairs: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    (*k).to_string(),
                    serde_json::Value::String((*v).to_string()),
                )
            })
            .collect()
    }

    #[test]
    fn normalize_canonicalises_case_and_separators() {
        assert_eq!(normalize_bcp47("EN_gb").as_deref(), Some("en-GB"));
        assert_eq!(normalize_bcp47("fr-fr").as_deref(), Some("fr-FR"));
        assert_eq!(normalize_bcp47("hi").as_deref(), Some("hi"));
        assert_eq!(normalize_bcp47("ZH-HANS").as_deref(), Some("zh-Hans"));
    }

    #[test]
    fn normalize_rejects_empty_and_garbage() {
        assert_eq!(normalize_bcp47(""), None);
        assert_eq!(normalize_bcp47("   "), None);
        assert_eq!(normalize_bcp47("english"), None);
        assert_eq!(normalize_bcp47("e"), None);
        assert_eq!(normalize_bcp47("en-"), None);
        assert_eq!(normalize_bcp47("12"), None);
    }

    #[test]
    fn normalize_keeps_numeric_and_variant_subtags_lowercase() {
        // Numeric UN M.49 region (len 3, not alphabetic) and long
        // variant subtags take the catch-all lowercase arm.
        assert_eq!(normalize_bcp47("ES-419").as_deref(), Some("es-419"));
        assert_eq!(
            normalize_bcp47("en-GB-OXENDICT").as_deref(),
            Some("en-GB-oxendict")
        );
        // Two-char subtag that is not purely alphabetic also falls
        // through to the lowercase arm rather than region uppercasing.
        assert_eq!(normalize_bcp47("en-a1").as_deref(), Some("en-a1"));
    }

    #[test]
    fn frontmatter_language_beats_site_default() {
        let lang =
            resolve_render_lang(&fm(&[("language", "hi")]), Some("en-GB"));
        assert_eq!(lang, "hi");
    }

    #[test]
    fn frontmatter_hreflang_used_when_language_absent() {
        let lang =
            resolve_render_lang(&fm(&[("hreflang", "en_gb")]), Some("en"));
        assert_eq!(lang, "en-GB");
    }

    #[test]
    fn invalid_frontmatter_language_falls_through() {
        let lang = resolve_render_lang(
            &fm(&[("language", "not a lang")]),
            Some("fr-FR"),
        );
        assert_eq!(lang, "fr-FR");
    }

    #[test]
    fn site_default_used_when_no_frontmatter_signal() {
        let lang = resolve_render_lang(&HashMap::new(), Some("en-GB"));
        assert_eq!(lang, "en-GB");
    }

    #[test]
    fn en_constant_when_nothing_resolves() {
        assert_eq!(resolve_render_lang(&HashMap::new(), None), "en");
        assert_eq!(
            resolve_render_lang(&HashMap::new(), Some("")),
            DEFAULT_PAGE_LANG
        );
    }

    #[test]
    fn non_string_frontmatter_values_are_ignored() {
        let mut map = HashMap::new();
        let _ = map.insert("language".to_string(), serde_json::json!(42));
        assert_eq!(resolve_render_lang(&map, Some("de")), "de");
    }
}
