// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::taxonomy`.

use ssg::plugin::Plugin;
use ssg::taxonomy::TaxonomyPlugin;

#[test]
fn taxonomy_plugin_name_is_stable() {
    assert!(!TaxonomyPlugin.name().is_empty());
}
