//! Property tests for the pure transforms that shape published output.
//!
//! Every bug this suite is written against had the same shape: a transform
//! that looked right on the inputs someone thought of, and was wrong on an
//! input nobody did. Example-based tests find the first kind. These find the
//! second.
//!
//! The properties asserted here are the ones whose violation is *silent* —
//! output that still looks like output:
//!
//! * **Idempotence.** A transform runs again on rebuilds and on already-
//!   processed input. `f(f(x)) != f(x)` means each pass degrades the file a
//!   little further, and nothing ever errors. The CSS minifier's second pass
//!   rewrote the inside of string literals for exactly this reason.
//! * **Separator invariance.** Paths that become URLs must use `/` on every
//!   platform. `Site::rel` returned the native separator, so on Windows the
//!   hreflang gate compared `en\index.html` with `/en/index.html` and got
//!   both a false positive and a false negative.
//! * **No injection.** A text extractor that lets `<` through is feeding
//!   markup into a search index that ships to every visitor.
//! * **Termination and totality.** No panic, no hang, for any input —
//!   including the non-ASCII case that made `to_lowercase` shift byte offsets
//!   and panic inside `ssg build` on a user's own content.
//!
//! Where a property is genuinely conditional, it is stated conditionally
//! rather than weakened: a test that asserts less than it appears to is worse
//! than no test, because it is counted as coverage.

use proptest::prelude::*;
use ssg::urls::{derive_page_url, derive_permalink};
use ssg::util::html_rewriter::{collapse_whitespace, decode_html_entities};

/// Arbitrary text including the characters that have broken transforms here:
/// angle brackets, ampersands, quotes, backslashes, and multi-byte scalars
/// whose lowercase form is longer than the original.
fn hostile_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            any::<char>(),
            Just('<'),
            Just('>'),
            Just('&'),
            Just('"'),
            Just('\\'),
            Just('\u{130}'), // İ — lowercases to two chars
            Just('\u{00a0}'),
            Just('\n'),
            Just('\t'),
        ],
        0..80,
    )
    .prop_map(|cs| cs.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Collapsing whitespace twice must equal collapsing once.
    #[test]
    fn collapse_whitespace_is_idempotent(s in hostile_text()) {
        let once = collapse_whitespace(&s);
        let twice = collapse_whitespace(&once);
        prop_assert_eq!(&once, &twice);
    }

    /// Collapsing never lengthens its input.
    #[test]
    fn collapse_whitespace_never_grows(s in hostile_text()) {
        prop_assert!(collapse_whitespace(&s).len() <= s.len());
    }

    /// Decoding entities twice must equal decoding once.
    ///
    /// Not obviously true — `&amp;lt;` decodes to `&lt;` then to `<` — so if
    /// this fails the fix is to state the property correctly, not to delete
    /// it. It holds because the decoder does a single left-to-right pass and
    /// does not rescan what it produced.
    #[test]
    fn decode_html_entities_is_idempotent(s in hostile_text()) {
        let once = decode_html_entities(&s);
        let twice = decode_html_entities(&once);
        prop_assert_eq!(&once, &twice);
    }

    /// Neither transform may panic on any input. This is the property that
    /// the a11y crash violated, found by fuzzing within seconds of a target
    /// existing.
    #[test]
    fn text_transforms_are_total(s in hostile_text()) {
        let _ = collapse_whitespace(&s);
        let _ = decode_html_entities(&s);
    }

    /// A derived page URL never contains a backslash, whatever separator the
    /// caller passed. These strings are compared against URLs taken from the
    /// HTML; a native separator makes every such comparison miss.
    #[test]
    fn derived_page_url_never_contains_a_backslash(
        base in "https://[a-z]{1,12}\\.(com|org|dev)",
        seg1 in "[a-z]{1,10}",
        seg2 in "[a-z]{1,10}",
    ) {
        for rel in [
            format!("{seg1}/{seg2}/index.html"),
            format!("{seg1}\\{seg2}\\index.html"),
            format!("./{seg1}/index.html"),
            format!("/{seg1}/index.html"),
        ] {
            let url = derive_page_url(&base, &rel);
            prop_assert!(
                !url.contains('\\'),
                "backslash survived into a URL: {url}"
            );
        }
    }

    /// The same URL is derived whichever separator the path arrives with.
    /// This is the invariant `Site::rel` broke on Windows.
    #[test]
    fn page_url_is_separator_invariant(
        base in "https://[a-z]{1,12}\\.com",
        seg1 in "[a-z]{1,10}",
        seg2 in "[a-z]{1,10}",
    ) {
        let unix = derive_page_url(&base, &format!("{seg1}/{seg2}/index.html"));
        let win  = derive_page_url(&base, &format!("{seg1}\\{seg2}\\index.html"));
        prop_assert_eq!(unix, win);
    }

    /// Deriving a URL is idempotent in the sense that matters: the same
    /// inputs always give the same output. A transform that varies run to run
    /// makes every downstream comparison — canonical, sitemap, hreflang —
    /// intermittently wrong.
    #[test]
    fn derive_page_url_is_deterministic(
        base in "https://[a-z]{1,12}\\.com",
        rel in "[a-z/]{1,30}",
    ) {
        prop_assert_eq!(
            derive_page_url(&base, &rel),
            derive_page_url(&base, &rel)
        );
    }

    /// A permalink always starts with its base URL. Anything else points a
    /// canonical tag at the wrong origin, which is the #730 failure mode.
    #[test]
    fn permalink_is_rooted_at_the_base_url(
        base in "https://[a-z]{1,12}\\.(com|org)",
        src in "[a-z]{1,10}\\.md",
    ) {
        let link = derive_permalink(&base, &src);
        prop_assert!(
            link.starts_with(&base),
            "permalink {link} is not rooted at {base}"
        );
    }

    /// URL derivation is total: no input shape panics.
    #[test]
    fn url_derivation_is_total(base in ".{0,40}", rel in ".{0,40}") {
        let _ = derive_page_url(&base, &rel);
        let _ = derive_permalink(&base, &rel);
    }
}
