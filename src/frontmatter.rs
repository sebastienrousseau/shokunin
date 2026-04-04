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
            .with_context(|| format!("Failed to read {:?}", md_path))?;

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

    let content = fs::read_to_string(&sidecar)
        .with_context(|| format!("Failed to read sidecar {:?}", sidecar))?;
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

/// Converts a `frontmatter_gen::Frontmatter` to a JSON-compatible HashMap.
fn frontmatter_to_json(
    fm: &frontmatter_gen::Frontmatter,
) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    for (key, value) in fm.0.iter() {
        let _ = map.insert(key.clone(), fm_value_to_json(value));
    }
    map
}

/// Converts a single frontmatter Value to serde_json::Value.
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
        // Fallback for any other variant
        _ => serde_json::Value::String(format!("{:?}", value)),
    }
}

/// Recursively collects `.md` files from a directory, bounded by depth.
fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];

    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DIR_DEPTH {
            continue;
        }
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_emit_and_read_sidecar() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path().join("content");
        let sidecar_dir = dir.path().join("sidecars");
        fs::create_dir_all(&content_dir).unwrap();

        let md = "---\ntitle: Hello World\ndate: 2026-01-01\n---\n# Content\n";
        fs::write(content_dir.join("index.md"), md).unwrap();

        let count = emit_sidecars(&content_dir, &sidecar_dir).unwrap();
        assert_eq!(count, 1);

        let sidecar_path = sidecar_dir.join("index.meta.json");
        assert!(sidecar_path.exists());

        // Verify sidecar file is valid JSON
        let content = fs::read_to_string(&sidecar_path).unwrap();
        let parsed: HashMap<String, serde_json::Value> =
            serde_json::from_str(&content).unwrap();
        assert!(parsed.contains_key("title"));
    }

    #[test]
    fn test_read_sidecar_missing() {
        let result = read_sidecar(Path::new("/nonexistent/file.meta.json"));
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_collect_md_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("b.txt"), "B").unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("c.md"), "# C").unwrap();

        let files = collect_md_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_no_frontmatter_skipped() {
        let dir = tempdir().unwrap();
        let content_dir = dir.path().join("content");
        let sidecar_dir = dir.path().join("sidecars");
        fs::create_dir_all(&content_dir).unwrap();

        fs::write(content_dir.join("plain.md"), "No frontmatter here.")
            .unwrap();

        let count = emit_sidecars(&content_dir, &sidecar_dir).unwrap();
        assert_eq!(count, 0);
    }
}
