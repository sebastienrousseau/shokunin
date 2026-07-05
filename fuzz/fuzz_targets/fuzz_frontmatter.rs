//! Fuzz `ssg_core::parse_frontmatter` — the TOML (`+++`), YAML (`---`),
//! and JSON (`{`) frontmatter dispatch — with arbitrary byte input.
//!
//! Invariant under test: no panic, no OOM, no hang for any input.
//! The function is infallible by signature (malformed frontmatter
//! degrades to an empty map + the raw body), so any crash is a bug.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let (_frontmatter, _body) = ssg_core::parse_frontmatter(&input);
});
