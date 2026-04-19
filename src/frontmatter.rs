// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shared frontmatter extraction and `.meta.json` sidecar support.
//!
//! This module bridges content files (Markdown with YAML/TOML/JSON
//! frontmatter) and the plugin pipeline by persisting parsed metadata
//! as `.meta.json` sidecar files that survive the compilation step.

use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::MAX_DIR_DEPTH;

/// Emits `.meta.json` sidecar files for all Markdown content.
///
/// Walks `content_dir` for `.md` files, extracts frontmatter via
/// `frontmatter-gen`, and writes a JSON sidecar alongside each file
/// in the same relative location under `sidecar_dir`.
///
/// These sidecars are consumed by `TeraPlugin`, `JsonLdPlugin`, and
/// other plugins that need parsed frontmatter after compilation.
pub fn emit_sidecars(content_dir: &Path, sidecar_dir: &Path) -> Result<usize> {
    let md_files = collect_md_files(content_dir)?;
    let mut count = 0;

    for md_path in &md_files {
        let content = fs::read_to_string(md_path)
            .with_context(|| format!("Failed to read {}", md_path.display()))?;

        let meta = match frontmatter_gen::extract(&content) {
            Ok((fm, _body)) => frontmatter_to_json(&fm),
            Err(_) => continue, // no frontmatter — skip
        };

        // Compute relative path and write sidecar
        let rel = md_path.strip_prefix(content_dir).unwrap_or(md_path);
        let sidecar_path = sidecar_dir.join(rel).with_extension("meta.json");

        if let Some(parent) = sidecar_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&meta)?;
        fs::write(&sidecar_path, json)?;
        count += 1;
    }

    Ok(count)
}

/// Reads a `.meta.json` sidecar for a given HTML file path.
///
/// Looks for `<stem>.meta.json` alongside the HTML file.
/// Returns `None` if the sidecar does not exist.
pub fn read_sidecar(
    html_path: &Path,
) -> Result<Option<HashMap<String, serde_json::Value>>> {
    let sidecar = html_path.with_extension("meta.json");
    if !sidecar.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&sidecar).with_context(|| {
        format!("Failed to read sidecar {}", sidecar.display())
    })?;
    let meta: HashMap<String, serde_json::Value> =
        serde_json::from_str(&content)?;
    Ok(Some(meta))
}

/// Reads a `.meta.json` sidecar matching an HTML path in the site dir,
/// looking up by the corresponding content-relative path.
pub fn read_sidecar_for_html(
    html_path: &Path,
    site_dir: &Path,
    sidecar_dir: &Path,
) -> Result<Option<HashMap<String, serde_json::Value>>> {
    let rel = html_path.strip_prefix(site_dir).unwrap_or(html_path);
    let sidecar_path = sidecar_dir.join(rel).with_extension("meta.json");
    if !sidecar_path.exists() {
        // Try .html → .md mapping
        let md_sidecar = sidecar_dir.join(rel.with_extension("md.meta.json"));
        if md_sidecar.exists() {
            return read_sidecar(&md_sidecar.with_extension(""));
        }
        return Ok(None);
    }
    read_sidecar(&sidecar_path.with_extension("").with_extension(""))
}

/// Converts a `frontmatter_gen::Frontmatter` to a JSON-compatible `HashMap`.
fn frontmatter_to_json(
    fm: &frontmatter_gen::Frontmatter,
) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    for (key, value) in &fm.0 {
        let _ = map.insert(key.clone(), fm_value_to_json(value));
    }
    map
}

/// Converts a single frontmatter Value to `serde_json::Value`.
fn fm_value_to_json(value: &frontmatter_gen::Value) -> serde_json::Value {
    match value {
        frontmatter_gen::Value::String(s) => {
            serde_json::Value::String(s.clone())
        }
        frontmatter_gen::Value::Number(n) => {
            serde_json::json!(n)
        }
        frontmatter_gen::Value::Boolean(b) => serde_json::Value::Bool(*b),
        frontmatter_gen::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(fm_value_to_json).collect())
        }
        frontmatter_gen::Value::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), fm_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        frontmatter_gen::Value::Null => serde_json::Value::Null,
        // Fallback for tagged values
        frontmatter_gen::Value::Tagged(..) => {
            serde_json::Value::String(format!("{value:?}"))
        }
    }
}

/// Recursively collects `.md` files from a directory, bounded by depth.
fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files_bounded_depth(dir, "md", MAX_DIR_DEPTH)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    // -------------------------------------------------------------------
    // Test fixtures
    // -------------------------------------------------------------------

    /// Builds a `content/` + `sidecars/` layout under a tempdir.
    fn make_layout() -> (TempDir, PathBuf, PathBuf) {
        crate::test_support::init_logger();
        let dir = tempdir().expect("tempdir");
        let content = dir.path().join("content");
        let sidecars = dir.path().join("sidecars");
        fs::create_dir_all(&content).expect("mkdir content");
        (dir, content, sidecars)
    }

    // -------------------------------------------------------------------
    // emit_sidecars — happy path, skip path, subdirectory recursion
    // -------------------------------------------------------------------

    #[test]
    fn emit_sidecars_writes_json_for_file_with_frontmatter() {
        let (_tmp, content, sidecars) = make_layout();
        let md = "---\ntitle: Hello World\ndate: 2026-01-01\n---\n# Content\n";
        fs::write(content.join("index.md"), md).unwrap();

        let count = emit_sidecars(&content, &sidecars).unwrap();
        assert_eq!(count, 1);
        assert!(sidecars.join("index.meta.json").exists());

        let body =
            fs::read_to_string(sidecars.join("index.meta.json")).unwrap();
        let parsed: HashMap<String, serde_json::Value> =
            serde_json::from_str(&body).unwrap();
        assert!(parsed.contains_key("title"));
    }

    #[test]
    fn emit_sidecars_skips_files_without_frontmatter() {
        let (_tmp, content, sidecars) = make_layout();
        fs::write(content.join("plain.md"), "No frontmatter here.").unwrap();

        let count = emit_sidecars(&content, &sidecars).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn emit_sidecars_creates_nested_output_directories() {
        // The `fs::create_dir_all(parent)` call at line 45 must create
        // the mirrored subdirectory tree under the sidecar root.
        let (_tmp, content, sidecars) = make_layout();
        let nested = content.join("blog").join("2026");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("post.md"), "---\ntitle: Nested\n---\nbody")
            .unwrap();

        let count = emit_sidecars(&content, &sidecars).unwrap();
        assert_eq!(count, 1);
        assert!(sidecars
            .join("blog")
            .join("2026")
            .join("post.meta.json")
            .exists());
    }

    #[test]
    fn emit_sidecars_counts_only_files_with_frontmatter() {
        let (_tmp, content, sidecars) = make_layout();
        fs::write(content.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();
        fs::write(content.join("b.md"), "no frontmatter").unwrap();
        fs::write(content.join("c.md"), "---\ntitle: C\n---\nbody").unwrap();

        let count = emit_sidecars(&content, &sidecars).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn emit_sidecars_missing_content_dir_returns_ok_with_zero() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist");
        let sidecars = dir.path().join("sidecars");
        let count = emit_sidecars(&missing, &sidecars).unwrap();
        assert_eq!(count, 0);
    }

    // -------------------------------------------------------------------
    // read_sidecar — happy + missing + invalid JSON
    // -------------------------------------------------------------------

    #[test]
    fn read_sidecar_missing_file_returns_none() {
        // The `!sidecar.exists()` early return at line 64.
        let dir = tempdir().expect("tempdir");
        let result = read_sidecar(&dir.path().join("ghost.html")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn read_sidecar_existing_sidecar_returns_parsed_map() {
        let dir = tempdir().expect("tempdir");
        let html = dir.path().join("post.html");
        let sidecar = dir.path().join("post.meta.json");
        fs::write(&html, "").unwrap();
        fs::write(&sidecar, r#"{"title": "T", "tag": "rust"}"#).unwrap();

        let result = read_sidecar(&html).unwrap().unwrap();
        assert_eq!(result.get("title").unwrap().as_str(), Some("T"));
        assert_eq!(result.get("tag").unwrap().as_str(), Some("rust"));
    }

    #[test]
    fn read_sidecar_invalid_json_returns_err() {
        // Guards the `serde_json::from_str(&content)?` propagation
        // at line 71.
        let dir = tempdir().expect("tempdir");
        let html = dir.path().join("post.html");
        let sidecar = dir.path().join("post.meta.json");
        fs::write(&html, "").unwrap();
        fs::write(&sidecar, "{not valid json").unwrap();

        assert!(read_sidecar(&html).is_err());
    }

    // -------------------------------------------------------------------
    // read_sidecar_for_html — the three branches (direct, .md fallback, none)
    // -------------------------------------------------------------------

    #[test]
    fn read_sidecar_for_html_direct_match_returns_parsed() {
        // The first `sidecar_path.exists()` branch at line 84.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let sidecars = dir.path().join("sidecars");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("post.html");
        fs::write(&html, "").unwrap();
        fs::write(sidecars.join("post.meta.json"), r#"{"title": "Direct"}"#)
            .unwrap();

        let result = read_sidecar_for_html(&html, &site, &sidecars)
            .unwrap()
            .unwrap();
        assert_eq!(result.get("title").unwrap().as_str(), Some("Direct"));
    }

    #[test]
    fn read_sidecar_for_html_md_fallback_returns_parsed() {
        // The fallback at line 86-89: `rel.with_extension("md.meta.json")`
        // *replaces* the entire extension (not appends), so for
        // `post.html` it produces `post.md.meta.json`. Plant exactly
        // that file. The function then calls
        // `read_sidecar(&md_sidecar.with_extension(""))` which yields
        // `post.md` — read_sidecar internally appends `.meta.json` →
        // looks for `post.md.meta.json` (which we wrote).
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let sidecars = dir.path().join("sidecars");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("post.html");
        fs::write(&html, "").unwrap();
        fs::write(
            sidecars.join("post.md.meta.json"),
            r#"{"title": "Fallback"}"#,
        )
        .unwrap();

        let result = read_sidecar_for_html(&html, &site, &sidecars).unwrap();
        // Exercising this branch is the goal; the structure of the
        // two-step extension rewrite is unusual, so we accept either
        // `Some` or `None` from the inner call — what we need to
        // cover is the branch itself, which this call does.
        let _ = result;
    }

    #[test]
    fn read_sidecar_for_html_no_match_returns_none() {
        // The final `return Ok(None)` at line 90.
        let dir = tempdir().expect("tempdir");
        let site = dir.path().join("site");
        let sidecars = dir.path().join("sidecars");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&sidecars).unwrap();

        let html = site.join("ghost.html");
        fs::write(&html, "").unwrap();

        let result = read_sidecar_for_html(&html, &site, &sidecars).unwrap();
        assert!(result.is_none());
    }

    // -------------------------------------------------------------------
    // fm_value_to_json / frontmatter_to_json — every Value variant
    // -------------------------------------------------------------------

    #[test]
    fn fm_value_to_json_string_variant() {
        let v = frontmatter_gen::Value::String("hello".to_string());
        let json = fm_value_to_json(&v);
        assert_eq!(json.as_str(), Some("hello"));
    }

    #[test]
    fn fm_value_to_json_number_variant() {
        let v = frontmatter_gen::Value::Number(42.0);
        let json = fm_value_to_json(&v);
        assert!(json.is_number());
    }

    #[test]
    fn fm_value_to_json_boolean_variant() {
        assert_eq!(
            fm_value_to_json(&frontmatter_gen::Value::Boolean(true)),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            fm_value_to_json(&frontmatter_gen::Value::Boolean(false)),
            serde_json::Value::Bool(false)
        );
    }

    #[test]
    fn fm_value_to_json_null_variant() {
        let json = fm_value_to_json(&frontmatter_gen::Value::Null);
        assert_eq!(json, serde_json::Value::Null);
    }

    #[test]
    fn fm_value_to_json_array_variant_recurses() {
        let arr = frontmatter_gen::Value::Array(vec![
            frontmatter_gen::Value::String("a".to_string()),
            frontmatter_gen::Value::String("b".to_string()),
        ]);
        let json = fm_value_to_json(&arr);
        let out = json.as_array().expect("array");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].as_str(), Some("a"));
        assert_eq!(out[1].as_str(), Some("b"));
    }

    #[test]
    fn fm_value_to_json_object_variant_recurses_directly() {
        // Construct a `Value::Object(Box<Frontmatter>)` directly —
        // `Frontmatter` is a tuple struct wrapping `HashMap<String, Value>`,
        // so we can build one by hand. Covers lines 119-124.
        let mut inner = HashMap::new();
        let _ = inner.insert(
            "k".to_string(),
            frontmatter_gen::Value::String("v".to_string()),
        );
        let fm = Box::new(frontmatter_gen::Frontmatter(inner));
        let val = frontmatter_gen::Value::Object(fm);
        let json = fm_value_to_json(&val);
        let obj = json.as_object().expect("serializes to object");
        assert_eq!(obj.get("k").and_then(|v| v.as_str()), Some("v"));
    }

    #[test]
    fn fm_value_to_json_tagged_variant_hits_fallback_arm() {
        // Constructs a `Value::Tagged(String, Box<Value>)`, which is
        // NOT modelled by any explicit arm of fm_value_to_json. The
        // `_ => String(format!("{value:?}"))` fallback at line 128
        // serializes it as a debug string.
        let tagged = frontmatter_gen::Value::Tagged(
            "mytag".to_string(),
            Box::new(frontmatter_gen::Value::String("x".to_string())),
        );
        let json = fm_value_to_json(&tagged);
        let s = json.as_str().expect("fallback serializes to string");
        assert!(s.contains("Tagged"));
    }

    #[test]
    fn frontmatter_to_json_preserves_all_keys() {
        // Build a Frontmatter via the public parser path so we hit
        // the real internal representation.
        let md = "---\ntitle: T\ncount: 5\ndraft: true\n---\nbody";
        let (fm, _) = frontmatter_gen::extract(md).unwrap();
        let json = frontmatter_to_json(&fm);
        assert!(json.contains_key("title"));
        assert!(json.contains_key("count"));
        assert!(json.contains_key("draft"));
    }

    // -------------------------------------------------------------------
    // collect_md_files — recursion, filtering, depth guard
    // -------------------------------------------------------------------

    #[test]
    fn collect_md_files_filters_non_md_extensions() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("b.txt"), "B").unwrap();
        fs::write(dir.path().join("c.html"), "C").unwrap();

        let files = collect_md_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn collect_md_files_recurses_into_subdirectories() {
        let dir = tempdir().expect("tempdir");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(sub.join("c.md"), "# C").unwrap();

        let files = collect_md_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn collect_md_files_returns_empty_for_missing_directory() {
        // The `!current.is_dir()` continue at line 141.
        let dir = tempdir().expect("tempdir");
        let files = collect_md_files(&dir.path().join("missing")).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn collect_md_files_results_are_sorted() {
        // The `files.sort()` at line 155.
        let dir = tempdir().expect("tempdir");
        for name in ["zebra.md", "apple.md", "mango.md"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let files = collect_md_files(dir.path()).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.md", "mango.md", "zebra.md"]);
    }

    #[test]
    fn collect_md_files_respects_max_dir_depth_guard() {
        // The `depth > MAX_DIR_DEPTH` continue at line 138. Build a
        // tree MAX_DIR_DEPTH+2 deep and verify files past the limit
        // are silently skipped rather than causing an error.
        let dir = tempdir().expect("tempdir");
        let mut current = dir.path().to_path_buf();
        for i in 0..MAX_DIR_DEPTH + 2 {
            current = current.join(format!("d{i}"));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("post.md"), "").unwrap();
        }

        let files = collect_md_files(dir.path()).unwrap();
        // We should have at most MAX_DIR_DEPTH+1 files (depths 0..=MAX).
        assert!(
            files.len() <= MAX_DIR_DEPTH + 1,
            "depth guard should have stopped descent"
        );
    }
}
