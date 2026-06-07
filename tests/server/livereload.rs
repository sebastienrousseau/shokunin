// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::livereload`.

use ssg::livereload::css_reload_message;

#[test]
fn css_reload_message_contains_path() {
    let msg = css_reload_message("css/theme.css");
    assert!(msg.contains("css/theme.css") || msg.contains("theme.css"));
}

#[test]
fn css_reload_message_is_nonempty_for_root_path() {
    let msg = css_reload_message("style.css");
    assert!(!msg.is_empty());
}
