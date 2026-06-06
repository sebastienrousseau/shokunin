// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Manifest fix plugin.

use super::helpers::{read_meta_sidecars, truncate_at_word};
use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::fs;

/// Fixes manifest.json description truncation by using full text or
/// word-boundary-safe truncation at 200 characters.
#[derive(Debug, Clone, Copy)]
pub struct ManifestFixPlugin;

impl Plugin for ManifestFixPlugin {
    fn name(&self) -> &'static str {
        "manifest-fix"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        let manifest_path = ctx.site_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&manifest_path).with_path(&manifest_path)?;

        let mut manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SsgError::io(e, &manifest_path))?;

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

        // Drop icon entries whose `src` is empty; Chrome logs
        // "Error while trying to use the following icon from the Manifest"
        // when it tries to fetch them.
        drop_empty_icons(&mut manifest);

        let output = serde_json::to_string_pretty(&manifest)
            .map_err(|e| SsgError::io(e, &manifest_path))?;
        fs::write(&manifest_path, output).with_path(&manifest_path)?;

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

/// Removes any entry from the manifest's `icons` array whose `src` is
/// missing or empty. Chrome logs a manifest icon download error for each
/// such entry, even though the manifest itself is otherwise valid.
fn drop_empty_icons(manifest: &mut serde_json::Value) {
    let Some(icons) = manifest.get_mut("icons").and_then(|v| v.as_array_mut())
    else {
        return;
    };
    icons.retain(|icon| {
        icon.get("src")
            .and_then(|s| s.as_str())
            .is_some_and(|s| !s.is_empty())
    });
    if icons.is_empty() {
        // An empty array is preferable to `[{src:""}]` — but if there are
        // truly no usable icons, drop the key entirely so the manifest
        // doesn't advertise an empty icon set.
        if let Some(map) = manifest.as_object_mut() {
            let _ = map.remove("icons");
        }
    }
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use anyhow::Result;
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
    fn test_drop_empty_icons_removes_empty_src() {
        let mut m: serde_json::Value = serde_json::from_str(
            r#"{"icons":[{"src":"","sizes":"512x512"},{"src":"/icon.svg","sizes":"512x512"}]}"#,
        )
        .unwrap();
        drop_empty_icons(&mut m);
        let icons = m["icons"].as_array().unwrap();
        assert_eq!(icons.len(), 1);
        assert_eq!(icons[0]["src"], "/icon.svg");
    }

    #[test]
    fn test_drop_empty_icons_removes_key_when_all_empty() {
        let mut m: serde_json::Value =
            serde_json::from_str(r#"{"name":"x","icons":[{"src":""}]}"#)
                .unwrap();
        drop_empty_icons(&mut m);
        assert!(m.get("icons").is_none(), "icons key should be dropped");
    }

    #[test]
    fn name_is_stable() {
        assert_eq!(ManifestFixPlugin.name(), "manifest-fix");
    }

    #[test]
    fn after_compile_no_op_when_manifest_missing() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        ManifestFixPlugin.after_compile(&ctx)?;
        assert!(!tmp.path().join("manifest.json").exists());
        Ok(())
    }

    #[test]
    fn after_compile_returns_error_on_invalid_json() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("manifest.json"), "not valid json").unwrap();
        let ctx = test_ctx(tmp.path());
        let err = ManifestFixPlugin.after_compile(&ctx).unwrap_err();
        assert!(
            err.to_string().contains("invalid JSON")
                || err.to_string().contains("manifest"),
            "expected JSON parse error, got: {err}"
        );
    }

    #[test]
    fn drop_empty_icons_keeps_array_with_real_entries() {
        let mut m: serde_json::Value = serde_json::from_str(
            r#"{"icons":[{"src":"/a.svg"},{"src":"/b.svg"}]}"#,
        )
        .unwrap();
        drop_empty_icons(&mut m);
        let icons = m["icons"].as_array().unwrap();
        assert_eq!(icons.len(), 2);
    }

    #[test]
    fn drop_empty_icons_no_op_when_no_icons_key() {
        let mut m: serde_json::Value =
            serde_json::from_str(r#"{"name":"x"}"#).unwrap();
        drop_empty_icons(&mut m);
        assert!(m.get("icons").is_none());
        assert_eq!(m["name"], "x");
    }

    #[test]
    fn drop_empty_icons_no_op_when_icons_not_array() {
        // Defensive: malformed manifest with non-array icons.
        let mut m: serde_json::Value =
            serde_json::from_str(r#"{"icons":"not an array"}"#).unwrap();
        drop_empty_icons(&mut m);
        assert_eq!(m["icons"], "not an array");
    }

    #[test]
    fn fix_truncated_description_returns_none_when_already_terminated() {
        assert!(fix_truncated_description("ends with period.").is_none());
        assert!(fix_truncated_description("ends with bang!").is_none());
        assert!(fix_truncated_description("ends with question?").is_none());
        assert!(fix_truncated_description("ends with ellipsis...").is_none());
    }

    #[test]
    fn fix_truncated_description_truncates_at_word_boundary() {
        let out =
            fix_truncated_description("a long description without ending");
        assert_eq!(out.as_deref(), Some("a long description without..."));
    }

    #[test]
    fn fix_truncated_description_no_space_appends_ellipsis() {
        // Edge case: a single very long word without spaces.
        let out = fix_truncated_description("supercalifragilistic");
        assert_eq!(out.as_deref(), Some("supercalifragilistic..."));
    }

    #[test]
    fn after_compile_drops_empty_icons_in_manifest() -> Result<()> {
        let tmp = tempdir()?;
        let manifest_path = tmp.path().join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{"name":"X","description":"Already terminated.","icons":[{"src":""}]}"#,
        )?;
        let ctx = test_ctx(tmp.path());
        ManifestFixPlugin.after_compile(&ctx)?;
        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
        assert!(after.get("icons").is_none(), "empty icon should be dropped");
        Ok(())
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

    #[test]
    fn test_manifest_fix_uses_sidecar_description() -> Result<()> {
        let tmp = tempdir()?;
        let manifest_path = tmp.path().join("manifest.json");
        fs::write(
            &manifest_path,
            r#"{"name":"Test","description":"Short description"}"#,
        )?;
        fs::write(
            tmp.path().join("index.meta.json"),
            r#"{"description":"This is a very long description that we are using to test manifest metadata sidecar description truncation logic in the manifest fix plugin. We need to make sure that the total length of this text exceeds two hundred characters so that the truncation is triggered."}"#,
        )?;

        let ctx = test_ctx(tmp.path());
        ManifestFixPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&result)?;
        let desc = manifest["description"].as_str().unwrap();
        assert!(desc.starts_with("This is a very long"));
        assert!(desc.ends_with("..."));
        Ok(())
    }

    #[test]
    fn test_find_full_description_fallback() {
        let mut entries = Vec::new();
        let mut meta1 = std::collections::HashMap::new();
        let _ = meta1.insert("title".to_string(), "No description".to_string());
        entries.push(("root".to_string(), meta1));

        let mut meta2 = std::collections::HashMap::new();
        let _ = meta2
            .insert("description".to_string(), "Fallback desc".to_string());
        entries.push(("subpage".to_string(), meta2));

        let desc = find_full_description(&entries);
        assert_eq!(desc.as_deref(), Some("Fallback desc"));
    }

    #[test]
    fn test_find_full_description_none() {
        let mut entries = Vec::new();
        let mut meta1 = std::collections::HashMap::new();
        let _ = meta1.insert("title".to_string(), "No description".to_string());
        entries.push(("root".to_string(), meta1));

        let desc = find_full_description(&entries);
        assert!(desc.is_none());
    }
}
