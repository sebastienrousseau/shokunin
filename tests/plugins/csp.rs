// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::CspPlugin`.

use ssg::csp::CspPlugin;
use ssg::plugin::Plugin;

#[test]
fn csp_plugin_name_is_stable() {
    assert!(!CspPlugin.name().is_empty());
}
