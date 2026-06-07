// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::IslandPlugin`.

use ssg::islands::IslandPlugin;
use ssg::plugin::Plugin;

#[test]
fn island_plugin_name_is_stable() {
    assert!(!IslandPlugin.name().is_empty());
}
