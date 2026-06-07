// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugins::{MinifyPlugin, ImageOptiPlugin}`.

use ssg::plugin::Plugin;
use ssg::plugins::{ImageOptiPlugin, MinifyPlugin};

#[test]
fn minify_plugin_name_is_stable() {
    assert!(!MinifyPlugin.name().is_empty());
}

#[test]
fn image_opti_plugin_name_is_stable() {
    assert!(!ImageOptiPlugin.name().is_empty());
}
