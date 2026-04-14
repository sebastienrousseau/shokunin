// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Plugin architecture for SSG
//!
//! Provides a trait-based plugin system with lifecycle hooks for
//! extending the static site generation pipeline.
//!
//! ## Lifecycle hooks
//!
//! Plugins can hook into three stages of site generation:
//!
//! 1. **`before_compile`** — Runs before compilation. Use for content
//!    preprocessing, metadata injection, or source transformation.
//! 2. **`after_compile`** — Runs after compilation. Use for HTML
//!    post-processing, asset optimization, or sitemap generation.
//! 3. **`on_serve`** — Runs before the dev server starts. Use for
//!    injecting dev-mode scripts or live-reload support.
//!
//! ## Example
//!
//! ```rust
//! use ssg::plugin::{Plugin, PluginContext};
//! use anyhow::Result;
//!
//! #[derive(Debug)]
//! struct MinifyPlugin;
//!
//! impl Plugin for MinifyPlugin {
//!     fn name(&self) -> &str { "minify" }
//!
//!     fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
//!         println!("Minifying files in {:?}", ctx.site_dir);
//!         // Walk site_dir and minify HTML/CSS/JS files
//!         Ok(())
//!     }
//! }
//! ```

use crate::cmd::SsgConfig;
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
};

// =====================================================================
// Content-addressed plugin cache
// =====================================================================

const CACHE_FILENAME: &str = ".ssg-plugin-cache.json";

/// Content-addressed cache that tracks file hashes so plugins can skip
/// unchanged files across incremental builds.
///
/// Stores `path → content_hash` mappings and persists to
/// `.ssg-plugin-cache.json` in the site directory.
#[derive(Debug, Clone, Default)]
pub struct PluginCache {
    entries: HashMap<PathBuf, u64>,
}

impl PluginCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Loads a cache from `site_dir/.ssg-plugin-cache.json`.
    ///
    /// Returns an empty cache if the file does not exist or cannot be
    /// parsed.
    #[must_use]
    pub fn load(site_dir: &Path) -> Self {
        let path = site_dir.join(CACHE_FILENAME);
        if !path.exists() {
            return Self::new();
        }
        let Ok(content) = fs::read_to_string(&path) else {
            return Self::new();
        };
        let Ok(map) = serde_json::from_str::<HashMap<String, u64>>(&content)
        else {
            return Self::new();
        };
        Self {
            entries: map
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v))
                .collect(),
        }
    }

    /// Persists the cache to `site_dir/.ssg-plugin-cache.json`.
    pub fn save(&self, site_dir: &Path) -> Result<()> {
        let path = site_dir.join(CACHE_FILENAME);
        let serialisable: HashMap<String, u64> = self
            .entries
            .iter()
            .map(|(k, v)| (k.to_string_lossy().into_owned(), *v))
            .collect();
        let json = serde_json::to_string_pretty(&serialisable)
            .context("failed to serialise plugin cache")?;
        fs::write(&path, json)
            .with_context(|| format!("cannot write {}", path.display()))?;
        Ok(())
    }

    /// Returns `true` if the file at `path` has changed since the last
    /// time it was recorded, or if it has never been recorded.
    pub fn has_changed(&self, path: &Path) -> bool {
        let Ok(content) = fs::read(path) else {
            return true;
        };
        let current = Self::hash_bytes(&content);
        match self.entries.get(path) {
            Some(&cached) => cached != current,
            None => true,
        }
    }

    /// Records the current content hash for `path`.
    pub fn update(&mut self, path: &Path) {
        if let Ok(content) = fs::read(path) {
            let hash = Self::hash_bytes(&content);
            let _ = self.entries.insert(path.to_path_buf(), hash);
        }
    }

    /// Simple FNV-1a 64-bit hash of a byte slice.
    fn hash_bytes(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &byte in data {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash
    }
}

/// Context passed to plugin hooks with paths and configuration.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// The content source directory.
    pub content_dir: PathBuf,
    /// The build/output directory.
    pub build_dir: PathBuf,
    /// The final site directory.
    pub site_dir: PathBuf,
    /// The template directory.
    pub template_dir: PathBuf,
    /// Site configuration (`base_url`, `site_name`, language, etc.).
    pub config: Option<SsgConfig>,
    /// Content-addressed plugin cache for incremental builds.
    pub cache: Option<PluginCache>,
}

impl PluginContext {
    /// Creates a new plugin context from directory paths.
    #[must_use]
    pub fn new(
        content_dir: &Path,
        build_dir: &Path,
        site_dir: &Path,
        template_dir: &Path,
    ) -> Self {
        Self {
            content_dir: content_dir.to_path_buf(),
            build_dir: build_dir.to_path_buf(),
            site_dir: site_dir.to_path_buf(),
            template_dir: template_dir.to_path_buf(),
            config: None,
            cache: None,
        }
    }

    /// Creates a new plugin context with site configuration.
    #[must_use]
    pub fn with_config(
        content_dir: &Path,
        build_dir: &Path,
        site_dir: &Path,
        template_dir: &Path,
        config: SsgConfig,
    ) -> Self {
        Self {
            content_dir: content_dir.to_path_buf(),
            build_dir: build_dir.to_path_buf(),
            site_dir: site_dir.to_path_buf(),
            template_dir: template_dir.to_path_buf(),
            config: Some(config),
            cache: None,
        }
    }
}

/// Trait for SSG plugins.
///
/// Implement this trait to create a plugin that hooks into the site
/// generation lifecycle. All hooks have default no-op implementations,
/// so you only need to override the ones you care about.
pub trait Plugin: fmt::Debug + Send + Sync {
    /// Returns the unique name of this plugin.
    fn name(&self) -> &str;

    /// Called before site compilation begins.
    ///
    /// Use this hook to preprocess content files, inject metadata,
    /// or validate source directories.
    fn before_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Called after site compilation completes.
    ///
    /// Use this hook to post-process generated HTML, optimize assets,
    /// generate sitemaps, or perform any output transformation.
    fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Called before the development server starts serving files.
    ///
    /// Use this hook to inject dev-mode scripts, set up live-reload,
    /// or modify the serve directory.
    fn on_serve(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}

/// Manages registered plugins and executes lifecycle hooks.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::{PluginManager, PluginContext, Plugin};
/// use anyhow::Result;
/// use std::path::Path;
///
/// #[derive(Debug)]
/// struct LogPlugin;
///
/// impl Plugin for LogPlugin {
///     fn name(&self) -> &str { "logger" }
///     fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
///         println!("Compiling from {:?}", ctx.content_dir);
///         Ok(())
///     }
/// }
///
/// let mut pm = PluginManager::new();
/// pm.register(LogPlugin);
/// assert_eq!(pm.len(), 1);
///
/// let ctx = PluginContext::new(
///     Path::new("content"),
///     Path::new("build"),
///     Path::new("public"),
///     Path::new("templates"),
/// );
/// pm.run_before_compile(&ctx).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    /// Creates a new empty plugin manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Registers a plugin.
    ///
    /// Plugins run in the order they are registered.
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// Returns the number of registered plugins.
    #[must_use]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns `true` if no plugins are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Returns the names of all registered plugins.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Runs the `before_compile` hook on all registered plugins.
    ///
    /// Plugins execute in registration order. If any plugin returns
    /// an error, execution stops and the error is propagated.
    pub fn run_before_compile(&self, ctx: &PluginContext) -> Result<()> {
        for plugin in &self.plugins {
            plugin.before_compile(ctx).map_err(|e| {
                anyhow::anyhow!(
                    "Plugin '{}' failed in before_compile: {}",
                    plugin.name(),
                    e
                )
            })?;
        }
        Ok(())
    }

    /// Runs the `after_compile` hook on all registered plugins.
    ///
    /// Plugins execute in registration order. If any plugin returns
    /// an error, execution stops and the error is propagated.
    pub fn run_after_compile(&self, ctx: &PluginContext) -> Result<()> {
        for plugin in &self.plugins {
            plugin.after_compile(ctx).map_err(|e| {
                anyhow::anyhow!(
                    "Plugin '{}' failed in after_compile: {}",
                    plugin.name(),
                    e
                )
            })?;
        }
        Ok(())
    }

    /// Runs the `on_serve` hook on all registered plugins.
    ///
    /// Plugins execute in registration order. If any plugin returns
    /// an error, execution stops and the error is propagated.
    pub fn run_on_serve(&self, ctx: &PluginContext) -> Result<()> {
        for plugin in &self.plugins {
            plugin.on_serve(ctx).map_err(|e| {
                anyhow::anyhow!(
                    "Plugin '{}' failed in on_serve: {}",
                    plugin.name(),
                    e
                )
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct CounterPlugin {
        name: &'static str,
        before: &'static AtomicUsize,
        after: &'static AtomicUsize,
        serve: &'static AtomicUsize,
    }

    impl Plugin for CounterPlugin {
        fn name(&self) -> &str {
            self.name
        }
        fn before_compile(&self, _ctx: &PluginContext) -> Result<()> {
            let _ = self.before.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
            let _ = self.after.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn on_serve(&self, _ctx: &PluginContext) -> Result<()> {
            let _ = self.serve.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FailPlugin {
        hook: &'static str,
    }

    impl Plugin for FailPlugin {
        fn name(&self) -> &'static str {
            "fail-plugin"
        }
        fn before_compile(&self, _ctx: &PluginContext) -> Result<()> {
            if self.hook == "before" {
                anyhow::bail!("before_compile failed");
            }
            Ok(())
        }
        fn after_compile(&self, _ctx: &PluginContext) -> Result<()> {
            if self.hook == "after" {
                anyhow::bail!("after_compile failed");
            }
            Ok(())
        }
        fn on_serve(&self, _ctx: &PluginContext) -> Result<()> {
            if self.hook == "serve" {
                anyhow::bail!("on_serve failed");
            }
            Ok(())
        }
    }

    #[derive(Debug)]
    struct NoopPlugin;

    impl Plugin for NoopPlugin {
        fn name(&self) -> &'static str {
            "noop"
        }
    }

    fn test_ctx() -> PluginContext {
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            Path::new("public"),
            Path::new("templates"),
        )
    }

    #[test]
    fn test_plugin_manager_new_is_empty() {
        let pm = PluginManager::new();
        assert!(pm.is_empty());
        assert_eq!(pm.len(), 0);
        assert!(pm.names().is_empty());
    }

    #[test]
    fn test_plugin_manager_default() {
        let pm = PluginManager::default();
        assert!(pm.is_empty());
    }

    #[test]
    fn test_register_and_count() {
        let mut pm = PluginManager::new();
        pm.register(NoopPlugin);
        assert_eq!(pm.len(), 1);
        assert!(!pm.is_empty());
        assert_eq!(pm.names(), vec!["noop"]);
    }

    #[test]
    fn test_multiple_plugins_run_in_order() {
        static BEFORE_A: AtomicUsize = AtomicUsize::new(0);
        static AFTER_A: AtomicUsize = AtomicUsize::new(0);
        static SERVE_A: AtomicUsize = AtomicUsize::new(0);
        static BEFORE_B: AtomicUsize = AtomicUsize::new(0);
        static AFTER_B: AtomicUsize = AtomicUsize::new(0);
        static SERVE_B: AtomicUsize = AtomicUsize::new(0);

        let mut pm = PluginManager::new();
        pm.register(CounterPlugin {
            name: "a",
            before: &BEFORE_A,
            after: &AFTER_A,
            serve: &SERVE_A,
        });
        pm.register(CounterPlugin {
            name: "b",
            before: &BEFORE_B,
            after: &AFTER_B,
            serve: &SERVE_B,
        });

        let ctx = test_ctx();
        pm.run_before_compile(&ctx).unwrap();
        pm.run_after_compile(&ctx).unwrap();
        pm.run_on_serve(&ctx).unwrap();

        assert_eq!(BEFORE_A.load(Ordering::SeqCst), 1);
        assert_eq!(BEFORE_B.load(Ordering::SeqCst), 1);
        assert_eq!(AFTER_A.load(Ordering::SeqCst), 1);
        assert_eq!(AFTER_B.load(Ordering::SeqCst), 1);
        assert_eq!(SERVE_A.load(Ordering::SeqCst), 1);
        assert_eq!(SERVE_B.load(Ordering::SeqCst), 1);
        assert_eq!(pm.names(), vec!["a", "b"]);
    }

    #[test]
    fn test_noop_plugin_all_hooks_succeed() {
        let mut pm = PluginManager::new();
        pm.register(NoopPlugin);
        let ctx = test_ctx();
        assert!(pm.run_before_compile(&ctx).is_ok());
        assert!(pm.run_after_compile(&ctx).is_ok());
        assert!(pm.run_on_serve(&ctx).is_ok());
    }

    #[test]
    fn test_before_compile_error_propagates() {
        let mut pm = PluginManager::new();
        pm.register(FailPlugin { hook: "before" });
        let ctx = test_ctx();
        let err = pm.run_before_compile(&ctx).unwrap_err();
        assert!(err.to_string().contains("fail-plugin"));
        assert!(err.to_string().contains("before_compile"));
    }

    #[test]
    fn test_after_compile_error_propagates() {
        let mut pm = PluginManager::new();
        pm.register(FailPlugin { hook: "after" });
        let ctx = test_ctx();
        let err = pm.run_after_compile(&ctx).unwrap_err();
        assert!(err.to_string().contains("fail-plugin"));
        assert!(err.to_string().contains("after_compile"));
    }

    #[test]
    fn test_on_serve_error_propagates() {
        let mut pm = PluginManager::new();
        pm.register(FailPlugin { hook: "serve" });
        let ctx = test_ctx();
        let err = pm.run_on_serve(&ctx).unwrap_err();
        assert!(err.to_string().contains("fail-plugin"));
        assert!(err.to_string().contains("on_serve"));
    }

    #[test]
    fn test_error_stops_subsequent_plugins() {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let mut pm = PluginManager::new();
        pm.register(FailPlugin { hook: "before" });
        pm.register(CounterPlugin {
            name: "second",
            before: &COUNTER,
            after: &COUNTER,
            serve: &COUNTER,
        });

        let ctx = test_ctx();
        assert!(pm.run_before_compile(&ctx).is_err());
        // Second plugin should not have run
        assert_eq!(COUNTER.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_empty_manager_hooks_succeed() {
        let pm = PluginManager::new();
        let ctx = test_ctx();
        assert!(pm.run_before_compile(&ctx).is_ok());
        assert!(pm.run_after_compile(&ctx).is_ok());
        assert!(pm.run_on_serve(&ctx).is_ok());
    }

    #[test]
    fn test_plugin_context_fields() {
        let ctx = PluginContext::new(
            Path::new("/a"),
            Path::new("/b"),
            Path::new("/c"),
            Path::new("/d"),
        );
        assert_eq!(ctx.content_dir, PathBuf::from("/a"));
        assert_eq!(ctx.build_dir, PathBuf::from("/b"));
        assert_eq!(ctx.site_dir, PathBuf::from("/c"));
        assert_eq!(ctx.template_dir, PathBuf::from("/d"));
    }

    #[test]
    fn test_plugin_context_clone() {
        let ctx = test_ctx();
        let cloned = ctx.clone();
        assert_eq!(ctx.content_dir, cloned.content_dir);
        assert_eq!(ctx.site_dir, cloned.site_dir);
    }

    #[test]
    fn test_plugin_context_debug() {
        let ctx = test_ctx();
        let debug = format!("{ctx:?}");
        assert!(debug.contains("content"));
        assert!(debug.contains("build"));
    }

    #[test]
    fn test_plugin_manager_debug() {
        let mut pm = PluginManager::new();
        pm.register(NoopPlugin);
        let debug = format!("{pm:?}");
        assert!(debug.contains("NoopPlugin"));
    }

    // -----------------------------------------------------------------
    // PluginCache tests
    // -----------------------------------------------------------------

    #[test]
    fn test_cache_new_is_empty() {
        let cache = PluginCache::new();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_has_changed_on_missing_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("hello.txt");
        fs::write(&file, "hello").unwrap();

        let cache = PluginCache::new();
        assert!(cache.has_changed(&file), "New file should count as changed");
    }

    #[test]
    fn test_cache_has_changed_detects_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("hello.txt");
        fs::write(&file, "hello").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&file);
        assert!(
            !cache.has_changed(&file),
            "File should not be changed after update"
        );
    }

    #[test]
    fn test_cache_has_changed_detects_modification() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("hello.txt");
        fs::write(&file, "hello").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&file);

        // Modify the file
        fs::write(&file, "world").unwrap();
        assert!(
            cache.has_changed(&file),
            "Modified file should be detected as changed"
        );
    }

    #[test]
    fn test_cache_persistence_save_load() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("data.txt");
        fs::write(&file, "content").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&file);
        cache.save(tmp.path()).unwrap();

        // Verify the cache file exists
        let cache_path = tmp.path().join(CACHE_FILENAME);
        assert!(cache_path.exists(), "Cache file should be persisted");

        // Load it back
        let loaded = PluginCache::load(tmp.path());
        assert!(
            !loaded.has_changed(&file),
            "Loaded cache should still recognise unchanged file"
        );
    }

    #[test]
    fn test_cache_load_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = PluginCache::load(tmp.path());
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_has_changed_nonexistent_file() {
        let cache = PluginCache::new();
        assert!(
            cache.has_changed(Path::new("/nonexistent/file.txt")),
            "Nonexistent file should count as changed"
        );
    }

    // -----------------------------------------------------------------
    // PluginCache: save/load round-trip, hash determinism, empty cache
    // -----------------------------------------------------------------

    #[test]
    fn test_cache_save_load_round_trip_with_multiple_files() {
        let tmp = tempfile::tempdir().unwrap();
        let f1 = tmp.path().join("one.txt");
        let f2 = tmp.path().join("two.txt");
        fs::write(&f1, "alpha").unwrap();
        fs::write(&f2, "beta").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&f1);
        cache.update(&f2);
        cache.save(tmp.path()).unwrap();

        let loaded = PluginCache::load(tmp.path());
        assert!(!loaded.has_changed(&f1));
        assert!(!loaded.has_changed(&f2));
    }

    #[test]
    fn test_cache_empty_save_load() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = PluginCache::new();
        cache.save(tmp.path()).unwrap();

        let loaded = PluginCache::load(tmp.path());
        assert!(loaded.entries.is_empty());
    }

    #[test]
    fn test_cache_hash_bytes_determinism() {
        let data = b"hello world";
        let h1 = PluginCache::hash_bytes(data);
        let h2 = PluginCache::hash_bytes(data);
        assert_eq!(h1, h2, "same input must produce same hash");
    }

    #[test]
    fn test_cache_hash_bytes_different_inputs() {
        let h1 = PluginCache::hash_bytes(b"aaa");
        let h2 = PluginCache::hash_bytes(b"bbb");
        assert_ne!(h1, h2, "different inputs should produce different hashes");
    }

    #[test]
    fn test_cache_hash_bytes_empty() {
        // Empty input should return the FNV offset basis
        let h = PluginCache::hash_bytes(b"");
        assert_eq!(h, 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn test_cache_has_changed_after_file_modification() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("data.txt");
        fs::write(&f, "version1").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&f);
        assert!(!cache.has_changed(&f));

        // Modify file content
        fs::write(&f, "version2").unwrap();
        assert!(cache.has_changed(&f));

        // Update cache, should no longer be changed
        cache.update(&f);
        assert!(!cache.has_changed(&f));
    }

    #[test]
    fn test_cache_load_corrupt_json() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_path = tmp.path().join(CACHE_FILENAME);
        fs::write(&cache_path, "this is not json").unwrap();

        let loaded = PluginCache::load(tmp.path());
        assert!(
            loaded.entries.is_empty(),
            "corrupt JSON should yield empty cache"
        );
    }

    #[test]
    fn test_cache_update_nonexistent_file_is_noop() {
        let mut cache = PluginCache::new();
        cache.update(Path::new("/nonexistent/file.txt"));
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_default_is_empty() {
        let cache = PluginCache::default();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_clone() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.txt");
        fs::write(&f, "x").unwrap();

        let mut cache = PluginCache::new();
        cache.update(&f);

        let cloned = cache.clone();
        assert!(!cloned.has_changed(&f));
    }

    #[test]
    fn test_plugin_context_with_config() {
        let config = SsgConfig::builder()
            .site_name("test".to_string())
            .base_url("https://example.com".to_string())
            .build()
            .expect("config");
        let ctx = PluginContext::with_config(
            Path::new("c"),
            Path::new("b"),
            Path::new("s"),
            Path::new("t"),
            config,
        );
        assert!(ctx.config.is_some());
        assert_eq!(ctx.config.unwrap().site_name, "test");
    }

    #[test]
    fn test_fail_plugin_non_matching_hooks_succeed() {
        let ctx = test_ctx();

        // FailPlugin("before") should succeed on after_compile and on_serve
        let p = FailPlugin { hook: "before" };
        assert!(p.after_compile(&ctx).is_ok());
        assert!(p.on_serve(&ctx).is_ok());

        // FailPlugin("after") should succeed on before_compile and on_serve
        let p = FailPlugin { hook: "after" };
        assert!(p.before_compile(&ctx).is_ok());
        assert!(p.on_serve(&ctx).is_ok());

        // FailPlugin("serve") should succeed on before_compile and after_compile
        let p = FailPlugin { hook: "serve" };
        assert!(p.before_compile(&ctx).is_ok());
        assert!(p.after_compile(&ctx).is_ok());
    }
}
