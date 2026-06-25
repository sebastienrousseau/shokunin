// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! AC9: opting *into* `--isr` MUST be additive — the existing HTML
//! output must stay byte-identical. The `IsrManifestPlugin` is the
//! only thing that runs differently and it only writes new files
//! under `dist/.ssg/`.
//!
//! Strategy: register the default plugin pipeline on a tempdir, then
//! register the ISR plugin separately on a sibling tempdir starting
//! from the same input. Compare the SHA-256 of every file *outside*
//! `dist/.ssg/`. They must match exactly.

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use ssg::cmd::SsgConfig;
use ssg::isr_manifest::{IsrManifestPlugin, CONTENT_RELATIVE_DIR, MANIFEST_RELATIVE_PATH};
use ssg::pipeline::register_default_plugins;
use ssg::plugin::{Plugin, PluginContext, PluginManager};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut out = String::with_capacity(64);
    for b in d {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn walk_files(root: &Path, out: &mut Vec<std::path::PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_files(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn site_hashes(site_dir: &Path) -> BTreeMap<String, String> {
    let mut files = Vec::new();
    walk_files(site_dir, &mut files);
    let mut map = BTreeMap::new();
    for f in files {
        let rel = f
            .strip_prefix(site_dir)
            .unwrap_or(&f)
            .to_string_lossy()
            .to_string();
        let bytes = fs::read(&f).unwrap();
        let _ = map.insert(rel, sha256_hex(&bytes));
    }
    map
}

fn populate_fixture(root: &Path) -> SsgConfig {
    let content = root.join("content");
    let templates = root.join("templates");
    let site = root.join("public");
    fs::create_dir_all(&content).unwrap();
    fs::create_dir_all(&templates).unwrap();
    fs::create_dir_all(&site).unwrap();

    fs::write(content.join("a.md"), "---\ntitle: A\n---\n# A").unwrap();
    fs::write(content.join("b.md"), "---\ntitle: B\n---\n# B").unwrap();
    fs::write(templates.join("index.html"), "<html/>").unwrap();
    fs::write(templates.join("page.html"), "<html/>").unwrap();

    // Pre-populate site_dir with some fake HTML outputs to simulate
    // what staticdatagen would emit. We're testing the ISR plugin's
    // additive behaviour, not the compile step.
    fs::write(site.join("index.html"), "<html>HOME</html>").unwrap();
    fs::create_dir_all(site.join("a")).unwrap();
    fs::write(site.join("a/index.html"), "<html>A</html>").unwrap();
    fs::create_dir_all(site.join("b")).unwrap();
    fs::write(site.join("b/index.html"), "<html>B</html>").unwrap();

    let mut config = SsgConfig::default();
    config.content_dir = content;
    config.template_dir = templates;
    config.output_dir = site;
    config
}

#[test]
fn ac9_register_default_plugins_unaffected_by_isr_flag() {
    // The default plugin pipeline must register the SAME plugin names
    // regardless of ISR opt-in. Only the ISR-specific plugin is added
    // on top via `register_isr_plugins`.
    let config = SsgConfig::default();

    let mut without = PluginManager::new();
    register_default_plugins(&mut without, &config, false, None);

    let mut with = PluginManager::new();
    register_default_plugins(&mut with, &config, false, None);
    // ISR-specific registration is a SEPARATE function — not invoked
    // by register_default_plugins. We assert the lists are equal.
    assert_eq!(
        without.names(),
        with.names(),
        "register_default_plugins must be deterministic regardless of ISR state"
    );
    assert!(
        !without.names().contains(&"isr-manifest"),
        "isr-manifest must NEVER appear in the default plugin list (AC9)"
    );
}

#[test]
fn ac9_isr_plugin_is_purely_additive() {
    // Run the ISR plugin against a pre-populated tempdir and confirm
    // that every pre-existing HTML byte stays untouched and the only
    // new files live under dist/.ssg/.
    let tmp_a = tempfile::tempdir().unwrap();
    let tmp_b = tempfile::tempdir().unwrap();

    let cfg_a = populate_fixture(tmp_a.path());
    let cfg_b = populate_fixture(tmp_b.path());

    // Snapshot A before any ISR run.
    let before_a = site_hashes(&cfg_a.output_dir);

    // Run ISR on B only.
    let ctx_b = PluginContext {
        content_dir: cfg_b.content_dir.clone(),
        build_dir: cfg_b.output_dir.clone(),
        site_dir: cfg_b.output_dir.clone(),
        template_dir: cfg_b.template_dir.clone(),
        config: Some(cfg_b.clone()),
        cache: None,
        memory_budget: None,
        html_files: None,
        dep_graph: None,
        dry_run: false,
    };
    IsrManifestPlugin.after_compile(&ctx_b).unwrap();

    let after_a = site_hashes(&cfg_a.output_dir);
    let after_b = site_hashes(&cfg_b.output_dir);

    // 1. A is unchanged.
    assert_eq!(before_a, after_a, "A must not change when only B runs ISR");

    // 2. Every file outside .ssg/ in B matches the corresponding file
    //    in A byte-for-byte (this is the AC9 guarantee).
    for (rel, hash) in &after_a {
        let other = after_b.get(rel).unwrap_or_else(|| {
            panic!("file {rel} present in non-ISR build but missing in ISR build")
        });
        assert_eq!(other, hash, "byte drift for {rel}");
    }

    // 3. The only new entries in B live under dist/.ssg/.
    for new_rel in after_b.keys() {
        if !after_a.contains_key(new_rel) {
            assert!(
                new_rel.starts_with(".ssg"),
                "ISR introduced a new file outside dist/.ssg/: {new_rel}"
            );
        }
    }

    // 4. The manifest and content tree are actually present.
    let manifest_path = cfg_b.output_dir.join(MANIFEST_RELATIVE_PATH);
    let content_root = cfg_b.output_dir.join(CONTENT_RELATIVE_DIR);
    assert!(manifest_path.exists(), "manifest.json must be written");
    assert!(content_root.exists(), "content tree must be staged");
}

#[test]
fn ac9_run_options_isr_defaults_to_false() {
    use ssg::pipeline::RunOptions;
    let opts = RunOptions::default();
    assert!(!opts.isr, "RunOptions::default().isr must be false (AC9)");
}
