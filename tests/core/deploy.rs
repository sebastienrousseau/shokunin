// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::deploy::{DeployPlugin, DeployTarget}`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::fs;

use ssg::deploy::{DeployPlugin, DeployTarget};
use ssg::plugin::{Plugin, PluginContext};
use tempfile::tempdir;

fn make_ctx(site: &std::path::Path) -> PluginContext {
    PluginContext::new(site, site, site, site)
}

#[test]
fn netlify_target_emits_netlify_toml() {
    let dir = tempdir().unwrap();
    let site = dir.path();
    fs::create_dir_all(site).unwrap();
    let plugin = DeployPlugin::new(DeployTarget::Netlify);
    plugin.after_compile(&make_ctx(site)).unwrap();
    assert!(site.join("netlify.toml").exists());
}

#[test]
fn vercel_target_emits_vercel_json() {
    let dir = tempdir().unwrap();
    let plugin = DeployPlugin::new(DeployTarget::Vercel);
    plugin.after_compile(&make_ctx(dir.path())).unwrap();
    assert!(dir.path().join("vercel.json").exists());
}

#[test]
fn cloudflare_target_emits_headers_and_redirects() {
    let dir = tempdir().unwrap();
    let plugin = DeployPlugin::new(DeployTarget::CloudflarePages);
    plugin.after_compile(&make_ctx(dir.path())).unwrap();
    assert!(dir.path().join("_headers").exists());
}

#[test]
fn github_pages_target_emits_nojekyll() {
    let dir = tempdir().unwrap();
    let plugin = DeployPlugin::new(DeployTarget::GithubPages);
    plugin.after_compile(&make_ctx(dir.path())).unwrap();
    assert!(dir.path().join(".nojekyll").exists());
}

#[test]
fn after_compile_noop_when_site_missing() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("nope");
    let plugin = DeployPlugin::new(DeployTarget::Netlify);
    let ctx = PluginContext::new(&missing, &missing, &missing, &missing);
    // Should not error on missing site directory.
    plugin.after_compile(&ctx).unwrap();
}
