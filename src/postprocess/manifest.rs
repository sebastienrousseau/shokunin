// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Manifest fix plugin.

use super::helpers::{read_meta_sidecars, truncate_at_word};
use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;

/// Fixes manifest.json description truncation by using full text or
/// word-boundary-safe truncation at 200 characters.
#[derive(Debug, Clone, Copy)]
pub struct ManifestFixPlugin;

impl Plugin for ManifestFixPlugin {
    fn name(&self) -> &'static str {
        "manifest-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        let manifest_path = ctx.site_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&manifest_path).with_context(|| {
                format!("cannot read {}", manifest_path.display())
            })?;

        let mut manifest: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| {
                format!("invalid JSON in {}", manifest_path.display())
            })?;

        let meta_entries =
            read_meta_sidecars(&ctx.site_dir).unwrap_or_default();

        let full_description = find_full_description(&meta_entries);

        if let Some(desc) = full_description {
            let truncated = truncate_at_word(&desc, 200);
            manifest["description"] = serde_json::Value::String(truncated);
        } else if let Some(current) =
            manifest.get("description").and_then(|v| v.as_str())
        {
            if let Some(fixed) = fix_truncated_description(current) {
                manifest["description"] = serde_json::Value::String(fixed);
            }
        }

        let output = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, output).with_context(|| {
            format!("cannot write {}", manifest_path.display())
        })?;

        log::info!("[manifest-fix] Fixed manifest.json description");
        Ok(())
    }
}

/// Finds the full description from meta sidecars, preferring the root page.
fn find_full_description(
    meta_entries: &[(String, std::collections::HashMap<String, String>)],
) -> Option<String> {
    meta_entries
        .iter()
        .find(|(rel, _)| rel.is_empty() || rel == ".")
        .and_then(|(_, meta)| meta.get("description"))
        .or_else(|| {
            meta_entries
                .iter()
                .find_map(|(_, meta)| meta.get("description"))
        })
        .cloned()
}

/// Fixes a truncated description by ensuring it ends at a word boundary.
/// Returns `None` if the description already ends with proper punctuation.
fn fix_truncated_description(current: &str) -> Option<String> {
    if current.ends_with('.')
        || current.ends_with('!')
        || current.ends_with('?')
        || current.ends_with("...")
    {
        return None;
    }
    Some(if let Some(last_space) = current.rfind(' ') {
        format!("{}...", &current[..last_space])
    } else {
        format!("{current}...")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx(site_dir: &Path) -> PluginContext {
        crate::test_support::init_logger();
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
        )
    }

    #[test]
    fn test_manifest_fix_repairs_truncated_description() -> Result<()> {
        let tmp = tempdir()?;
        let manifest_path = tmp.path().join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{"name":"Test","description":"A new paper suggests Shor's algorithm could run on as few as 10,000 qubits. The threshold for cryptographically relevant"}"#,
        )?;

        let ctx = test_ctx(tmp.path());
        ManifestFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&result)?;
        let desc = manifest["description"].as_str().unwrap();
        assert!(
            desc.ends_with("...") || desc.ends_with('.') || desc.ends_with('!'),
            "Description should end cleanly, got: {desc}"
        );
        Ok(())
    }
}
