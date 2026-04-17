// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Streaming compilation for large sites.
//!
//! Processes content files in batches to cap peak memory usage, enabling
//! compilation of 100K+ page sites within a configurable memory budget.
//!
//! The streaming compiler divides content files into chunks based on the
//! memory budget, compiles each chunk, then releases it before processing
//! the next. After all chunks, a merge pass unifies cross-page artefacts
//! (sitemap, search index, feeds).

use crate::walk;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Default peak memory budget: 512 MB.
pub const DEFAULT_MEMORY_BUDGET_MB: usize = 512;

/// Estimated memory per page in bytes (HTML + metadata + buffers).
/// Conservative estimate for batch sizing.
const ESTIMATED_BYTES_PER_PAGE: usize = 64 * 1024; // 64 KB

/// Memory budget configuration for streaming compilation.
#[derive(Debug, Clone, Copy)]
pub struct MemoryBudget {
    /// Maximum memory in bytes.
    pub max_bytes: usize,
    /// Pages per batch, derived from `max_bytes`.
    pub batch_size: usize,
}

impl MemoryBudget {
    /// Creates a memory budget from a megabyte limit.
    #[must_use]
    pub fn from_mb(mb: usize) -> Self {
        let max_bytes = mb * 1024 * 1024;
        let batch_size = (max_bytes / ESTIMATED_BYTES_PER_PAGE).max(10);
        Self {
            max_bytes,
            batch_size,
        }
    }

    /// Creates the default 512 MB budget.
    #[must_use]
    pub fn default_budget() -> Self {
        Self::from_mb(DEFAULT_MEMORY_BUDGET_MB)
    }
}

/// Collects content files and returns them as batches.
///
/// Each batch contains at most `budget.batch_size` files.
pub fn batched_content_files(
    content_dir: &Path,
    budget: &MemoryBudget,
) -> Result<Vec<Vec<PathBuf>>> {
    let all_files = walk::walk_files(content_dir, "md")
        .with_context(|| format!("cannot walk {}", content_dir.display()))?;

    if all_files.is_empty() {
        return Ok(vec![]);
    }

    let batches: Vec<Vec<PathBuf>> = all_files
        .chunks(budget.batch_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    log::info!(
        "[streaming] {} file(s) in {} batch(es) (budget: {} MB, {} pages/batch)",
        all_files.len(),
        batches.len(),
        budget.max_bytes / (1024 * 1024),
        budget.batch_size,
    );

    Ok(batches)
}

/// Compiles a single batch of content files into the build directory.
///
/// Creates a temporary content directory containing only the batch files,
/// runs `staticdatagen::compile` on it, then merges the output into the
/// final site directory.
pub fn compile_batch(
    batch: &[PathBuf],
    content_dir: &Path,
    build_dir: &Path,
    site_dir: &Path,
    template_dir: &Path,
    batch_idx: usize,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    // Create a temporary batch content directory
    let batch_content = build_dir.join(format!(".batch-{batch_idx}"));
    fs::create_dir_all(&batch_content)?;

    // Copy batch files preserving directory structure
    for file in batch {
        let rel = file.strip_prefix(content_dir).unwrap_or(file);
        let dest = batch_content.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let _ = fs::copy(file, &dest)?;
    }

    // Compile the batch
    let batch_build = build_dir.join(format!(".batch-{batch_idx}-build"));
    let batch_site = build_dir.join(format!(".batch-{batch_idx}-site"));
    fs::create_dir_all(&batch_build)?;
    fs::create_dir_all(&batch_site)?;

    let compile_result = staticdatagen::compile(
        &batch_build,
        &batch_content,
        &batch_site,
        template_dir,
    );

    // Merge batch output into the main site directory
    if compile_result.is_ok() {
        fs::create_dir_all(site_dir)?;
        merge_dir(&batch_site, site_dir)?;
    }

    // Clean up batch temporaries
    let _ = fs::remove_dir_all(&batch_content);
    let _ = fs::remove_dir_all(&batch_build);
    let _ = fs::remove_dir_all(&batch_site);

    compile_result.map_err(|e| anyhow::anyhow!("batch {batch_idx}: {e:?}"))
}

/// Recursively merges files from `src` into `dst`, overwriting on conflict.
fn merge_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest)?;
            merge_dir(&path, &dest)?;
        } else {
            let _ = fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// Determines whether streaming compilation should be used.
///
/// Returns `true` if the content directory has more files than a single
/// batch can hold, or if `--max-memory` was explicitly set.
#[must_use]
pub fn should_stream(
    content_dir: &Path,
    budget: &MemoryBudget,
    explicitly_set: bool,
) -> bool {
    if explicitly_set {
        return true;
    }

    let count = walk::walk_files(content_dir, "md")
        .map(|f| f.len())
        .unwrap_or(0);

    count > budget.batch_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn memory_budget_from_mb() {
        let budget = MemoryBudget::from_mb(256);
        assert_eq!(budget.max_bytes, 256 * 1024 * 1024);
        assert!(budget.batch_size > 0);
    }

    #[test]
    fn memory_budget_default() {
        let budget = MemoryBudget::default_budget();
        assert_eq!(budget.max_bytes, 512 * 1024 * 1024);
    }

    #[test]
    fn memory_budget_minimum_batch_size() {
        let budget = MemoryBudget::from_mb(0);
        assert!(
            budget.batch_size >= 10,
            "batch size should have a floor of 10"
        );
    }

    #[test]
    fn batched_content_files_empty_dir() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        let budget = MemoryBudget::from_mb(512);
        let batches = batched_content_files(&content, &budget).unwrap();
        assert!(batches.is_empty());
    }

    #[test]
    fn batched_content_files_splits_correctly() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        for i in 0..25 {
            fs::write(
                content.join(format!("page{i}.md")),
                format!("# Page {i}"),
            )
            .unwrap();
        }

        let budget = MemoryBudget {
            max_bytes: 0,
            batch_size: 10,
        };
        let batches = batched_content_files(&content, &budget).unwrap();

        assert_eq!(batches.len(), 3); // 10 + 10 + 5
        assert_eq!(batches[0].len(), 10);
        assert_eq!(batches[1].len(), 10);
        assert_eq!(batches[2].len(), 5);
    }

    #[test]
    fn merge_dir_combines_files() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();

        fs::write(src.join("a.html"), "from src").unwrap();
        fs::write(dst.join("b.html"), "existing").unwrap();

        merge_dir(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.html")).unwrap(), "from src");
        assert_eq!(fs::read_to_string(dst.join("b.html")).unwrap(), "existing");
    }

    #[test]
    fn merge_dir_overwrites_on_conflict() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();

        fs::write(src.join("a.html"), "new").unwrap();
        fs::write(dst.join("a.html"), "old").unwrap();

        merge_dir(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.html")).unwrap(), "new");
    }

    #[test]
    fn should_stream_when_explicitly_set() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        let budget = MemoryBudget::default_budget();
        assert!(should_stream(&content, &budget, true));
    }

    #[test]
    fn should_not_stream_small_site() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(content.join("index.md"), "# Home").unwrap();

        let budget = MemoryBudget::default_budget();
        assert!(!should_stream(&content, &budget, false));
    }
}
