// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ssg::plugin::Plugin;
use ssg::seo::RobotsPlugin;

#[test]
fn robots_plugin_name_is_stable() {
    let p = RobotsPlugin::new("https://example.com");
    assert!(!p.name().is_empty());
}
