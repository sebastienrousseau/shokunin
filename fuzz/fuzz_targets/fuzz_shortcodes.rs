//! Fuzz `ssg::shortcodes::expand_shortcodes` — block
//! (`{{< warning >}}…{{< /warning >}}`) and inline
//! (`{{< name key="value" >}}`) shortcode expansion — with arbitrary
//! byte input.
//!
//! Invariant under test: no panic, no OOM, no hang for any input,
//! including unbalanced/nested/interleaved shortcode delimiters.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _expanded = ssg::shortcodes::expand_shortcodes(&input);
});
