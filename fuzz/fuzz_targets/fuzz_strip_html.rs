//! Fuzz `ssg_core::strip_html_tags` and `build_search_entry`.
//!
//! `strip_html_tags` is a character-level state machine over untrusted HTML,
//! and its output feeds the search index that ships to every visitor. Two
//! properties are asserted beyond "does not panic", because a silent failure
//! here degrades search rather than crashing anything:
//!
//!   * stripping never lengthens the input
//!   * no `<` or `>` survives into indexed content
//!
//! Invariant under test: no panic, no hang, and both properties hold.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let html = String::from_utf8_lossy(data);

    let plain = ssg_core::strip_html_tags(&html);
    assert!(
        plain.len() <= html.len(),
        "stripping tags lengthened the input"
    );

    let entry = ssg_core::build_search_entry("t", "/u", &html);
    assert!(
        !entry.content.contains('<') && !entry.content.contains('>'),
        "raw angle bracket survived into a search entry"
    );
});
