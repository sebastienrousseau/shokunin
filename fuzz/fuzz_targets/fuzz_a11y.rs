//! Fuzz `ssg_a11y::check_page` — the WCAG rule engine that runs on every
//! built page — with arbitrary input.
//!
//! The checker parses HTML with hand-written scanners rather than a full
//! parser, which is exactly where malformed markup tends to walk off the end
//! of a slice. It also runs inside `ssg build`, so a panic here fails a user's
//! build on their own content.
//!
//! Invariant under test: no panic, no hang, for any input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let html = String::from_utf8_lossy(data);
    let issues = ssg_a11y::check_page(&html);
    // Force the results to be materialised so a lazy iterator cannot hide a
    // panic behind an unevaluated chain.
    std::hint::black_box(issues.len());
});
