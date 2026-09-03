// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::scaffold`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::scaffold::scaffold_project_at;
use tempfile::tempdir;

#[test]
fn scaffold_project_at_creates_project_dirs() {
    let dir = tempdir().unwrap();
    scaffold_project_at("mysite", dir.path()).unwrap();
    let root = dir.path().join("mysite");
    assert!(root.exists(), "project root created");
    assert!(root.join("content").exists() || root.join("templates").exists());
}

#[test]
fn scaffold_project_at_is_idempotent_on_existing_path() {
    let dir = tempdir().unwrap();
    scaffold_project_at("twice", dir.path()).unwrap();
    let _second = scaffold_project_at("twice", dir.path());
    // second run may succeed or report an existing-project error;
    // either way it should not panic.
}

/// A scaffolded project must build. Issue #752.
///
/// This is the first thing a new user does, and it did not work: the
/// scaffolder wrote `MiniJinja` templates under `templates/tera/` and
/// nothing at the template-directory root, while the compile step
/// delegates to `StaticWeaver`, which reads `{{variable}}` templates from
/// the root. The first stage failed before the plugin that would have
/// used the `MiniJinja` set ever ran, and the error named the output
/// staging directory rather than the missing template:
///
///     I/O error at 'public.build-tmp': No such file or directory
///
/// Asserting on artefacts rather than on the exit status is deliberate:
/// `compile_site` returned `Ok` in an early version of this
/// investigation while producing nothing, and a probe that counted only
/// the top level of the output directory reported zero files for a
/// build that had actually succeeded.
#[test]
fn scaffolded_project_compiles_and_emits_pages() {
    use std::fs;
    use std::path::Path;

    fn count_html(dir: &Path) -> usize {
        let mut n = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    n += count_html(&path);
                } else if path.extension().is_some_and(|x| x == "html") {
                    n += 1;
                }
            }
        }
        n
    }

    let dir = tempdir().expect("tempdir");
    scaffold_project_at("buildable", dir.path()).expect("scaffold");
    let root = dir.path().join("buildable");

    let content = root.join("content");
    let build = root.join("build");
    let site = root.join("public");
    let template = root.join("templates");
    fs::create_dir_all(&build).expect("build dir");
    fs::create_dir_all(&site).expect("site dir");

    ssg::compile_site(&build, &content, &site, &template)
        .expect("a freshly scaffolded project must compile");

    let pages = count_html(&site);
    assert!(
        pages >= 3,
        "the scaffolded project produced {pages} HTML page(s); its \
         content directory ships an index, an about page and a blog \
         post, so anything less means the build silently emitted \
         nothing useful"
    );
}

/// The four `StaticWeaver` templates the compile step needs must exist at
/// the template-directory root, not only under `tera/`.
///
/// The required set was measured by adding them one at a time to a
/// scaffolded project: with only three of the four the build still
/// fails. Naming them here means a future tidy-up that moves or renames
/// one fails with the reason rather than with `os error 2`.
#[test]
fn scaffold_writes_the_root_templates_the_compiler_requires() {
    let dir = tempdir().expect("tempdir");
    scaffold_project_at("roots", dir.path()).expect("scaffold");
    let templates = dir.path().join("roots").join("templates");

    for required in ["template.html", "index.html", "page.html", "post.html"] {
        assert!(
            templates.join(required).is_file(),
            "missing root template {required}; the compile step reads \
             StaticWeaver templates from the template-directory root, \
             and without all four it aborts before any plugin runs"
        );
    }
}
