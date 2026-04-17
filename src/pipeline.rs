// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Build pipeline: plugin orchestration and site compilation.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use staticdatagen::compile;

use crate::cmd::SsgConfig;
use crate::{
    accessibility, ai, assets, content, deploy, drafts, highlight, i18n,
    livereload, pagination, plugin, plugins as plugins_mod, postprocess,
    search, seo, shortcodes, taxonomy, walk,
};

/// CLI-driven options that don't live in `SsgConfig` itself.
///
/// Extracted from clap matches so the run pipeline can be unit-tested
/// without going through `Cli::build()`.
#[derive(Debug, Clone)]
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

    let ctx = plugin::PluginContext::with_config(
        &config.content_dir,
        &build_dir,
        &site_dir,
        &config.template_dir,
        config.clone(),
    );

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
    let mut ctx = ctx.clone();
    ctx.cache = Some(cache);

    plugins.run_before_compile(&ctx)?;
    compile_site(build_dir, content_dir, site_dir, template_dir)?;
    plugins.run_after_compile(&ctx)?;

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
    plugins.register(crate::template_plugin::TemplatePlugin::from_template_dir(
        &config.template_dir,
    ));

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
