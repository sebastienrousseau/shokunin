// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! ISR manifest shape (issue #546).
//!
//! AC1: `dist/.ssg/manifest.json` exists when `--isr` is passed, and
//! every output URL has an entry mapping to its source dep list +
//! topological hash. AC9 (back-compat): without `--isr`, the manifest
//! is NOT written.

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use std::fs;
use std::path::Path;

use ssg::isr_manifest::{
    build_manifest, IsrManifestPlugin, CONTENT_RELATIVE_DIR,
    MANIFEST_RELATIVE_PATH,
};
use ssg::plugin::{Plugin, PluginContext};
use ssg_core::Manifest;

fn fixture_site(root: &Path) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let content = root.join("content");
    let templates = root.join("templates");
    let site = root.join("public");
    fs::create_dir_all(&content).unwrap();
    fs::create_dir_all(&templates).unwrap();
    fs::create_dir_all(&site).unwrap();

    fs::write(content.join("index.md"), "---\ntitle: Home\n---\n# Home").unwrap();
    fs::create_dir_all(content.join("posts")).unwrap();
    fs::write(
        content.join("posts/alpha.md"),
        "---\ntitle: Alpha\nisr:\n  s_maxage: 300\n  swr: 1800\n---\n# Alpha",
    )
    .unwrap();
    fs::write(
        content.join("posts/bravo.md"),
        "---\ntitle: Bravo\n---\n# Bravo",
    )
    .unwrap();

    fs::write(templates.join("index.html"), "<html><body>{{ content }}</body></html>").unwrap();
    fs::write(templates.join("page.html"), "<html><body>{{ content }}</body></html>").unwrap();

    (content, templates, site)
}

#[test]
fn ac1_manifest_emitted_when_isr_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    let (content, templates, site) = fixture_site(tmp.path());

    let ctx = PluginContext {
        content_dir: content.clone(),
        build_dir: site.clone(),
        site_dir: site.clone(),
        template_dir: templates.clone(),
        config: None,
        cache: None,
        memory_budget: None,
        html_files: None,
        dep_graph: None,
        dry_run: false,
    };

    IsrManifestPlugin.after_compile(&ctx).unwrap();

    // 1. manifest.json exists.
    let mf_path = site.join(MANIFEST_RELATIVE_PATH);
    assert!(mf_path.exists(), "manifest.json must be written when ISR enabled");

    let json = fs::read_to_string(&mf_path).unwrap();
    let parsed: Manifest = serde_json::from_str(&json).unwrap();

    // 2. Each output URL has a mapping.
    let urls: Vec<&String> = parsed.entries.keys().collect();
    assert!(urls.iter().any(|u| u.as_str() == "/index.html"));
    assert!(urls.iter().any(|u| u.as_str() == "/posts/alpha/index.html"));
    assert!(urls.iter().any(|u| u.as_str() == "/posts/bravo/index.html"));

    // 3. Each entry carries sources + hash.
    for (url, entry) in &parsed.entries {
        assert!(
            !entry.sources.is_empty(),
            "{url} must list source dependencies"
        );
        assert_eq!(entry.hash.len(), 64, "{url} hash must be sha256 hex");
        assert!(
            entry.sources.iter().any(|s| s.starts_with("content/")),
            "{url} must list at least one content source"
        );
    }

    // 4. Per-route frontmatter override threaded through.
    let alpha = parsed.get("/posts/alpha/index.html").unwrap();
    let cache = alpha.cache.as_ref().unwrap();
    assert_eq!(cache.s_maxage, 300);
    assert_eq!(cache.swr, 1800);

    // 5. Bravo has no override → no per-entry cache field.
    let bravo = parsed.get("/posts/bravo/index.html").unwrap();
    assert!(bravo.cache.is_none());

    // 6. Raw sources staged under dist/.ssg/content/.
    let staged_md = site
        .join(CONTENT_RELATIVE_DIR)
        .join("content/posts/alpha.md");
    assert!(staged_md.exists(), "alpha markdown should be staged for KV upload");
    let staged_tpl = site
        .join(CONTENT_RELATIVE_DIR)
        .join("templates/index.html");
    assert!(staged_tpl.exists(), "templates must be staged");
}

#[test]
fn ac9_no_manifest_when_isr_disabled() {
    // The IsrManifestPlugin is the ONLY thing that writes
    // dist/.ssg/manifest.json. When register_isr_plugins is not
    // called, the file must not appear.
    use ssg::pipeline::{build_pipeline, RunOptions};
    use ssg::cmd::SsgConfig;

    let tmp = tempfile::tempdir().unwrap();
    let (content, templates, site) = fixture_site(tmp.path());

    let mut config = SsgConfig::default();
    config.content_dir = content;
    config.template_dir = templates;
    config.output_dir = site.clone();

    let opts = RunOptions {
        quiet: true,
        ..Default::default()
    };
    assert!(!opts.isr, "default RunOptions must have isr=false");

    let (plugins, _ctx, _build, _site) = build_pipeline(&config, &opts);

    // Plugin list must not contain isr-manifest.
    let names = plugins.names();
    assert!(
        !names.contains(&"isr-manifest"),
        "isr-manifest plugin must NOT be registered without --isr (AC9)"
    );

    // And the manifest file must not exist.
    assert!(!site.join(MANIFEST_RELATIVE_PATH).exists());
}

#[test]
fn ac9_manifest_present_when_isr_enabled() {
    use ssg::pipeline::{build_pipeline, RunOptions};
    use ssg::cmd::SsgConfig;

    let tmp = tempfile::tempdir().unwrap();
    let (content, templates, site) = fixture_site(tmp.path());

    let mut config = SsgConfig::default();
    config.content_dir = content;
    config.template_dir = templates;
    config.output_dir = site;

    let opts = RunOptions {
        quiet: true,
        isr: true,
        ..Default::default()
    };
    let (plugins, _ctx, _build, _site) = build_pipeline(&config, &opts);

    let names = plugins.names();
    assert!(
        names.contains(&"isr-manifest"),
        "isr-manifest plugin must register when --isr is set"
    );
}

#[test]
fn manifest_is_deterministic_for_same_input() {
    let tmp1 = tempfile::tempdir().unwrap();
    let tmp2 = tempfile::tempdir().unwrap();

    let (c1, t1, _) = fixture_site(tmp1.path());
    let (c2, t2, _) = fixture_site(tmp2.path());

    let m1 = build_manifest(&c1, &t1, tmp1.path()).unwrap();
    let m2 = build_manifest(&c2, &t2, tmp2.path()).unwrap();

    // Hashes per URL must agree byte-for-byte across runs.
    for (url, e1) in &m1.entries {
        let e2 = m2.get(url).unwrap();
        assert_eq!(e1.hash, e2.hash, "hash drift for {url}");
        assert_eq!(e1.sources, e2.sources, "source drift for {url}");
    }
}
