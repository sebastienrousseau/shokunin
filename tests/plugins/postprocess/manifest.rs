// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use ssg::plugin::Plugin;
use ssg::postprocess::ManifestFixPlugin;

#[test]
fn manifest_fix_plugin_name_is_stable() {
    assert!(!ManifestFixPlugin.name().is_empty());
}
