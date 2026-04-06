// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Draft filtering plugin.
//!
//! Removes content files with `draft: true` in their frontmatter
//! before compilation, unless the `--drafts` flag is set.

use crate::plugin::{Plugin, PluginContext};
use crate::MAX_DIR_DEPTH;
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Plugin that filters draft content before compilation.
///
/// In `before_compile`, scans content files for `draft: true` in
/// frontmatter and renames them to `.md.draft` so staticdatagen
/// skips them. In `after_compile`, restores the originals.
#[derive(Debug, Clone, Copy)]
pub struct DraftPlugin {
    include_drafts: bool,
}

impl DraftPlugin {
    /// Creates a new `DraftPlugin`.
    ///
    /// If `include_drafts` is true, draft files are left in place.
    #[must_use]
    pub const fn new(include_drafts: bool) -> Self {
        Self { include_drafts }
    }
}

impl Plugin for DraftPlugin {
    fn name(&self) -> &'static str {
        "drafts"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        if self.include_drafts || !ctx.content_dir.exists() {
            return Ok(());
        }

        let md_files = collect_md_files(&ctx.content_dir)?;
        let mut hidden = 0usize;

        for path in &md_files {
            if is_draft(path)? {
                let draft_path = path.with_extension("md.draft");
                fs::rename(path, &draft_path)?;
                hidden += 1;
            }
        }

        if hidden > 0 {
            log::info!(
                "[drafts] Hidden {hidden} draft file(s) (use --drafts to include)"
            );
        }
        Ok(())
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if self.include_drafts || !ctx.content_dir.exists() {
            return Ok(());
        }

        // Restore hidden drafts
        let draft_files = collect_draft_files(&ctx.content_dir)?;
        for draft_path in &draft_files {
            let original = draft_path.with_extension("");
            if !original.exists() {
                fs::rename(draft_path, &original)?;
            }
        }
        Ok(())
    }
}

/// Checks if a Markdown file has `draft: true` in its frontmatter.
fn is_draft(path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;

    // Quick check: look for draft field in YAML frontmatter
    if !content.starts_with("---") {
        return Ok(false);
    }

    // Find the closing ---
    if let Some(end) = content[3..].find("---") {
        let frontmatter = &content[3..3 + end];
        // Check for draft: true (handles various YAML formats)
        for line in frontmatter.lines() {
            let trimmed = line.trim();
            if trimmed == "draft: true"
                || trimmed == "draft: True"
                || trimmed == "draft: TRUE"
                || trimmed == "draft: yes"
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DIR_DEPTH || !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if path.extension().is_some_and(|e| e == "md") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn collect_draft_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DIR_DEPTH || !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if path.extension().is_some_and(|e| e == "draft") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_draft_true() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("post.md");
        fs::write(&path, "---\ntitle: Draft\ndraft: true\n---\n# Content\n")
            .unwrap();
        assert!(is_draft(&path).unwrap());
    }

    #[test]
    fn test_is_draft_false() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("post.md");
        fs::write(&path, "---\ntitle: Published\n---\n# Content\n").unwrap();
        assert!(!is_draft(&path).unwrap());
    }

    #[test]
    fn test_is_draft_no_frontmatter() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("plain.md");
        fs::write(&path, "# No frontmatter\n").unwrap();
        assert!(!is_draft(&path).unwrap());
    }

    #[test]
    fn test_draft_plugin_hides_and_restores() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        fs::write(
            content.join("draft.md"),
            "---\ntitle: Draft\ndraft: true\n---\nDraft content",
        )
        .unwrap();
        fs::write(
            content.join("published.md"),
            "---\ntitle: Published\n---\nPublished content",
        )
        .unwrap();

        let plugin = DraftPlugin::new(false);
        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());

        // before_compile hides drafts
        plugin.before_compile(&ctx).unwrap();
        assert!(!content.join("draft.md").exists());
        assert!(content.join("draft.md.draft").exists());
        assert!(content.join("published.md").exists());

        // after_compile restores drafts
        plugin.after_compile(&ctx).unwrap();
        assert!(content.join("draft.md").exists());
        assert!(!content.join("draft.md.draft").exists());
    }

    #[test]
    fn test_draft_plugin_include_drafts() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();

        fs::write(
            content.join("draft.md"),
            "---\ntitle: Draft\ndraft: true\n---\n",
        )
        .unwrap();

        let plugin = DraftPlugin::new(true);
        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());

        plugin.before_compile(&ctx).unwrap();
        // Draft should still be there
        assert!(content.join("draft.md").exists());
    }
}
