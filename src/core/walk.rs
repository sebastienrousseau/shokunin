// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shared bounded directory walkers.
//!
//! Replaces the per-plugin `collect_*_files` helpers that previously
//! lived in nearly every module. Each function performs an iterative
//! (no-recursion) walk with optional bounds and returns a sorted
//! `Vec<PathBuf>` for deterministic test output.
//!
//! ## Variants
//!
//! - [`walk_files`] — single-extension filter, no bounds.
//! - [`walk_files_multi`] — multiple extensions (case-insensitive).
//! - [`walk_files_bounded_depth`] — single extension with a maximum
//!   directory depth (for content trees).
//! - [`walk_files_bounded_count`] — single extension with a maximum
//!   file-count cap (for live-reload / batch I/O fast-paths).
//!
//! All variants return `Ok(Vec::new())` when the root directory does
//! not exist or is not a directory — matching the convention used by
//! every previous local collector in the crate.

use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Recursively collects files matching `extension` under `dir`.
///
/// Sorted output, no recursion (uses an explicit stack), no depth or
/// count bounds. Returns `Ok(Vec::new())` if `dir` does not exist.
pub fn walk_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Recursively collects files matching any of `extensions` under `dir`.
///
/// Extension matching is **case-insensitive** so `IMG.JPG` and
/// `img.jpg` are both collected when `extensions` contains `"jpg"`.
/// Sorted output.
pub fn walk_files_multi(
    dir: &Path,
    extensions: &[&str],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if extensions.contains(&ext_lower.as_str()) {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Recursively collects files matching `extension`, bounded by depth.
///
/// Subdirectories beyond `max_depth` are silently skipped. Used by
/// content walkers that respect [`crate::MAX_DIR_DEPTH`] as a guard
/// against pathological symlink loops.
pub fn walk_files_bounded_depth(
    dir: &Path,
    extension: &str,
    max_depth: usize,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];
    while let Some((current, depth)) = stack.pop() {
        if depth > max_depth || !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if path.extension().is_some_and(|e| e == extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Recursively collects files matching `extension`, capped at
/// `max_files`. Provides `with_context` on the underlying `read_dir`
/// failure.
///
/// Used by `livereload` (50 000 file cap) and similar fast-path
/// walkers that need a bounded latency upper bound.
pub fn walk_files_bounded_count(
    dir: &Path,
    extension: &str,
    max_files: usize,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if files.len() >= max_files {
            break;
        }
        if !current.is_dir() {
            continue;
        }
        let entries = fs::read_dir(&current)
            .with_context(|| format!("cannot read {}", current.display()))?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == extension) {
                files.push(path);
                if files.len() >= max_files {
                    break;
                }
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // -------------------------------------------------------------------
    // walk_files
    // -------------------------------------------------------------------

    #[test]
    fn walk_files_returns_empty_for_missing_directory() {
        let dir = tempdir().unwrap();
        let result = walk_files(&dir.path().join("missing"), "html").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn walk_files_filters_by_extension() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(dir.path().join("b.css"), "").unwrap();
        fs::write(dir.path().join("c.js"), "").unwrap();

        let result = walk_files(dir.path(), "html").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("a.html"));
    }

    #[test]
    fn walk_files_recurses_into_subdirectories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("top.md"), "").unwrap();
        fs::write(nested.join("deep.md"), "").unwrap();

        let result = walk_files(dir.path(), "md").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn walk_files_returns_results_sorted() {
        let dir = tempdir().unwrap();
        for name in ["zebra.html", "apple.html", "mango.html"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let result = walk_files(dir.path(), "html").unwrap();
        let names: Vec<_> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["apple.html", "mango.html", "zebra.html"]);
    }

    // -------------------------------------------------------------------
    // walk_files_multi
    // -------------------------------------------------------------------

    #[test]
    fn walk_files_multi_collects_each_supplied_extension() {
        let dir = tempdir().unwrap();
        for name in ["a.jpg", "b.jpeg", "c.png", "d.gif", "e.txt"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let result =
            walk_files_multi(dir.path(), &["jpg", "jpeg", "png"]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn walk_files_multi_extension_match_is_case_insensitive() {
        let dir = tempdir().unwrap();
        for name in ["A.JPG", "B.PNG", "C.JPEG"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let result =
            walk_files_multi(dir.path(), &["jpg", "jpeg", "png"]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn walk_files_multi_returns_empty_for_missing_directory() {
        let dir = tempdir().unwrap();
        let result =
            walk_files_multi(&dir.path().join("missing"), &["jpg"]).unwrap();
        assert!(result.is_empty());
    }

    // -------------------------------------------------------------------
    // walk_files_bounded_depth
    // -------------------------------------------------------------------

    #[test]
    fn walk_files_bounded_depth_respects_max_depth() {
        let dir = tempdir().unwrap();
        let mut current = dir.path().to_path_buf();
        for i in 0..5 {
            current = current.join(format!("d{i}"));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("p.md"), "").unwrap();
        }
        // max_depth=2 → only files at depths 0..=2 should be returned.
        let result = walk_files_bounded_depth(dir.path(), "md", 2).unwrap();
        assert!(result.len() <= 3);
    }

    #[test]
    fn walk_files_bounded_depth_returns_empty_for_missing_directory() {
        let dir = tempdir().unwrap();
        let result =
            walk_files_bounded_depth(&dir.path().join("missing"), "md", 8)
                .unwrap();
        assert!(result.is_empty());
    }

    // -------------------------------------------------------------------
    // walk_files_bounded_count
    // -------------------------------------------------------------------

    #[test]
    fn walk_files_bounded_count_respects_max_files() {
        let dir = tempdir().unwrap();
        for i in 0..10 {
            fs::write(dir.path().join(format!("f{i}.html")), "").unwrap();
        }
        let result = walk_files_bounded_count(dir.path(), "html", 5).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn walk_files_bounded_count_returns_empty_for_missing_directory() {
        let dir = tempdir().unwrap();
        let result =
            walk_files_bounded_count(&dir.path().join("missing"), "html", 100)
                .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn walk_files_bounded_count_outer_loop_breaks_on_saturation() {
        // Files spread across two subdirectories so the outer-loop
        // saturation `break` fires (not the inner one).
        let dir = tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        for i in 0..3 {
            fs::write(a.join(format!("f{i}.html")), "").unwrap();
            fs::write(b.join(format!("f{i}.html")), "").unwrap();
        }
        let result = walk_files_bounded_count(dir.path(), "html", 2).unwrap();
        assert!(result.len() <= 4);
    }
}
