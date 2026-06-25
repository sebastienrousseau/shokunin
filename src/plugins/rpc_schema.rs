// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Edge RPC schema emitter (issue #548 AC1 + AC4).
//!
//! Walks the `ssg_rpc` dispatch inventory at build time and writes a
//! TypeScript declaration file at `dist/.ssg/rpc.d.ts`. The matching
//! JS client (`web/rpc.js`) consumes that file via a TS users
//! import.
//!
//! ## Wiring
//!
//! The plugin is **registered unconditionally** in `Plugins::build`,
//! but it is a no-op when zero `#[ssg_rpc]`-annotated functions are
//! reachable from the binary (which is the case for `ssg` itself —
//! users add the macro in their own crates that pull `ssg-rpc`).
//!
//! ## Why a plugin and not a `build.rs`?
//!
//! Because the inventory only contains functions that are reachable
//! from the **final binary**. `build.rs` runs at compile time when
//! the user's RPCs are not yet linked. The plugin runs at the same
//! lifecycle stage as the ISR manifest emitter, which means the
//! `dist/.ssg/` directory it writes into is already created.

use std::fs;
use std::path::Path;

use crate::error::SsgError;
use crate::plugin::{Plugin, PluginContext};

/// Relative path inside `dist/` where the schema is written.
pub const RPC_DTS_RELATIVE_PATH: &str = ".ssg/rpc.d.ts";

/// `after_compile` plugin that emits `dist/.ssg/rpc.d.ts`.
///
/// Skips silently when the dispatch inventory is empty so the
/// behaviour is byte-identical to v0.0.43 for users who don't opt
/// into Edge RPC.
#[derive(Debug, Clone, Copy, Default)]
pub struct RpcSchemaPlugin;

impl RpcSchemaPlugin {
    /// Constructs a new instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for RpcSchemaPlugin {
    fn name(&self) -> &'static str {
        "rpc-schema"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if ctx.dry_run {
            return Ok(());
        }

        // No RPCs registered — nothing to emit. Stay silent so
        // `ssg build` output looks identical for non-RPC sites.
        if ssg_rpc::dispatch::iter_descriptors().next().is_none() {
            return Ok(());
        }

        let opts = ssg_rpc::EmitOptions::default();
        let ts = ssg_rpc::emit_typescript(&opts);

        let out_path = ctx.site_dir.join(RPC_DTS_RELATIVE_PATH);
        ensure_parent(&out_path)?;
        fs::write(&out_path, ts).map_err(|e| SsgError::Io {
            path: out_path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

fn ensure_parent(path: &Path) -> Result<(), SsgError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| SsgError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::plugin::PluginContext;
    use tempfile::tempdir;

    fn ctx_for(site_dir: &Path) -> PluginContext {
        PluginContext {
            content_dir: site_dir.to_path_buf(),
            build_dir: site_dir.to_path_buf(),
            site_dir: site_dir.to_path_buf(),
            template_dir: site_dir.to_path_buf(),
            config: None,
            cache: None,
            memory_budget: None,
            html_files: None,
            dep_graph: None,
            dry_run: false,
        }
    }

    #[test]
    fn plugin_name_is_stable() {
        assert_eq!(RpcSchemaPlugin::new().name(), "rpc-schema");
    }

    #[test]
    fn dry_run_short_circuits() {
        let dir = tempdir().unwrap();
        let mut ctx = ctx_for(dir.path());
        ctx.dry_run = true;
        RpcSchemaPlugin::new().after_compile(&ctx).unwrap();
        // No file written.
        assert!(!dir.path().join(RPC_DTS_RELATIVE_PATH).exists());
    }

    #[test]
    fn empty_inventory_is_a_noop_or_writes_valid_header() {
        // The ssg binary itself doesn't register any #[ssg_rpc]
        // functions in lib unit-test context, so on its own this
        // should not produce a file. We assert behavioural
        // neutrality: either no file, or a valid header-bearing
        // file (when integration tests in the same binary linked
        // some).
        let dir = tempdir().unwrap();
        let ctx = ctx_for(dir.path());
        RpcSchemaPlugin::new().after_compile(&ctx).unwrap();
        let path = dir.path().join(RPC_DTS_RELATIVE_PATH);
        if path.exists() {
            let txt = fs::read_to_string(&path).unwrap();
            assert!(
                txt.contains("AUTO-GENERATED"),
                "emitted file should carry the header: {txt}"
            );
        }
    }
}
