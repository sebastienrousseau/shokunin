//! Fuzz the public lol_html entry points in `ssg::util::html_rewriter`
//! — `rewrite_html` (the rewrite chain with zero handlers, which still
//! drives the full streaming parser) and `extract_text_with_filter`
//! (selector-matched text extraction + entity decoding) — with
//! arbitrary byte input.
//!
//! Invariant under test: no panic, no OOM, no hang for any input.
//! Errors (`SsgError`) are the documented failure mode and are fine.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _ = ssg::util::html_rewriter::rewrite_html(&input, Vec::new());
    let _ = ssg::util::html_rewriter::extract_text_with_filter(&input, "p");
});
