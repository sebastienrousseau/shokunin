//! Fuzz `ssg_core::compile_markdown` — the pulldown-cmark GFM pipeline
//! (tables, strikethrough, task lists) — with arbitrary byte input.
//!
//! Invariant under test: no panic, no OOM, no hang for any input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _html = ssg_core::compile_markdown(&input);
});
