// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::postprocess::sbom::SbomPlugin` (post-process
//! `sbom-link` HTML rewriter — distinct from the top-level
//! `ssg::sbom::SbomPlugin` which emits the JSON manifest itself).

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::plugin::Plugin;
use ssg::postprocess::SbomPlugin;

#[test]
fn postprocess_sbom_plugin_name_is_stable() {
    assert!(!SbomPlugin.name().is_empty());
}
