// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::plugin::{PluginContext, PluginManager}`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::plugin::{PluginContext, PluginManager};
use tempfile::tempdir;

#[test]
fn plugin_context_new_records_supplied_dirs() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    let ctx = PluginContext::new(p, p, p, p);
    assert_eq!(ctx.site_dir, p);
}

#[test]
fn plugin_manager_default_constructs_empty() {
    let m = PluginManager::default();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}
