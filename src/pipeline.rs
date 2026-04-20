// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build pipeline: plugin orchestration and site compilation.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use staticdatagen::compile;

use crate::cmd::SsgConfig;
use crate::{
    accessibility, ai, assets, content, csp, deploy, drafts, highlight, i18n,
    islands, livereload, pagination, plugin, plugins as plugins_mod,
    postprocess, search, seo, shortcodes, streaming, taxonomy, walk,
};

// ---------------------------------------------------------------------------
// BuildError — serialisable build error for browser overlay delivery
// ---------------------------------------------------------------------------

/// Serialisable build error for browser overlay delivery.
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct BuildError {
    /// Source file path (if extractable from the error chain).
    pub file: Option<String>,
    /// Line number (if extractable).
    pub line: Option<usize>,
    /// Human-readable error message.
    pub message: String,
}

impl BuildError {
    /// Creates a `BuildError` from an `anyhow` error, attempting to extract
    /// file path and line number from the error chain.
    #[must_use]
    #[allow(dead_code)]
    pub fn from_anyhow(err: &anyhow::Error) -> Self {
        let message = format!("{err:#}");
        let file = extract_file_from_error(&message);
        Self {
            file,
            line: None,
            message,
        }
    }

    /// Serializes to a WebSocket JSON message.
    #[must_use]
    #[allow(dead_code)]
    pub fn to_ws_message(&self) -> String {
        serde_json::json!({
            "type": "error",
            "file": self.file,
            "line": self.line,
            "message": self.message,
        })
        .to_string()
    }
}

/// Returns the JSON message to clear the error overlay.
#[must_use]
#[allow(dead_code)]
pub fn clear_error_message() -> String {
    r#"{"type":"clear-error"}"#.to_string()
}

/// Extracts a file path from an error message by scanning for path-like
/// tokens ending in known extensions.
#[allow(dead_code)]
fn extract_file_from_error(msg: &str) -> Option<String> {
    for word in msg.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| {
            !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-'
        });
        if trimmed.contains('/')
            && (trimmed.ends_with(".md")
                || trimmed.ends_with(".html")
                || trimmed.ends_with(".toml")
                || trimmed.ends_with(".yml")
                || trimmed.ends_with(".yaml"))
        {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// CLI-driven options that don't live in `SsgConfig` itself.
///
/// Extracted from clap matches so the run pipeline can be unit-tested
/// without going through `Cli::build()`.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RunOptions {
    /// Suppress banner and timing print-outs.
    pub quiet: bool,
    /// Include draft files (skip the `DraftPlugin` filter).
    pub include_drafts: bool,
    /// Optional deploy target — `netlify`, `vercel`, `cloudflare`, `github`.
    pub deploy_target: Option<String>,
    /// Validate content schemas only (no build).
    pub validate_only: bool,
    /// Number of parallel threads for Rayon (`--jobs`).
    /// `None` means use all available CPUs.
    pub jobs: Option<usize>,
    /// Peak memory budget in MB for streaming compilation.
    /// `None` means use the default (512 MB).
    pub max_memory_mb: Option<usize>,
    /// Run the agentic AI pipeline to audit and fix content.
    #[allow(dead_code)]
    pub ai_fix: bool,
    /// Preview AI fixes without writing files.
    #[allow(dead_code)]
    pub ai_fix_dry_run: bool,
}

impl RunOptions {
    /// Builds a `RunOptions` from a parsed `clap::ArgMatches`.
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            quiet: matches.get_flag("quiet"),
            include_drafts: matches.get_flag("drafts"),
            deploy_target: matches.get_one::<String>("deploy").cloned(),
            validate_only: matches.get_flag("validate"),
            jobs: matches.get_one::<usize>("jobs").copied(),
            max_memory_mb: matches.get_one::<usize>("max-memory").copied(),
            ai_fix: matches.get_flag("ai-fix"),
            ai_fix_dry_run: matches.get_flag("ai-fix-dry-run"),
        }
    }
}

/// Resolves distinct build and site directories for compilation.
///
/// `staticdatagen::compile` finalizes output by renaming the build directory
/// into the site directory. If both paths are identical, finalization fails.
/// This helper guarantees distinct paths when needed.
pub fn resolve_build_and_site_dirs(config: &SsgConfig) -> (PathBuf, PathBuf) {
    let site_dir = config
        .serve_dir
        .clone()
        .unwrap_or_else(|| config.output_dir.clone());

    let build_dir = if site_dir == config.output_dir {
        config.output_dir.with_extension("build-tmp")
    } else {
        config.output_dir.clone()
    };

    (build_dir, site_dir)
}

/// Builds a fully-populated plugin manager and plugin context for a build.
///
/// Extracted so unit tests can construct the same wiring without
/// needing to fake CLI argument parsing.
pub fn build_pipeline(
    config: &SsgConfig,
    opts: &RunOptions,
) -> (
    plugin::PluginManager,
    plugin::PluginContext,
    PathBuf,
    PathBuf,
) {
    let (build_dir, site_dir) = resolve_build_and_site_dirs(config);

    let mut ctx = plugin::PluginContext::with_config(
        &config.content_dir,
        &build_dir,
        &site_dir,
        &config.template_dir,
        config.clone(),
    );

    // Set memory budget if --max-memory was specified
    if let Some(mb) = opts.max_memory_mb {
        ctx.memory_budget = Some(streaming::MemoryBudget::from_mb(mb));
    }

    let mut plugins = plugin::PluginManager::new();
    register_default_plugins(
        &mut plugins,
        config,
        opts.include_drafts,
        opts.deploy_target.as_deref(),
    );

    (plugins, ctx, build_dir, site_dir)
}

/// Runs the build half of the pipeline: `before_compile` → compile →
/// `after_compile`. Does not start the dev server.
///
/// Extracted from `run()` so the actual build can be unit-tested
/// against a tempdir without booting an HTTP server.
pub fn execute_build_pipeline(
    plugins: &plugin::PluginManager,
    ctx: &plugin::PluginContext,
    build_dir: &Path,
    content_dir: &Path,
    site_dir: &Path,
    template_dir: &Path,
    quiet: bool,
) -> Result<()> {
    let start = std::time::Instant::now();

    // Load plugin cache for incremental builds
    let cache = plugin::PluginCache::load(site_dir);
    let dep_graph = crate::depgraph::DepGraph::load(site_dir);
    let mut ctx = ctx.clone();
    ctx.cache = Some(cache);
    ctx.dep_graph = Some(dep_graph);

    plugins.run_before_compile(&ctx)?;

    // Use streaming compilation for large sites when --max-memory is set
    // or the site exceeds the default batch size.
    let budget = ctx
        .memory_budget
        .unwrap_or_else(streaming::MemoryBudget::default_budget);
    let explicitly_set = ctx.memory_budget.is_some();

    if streaming::should_stream(content_dir, &budget, explicitly_set) {
        let batches = streaming::batched_content_files(content_dir, &budget)?;
        for (i, batch) in batches.iter().enumerate() {
            streaming::compile_batch(
                batch,
                content_dir,
                build_dir,
                site_dir,
                template_dir,
                i,
            )?;
        }
    } else {
        compile_site(build_dir, content_dir, site_dir, template_dir)?;
    }

    // Cache HTML file list once — shared by all after_compile plugins,
    // eliminating 8+ redundant directory walks.
    ctx.cache_html_files();

    plugins.run_after_compile(&ctx)?;

    // Fused transform pass: read each HTML once → pipe through all
    // transform plugins → write once. Eliminates redundant I/O.
    plugins.run_fused_transforms(&ctx)?;

    // Rebuild and save cache: snapshot all HTML files in site_dir
    if let Some(ref mut cache) = ctx.cache {
        if let Ok(files) = walk::walk_files(site_dir, "html") {
            for file in &files {
                cache.update(file);
            }
        }
        if let Err(e) = cache.save(site_dir) {
            log::warn!("Failed to save plugin cache: {e}");
        }
    }

    // Persist the dependency graph for next incremental build
    if let Some(ref dg) = ctx.dep_graph {
        if let Err(e) = dg.save(site_dir) {
            log::warn!("Failed to save dependency graph: {e}");
        }
    }

    let elapsed = start.elapsed();
    if !quiet {
        println!(
            "Site built in {:.2}s ({} plugin(s))",
            elapsed.as_secs_f64(),
            plugins.len()
        );
    }
    Ok(())
}

/// Compiles the static site from source directories.
pub fn compile_site(
    build_dir: &Path,
    content_dir: &Path,
    site_dir: &Path,
    template_dir: &Path,
) -> Result<()> {
    compile(build_dir, content_dir, site_dir, template_dir).map_err(|e| {
        eprintln!("    Error compiling site: {e:?}");
        anyhow!("Failed to compile site: {e:?}")
    })
}

/// Registers the default plugin pipeline.
///
/// Plugins execute in registration order. The ordering is:
/// 1. SEO plugins (meta tags, canonical URLs, robots.txt)
/// 2. Search index generation
/// 3. HTML minification (must be last content transform)
/// 4. Live reload (`on_serve` only)
pub fn register_default_plugins(
    plugins: &mut plugin::PluginManager,
    config: &SsgConfig,
    include_drafts: bool,
    deploy_target: Option<&str>,
) {
    let base_url = config.base_url.clone();

    // Before-compile plugins
    plugins.register(content::ContentValidationPlugin);
    plugins.register(drafts::DraftPlugin::new(include_drafts));
    plugins.register(shortcodes::ShortcodePlugin);

    // Template engine (must run first in after_compile)
    #[cfg(feature = "templates")]
    plugins.register(
        crate::template_plugin::TemplatePlugin::from_template_dir(
            &config.template_dir,
        ),
    );

    // Post-processing fixes for staticdatagen output (run early,
    // before SEO plugins read/modify the HTML)
    plugins.register(postprocess::SitemapFixPlugin);
    plugins.register(postprocess::NewsSitemapFixPlugin);
    plugins.register(postprocess::RssAggregatePlugin);
    plugins.register(postprocess::AtomFeedPlugin);
    plugins.register(postprocess::ManifestFixPlugin);
    plugins.register(postprocess::HtmlFixPlugin);

    // Syntax highlighting
    plugins.register(highlight::HighlightPlugin::default());

    // SEO plugins
    plugins.register(seo::SeoPlugin);
    plugins
        .register(seo::JsonLdPlugin::from_site(&base_url, &config.site_name));
    plugins.register(seo::CanonicalPlugin::new(base_url.clone()));
    plugins.register(seo::RobotsPlugin::new(base_url));

    // AI readiness
    plugins.register(ai::AiPlugin);

    // Taxonomy and pagination
    plugins.register(taxonomy::TaxonomyPlugin);
    plugins.register(pagination::PaginationPlugin::default());

    // Search & optimization
    plugins.register(search::SearchPlugin);

    // Accessibility validation
    plugins.register(accessibility::AccessibilityPlugin);

    // Image optimization (WebP, responsive srcset)
    #[cfg(feature = "image-optimization")]
    plugins.register(crate::image_plugin::ImageOptimizationPlugin::default());

    // I18n hreflang injection and per-locale sitemaps
    if let Some(ref i18n_cfg) = config.i18n {
        if i18n_cfg.locales.len() > 1 {
            plugins.register(i18n::I18nPlugin::new(i18n_cfg.clone()));
        }
    }

    // Interactive islands (Web Components)
    plugins.register(islands::IslandPlugin);

    // CSP hardening: extract inline styles/scripts to external files with SRI
    plugins.register(csp::CspPlugin);

    // Asset fingerprinting + SRI (after all content transforms)
    plugins.register(assets::FingerprintPlugin);

    // Minification (must be last content transform)
    plugins.register(plugins_mod::MinifyPlugin);

    // Deployment config generation (opt-in via --deploy flag)
    if let Some(target) = deploy_target {
        let dt = match target {
            "netlify" => Some(deploy::DeployTarget::Netlify),
            "vercel" => Some(deploy::DeployTarget::Vercel),
            "cloudflare" => Some(deploy::DeployTarget::CloudflarePages),
            "github" => Some(deploy::DeployTarget::GithubPages),
            _ => {
                log::warn!("Unknown deploy target: {target}");
                None
            }
        };
        if let Some(dt) = dt {
            plugins.register(deploy::DeployPlugin::new(dt));
        }
    }

    // Dev server
    plugins.register(livereload::LiveReloadPlugin::default());
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_build_error_serialization() {
        let err = BuildError {
            file: Some("content/post.md".to_string()),
            line: Some(42),
            message: "unexpected token".to_string(),
        };
        let json = err.to_ws_message();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["type"], "error");
        assert_eq!(parsed["file"], "content/post.md");
        assert_eq!(parsed["line"], 42);
        assert_eq!(parsed["message"], "unexpected token");
    }

    #[test]
    fn test_clear_error_message() {
        let msg = clear_error_message();
        let parsed: serde_json::Value =
            serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["type"], "clear-error");
    }

    #[test]
    fn test_extract_file_from_error_md() {
        let msg = "cannot read content/posts/hello.md: permission denied";
        assert_eq!(
            extract_file_from_error(msg),
            Some("content/posts/hello.md".to_string())
        );
    }

    #[test]
    fn test_extract_file_from_error_html() {
        let msg = "template error in templates/base.html";
        assert_eq!(
            extract_file_from_error(msg),
            Some("templates/base.html".to_string())
        );
    }

    #[test]
    fn test_extract_file_from_error_toml() {
        let msg = "parse error in config/site.toml at line 5";
        assert_eq!(
            extract_file_from_error(msg),
            Some("config/site.toml".to_string())
        );
    }

    #[test]
    fn test_extract_file_from_error_none() {
        let msg = "something went wrong with no file path";
        assert_eq!(extract_file_from_error(msg), None);
    }

    #[test]
    fn test_build_error_from_anyhow() {
        let err = anyhow::anyhow!("cannot write output/index.html: disk full");
        let be = BuildError::from_anyhow(&err);
        assert_eq!(be.file, Some("output/index.html".to_string()));
        assert!(be.line.is_none());
        assert!(be.message.contains("disk full"));
    }

    // -----------------------------------------------------------------
    // BuildError — additional coverage
    // -----------------------------------------------------------------

    #[test]
    fn test_build_error_no_file_no_line() {
        let err = BuildError {
            file: None,
            line: None,
            message: "something broke".to_string(),
        };
        let json = err.to_ws_message();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["type"], "error");
        assert!(parsed["file"].is_null());
        assert!(parsed["line"].is_null());
        assert_eq!(parsed["message"], "something broke");
    }

    #[test]
    fn test_build_error_clone() {
        let err = BuildError {
            file: Some("a/b.md".to_string()),
            line: Some(10),
            message: "oops".to_string(),
        };
        let cloned = err.clone();
        assert_eq!(cloned.file, err.file);
        assert_eq!(cloned.line, err.line);
        assert_eq!(cloned.message, err.message);
    }

    #[test]
    fn test_build_error_debug() {
        let err = BuildError {
            file: None,
            line: None,
            message: "debug test".to_string(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("BuildError"));
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_build_error_from_anyhow_no_file() {
        let err = anyhow::anyhow!("generic error without any file path");
        let be = BuildError::from_anyhow(&err);
        assert!(be.file.is_none());
        assert!(be.message.contains("generic error"));
    }

    #[test]
    fn test_build_error_from_anyhow_yml_extension() {
        let err = anyhow::anyhow!("parse error in config/site.yml");
        let be = BuildError::from_anyhow(&err);
        assert_eq!(be.file, Some("config/site.yml".to_string()));
    }

    #[test]
    fn test_build_error_from_anyhow_yaml_extension() {
        let err = anyhow::anyhow!("error in data/settings.yaml at line 3");
        let be = BuildError::from_anyhow(&err);
        assert_eq!(be.file, Some("data/settings.yaml".to_string()));
    }

    // -----------------------------------------------------------------
    // extract_file_from_error — additional coverage
    // -----------------------------------------------------------------

    #[test]
    fn test_extract_file_with_punctuation_around_path() {
        let msg = "error: 'templates/base.html' not found";
        let result = extract_file_from_error(msg);
        assert_eq!(result, Some("templates/base.html".to_string()));
    }

    #[test]
    fn test_extract_file_no_slash_in_word() {
        let msg = "file not found: base.html";
        let result = extract_file_from_error(msg);
        assert!(result.is_none(), "no slash means no file path extraction");
    }

    #[test]
    fn test_extract_file_multiple_paths_returns_first() {
        let msg = "failed to read src/a.md and src/b.html";
        let result = extract_file_from_error(msg);
        assert_eq!(result, Some("src/a.md".to_string()));
    }

    #[test]
    fn test_extract_file_toml_with_trailing_colon() {
        let msg = "invalid key in config/site.toml: 'foo'";
        let result = extract_file_from_error(msg);
        assert_eq!(result, Some("config/site.toml".to_string()));
    }

    // -----------------------------------------------------------------
    // clear_error_message — sanity
    // -----------------------------------------------------------------

    #[test]
    fn test_clear_error_message_is_valid_json() {
        let msg = clear_error_message();
        let parsed: serde_json::Value =
            serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["type"], "clear-error");
        // Ensure no extra keys leak
        assert_eq!(parsed.as_object().unwrap().len(), 1);
    }

    // -----------------------------------------------------------------
    // resolve_build_and_site_dirs — coverage from pipeline module
    // -----------------------------------------------------------------

    #[test]
    fn test_resolve_dirs_no_serve_dir() {
        use crate::cmd::SsgConfig;
        use std::path::PathBuf;
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("out");
        config.serve_dir = None;

        let (build, site) = resolve_build_and_site_dirs(&config);
        assert_eq!(site, PathBuf::from("out"));
        // build should differ from site
        assert_ne!(build, site);
    }

    #[test]
    fn test_resolve_dirs_serve_differs_from_output() {
        use crate::cmd::SsgConfig;
        use std::path::PathBuf;
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("build");
        config.serve_dir = Some(PathBuf::from("public"));

        let (build, site) = resolve_build_and_site_dirs(&config);
        assert_eq!(build, PathBuf::from("build"));
        assert_eq!(site, PathBuf::from("public"));
    }

    #[test]
    fn test_resolve_dirs_serve_equals_output() {
        use crate::cmd::SsgConfig;
        use std::path::PathBuf;
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("dist");
        config.serve_dir = Some(PathBuf::from("dist"));

        let (build, site) = resolve_build_and_site_dirs(&config);
        assert_eq!(site, PathBuf::from("dist"));
        assert_ne!(build, site);
        assert!(build.to_string_lossy().contains("build-tmp"));
    }

    // -----------------------------------------------------------------
    // RunOptions — construction from matches
    // -----------------------------------------------------------------

    #[test]
    fn test_run_options_defaults() {
        use crate::cmd::Cli;
        let cli = Cli::build();
        let matches = cli.try_get_matches_from(vec!["ssg"]).unwrap();
        let opts = RunOptions::from_matches(&matches);

        assert!(!opts.quiet);
        assert!(!opts.include_drafts);
        assert!(opts.deploy_target.is_none());
        assert!(!opts.validate_only);
        assert!(opts.jobs.is_none());
        assert!(opts.max_memory_mb.is_none());
        assert!(!opts.ai_fix);
        assert!(!opts.ai_fix_dry_run);
    }

    #[test]
    fn test_run_options_ai_fix_flags() {
        use crate::cmd::Cli;
        let cli = Cli::build();
        let matches = cli
            .try_get_matches_from(vec!["ssg", "--ai-fix", "--ai-fix-dry-run"])
            .unwrap();
        let opts = RunOptions::from_matches(&matches);
        assert!(opts.ai_fix);
        assert!(opts.ai_fix_dry_run);
    }

    #[test]
    fn test_run_options_debug() {
        use crate::cmd::Cli;
        let cli = Cli::build();
        let matches = cli.try_get_matches_from(vec!["ssg"]).unwrap();
        let opts = RunOptions::from_matches(&matches);
        let debug = format!("{opts:?}");
        assert!(debug.contains("RunOptions"));
        assert!(debug.contains("quiet"));
    }

    #[test]
    fn test_run_options_clone() {
        use crate::cmd::Cli;
        let cli = Cli::build();
        let matches = cli
            .try_get_matches_from(vec!["ssg", "--quiet", "--jobs", "2"])
            .unwrap();
        let opts = RunOptions::from_matches(&matches);
        let cloned = opts.clone();
        assert_eq!(cloned.quiet, opts.quiet);
        assert_eq!(cloned.jobs, opts.jobs);
    }

    // -----------------------------------------------------------------
    // register_default_plugins — plugin count and ordering
    // -----------------------------------------------------------------

    #[test]
    fn test_register_default_plugins_minimum_count() {
        use crate::cmd::SsgConfig;
        use crate::plugin::PluginManager;

        let config = SsgConfig::default();
        let mut pm = PluginManager::new();
        register_default_plugins(&mut pm, &config, false, None);

        // We expect a substantial number of default plugins
        assert!(
            pm.len() >= 15,
            "expected at least 15 default plugins, got {}",
            pm.len()
        );
    }

    #[test]
    fn test_register_default_plugins_includes_key_plugins() {
        use crate::cmd::SsgConfig;
        use crate::plugin::PluginManager;

        let config = SsgConfig::default();
        let mut pm = PluginManager::new();
        register_default_plugins(&mut pm, &config, false, None);

        let names = pm.names();
        assert!(names.contains(&"content-validation"));
        assert!(names.contains(&"drafts"));
        assert!(names.contains(&"shortcodes"));
        assert!(names.contains(&"seo"));
        assert!(names.contains(&"search"));
        assert!(names.contains(&"minify"));
        assert!(names.contains(&"livereload"));
    }

    #[test]
    fn test_register_default_plugins_with_deploy_adds_deploy_plugin() {
        use crate::cmd::SsgConfig;
        use crate::plugin::PluginManager;

        let config = SsgConfig::default();
        let mut pm_without = PluginManager::new();
        register_default_plugins(&mut pm_without, &config, false, None);
        let count_without = pm_without.len();

        let mut pm_with = PluginManager::new();
        register_default_plugins(&mut pm_with, &config, false, Some("netlify"));

        assert_eq!(pm_with.len(), count_without + 1);
        assert!(pm_with.names().contains(&"deploy"));
    }

    #[test]
    fn test_register_default_plugins_unknown_deploy_skipped() {
        use crate::cmd::SsgConfig;
        use crate::plugin::PluginManager;

        let config = SsgConfig::default();
        let mut pm = PluginManager::new();
        register_default_plugins(
            &mut pm,
            &config,
            false,
            Some("nonexistent-platform"),
        );

        assert!(
            !pm.names().contains(&"deploy"),
            "unknown deploy target should not register a deploy plugin"
        );
    }

    // -----------------------------------------------------------------
    // build_pipeline — basic wiring
    // -----------------------------------------------------------------

    #[test]
    fn test_build_pipeline_returns_valid_dirs() {
        use crate::cmd::SsgConfig;

        let temp = tempfile::tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");
        config.template_dir = temp.path().join("templates");

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
            ai_fix: false,
            ai_fix_dry_run: false,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        assert!(!plugins.is_empty());
        assert_ne!(build_dir, site_dir);
        assert_eq!(ctx.content_dir, temp.path().join("content"));
    }
}
