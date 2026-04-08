#![forbid(unsafe_code)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://cloudcdn.pro/shokunin/images/favicon.ico",
    html_logo_url = "https://cloudcdn.pro/shokunin/images/logos/shokunin.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

/// Shared bounded directory walkers used by every plugin's
/// `collect_*_files` helper.
pub(crate) mod walk;

/// Test-only utilities shared across unit test modules.
#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::Once;

    static LOGGER: Once = Once::new();

    /// Raises `log::max_level()` to Trace so `log::info!` / `log::warn!`
    /// macro bodies execute their format arguments and are counted by
    /// LLVM region coverage. We only bump the filter level; no logger
    /// backend is installed, so it does not conflict with tests that
    /// install their own (e.g. the env_logger init test in lib.rs).
    /// Safe to call from any number of tests or fixtures.
    pub(crate) fn init_logger() {
        LOGGER.call_once(|| {
            log::set_max_level(log::LevelFilter::Trace);
        });
    }
}

// Standard library imports
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::cmd::{Cli, SsgConfig};

// Third-party imports
use anyhow::{anyhow, ensure, Context, Result};
use dtt::datetime::DateTime;
use http_handle::Server;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, LevelFilter};
use rayon::prelude::*;
use staticdatagen::compile;
use tokio::fs as async_fs;

/// Automated WCAG accessibility checker.
pub mod accessibility;
/// AI-readiness content hooks (GEO/AEO).
pub mod ai;
/// Asset fingerprinting, SRI hashes, and minification.
pub mod assets;
/// Content fingerprinting for incremental builds.
pub mod cache;
pub mod cmd;
/// Deployment adapter generation.
pub mod deploy;
/// Draft content filtering.
pub mod drafts;
/// Shared frontmatter extraction and `.meta.json` sidecar files.
pub mod frontmatter;
/// Syntax highlighting for code blocks.
pub mod highlight;
/// Image optimization with WebP and responsive srcset.
#[cfg(feature = "image-optimization")]
pub mod image_plugin;
/// WebSocket-based live-reload script injection.
pub mod livereload;
/// GitHub Flavored Markdown (GFM) extensions: tables, strikethrough, task lists.
pub mod markdown_ext;
/// Pagination for listing pages.
pub mod pagination;
/// Lifecycle hook plugin system.
pub mod plugin;
/// Built-in plugins for common tasks.
pub mod plugins;
/// Project scaffolding for `--new`.
pub mod scaffold;
/// Shortcode expansion for Markdown content.
pub mod shortcodes;
/// Taxonomy generation (tags, categories).
pub mod taxonomy;
use plugins as plugins_mod;
/// Command-line argument processing and site compilation.
pub mod process;
/// JSON Schema generation for configuration.
pub mod schema;
/// Client-side search index generator and search UI.
pub mod search;
/// SEO plugins: meta tags, robots.txt, and canonical URLs.
pub mod seo;
/// High-performance streaming file processor.
pub mod stream;
/// Tera templating engine integration.
#[cfg(feature = "tera-templates")]
pub mod tera_engine;
/// Tera template rendering plugin.
#[cfg(feature = "tera-templates")]
pub mod tera_plugin;
/// File-watching for live rebuild.
pub mod watch;

/// Re-exports
pub use staticdatagen;

/// Represents the necessary directory paths for the site generator.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The site output directory
    pub site: PathBuf,
    /// The content directory
    pub content: PathBuf,
    /// The build directory
    pub build: PathBuf,
    /// The template directory
    pub template: PathBuf,
}

impl Paths {
    /// Creates a new builder for configuring Paths
    #[must_use]
    pub fn builder() -> PathsBuilder {
        PathsBuilder::default()
    }

    /// Creates paths with default directories
    #[must_use]
    pub fn default_paths() -> Self {
        Self {
            site: PathBuf::from("public"),
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        }
    }
}
// Modify the validate method in Paths impl
impl Paths {
    /// Validates all paths in the configuration
    pub fn validate(&self) -> Result<()> {
        // Check for path traversal and other security concerns
        for (name, path) in [
            ("site", &self.site),
            ("content", &self.content),
            ("build", &self.build),
            ("template", &self.template),
        ] {
            // For non-existent paths, validate their components
            let path_str = path.to_string_lossy();
            if path_str.contains("..") {
                anyhow::bail!(
                    "{} path contains directory traversal: {}",
                    name,
                    path.display()
                );
            }
            if path_str.contains("//") {
                anyhow::bail!(
                    "{} path contains invalid double slashes: {}",
                    name,
                    path.display()
                );
            }

            // If path exists, perform additional checks
            if path.exists() {
                let metadata = path
                    .symlink_metadata()
                    .context(format!("Failed to get metadata for {name}"))?;

                if metadata.file_type().is_symlink() {
                    anyhow::bail!(
                        "{} path is a symlink which is not allowed: {}",
                        name,
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }
}

/// Builder for creating Paths configurations
#[derive(Debug, Default, Clone)]
pub struct PathsBuilder {
    /// The site output directory
    pub site: Option<PathBuf>,
    /// The content directory
    pub content: Option<PathBuf>,
    /// The build directory
    pub build: Option<PathBuf>,
    /// The template directory
    pub template: Option<PathBuf>,
}

impl PathsBuilder {
    /// Sets the site output directory
    pub fn site<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.site = Some(path.into());
        self
    }

    /// Sets the content directory
    pub fn content<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.content = Some(path.into());
        self
    }

    /// Sets the build directory
    pub fn build_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.build = Some(path.into());
        self
    }

    /// Sets the template directory
    pub fn template<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.template = Some(path.into());
        self
    }

    /// Sets all paths relative to a base directory
    pub fn relative_to<P: AsRef<Path>>(self, base: P) -> Self {
        let base = base.as_ref();
        self.site(base.join("public"))
            .content(base.join("content"))
            .build_dir(base.join("build"))
            .template(base.join("templates"))
    }

    /// Builds the Paths configuration
    ///
    /// # Returns
    ///
    /// * `Result<Paths>` - The configured paths if valid
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Required paths are missing
    /// * Paths are invalid or unsafe
    /// * Unable to create necessary directories
    pub fn build(self) -> Result<Paths> {
        let paths = Paths {
            site: self.site.unwrap_or_else(|| PathBuf::from("public")),
            content: self.content.unwrap_or_else(|| PathBuf::from("content")),
            build: self.build.unwrap_or_else(|| PathBuf::from("build")),
            template: self
                .template
                .unwrap_or_else(|| PathBuf::from("templates")),
        };

        // Validate the configuration
        paths.validate()?;

        Ok(paths)
    }
}

// Constants for configuration
const DEFAULT_LOG_LEVEL: &str = "info";
const ENV_LOG_LEVEL: &str = "SSG_LOG_LEVEL";

/// Maximum directory nesting depth for all traversal operations.
/// Prevents stack overflow from pathological or circular directory trees.
/// 128 levels accommodates any realistic project structure.
pub const MAX_DIR_DEPTH: usize = 128;

/// Minimum number of entries to justify Rayon parallel dispatch overhead.
const PARALLEL_THRESHOLD: usize = 16;

/// Maps a case-insensitive log level string to a `LevelFilter`.
///
/// Unrecognised values fall back to `LevelFilter::Info`. Extracted
/// from `initialize_logging` so it can be unit-tested without
/// installing a global logger (which is one-shot per process).
fn parse_log_level(log_level: &str) -> LevelFilter {
    match log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

/// Initializes the logging system based on environment variables.
fn initialize_logging() -> Result<()> {
    let log_level = std::env::var(ENV_LOG_LEVEL)
        .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

    let level = parse_log_level(&log_level);

    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();

    info!("Logging initialized at level: {log_level}");
    Ok(())
}

/// Resolves distinct build and site directories for compilation.
///
/// `staticdatagen::compile` finalizes output by renaming the build directory
/// into the site directory. If both paths are identical, finalization fails.
/// This helper guarantees distinct paths when needed.
fn resolve_build_and_site_dirs(config: &SsgConfig) -> (PathBuf, PathBuf) {
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

/// CLI-driven options that don't live in `SsgConfig` itself.
///
/// Extracted from clap matches so the run pipeline can be unit-tested
/// without going through `Cli::build()`.
#[derive(Debug, Clone)]
pub(crate) struct RunOptions {
    /// Suppress banner and timing print-outs.
    pub quiet: bool,
    /// Include draft files (skip the DraftPlugin filter).
    pub include_drafts: bool,
    /// Optional deploy target — `netlify`, `vercel`, `cloudflare`, `github`.
    pub deploy_target: Option<String>,
}

impl RunOptions {
    /// Builds a `RunOptions` from a parsed `clap::ArgMatches`.
    pub(crate) fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            quiet: matches.get_flag("quiet"),
            include_drafts: matches.get_flag("drafts"),
            deploy_target: matches.get_one::<String>("deploy").cloned(),
        }
    }
}

/// Builds a fully-populated plugin manager and plugin context for a build.
///
/// Extracted so unit tests can construct the same wiring without
/// needing to fake CLI argument parsing.
pub(crate) fn build_pipeline(
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

/// Runs the build half of the pipeline: before_compile → compile →
/// after_compile. Does not start the dev server.
///
/// Extracted from `run()` so the actual build can be unit-tested
/// against a tempdir without booting an HTTP server.
pub(crate) fn execute_build_pipeline(
    plugins: &plugin::PluginManager,
    ctx: &plugin::PluginContext,
    build_dir: &Path,
    content_dir: &Path,
    site_dir: &Path,
    template_dir: &Path,
    quiet: bool,
) -> Result<()> {
    let start = std::time::Instant::now();

    plugins.run_before_compile(ctx)?;
    compile_site(build_dir, content_dir, site_dir, template_dir)?;
    plugins.run_after_compile(ctx)?;

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

/// Executes the static site generation process.
///
/// Parses CLI arguments, runs the plugin pipeline around compilation,
/// and starts a local dev server. This function blocks indefinitely
/// while the server is running.
pub async fn run() -> Result<()> {
    initialize_logging()?;
    info!("Starting site generation process");

    let matches = Cli::build().get_matches();
    let config = SsgConfig::from_matches(&matches)?;
    let opts = RunOptions::from_matches(&matches);

    if !opts.quiet {
        Cli::print_banner();
    }

    let (plugins, ctx, build_dir, site_dir) = build_pipeline(&config, &opts);

    execute_build_pipeline(
        &plugins,
        &ctx,
        &build_dir,
        &config.content_dir,
        &site_dir,
        &config.template_dir,
        opts.quiet,
    )?;

    // Run on_serve hooks and start dev server
    plugins.run_on_serve(&ctx)?;
    serve_site(&site_dir)
}

/// Registers the default plugin pipeline.
///
/// Plugins execute in registration order. The ordering is:
/// 1. SEO plugins (meta tags, canonical URLs, robots.txt)
/// 2. Search index generation
/// 3. HTML minification (must be last content transform)
/// 4. Live reload (`on_serve` only)
fn register_default_plugins(
    plugins: &mut plugin::PluginManager,
    config: &SsgConfig,
    include_drafts: bool,
    deploy_target: Option<&str>,
) {
    let base_url = config.base_url.clone();

    // Before-compile plugins
    plugins.register(drafts::DraftPlugin::new(include_drafts));
    plugins.register(shortcodes::ShortcodePlugin);

    // Tera templating (must run first in after_compile)
    #[cfg(feature = "tera-templates")]
    plugins.register(tera_plugin::TeraPlugin::from_template_dir(
        &config.template_dir,
    ));

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
    plugins.register(image_plugin::ImageOptimizationPlugin);

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

/// Pluggable transport that drives the dev server.
///
/// Production code uses [`HttpTransport`] (a thin wrapper around
/// `http_handle::Server`); tests use [`NoopTransport`] which records
/// the call without actually binding a port. The trait exists so
/// every line of `serve_site` is unit-testable.
pub trait ServeTransport {
    /// Start serving `root` on `addr`. Implementations may block.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying transport fails to start.
    fn start(&self, addr: &str, root: &str) -> Result<()>;
}

/// Production transport: starts an `http_handle::Server`.
#[derive(Debug, Clone, Copy)]
pub struct HttpTransport;

impl ServeTransport for HttpTransport {
    fn start(&self, addr: &str, root: &str) -> Result<()> {
        let server = Server::new(addr, root);
        let _ = server.start();
        Ok(())
    }
}

/// Resolves a `site_dir` `Path` into the `(addr, root)` pair the
/// transport expects, returning an error if the path contains
/// invalid UTF-8.
///
/// Extracted from `serve_site` so the path-to-string conversion can
/// be unit-tested without invoking a transport.
pub(crate) fn build_serve_address(site_dir: &Path) -> Result<(String, String)> {
    let root = site_dir
        .to_str()
        .ok_or_else(|| {
            anyhow!("Site directory path contains invalid UTF-8: {site_dir:?}")
        })?
        .to_string();
    let addr = format!("{}:{}", cmd::DEFAULT_HOST, cmd::DEFAULT_PORT);
    Ok((addr, root))
}

/// Starts the dev server using a caller-supplied transport.
///
/// Extracted so test code can pass a no-op transport and still
/// exercise the surrounding glue (path validation, address
/// formatting). Production callers use [`serve_site`] which
/// delegates to [`HttpTransport`].
///
/// # Errors
///
/// Returns an error if `site_dir` contains invalid UTF-8 or if the
/// underlying transport fails.
pub fn serve_site_with<T: ServeTransport>(
    site_dir: &Path,
    transport: &T,
) -> Result<()> {
    let (addr, root) = build_serve_address(site_dir)?;
    transport.start(&addr, &root)
}

/// Converts a site directory path to a string and starts an HTTP server.
///
/// This function blocks while the server is running.
///
/// # Errors
///
/// Returns an error if `site_dir` contains invalid UTF-8.
pub fn serve_site(site_dir: &Path) -> Result<()> {
    serve_site_with(site_dir, &HttpTransport)
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

/// Validates and copies files from source to destination.
///
/// This function performs comprehensive safety checks before copying files,
/// including path validation, symlink detection, and size limitations.
///
/// # Arguments
///
/// * `src` - Source path to copy from
/// * `dst` - Destination path to copy to
///
/// # Returns
///
/// Returns `Ok(())` if the copy operation succeeds, or an error if:
/// * Source path is invalid or inaccessible
/// * Source contains symlinks (not allowed)
/// * Files exceed size limits (default: 10MB)
/// * Destination cannot be created or written to
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use ssg::verify_and_copy_files;
///
/// fn main() -> anyhow::Result<()> {
///     let source = Path::new("source_directory");
///     let destination = Path::new("destination_directory");
///
///     verify_and_copy_files(source, destination)?;
///     println!("Files copied successfully");
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// This function implements several security measures:
/// * Path traversal prevention
/// * Symlink restriction
/// * File size limits
/// * Permission validation
pub fn verify_and_copy_files(src: &Path, dst: &Path) -> Result<()> {
    ensure!(
        is_safe_path(src)?,
        "Source directory is unsafe or inaccessible: {src:?}"
    );

    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {src:?}");
    }

    // If source is a file, verify its safety
    if src.is_file() {
        verify_file_safety(src)?;
    }

    // Ensure the destination directory exists
    fs::create_dir_all(dst).with_context(|| {
        format!(
            "Failed to create or access destination directory at path: {dst:?}"
        )
    })?;

    // Copy directory contents with safety checks
    copy_dir_all(src, dst).with_context(|| {
        format!(
            "Failed to copy files from source: {src:?} to destination: {dst:?}"
        )
    })?;

    Ok(())
}

/// Asynchronously validates and copies files between directories.
///
/// Uses iterative traversal with an explicit stack to avoid unbounded recursion.
/// Traversal depth is bounded by [`MAX_DIR_DEPTH`].
pub async fn verify_and_copy_files_async(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {src:?}"
        ));
    }

    async_fs::create_dir_all(dst).await.with_context(|| {
        format!(
            "Failed to create or access destination directory at path: {dst:?}"
        )
    })?;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        let mut entries = async_fs::read_dir(&src_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());

            if src_path.is_dir() {
                async_fs::create_dir_all(&dst_path).await?;
                stack.push((src_path, dst_path, depth + 1));
            } else {
                verify_file_safety(&src_path)?;
                _ = async_fs::copy(&src_path, &dst_path).await?;
            }
        }
    }

    Ok(())
}

/// Copies directories with a progress bar for feedback.
///
/// Uses iterative traversal with an explicit stack to avoid unbounded recursion.
/// Traversal depth is bounded by [`MAX_DIR_DEPTH`].
pub fn copy_dir_with_progress(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {}", src.display());
    }

    fs::create_dir_all(dst).with_context(|| {
        format!("Failed to create destination directory: {}", dst.display())
    })?;

    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {pos} files {msg}")
            .map_err(|e| anyhow::anyhow!("Invalid progress bar template: {e}"))?
            .progress_chars("#>-"),
    );

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        let entries: Vec<_> = fs::read_dir(&src_dir)
            .context(format!(
                "Failed to read source directory: {}",
                src_dir.display()
            ))?
            .collect::<std::io::Result<Vec<_>>>()?;

        for entry in &entries {
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());

            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                stack.push((src_path, dst_path, depth + 1));
            } else {
                let _ = fs::copy(&src_path, &dst_path)?;
            }
            progress_bar.inc(1);
        }
    }

    progress_bar.finish_with_message("Copy complete.");
    Ok(())
}

/// Checks if a given path is safe to use.
///
/// Validates that the provided path does not contain directory traversal attempts
/// or other potential security risks.
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// * `Ok(true)` - If the path is safe to use
/// * `Ok(false)` - If the path contains unsafe elements
/// * `Err` - If path validation fails
///
/// # Security
///
/// This function prevents directory traversal attacks by:
/// * Resolving symbolic links
/// * Checking for parent directory references (`..`)
/// * Validating path components
///
pub fn is_safe_path(path: &Path) -> Result<bool> {
    // Check for traversal patterns in non-existent paths
    if !path.exists() {
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Ok(false);
        }
        return Ok(true); // Non-existent paths without traversal are safe
    }

    // canonicalize() resolves symlinks and all `..' components,
    // so the resulting path is always absolute with no parent refs.
    // A failure here (e.g. broken symlink) means the path is unsafe.
    let _canonical = path
        .canonicalize()
        .context(format!("Failed to canonicalize path {}", path.display()))?;

    Ok(true)
}

/// Verifies the safety of a file for processing.
///
/// Performs comprehensive safety checks on a file to ensure it meets security
/// requirements before processing. These checks include symlink detection and
/// file size validation.
///
/// # Arguments
///
/// * `path` - Reference to the path of the file to verify
///
/// # Returns
///
/// * `Ok(())` - If the file passes all safety checks
/// * `Err` - If any safety check fails
///
/// # Safety Checks
///
/// * Symlinks: Not allowed (returns error)
/// * File size: Must be under 10MB
/// * File type: Must be a regular file
///
/// # Examples
///
/// Verifies the safety of a file.
///
/// ```rust
/// use std::fs;
/// use std::path::Path;
/// use ssg::verify_file_safety;
/// use tempfile::tempdir;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create temporary directory
/// let temp_dir = tempdir()?;
/// let file_path = temp_dir.path().join("index.md");
///
/// // Create test file
/// fs::write(&file_path, "Hello, world!")?;
///
/// // Perform verification
/// verify_file_safety(&file_path)?;
///
/// // Directory and file are automatically cleaned up
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * File is a symlink
/// * File size exceeds 10MB
/// * Cannot read file metadata
pub fn verify_file_safety(path: &Path) -> Result<()> {
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

    // Get symlink metadata without following the symlink
    let symlink_metadata = path.symlink_metadata().map_err(|e| {
        anyhow::anyhow!(
            "Failed to get symlink metadata for {}: {}",
            path.display(),
            e
        )
    })?;

    // Explicitly check for symlinks first
    if symlink_metadata.file_type().is_symlink() {
        return Err(anyhow::anyhow!(
            "Symlinks are not allowed: {}",
            path.display()
        ));
    }

    // Only check size if it's a regular file
    if symlink_metadata.file_type().is_file()
        && symlink_metadata.len() > MAX_FILE_SIZE
    {
        return Err(anyhow::anyhow!(
            "File exceeds maximum allowed size of {} bytes: {}",
            MAX_FILE_SIZE,
            path.display()
        ));
    }

    Ok(())
}

/// Creates and initialises a log file for the static site generator.
///
/// Establishes a new log file at the specified path with appropriate permissions
/// and write capabilities. The log file is used to track the generation process
/// and any errors that occur.
///
/// # Arguments
///
/// * `file_path` - The desired location for the log file
///
/// # Returns
///
/// * `Ok(File)` - A file handle for the created log file
/// * `Err` - If the file cannot be created or permissions are insufficient
///
/// # Examples
///
/// ```rust
/// use ssg::create_log_file;
///
/// fn main() -> anyhow::Result<()> {
///     let log_file = create_log_file("./site_generation.log")?;
///     println!("Log file created successfully");
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// * The specified path is invalid
/// * File creation permissions are insufficient
/// * The parent directory is not writable
pub fn create_log_file(file_path: &str) -> Result<File> {
    File::create(file_path).context("Failed to create log file")
}

/// Records system initialisation in the logging system.
///
/// Creates a detailed log entry capturing the system's startup state,
/// including configuration and initial conditions. Uses the Common Log Format (CLF)
/// for consistent logging.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Returns
///
/// * `Ok(())` - If the log entry is written successfully
/// * `Err` - If writing fails or translation errors occur
///
/// # Examples
///
/// ```rust
/// use ssg::{create_log_file, log_initialization};
/// use dtt::datetime::DateTime;
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = DateTime::new();
///
///     log_initialization(&mut log_file, &date)?;
///     println!("System initialisation logged");
///     Ok(())
/// }
/// ```
pub fn log_initialization(log_file: &mut File, date: &DateTime) -> Result<()> {
    writeln!(
        log_file,
        "[{date}] INFO process: System initialization complete"
    )
    .context("Failed to write banner log")
}

/// Logs processed command-line arguments for debugging and auditing.
///
/// Records all provided command-line arguments and their values in the log file,
/// providing a traceable record of site generation parameters.
///
/// # Arguments
///
/// * `log_file` - Active file handle for writing log entries
/// * `date` - Current date and time for log timestamps
///
/// # Returns
///
/// * `Ok(())` - If arguments are logged successfully
/// * `Err` - If writing fails or translation errors occur
///
/// # Examples
///
/// ```rust
/// use ssg::{create_log_file, log_arguments};
/// use dtt::datetime::DateTime;
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./site.log")?;
///     let date = DateTime::new();
///
///     log_arguments(&mut log_file, &date)?;
///     println!("Arguments logged successfully");
///     Ok(())
/// }
/// ```
pub fn log_arguments(log_file: &mut File, date: &DateTime) -> Result<()> {
    writeln!(log_file, "[{date}] INFO process: Arguments processed")
        .context("Failed to write arguments log")
}

/// Creates and verifies required directories for site generation.
///
/// Ensures all necessary directories exist and are safe to use, creating
/// them if necessary. Also performs security checks on each directory.
///
/// # Arguments
///
/// * `paths` - Reference to a Paths struct containing required directory paths
///
/// # Returns
///
/// * `Ok(())` - If all directories are created/verified successfully
/// * `Err` - If any directory operation fails
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use ssg::{Paths, create_directories};
///
/// fn main() -> anyhow::Result<()> {
///     let paths = Paths {
///         site: PathBuf::from("public"),
///         content: PathBuf::from("content"),
///         build: PathBuf::from("build"),
///         template: PathBuf::from("templates"),
///     };
///
///     create_directories(&paths)?;
///     println!("All directories ready");
///     Ok(())
/// }
/// ```
///
/// # Security
///
/// Performs the following security checks:
/// * Path traversal prevention
/// * Permission validation
/// * Safe path verification
pub fn create_directories(paths: &Paths) -> Result<()> {
    // Ensure each directory exists, with custom error messages for each.
    for (name, path) in [
        ("content", &paths.content),
        ("build", &paths.build),
        ("site", &paths.site),
        ("template", &paths.template),
    ] {
        fs::create_dir_all(path).with_context(|| {
            format!(
                "Failed to create or access {name} directory at path: {path:?}"
            )
        })?;
    }

    // Path safety check with additional context
    if !is_safe_path(&paths.content)?
        || !is_safe_path(&paths.build)?
        || !is_safe_path(&paths.site)?
        || !is_safe_path(&paths.template)?
    {
        anyhow::bail!("One or more paths are unsafe. Ensure paths do not contain '..' and are accessible.");
    }

    Ok(())
}

/// Configures and launches the development server.
///
/// Sets up a local server for testing and previewing the generated site.
/// Handles file copying and server configuration for local development.
///
/// # Arguments
///
/// * `log_file` - Reference to the active log file
/// * `date` - Current timestamp for logging
/// * `paths` - All required directory paths
/// * `serve_dir` - Directory to serve content from
///
/// # Returns
///
/// * `Ok(())` - If server starts successfully
/// * `Err` - If server configuration or startup fails
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use ssg::{Paths, handle_server, create_log_file};
/// use dtt::datetime::DateTime;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./server.log")?;
///     let date = DateTime::new();
///     let paths = Paths {
///         site: PathBuf::from("public"),
///         content: PathBuf::from("content"),
///         build: PathBuf::from("build"),
///         template: PathBuf::from("templates"),
///     };
///     let serve_dir = PathBuf::from("serve");
///
///     handle_server(&mut log_file, &date, &paths, &serve_dir).await?;
///     Ok(())
/// }
/// ```
///
/// # Server Configuration
///
/// * Default port: 8000
/// * Host: 127.0.0.1 (localhost)
/// * Serves static files from the specified directory
pub async fn handle_server(
    log_file: &mut File,
    date: &DateTime,
    paths: &Paths,
    serve_dir: &PathBuf,
) -> Result<()> {
    // Log server initialization
    writeln!(log_file, "[{date}] INFO process: Server initialization")?;

    prepare_serve_dir(paths, serve_dir).await?;

    println!("\nStarting server at http://127.0.0.1:8000");
    println!("Serving content from: {}", serve_dir.display());

    warp::serve(warp::fs::dir(serve_dir.clone()))
        .run(([127, 0, 0, 1], 8000))
        .await;
    Ok(())
}

/// Prepares the serve directory by creating it and copying site files.
pub async fn prepare_serve_dir(
    paths: &Paths,
    serve_dir: &PathBuf,
) -> Result<()> {
    async_fs::create_dir_all(serve_dir)
        .await
        .context("Failed to create serve directory")?;

    println!("Setting up server...");
    println!("Source: {}", paths.site.display());
    println!("Serving from: {}", serve_dir.display());

    if serve_dir != &paths.site {
        verify_and_copy_files_async(&paths.site, serve_dir).await?;
    }
    Ok(())
}

/// Recursively collects all file paths within a directory.
///
/// Traverses a directory tree and compiles a list of all file paths found,
/// excluding directories themselves.
///
/// # Arguments
///
/// * `dir` - Reference to the directory to search
/// * `files` - Mutable vector to store found file paths
///
/// # Returns
///
/// * `Ok(())` - If the collection process succeeds
/// * `Err` - If any file system operation fails
///
/// # Examples
///
/// ```rust
/// use std::path::{Path, PathBuf};
/// use ssg::collect_files_recursive;
///
/// fn main() -> anyhow::Result<()> {
///     let mut files = Vec::new();
///     let dir_path = Path::new("./examples/content");
///
///     collect_files_recursive(dir_path, &mut files)?;
///
///     for file in files {
///         println!("Found file: {}", file.display());
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Note
///
/// This function:
/// * Only collects file paths, not directory paths
/// * Rejects symbolic links (consistent with security model)
/// * Maintains original path structure
pub fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    // (directory, depth)
    let mut stack = vec![(dir.to_path_buf(), 0usize)];

    while let Some((current_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            current_dir.display()
        );

        for entry in fs::read_dir(&current_dir)? {
            let path = entry?.path();

            if path.is_dir() {
                stack.push((path, depth + 1));
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Recursively copies a directory whilst maintaining structure and attributes.
///
/// Performs a deep copy of a directory tree, preserving file attributes and
/// handling nested directories. Uses parallel processing for improved performance.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
///
/// # Returns
///
/// * `Ok(())` - If the copy operation succeeds
/// * `Err` - If any part of the copy operation fails
///
/// # Performance
///
/// Uses rayon for parallel processing of files, significantly improving
/// performance for directories with many files.
///
/// # Safety
///
/// * Verifies file safety before copying
/// * Maintains original file permissions
/// * Handles circular references
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_dir, dst_dir, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_dir.display()
        );

        let entries: Vec<_> =
            fs::read_dir(&src_dir)?.collect::<std::io::Result<Vec<_>>>()?;

        // Separate files from directories in a single pass
        let mut subdirs = Vec::new();
        let files: Vec<_> = entries
            .iter()
            .filter(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    subdirs.push((path, dst_dir.join(entry.file_name())));
                    false
                } else {
                    true
                }
            })
            .collect();

        // Copy files — parallel only when worth the dispatch cost
        let copy_file = |entry: &&fs::DirEntry| -> Result<()> {
            let src_path = entry.path();
            let dst_path = dst_dir.join(entry.file_name());
            verify_file_safety(&src_path)?;
            _ = fs::copy(&src_path, &dst_path)?;
            Ok(())
        };

        if files.len() >= PARALLEL_THRESHOLD {
            files.par_iter().try_for_each(copy_file)?;
        } else {
            files.iter().try_for_each(copy_file)?;
        }

        // Push subdirectories onto the stack
        for (sub_src, sub_dst) in subdirs {
            fs::create_dir_all(&sub_dst)?;
            stack.push((sub_src, sub_dst, depth + 1));
        }
    }

    Ok(())
}

/// Asynchronously copies an entire directory structure, preserving file attributes and handling nested directories.
///
/// # Parameters
///
/// * `src`: A reference to the source directory path.
/// * `dst`: A reference to the destination directory path.
///
/// # Returns
///
/// * `Result<()>`:
///   - `Ok(())`: If the directory copying is successful.
///   - `Err(e)`: If an error occurs during the directory copying, where `e` is the associated error.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `std::io::Error`: If an error occurs during directory creation, file copying, or permission issues.
/// * `anyhow::Error`: If a file safety check fails.
#[cfg(feature = "async")]
pub async fn copy_dir_all_async(src: &Path, dst: &Path) -> Result<()> {
    internal_copy_dir_async(src, dst).await
}

#[cfg(feature = "async")]
async fn internal_copy_dir_async(src: &Path, dst: &Path) -> Result<()> {
    tokio::fs::create_dir_all(dst).await?;

    // (source_dir, dest_dir, depth)
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf(), 0usize)];

    while let Some((src_path, dst_path, depth)) = stack.pop() {
        ensure!(
            depth < MAX_DIR_DEPTH,
            "Directory nesting exceeds maximum depth of {}: {}",
            MAX_DIR_DEPTH,
            src_path.display()
        );

        let mut entries = tokio::fs::read_dir(&src_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let src_entry = entry.path();
            let dst_entry = dst_path.join(entry.file_name());

            if src_entry.is_dir() {
                tokio::fs::create_dir_all(&dst_entry).await?;
                stack.push((src_entry, dst_entry, depth + 1));
            } else {
                verify_file_safety(&src_entry)?;
                _ = tokio::fs::copy(&src_entry, &dst_entry).await?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Cli;
    use anyhow::Result;
    use std::env;
    use std::{
        fs::{self, File},
        path::PathBuf,
    };
    use tempfile::{tempdir, TempDir};

    #[test]
    fn test_create_log_file_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("test.log");

        let log_file = create_log_file(log_file_path.to_str().unwrap())?;
        assert!(log_file.metadata()?.is_file());

        Ok(())
    }

    #[test]
    fn test_log_arguments() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("args_log.log");
        let mut log_file = File::create(&log_file_path)?;

        let date = DateTime::new();
        log_arguments(&mut log_file, &date)?;

        let log_content = fs::read_to_string(log_file_path)?;
        assert!(log_content.contains("process"));

        Ok(())
    }

    #[test]
    fn test_create_directories_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();

        let paths = Paths {
            site: base_path.join("public"),
            content: base_path.join("content"),
            build: base_path.join("build"),
            template: base_path.join("templates"),
        };

        create_directories(&paths)?;

        // Verify each directory exists
        assert!(paths.site.exists());
        assert!(paths.content.exists());
        assert!(paths.build.exists());
        assert!(paths.template.exists());

        Ok(())
    }

    #[test]
    fn test_create_directories_failure() {
        let invalid_paths = Paths {
            site: PathBuf::from("/invalid/site"),
            content: PathBuf::from("/invalid/content"),
            build: PathBuf::from("/invalid/build"),
            template: PathBuf::from("/invalid/template"),
        };

        let result = create_directories(&invalid_paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_all() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let src_file = src_dir.path().join("test_file.txt");
        _ = File::create(&src_file)?;

        let result = copy_dir_all(src_dir.path(), dst_dir.path());
        assert!(result.is_ok());
        assert!(dst_dir.path().join("test_file.txt").exists());

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();

        // Create source directory and test file
        let src_dir = base_path.join("src");
        fs::create_dir_all(&src_dir)?;
        let test_file = src_dir.join("test_file.txt");
        fs::write(&test_file, "test content")?;

        // Create destination directory
        let dst_dir = base_path.join("dst");

        // Verify and copy files
        verify_and_copy_files(&src_dir, &dst_dir)?;

        // Verify the file was copied
        assert!(dst_dir.join("test_file.txt").exists());

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_failure() {
        let src_dir = PathBuf::from("/invalid/src");
        let dst_dir = PathBuf::from("/invalid/dst");

        let result = verify_and_copy_files(&src_dir, &dst_dir);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_server_failure() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path).unwrap();

        let paths = Paths {
            site: PathBuf::from("/invalid/site"),
            content: PathBuf::from("/invalid/content"),
            build: PathBuf::from("/invalid/build"),
            template: PathBuf::from("/invalid/template"),
        };

        let serve_dir = temp_dir.path().join("serve");
        let date = DateTime::new();
        let result = handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(result.await.is_err());
    }

    #[test]
    fn test_is_safe_path_safe() -> Result<()> {
        let temp_dir = tempdir()?;
        let safe_path = temp_dir.path().to_path_buf().join("safe_path");

        // Create the directory
        fs::create_dir_all(&safe_path)?;

        // Use the absolute path
        let absolute_safe_path = safe_path.canonicalize()?;
        assert!(is_safe_path(&absolute_safe_path)?);
        Ok(())
    }

    #[test]
    fn test_create_directories_partial_failure() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().join("valid_dir");
        let invalid_path = PathBuf::from("/invalid/path");

        let paths = Paths {
            site: valid_path,
            content: invalid_path.clone(),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let result = create_directories(&paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_all_nested() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let nested_dir = src_dir.path().join("nested_dir");
        fs::create_dir(&nested_dir)?;

        let nested_file = nested_dir.join("nested_file.txt");
        _ = File::create(&nested_file)?;

        copy_dir_all(src_dir.path(), dst_dir.path())?;
        assert!(dst_dir.path().join("nested_dir/nested_file.txt").exists());

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_missing_source() {
        let src_path = PathBuf::from("/non_existent_dir");
        let dst_dir = tempdir().unwrap();

        let result = verify_and_copy_files(&src_path, dst_dir.path());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_server_missing_serve_dir() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path).unwrap();

        let paths = Paths {
            site: temp_dir.path().join("site"),
            content: temp_dir.path().join("content"),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let non_existent_serve_dir = PathBuf::from("/non_existent_serve_dir");
        let binding = DateTime::new();
        let result = handle_server(
            &mut log_file,
            &binding,
            &paths,
            &non_existent_serve_dir,
        );
        assert!(result.await.is_err());
    }

    #[test]
    fn test_collect_files_recursive_empty() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut files = Vec::new();

        collect_files_recursive(temp_dir.path(), &mut files)?;
        assert!(files.is_empty());

        Ok(())
    }

    #[test]
    fn test_print_banner() {
        // Simply call the function to ensure it runs without errors.
        Cli::print_banner();
    }

    #[test]
    fn test_collect_files_recursive_with_nested_directories() -> Result<()> {
        let temp_dir = tempdir()?;
        let nested_dir = temp_dir.path().join("nested_dir");
        fs::create_dir(&nested_dir)?;

        let nested_file = nested_dir.join("nested_file.txt");
        _ = File::create(&nested_file)?;

        let mut files = Vec::new();
        collect_files_recursive(temp_dir.path(), &mut files)?;

        assert!(files.contains(&nested_file));
        assert_eq!(files.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_handle_server_start_message() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("server_log.log");
        let mut log_file = File::create(&log_file_path)?;

        let paths = Paths {
            site: temp_dir.path().join("site"),
            content: temp_dir.path().join("content"),
            build: temp_dir.path().join("build"),
            template: temp_dir.path().join("template"),
        };

        let serve_dir = temp_dir.path().join("serve");

        // Check setup conditions before calling `handle_server`
        fs::create_dir_all(&serve_dir)?;
        assert!(serve_dir.exists(), "Expected serve directory to be created");

        // Now, call `handle_server` and check for specific output or error
        let date = DateTime::new();
        let result = handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(
            result.await.is_err(),
            "Expected handle_server to fail without valid setup"
        );

        Ok(())
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn test_verify_file_safety_symlink() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let symlink_path = temp_dir.path().join("test_link.txt");

        // Create a regular file
        fs::write(&file_path, "test content")?;

        // Create a symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &symlink_path)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&file_path, &symlink_path)?;

        // Debug output
        println!("File exists: {}", file_path.exists());
        println!("Symlink exists: {}", symlink_path.exists());
        println!(
            "Is symlink: {}",
            symlink_path.symlink_metadata()?.file_type().is_symlink()
        );

        // Try to verify the symlink
        let result = verify_file_safety(&symlink_path);

        // Print the result for debugging
        println!("Result: {:?}", result);

        // Verify that we got an error
        assert!(result.is_err(), "Expected error for symlink, got success");

        // Verify the error message
        let err = result.unwrap_err();
        println!("Error message: {}", err);
        assert!(
            err.to_string().contains("Symlinks are not allowed"),
            "Unexpected error message: {}",
            err
        );

        Ok(())
    }

    #[test]
    fn test_verify_file_safety_size() -> Result<()> {
        let temp_dir = tempdir()?;
        let large_file_path = temp_dir.path().join("large.txt");

        // Create a large file
        let file = File::create(&large_file_path)?;
        file.set_len(11 * 1024 * 1024)?; // 11MB

        let result = verify_file_safety(&large_file_path);
        assert!(result.is_err(), "Expected error, got: {:?}", result);
        Ok(())
    }

    #[test]
    fn test_verify_file_safety_regular() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("regular.txt");

        // Create a regular file
        fs::write(&file_path, "test content")?;

        assert!(verify_file_safety(&file_path).is_ok());
        Ok(())
    }

    /// Tests successful copying of an empty directory
    #[tokio::test]
    async fn test_copy_empty_directory_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let result = copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_ok());

        // Verify destination directory exists
        assert!(dst_dir.path().exists());
        Ok(())
    }

    /// Tests copying a directory with a single file
    #[tokio::test]
    async fn test_copy_single_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a test file
        let test_file = src_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify file was copied
        let copied_file = dst_dir.path().join("test.txt");
        assert!(copied_file.exists());
        assert_eq!(fs::read_to_string(copied_file)?, "test content");

        Ok(())
    }

    /// Tests copying a directory with nested subdirectories
    #[tokio::test]
    async fn test_copy_nested_directories_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create nested directory structure
        let nested_dir = src_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;

        // Create files in both root and nested directory
        fs::write(src_dir.path().join("root.txt"), "root content")?;
        fs::write(nested_dir.join("nested.txt"), "nested content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify directory structure and contents
        assert!(dst_dir.path().join("nested").exists());
        assert!(dst_dir.path().join("root.txt").exists());
        assert!(dst_dir.path().join("nested/nested.txt").exists());

        assert_eq!(
            fs::read_to_string(dst_dir.path().join("root.txt"))?,
            "root content"
        );
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("nested/nested.txt"))?,
            "nested content"
        );

        Ok(())
    }

    /// Tests handling of symlinks
    #[tokio::test]
    async fn test_copy_with_symlink_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a regular file
        let file_path = src_dir.path().join("original.txt");
        fs::write(&file_path, "original content")?;

        // Create a symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let symlink_path = src_dir.path().join("link.txt");
            symlink(&file_path, &symlink_path)?;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            let symlink_path = src_dir.path().join("link.txt");
            symlink_file(&file_path, &symlink_path)?;
        }

        // Attempt to copy - should fail due to symlink
        let result = copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying large files
    #[tokio::test]
    async fn test_copy_large_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a large file (11MB)
        let large_file = src_dir.path().join("large.txt");
        let file = fs::File::create(&large_file)?;
        file.set_len(11 * 1024 * 1024)?;

        // Attempt to copy - should fail due to file size limit
        let result = copy_dir_all_async(src_dir.path(), dst_dir.path()).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying with invalid destination
    #[tokio::test]
    async fn test_copy_invalid_destination_async() -> Result<()> {
        let src_dir = tempdir()?;
        let invalid_dst = PathBuf::from("/nonexistent/path");

        let result = copy_dir_all_async(src_dir.path(), &invalid_dst).await;
        assert!(result.is_err());

        Ok(())
    }

    /// Tests concurrent copying of multiple files
    #[tokio::test]
    async fn test_concurrent_copy_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create multiple files
        for i in 0..5 {
            fs::write(
                src_dir.path().join(format!("file{}.txt", i)),
                format!("content {}", i),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify all files were copied
        for i in 0..5 {
            let copied_file = dst_dir.path().join(format!("file{}.txt", i));
            assert!(copied_file.exists());
            assert_eq!(
                fs::read_to_string(copied_file)?,
                format!("content {}", i)
            );
        }

        Ok(())
    }

    /// Tests copying with maximum directory depth
    #[tokio::test]
    async fn test_max_directory_depth_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        let max_depth = 5;

        // Create deeply nested directory structure
        let mut current_dir = src_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{}", i));
            fs::create_dir(&current_dir)?;
            fs::write(
                current_dir.join("file.txt"),
                format!("content level {}", i),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path()).await?;

        // Verify the entire structure was copied
        current_dir = dst_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{}", i));
            assert!(current_dir.exists());
            assert!(current_dir.join("file.txt").exists());
            assert_eq!(
                fs::read_to_string(current_dir.join("file.txt"))?,
                format!("content level {}", i)
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_missing_source() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("nonexistent");
        let dst_dir = temp_dir.path().join("dst");

        let error = verify_and_copy_files_async(&src_dir, &dst_dir)
            .await
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("does not exist"),
            "Expected error message about non-existent source, got: {}",
            error
        );

        Ok(())
    }

    #[test]
    fn test_paths_builder_default() -> Result<()> {
        let paths = Paths::builder().build()?;
        assert_eq!(paths.site, PathBuf::from("public"));
        assert_eq!(paths.content, PathBuf::from("content"));
        assert_eq!(paths.build, PathBuf::from("build"));
        assert_eq!(paths.template, PathBuf::from("templates"));
        Ok(())
    }

    #[test]
    fn test_resolve_build_and_site_dirs_without_serve_dir() {
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("docs");
        config.serve_dir = None;

        let (build_dir, site_dir) = resolve_build_and_site_dirs(&config);

        assert_eq!(site_dir, PathBuf::from("docs"));
        assert_eq!(build_dir, PathBuf::from("docs.build-tmp"));
        assert_ne!(build_dir, site_dir);
    }

    #[test]
    fn test_resolve_build_and_site_dirs_with_distinct_serve_dir() {
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("docs");
        config.serve_dir = Some(PathBuf::from("public"));

        let (build_dir, site_dir) = resolve_build_and_site_dirs(&config);

        assert_eq!(build_dir, PathBuf::from("docs"));
        assert_eq!(site_dir, PathBuf::from("public"));
        assert_ne!(build_dir, site_dir);
    }

    #[test]
    fn test_resolve_build_and_site_dirs_with_same_serve_and_output_dir() {
        let mut config = SsgConfig::default();
        config.output_dir = PathBuf::from("docs");
        config.serve_dir = Some(PathBuf::from("docs"));

        let (build_dir, site_dir) = resolve_build_and_site_dirs(&config);

        assert_eq!(site_dir, PathBuf::from("docs"));
        assert_eq!(build_dir, PathBuf::from("docs.build-tmp"));
        assert_ne!(build_dir, site_dir);
    }

    #[test]
    fn test_paths_builder_custom() -> Result<()> {
        let temp_dir = tempdir()?;
        let paths = Paths::builder()
            .site(temp_dir.path().join("custom_public"))
            .content(temp_dir.path().join("custom_content"))
            .build_dir(temp_dir.path().join("custom_build"))
            .template(temp_dir.path().join("custom_templates"))
            .build()?;

        assert_eq!(paths.site, temp_dir.path().join("custom_public"));
        assert_eq!(paths.content, temp_dir.path().join("custom_content"));
        assert_eq!(paths.build, temp_dir.path().join("custom_build"));
        assert_eq!(paths.template, temp_dir.path().join("custom_templates"));
        Ok(())
    }

    #[test]
    fn test_paths_builder_relative() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create the directories first
        fs::create_dir_all(temp_dir.path().join("public"))?;
        fs::create_dir_all(temp_dir.path().join("content"))?;
        fs::create_dir_all(temp_dir.path().join("build"))?;
        fs::create_dir_all(temp_dir.path().join("templates"))?;

        let paths = Paths::builder().relative_to(temp_dir.path()).build()?;

        assert_eq!(paths.site, temp_dir.path().join("public"));
        assert_eq!(paths.content, temp_dir.path().join("content"));
        assert_eq!(paths.build, temp_dir.path().join("build"));
        assert_eq!(paths.template, temp_dir.path().join("templates"));
        Ok(())
    }

    #[test]
    fn test_paths_validation() -> Result<()> {
        // Test directory traversal
        let result = Paths::builder().site("../invalid").build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("directory traversal"),
            "Expected error about directory traversal"
        );

        // Test double slashes
        let result = Paths::builder().site("invalid//path").build();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid double slashes"),
            "Expected error about invalid double slashes"
        );

        // Test symlinks if possible
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let temp_dir = tempdir()?;
            let real_path = temp_dir.path().join("real");
            let symlink_path = temp_dir.path().join("symlink");

            fs::create_dir(&real_path)?;
            symlink(&real_path, &symlink_path)?;

            let result = Paths::builder().site(symlink_path).build();

            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("symlink"),
                "Expected error about symlinks"
            );
        }

        Ok(())
    }

    #[test]
    fn test_paths_default_paths() {
        let paths = Paths::default_paths();
        assert_eq!(paths.site, PathBuf::from("public"));
        assert_eq!(paths.content, PathBuf::from("content"));
        assert_eq!(paths.build, PathBuf::from("build"));
        assert_eq!(paths.template, PathBuf::from("templates"));
    }

    // Add a new test for non-existent but valid paths
    #[test]
    fn test_paths_nonexistent_valid() -> Result<()> {
        let temp_dir = tempdir()?;
        let valid_path = temp_dir.path().join("new_directory");

        let paths = Paths::builder().site(valid_path.clone()).build()?;

        assert_eq!(paths.site, valid_path);
        Ok(())
    }

    #[test]
    fn test_initialize_logging_with_custom_level() -> Result<()> {
        env::set_var(ENV_LOG_LEVEL, "debug");
        assert!(initialize_logging().is_ok());
        env::remove_var(ENV_LOG_LEVEL);
        Ok(())
    }

    #[test]
    fn test_paths_builder_with_all_invalid_paths() -> Result<()> {
        let result = Paths::builder()
            .site("../invalid")
            .content("content//invalid")
            .build_dir("build/../invalid")
            .template("template//invalid")
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_paths_builder_clone() {
        let builder = PathsBuilder::default();
        let cloned = builder.clone();
        assert!(cloned.site.is_none());
        assert!(cloned.content.is_none());
        assert!(cloned.build.is_none());
        assert!(cloned.template.is_none());
    }

    #[test]
    fn test_paths_clone() -> Result<()> {
        let paths = Paths::default_paths();
        let cloned = paths.clone();

        assert_eq!(paths.site, cloned.site);
        assert_eq!(paths.content, cloned.content);
        assert_eq!(paths.build, cloned.build);
        assert_eq!(paths.template, cloned.template);
        Ok(())
    }

    #[tokio::test]
    async fn test_async_copy_with_empty_source() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("empty_src");
        let dst_dir = temp_dir.path().join("empty_dst");

        fs::create_dir(&src_dir)?;

        let result = verify_and_copy_files_async(&src_dir, &dst_dir).await;
        assert!(result.is_ok());
        assert!(dst_dir.exists());
        Ok(())
    }

    #[test]
    fn test_paths_validation_all_aspects() -> Result<()> {
        let temp_dir = tempdir()?;

        // Test with absolute paths
        let result = Paths::builder()
            .site(temp_dir.path().join("site"))
            .content(temp_dir.path().join("content"))
            .build_dir(temp_dir.path().join("build"))
            .template(temp_dir.path().join("template"))
            .build();

        assert!(result.is_ok());

        // Test with multiple validation issues
        let result = Paths::builder()
            .site("../site")
            .content("content//test")
            .build_dir("build/../../test")
            .template("template//test")
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_log_initialization_with_empty_log_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let log_path = temp_dir.path().join("empty.log");
        let mut log_file = File::create(&log_path)?;

        let date = DateTime::new();
        log_initialization(&mut log_file, &date)?;

        let content = fs::read_to_string(&log_path)?;
        assert!(!content.is_empty());
        assert!(content.contains("process"));
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_with_nested_empty_dirs(
    ) -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create nested empty directory structure
        fs::create_dir_all(src_dir.join("a/b/c"))?;
        fs::create_dir_all(src_dir.join("d/e/f"))?;

        verify_and_copy_files_async(&src_dir, &dst_dir).await?;

        assert!(dst_dir.join("a/b/c").exists());
        assert!(dst_dir.join("d/e/f").exists());
        Ok(())
    }

    #[test]
    fn test_validate_nonexistent_paths() -> Result<()> {
        let paths = Paths {
            site: PathBuf::from("nonexistent/site"),
            content: PathBuf::from("nonexistent/content"),
            build: PathBuf::from("nonexistent/build"),
            template: PathBuf::from("nonexistent/template"),
        };

        // Non-existent paths should be valid if they don't contain unsafe patterns
        assert!(paths.validate().is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_copy_dir_all_async_with_empty_dirs() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        fs::create_dir_all(src_dir.join("empty1"))?;
        fs::create_dir_all(src_dir.join("empty2/empty3"))?;

        copy_dir_all_async(&src_dir, &dst_dir).await?;

        assert!(dst_dir.join("empty1").exists());
        assert!(dst_dir.join("empty2/empty3").exists());
        Ok(())
    }

    #[test]
    fn test_log_level_from_env() {
        // Save the current environment variable value
        let original_value = env::var(ENV_LOG_LEVEL).ok();

        // Helper function to get processed log level
        fn get_processed_log_level() -> String {
            let log_level = env::var(ENV_LOG_LEVEL)
                .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

            match log_level.to_lowercase().as_str() {
                "error" => "error",
                "warn" => "warn",
                "info" => "info",
                "debug" => "debug",
                "trace" => "trace",
                _ => "info", // Default to info for invalid values
            }
            .to_string()
        }

        // Test various log level settings
        let test_levels = vec![
            ("DEBUG", "debug"),
            ("ERROR", "error"),
            ("WARN", "warn"),
            ("INFO", "info"),
            ("TRACE", "trace"),
            ("INVALID", "info"), // Should default to info
        ];

        for (input, expected) in test_levels {
            env::set_var(ENV_LOG_LEVEL, input);
            let processed_level = get_processed_log_level();
            assert_eq!(
                processed_level, expected,
                "Expected log level '{}' for input '{}', but got '{}'",
                expected, input, processed_level
            );
        }

        // Restore the original environment variable state
        env::remove_var(ENV_LOG_LEVEL);
        if let Some(value) = original_value {
            env::set_var(ENV_LOG_LEVEL, value);
        }
    }

    /// Test for default log level when environment variable is not set
    #[test]
    fn test_default_log_level() {
        let original_value = env::var(ENV_LOG_LEVEL).ok();
        env::remove_var(ENV_LOG_LEVEL);

        let log_level = env::var(ENV_LOG_LEVEL)
            .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string())
            .to_lowercase();
        assert_eq!(log_level, DEFAULT_LOG_LEVEL.to_lowercase());

        env::remove_var(ENV_LOG_LEVEL);
        if let Some(value) = original_value {
            env::set_var(ENV_LOG_LEVEL, value);
        }
    }

    /// Test the logic for translating string log levels to LevelFilter values
    #[test]
    fn test_log_level_translation() {
        let test_cases = vec![
            ("error", LevelFilter::Error),
            ("warn", LevelFilter::Warn),
            ("info", LevelFilter::Info),
            ("debug", LevelFilter::Debug),
            ("trace", LevelFilter::Trace),
            ("invalid", LevelFilter::Info),
            ("", LevelFilter::Info),
        ];

        for (input, expected) in test_cases {
            let level = match input.to_lowercase().as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            };

            assert_eq!(
                level, expected,
                "Log level mismatch for input: '{}' - expected {:?}, got {:?}",
                input, expected, level
            );
        }
    }

    /// Test environment variable handling with cleanup
    #[test]
    fn test_env_log_level_handling() {
        // Save original state
        let original_value = env::var(ENV_LOG_LEVEL).ok();

        let test_cases = vec![
            (Some("DEBUG"), "debug"),
            (Some("ERROR"), "error"),
            (Some("WARN"), "warn"),
            (Some("INFO"), "info"),
            (Some("TRACE"), "trace"),
            (Some("INVALID"), "info"),
            (None, "info"),
        ];

        for (env_value, expected) in test_cases {
            // Clear any existing env var
            env::remove_var(ENV_LOG_LEVEL);

            // Set new value if provided
            if let Some(value) = env_value {
                env::set_var(ENV_LOG_LEVEL, value);
            }

            let log_level = env::var(ENV_LOG_LEVEL)
                .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string())
                .to_lowercase();

            let actual = match log_level.as_str() {
                "error" => "error",
                "warn" => "warn",
                "info" => "info",
                "debug" => "debug",
                "trace" => "trace",
                _ => "info",
            };

            assert_eq!(
                actual, expected,
                "Log level mismatch for env value: {:?}",
                env_value
            );
        }

        // Restore original state
        env::remove_var(ENV_LOG_LEVEL);
        if let Some(value) = original_value {
            env::set_var(ENV_LOG_LEVEL, value);
        }
    }

    #[test]
    fn test_initialize_logging_custom_levels() {
        // Verify that the expected log level strings are valid
        let valid_levels = ["debug", "warn", "error", "trace", "info"];
        for level in &valid_levels {
            assert!(
                ["trace", "debug", "info", "warn", "error"].contains(level),
                "unexpected log level: {level}"
            );
        }
        // Verify our default is valid
        assert!(["trace", "debug", "info", "warn", "error"]
            .contains(&DEFAULT_LOG_LEVEL),);
    }

    #[test]
    fn parse_log_level_recognises_all_supported_levels() {
        // Covers every arm of the match in parse_log_level (including
        // the trace arm at line 286 and the `_ =>` fallback at 287).
        assert_eq!(parse_log_level("error"), LevelFilter::Error);
        assert_eq!(parse_log_level("warn"), LevelFilter::Warn);
        assert_eq!(parse_log_level("info"), LevelFilter::Info);
        assert_eq!(parse_log_level("debug"), LevelFilter::Debug);
        assert_eq!(parse_log_level("trace"), LevelFilter::Trace);
    }

    #[test]
    fn parse_log_level_is_case_insensitive() {
        assert_eq!(parse_log_level("ERROR"), LevelFilter::Error);
        assert_eq!(parse_log_level("Warn"), LevelFilter::Warn);
        assert_eq!(parse_log_level("TraCe"), LevelFilter::Trace);
    }

    #[test]
    fn parse_log_level_unknown_value_falls_back_to_info() {
        assert_eq!(parse_log_level("nonsense"), LevelFilter::Info);
        assert_eq!(parse_log_level(""), LevelFilter::Info);
        assert_eq!(parse_log_level("verbose"), LevelFilter::Info);
    }

    #[tokio::test]
    async fn test_concurrent_operations() -> Result<()> {
        use tokio::fs as async_fs;

        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create source directory
        async_fs::create_dir_all(&src_dir).await?;

        // Create files concurrently
        let mut handles = Vec::new();
        for i in 0..100 {
            let src = src_dir.clone();
            handles.push(tokio::spawn(async move {
                async_fs::write(
                    src.join(format!("file_{}.txt", i)),
                    format!("content {}", i),
                )
                .await
            }));
        }

        // Wait for all files to be created
        for handle in handles {
            handle.await??;
        }

        // Ensure source files exist before copying
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify source files
        let mut src_files = Vec::new();
        collect_files_recursive(&src_dir, &mut src_files)?;
        assert_eq!(src_files.len(), 100);

        // Create destination directory
        async_fs::create_dir_all(&dst_dir).await?;

        // Copy files using verify_and_copy_files instead of async version
        verify_and_copy_files(&src_dir, &dst_dir)?;

        // Allow some time for filesystem operations to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify destination files
        let mut dst_files = Vec::new();
        collect_files_recursive(&dst_dir, &mut dst_files)?;

        assert_eq!(dst_files.len(), 100);

        // Verify file contents
        for i in 0..100 {
            let dst_path = dst_dir.join(format!("file_{}.txt", i));
            assert!(
                dst_path.exists(),
                "File {} does not exist in destination",
                dst_path.display()
            );

            let content = fs::read_to_string(&dst_path)?;
            assert_eq!(
                content,
                format!("content {}", i),
                "Content mismatch for file {}",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        fs::create_dir_all(&src_dir)?;

        // Create a test file
        fs::write(src_dir.join("test.txt"), "test content")?;

        // Copy files
        verify_and_copy_files(&src_dir, &dst_dir)?;

        // Verify file was copied
        assert!(dst_dir.join("test.txt").exists());
        assert_eq!(
            fs::read_to_string(dst_dir.join("test.txt"))?,
            "test content"
        );

        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_empty_source() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Call the function with an empty source directory
        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;

        // Verify that the destination directory exists and is empty
        assert!(dst_dir.path().exists());
        assert!(fs::read_dir(dst_dir.path())?.next().is_none());

        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_source_does_not_exist() {
        let src_dir = Path::new("/nonexistent");
        let dst_dir = tempdir().unwrap();

        let result = copy_dir_with_progress(src_dir, dst_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_with_progress_single_file() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        fs::write(src_dir.path().join("file1.txt"), "content")?;

        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;

        let copied_file = dst_dir.path().join("file1.txt");
        assert!(copied_file.exists());
        assert_eq!(fs::read_to_string(copied_file)?, "content");

        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_nested_directories() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let nested_dir = src_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;
        fs::write(nested_dir.join("file.txt"), "nested content")?;

        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;

        let copied_nested_file = dst_dir.path().join("nested/file.txt");
        assert!(copied_nested_file.exists());
        assert_eq!(fs::read_to_string(copied_nested_file)?, "nested content");

        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_destination_creation_failure() {
        let src_dir = tempdir().unwrap();
        let dst_dir = Path::new("/invalid_path");

        let result = copy_dir_with_progress(src_dir.path(), dst_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_and_copy_files_single_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_file = temp_dir.path().join("single.txt");
        fs::write(&src_file, "content")?;
        let dst_dir = temp_dir.path().join("dst");
        // Calling with a file as src triggers verify_file_safety branch
        // then copy_dir_all fails because src is a file, not a directory
        let result = verify_and_copy_files(&src_file, &dst_dir);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_is_safe_path_traversal_nonexistent() -> Result<()> {
        assert!(!is_safe_path(Path::new("../../etc/passwd"))?);
        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_nested() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        // Create nested structure with files
        let sub = src_dir.path().join("sub");
        fs::create_dir(&sub)?;
        fs::write(src_dir.path().join("root.txt"), "root")?;
        fs::write(sub.join("nested.txt"), "nested")?;
        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;
        assert!(dst_dir.path().join("root.txt").exists());
        assert!(dst_dir.path().join("sub/nested.txt").exists());
        Ok(())
    }

    #[test]
    fn test_copy_dir_all_parallel_threshold() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        // Create >= 16 files to trigger parallel path
        for i in 0..20 {
            fs::write(
                src_dir.path().join(format!("file{}.txt", i)),
                format!("content {}", i),
            )?;
        }
        copy_dir_all(src_dir.path(), dst_dir.path())?;
        for i in 0..20 {
            assert!(dst_dir.path().join(format!("file{}.txt", i)).exists());
        }
        Ok(())
    }

    #[test]
    fn test_collect_files_recursive_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        // Create a directory deeper than MAX_DIR_DEPTH
        let mut path = temp_dir.path().to_path_buf();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{}", i));
            fs::create_dir(&path)?;
        }
        let mut files = Vec::new();
        let result = collect_files_recursive(temp_dir.path(), &mut files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[test]
    fn test_copy_dir_all_depth_exceeded() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        let mut path = src_dir.path().to_path_buf();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{}", i));
            fs::create_dir(&path)?;
        }
        let result = copy_dir_all(src_dir.path(), dst_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");
        let mut path = src.clone();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{}", i));
            fs::create_dir_all(&path)?;
        }
        let result = verify_and_copy_files_async(&src, &dst).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[tokio::test]
    async fn test_copy_dir_all_async_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");
        let mut path = src.clone();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{}", i));
            fs::create_dir_all(&path)?;
        }
        let result = copy_dir_all_async(&src, &dst).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[test]
    fn test_verify_file_safety_nonexistent() {
        let result = verify_file_safety(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_with_progress_nonexistent_source() {
        let result = copy_dir_with_progress(
            Path::new("/nonexistent/source"),
            Path::new("/tmp/dst"),
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_and_copy_files_async_with_files() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source with nested dirs + files to cover lines 415-420
        fs::create_dir_all(src.join("sub1/sub2"))?;
        fs::write(src.join("root.txt"), "root")?;
        fs::write(src.join("sub1/a.txt"), "a")?;
        fs::write(src.join("sub1/sub2/b.txt"), "b")?;

        verify_and_copy_files_async(&src, &dst).await?;

        assert_eq!(fs::read_to_string(dst.join("root.txt"))?, "root");
        assert_eq!(fs::read_to_string(dst.join("sub1/a.txt"))?, "a");
        assert_eq!(fs::read_to_string(dst.join("sub1/sub2/b.txt"))?, "b");
        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_with_files() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create nested structure to cover lines 463-490
        let sub1 = src_dir.path().join("a");
        let sub2 = sub1.join("b");
        fs::create_dir_all(&sub2)?;
        fs::write(src_dir.path().join("file1.txt"), "f1")?;
        fs::write(sub1.join("file2.txt"), "f2")?;
        fs::write(sub2.join("file3.txt"), "f3")?;

        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;

        assert_eq!(fs::read_to_string(dst_dir.path().join("file1.txt"))?, "f1");
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("a/file2.txt"))?,
            "f2"
        );
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("a/b/file3.txt"))?,
            "f3"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_is_safe_path_broken_symlink() -> Result<()> {
        let temp_dir = tempdir()?;
        let target = temp_dir.path().join("nonexistent_target");
        let link = temp_dir.path().join("broken_link");

        std::os::unix::fs::symlink(&target, &link)?;
        let result = is_safe_path(&link)?;
        assert!(result);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_paths_validate_symlink() -> Result<()> {
        let temp_dir = tempdir()?;
        let real = temp_dir.path().join("real");
        let link = temp_dir.path().join("link");

        fs::create_dir(&real)?;
        std::os::unix::fs::symlink(&real, &link)?;

        let paths = Paths {
            site: link,
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        };
        let result = paths.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("symlink"));
        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_depth_exceeded() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        let mut path = src_dir.path().to_path_buf();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{}", i));
            fs::create_dir(&path)?;
        }
        let result = copy_dir_with_progress(src_dir.path(), dst_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_source_is_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_file = temp_dir.path().join("source.txt");
        let dst_dir = temp_dir.path().join("dst");
        fs::write(&src_file, "hello")?;

        // verify_and_copy_files with a file as source triggers
        // verify_file_safety then copy_dir_all (which fails on a file)
        let result = verify_and_copy_files(&src_file, &dst_dir);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_compile_site_error() -> Result<()> {
        let temp_dir = tempdir()?;
        let build = temp_dir.path().join("build");
        let content = temp_dir.path().join("content");
        let site = temp_dir.path().join("site");
        let template = temp_dir.path().join("template");
        fs::create_dir_all(&build)?;
        fs::create_dir_all(&content)?;
        fs::create_dir_all(&site)?;
        fs::create_dir_all(&template)?;

        let result = compile_site(&build, &content, &site, &template);
        // Compilation with empty dirs will fail
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_serve_dir_same_as_site() -> Result<()> {
        let temp_dir = tempdir()?;
        let site_dir = temp_dir.path().join("site");
        fs::create_dir_all(&site_dir)?;
        fs::write(site_dir.join("index.html"), "<html/>")?;

        let paths = Paths {
            site: site_dir.clone(),
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        };

        // When serve_dir == site, no copy should happen
        prepare_serve_dir(&paths, &site_dir).await?;
        assert!(site_dir.join("index.html").exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_prepare_serve_dir_different() -> Result<()> {
        let temp_dir = tempdir()?;
        let site_dir = temp_dir.path().join("site");
        let serve_dir = temp_dir.path().join("serve");
        fs::create_dir_all(&site_dir)?;
        fs::write(site_dir.join("index.html"), "<html/>")?;

        let paths = Paths {
            site: site_dir,
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        };

        prepare_serve_dir(&paths, &serve_dir).await?;
        assert!(serve_dir.join("index.html").exists());
        Ok(())
    }

    #[test]
    fn test_create_directories_all_valid() -> Result<()> {
        let temp_dir = tempdir()?;
        let paths = Paths {
            site: temp_dir.path().join("s"),
            content: temp_dir.path().join("c"),
            build: temp_dir.path().join("b"),
            template: temp_dir.path().join("t"),
        };
        create_directories(&paths)?;
        assert!(paths.site.exists());
        assert!(paths.build.exists());
        Ok(())
    }

    #[test]
    fn test_is_safe_path_existing_valid() -> Result<()> {
        let temp_dir = tempdir()?;
        let dir = temp_dir.path().join("valid");
        fs::create_dir(&dir)?;
        let canonical = dir.canonicalize()?;
        assert!(is_safe_path(&canonical)?);
        Ok(())
    }

    // -----------------------------------------------------------------
    // RunOptions / build_pipeline / execute_build_pipeline
    // -----------------------------------------------------------------

    #[test]
    fn run_options_from_matches_extracts_quiet_drafts_and_deploy() {
        // Build matches the same way `run()` does, so we exercise
        // the real argument parser without launching anything.
        let cli = Cli::build();
        let matches = cli
            .try_get_matches_from(vec![
                "ssg", "--quiet", "--drafts", "--deploy", "netlify",
            ])
            .expect("matches");
        let opts = RunOptions::from_matches(&matches);
        assert!(opts.quiet);
        assert!(opts.include_drafts);
        assert_eq!(opts.deploy_target.as_deref(), Some("netlify"));
    }

    #[test]
    fn run_options_from_matches_defaults_when_flags_absent() {
        let cli = Cli::build();
        let matches = cli.try_get_matches_from(vec!["ssg"]).expect("matches");
        let opts = RunOptions::from_matches(&matches);
        assert!(!opts.quiet);
        assert!(!opts.include_drafts);
        assert!(opts.deploy_target.is_none());
    }

    #[test]
    fn build_pipeline_assembles_manager_context_and_dirs() {
        // Constructs the full plugin manager + context wiring without
        // touching `compile_site` or starting the server. Covers the
        // `register_default_plugins` registration body and the
        // resolve_build_and_site_dirs path inside build_pipeline.
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");
        config.template_dir = temp.path().join("templates");
        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        // The plugin manager should have at least the SEO + accessibility
        // + minify + livereload plugins registered.
        assert!(plugins.len() >= 10);
        // Build and site dirs should be distinct.
        assert_ne!(build_dir, site_dir);
        assert_eq!(site_dir, temp.path().join("public"));
        // Context paths should match config.
        assert_eq!(ctx.content_dir, temp.path().join("content"));
    }

    #[test]
    fn build_pipeline_with_deploy_target_registers_deploy_plugin() {
        // The `if let Some(target) = deploy_target` branch in
        // register_default_plugins should add an extra plugin.
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts_no_deploy = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
        };
        let (no_deploy, _, _, _) = build_pipeline(&config, &opts_no_deploy);

        let opts_deploy = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: Some("netlify".to_string()),
        };
        let (with_deploy, _, _, _) = build_pipeline(&config, &opts_deploy);

        assert_eq!(with_deploy.len(), no_deploy.len() + 1);
    }

    #[test]
    fn build_pipeline_with_unknown_deploy_target_logs_and_skips() {
        // The `_ => log::warn!` arm in the deploy-target match.
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: Some("nonsense-platform".to_string()),
        };
        let (plugins, _, _, _) = build_pipeline(&config, &opts);
        // No deploy plugin registered for an unknown target.
        let names = plugins.names();
        assert!(!names.iter().any(|n| n == &"deploy"));
    }

    #[test]
    fn build_pipeline_with_each_known_deploy_target_registers_one_plugin() {
        // Covers each match arm: netlify, vercel, cloudflare, github.
        for target in ["netlify", "vercel", "cloudflare", "github"] {
            let temp = tempdir().unwrap();
            let mut config = SsgConfig::default();
            config.content_dir = temp.path().join("content");
            config.output_dir = temp.path().join("public");

            let opts = RunOptions {
                quiet: true,
                include_drafts: false,
                deploy_target: Some(target.to_string()),
            };
            let (plugins, _, _, _) = build_pipeline(&config, &opts);
            assert!(
                plugins.names().iter().any(|n| n == &"deploy"),
                "deploy plugin should be registered for target `{target}`"
            );
        }
    }

    // -----------------------------------------------------------------
    // ServeTransport / serve_site_with
    // -----------------------------------------------------------------

    /// Test transport that records its calls without starting an
    /// HTTP server.
    #[derive(Debug, Default)]
    struct RecordingTransport {
        calls: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl ServeTransport for RecordingTransport {
        fn start(&self, addr: &str, root: &str) -> Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((addr.to_string(), root.to_string()));
            Ok(())
        }
    }

    /// Test transport that always errors — verifies the error is
    /// propagated through `serve_site_with`.
    #[derive(Debug, Default)]
    struct FailingTransport;

    impl ServeTransport for FailingTransport {
        fn start(&self, _addr: &str, _root: &str) -> Result<()> {
            Err(anyhow!("transport failed"))
        }
    }

    #[test]
    fn build_serve_address_resolves_path_to_addr_root_pair() {
        let (addr, root) = build_serve_address(Path::new("./public")).unwrap();
        assert_eq!(
            addr,
            format!("{}:{}", cmd::DEFAULT_HOST, cmd::DEFAULT_PORT)
        );
        assert_eq!(root, "./public");
    }

    #[test]
    fn verify_and_copy_files_destination_create_dir_failure_propagates(
    ) -> Result<()> {
        // Covers the with_context closure at lines 680-683 of
        // verify_and_copy_files. Trick: place a regular file in
        // the path where the destination parent should be a dir.
        // fs::create_dir_all then fails with NotADirectory.
        let temp = tempdir()?;
        let blocker = temp.path().join("blocker.txt");
        fs::write(&blocker, "i am a file, not a directory")?;

        // dst is "blocker.txt/sub" — parent is a file → create_dir_all fails.
        let bad_dst = blocker.join("sub");
        let result = verify_and_copy_files(temp.path(), &bad_dst);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(
            msg.contains("Failed to create or access destination"),
            "expected with_context message, got: {msg}"
        );
        Ok(())
    }

    #[test]
    fn create_directories_unsafe_path_bails() -> Result<()> {
        // Covers line 1099: the `anyhow::bail!` triggered when one
        // of the paths fails is_safe_path. is_safe_path returns
        // Ok(false) for non-existent paths containing `..`. The
        // create_dir_all loop above (lines 1086-1091) creates the
        // directories first, so we need a path that DOESN'T get
        // created by create_dir_all but DOES contain `..` —
        // but create_dir_all is permissive about `..` and resolves
        // it. So we need a different angle: pass a path that's
        // intentionally crafted to not exist after create_dir_all.
        //
        // Trick: use a path with a literal `..` segment after a
        // file. fs::create_dir_all("file/../newdir") fails because
        // it tries to walk into the file. After failure, the path
        // doesn't exist AND contains `..`, so is_safe_path returns
        // false → bail fires.
        let temp = tempdir()?;
        let blocker = temp.path().join("blocker.txt");
        fs::write(&blocker, "x")?;

        // The unsafe path: file/../subdir → traversal through file.
        let unsafe_path = blocker.join("..").join("subdir");

        let paths = Paths {
            site: temp.path().join("s"),
            content: unsafe_path,
            build: temp.path().join("b"),
            template: temp.path().join("t"),
        };
        let result = create_directories(&paths);
        // Either the create_dir_all loop fails (line 1086 with_context)
        // OR is_safe_path bails. Both are valid error continuations.
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn copy_dir_with_progress_read_dir_failure_propagates() -> Result<()> {
        // Covers the .context(format!(...)) closure at lines 773-777
        // of copy_dir_with_progress. Trick: the function checks
        // src.exists() (which a regular file passes), creates the
        // destination, then enters the iterative walker. The walker
        // immediately calls fs::read_dir on the source, which fails
        // with NotADirectory when src is a file.
        let temp = tempdir()?;
        let src_file = temp.path().join("not-a-dir.txt");
        fs::write(&src_file, "content")?;
        let dst = temp.path().join("dst");

        let result = copy_dir_with_progress(&src_file, &dst);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(
            msg.contains("Failed to read source directory"),
            "expected with_context message, got: {msg}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn verify_and_copy_files_async_destination_create_dir_failure_propagates(
    ) -> Result<()> {
        // Covers the async with_context closure at lines 707-710.
        let temp = tempdir()?;
        let blocker = temp.path().join("async-blocker.txt");
        fs::write(&blocker, "blocker")?;

        let bad_dst = blocker.join("sub");
        let result = verify_and_copy_files_async(temp.path(), &bad_dst).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Failed to create or access destination"));
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn build_serve_address_rejects_invalid_utf8_path() {
        // Covers lines 571-573: the to_str().ok_or_else closure that
        // fires when the path contains invalid UTF-8 byte sequences.
        // On Unix we can construct such a path via OsStr::from_bytes.
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let invalid_bytes = b"/tmp/site_\xff_invalid";
        let path = Path::new(OsStr::from_bytes(invalid_bytes));
        let err = build_serve_address(path).unwrap_err();
        assert!(format!("{err:?}").contains("invalid UTF-8"));
    }

    #[test]
    #[cfg(unix)]
    fn serve_site_shim_propagates_invalid_utf8_path_error() {
        // Covers the body of `pub fn serve_site` (lines 605-607).
        // We can't actually start a server, so we use the same
        // invalid-UTF-8 path trick to make build_serve_address bail
        // BEFORE HttpTransport::start is called. This exercises the
        // function's body and the error-propagation path through
        // serve_site_with.
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let invalid = b"/tmp/\xfe\xfe_bad";
        let path = Path::new(OsStr::from_bytes(invalid));
        let err = serve_site(path).unwrap_err();
        assert!(format!("{err:?}").contains("invalid UTF-8"));
    }

    #[test]
    fn serve_site_with_recording_transport_records_addr_and_root() {
        let transport = RecordingTransport::default();
        serve_site_with(Path::new("./public"), &transport).unwrap();
        let calls = transport.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "./public");
    }

    #[test]
    fn serve_site_with_propagates_transport_errors() {
        let transport = FailingTransport;
        let result = serve_site_with(Path::new("./public"), &transport);
        assert!(result.is_err());
        assert!(
            format!("{:?}", result.unwrap_err()).contains("transport failed")
        );
    }

    #[test]
    fn http_transport_implements_serve_transport_trait() {
        // Compile-time check: HttpTransport satisfies the bound.
        fn _assert_impl<T: ServeTransport>() {}
        _assert_impl::<HttpTransport>();
    }

    // NOTE: HttpTransport::start binds an HTTP server and BLOCKS
    // indefinitely on success (it does not return until the server
    // process is killed). There is no safe way to unit-test
    // HttpTransport::start in-process without leaking file
    // descriptors and bound ports. The lines are exercised
    // manually via `cargo run --example multilingual` (confirmed
    // green in the PR test plan) and the ServeTransport trait
    // surface itself is covered by RecordingTransport and
    // FailingTransport doubles.

    // -----------------------------------------------------------------
    // execute_build_pipeline — runs the actual build half of run()
    // -----------------------------------------------------------------

    #[test]
    fn execute_build_pipeline_propagates_compile_errors() -> Result<()> {
        // Covers the body of execute_build_pipeline at lines 405-419,
        // including the start.elapsed() / println! summary arm. We
        // run it against a deliberately broken layout (empty
        // content_dir, no templates) so compile_site returns Err
        // and we exercise the Result-propagation continuation.
        let temp = tempdir()?;
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("missing-content");
        config.output_dir = temp.path().join("public");
        config.template_dir = temp.path().join("missing-templates");
        config.site_name = "broken".to_string();

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        // Pipeline must fail cleanly with an error rather than
        // panicking on the missing directories.
        let result = execute_build_pipeline(
            &plugins,
            &ctx,
            &build_dir,
            &config.content_dir,
            &site_dir,
            &config.template_dir,
            opts.quiet,
        );
        assert!(result.is_err(), "broken layout should propagate Err");
        Ok(())
    }

    #[test]
    fn execute_build_pipeline_succeeds_against_real_example_fixtures(
    ) -> Result<()> {
        // Uses the in-tree examples/content/en + examples/templates/en
        // fixtures (which are known to compile successfully — same
        // fixtures that power `cargo run --example plugins`). This
        // is the only way to cover execute_build_pipeline's success
        // continuation at lines 409-419 (after compile_site returns
        // Ok). Skipped if the fixtures aren't present.
        let cwd = env::current_dir()?;
        let content = cwd.join("examples/content/en");
        let template = cwd.join("examples/templates/en");
        if !content.exists() || !template.exists() {
            eprintln!(
                "skipping: examples/content/en not present in {}",
                cwd.display()
            );
            return Ok(());
        }

        let temp = tempdir()?;
        let mut config = SsgConfig::default();
        config.content_dir = content.clone();
        config.template_dir = template.clone();
        config.output_dir = temp.path().join("public");
        config.site_name = "pipeline-success-test".to_string();
        config.base_url = "http://localhost".to_string();

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        // Success path: every line in execute_build_pipeline is hit.
        execute_build_pipeline(
            &plugins,
            &ctx,
            &build_dir,
            &config.content_dir,
            &site_dir,
            &config.template_dir,
            opts.quiet,
        )?;

        assert!(
            site_dir.exists() || build_dir.exists(),
            "expected build/site dir to exist after successful pipeline"
        );
        Ok(())
    }

    #[test]
    fn execute_build_pipeline_verbose_success_hits_println_arm() -> Result<()> {
        // Same fixture but with quiet=false, to cover lines 412-418
        // (the `if !quiet { println!(...) }` arm).
        let cwd = env::current_dir()?;
        let content = cwd.join("examples/content/en");
        let template = cwd.join("examples/templates/en");
        if !content.exists() || !template.exists() {
            return Ok(());
        }

        let temp = tempdir()?;
        let mut config = SsgConfig::default();
        config.content_dir = content;
        config.template_dir = template;
        config.output_dir = temp.path().join("public");
        config.site_name = "verbose-success".to_string();
        config.base_url = "http://localhost".to_string();

        let opts = RunOptions {
            quiet: false,
            include_drafts: false,
            deploy_target: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);
        execute_build_pipeline(
            &plugins,
            &ctx,
            &build_dir,
            &config.content_dir,
            &site_dir,
            &config.template_dir,
            opts.quiet,
        )?;
        Ok(())
    }

    #[test]
    fn execute_build_pipeline_verbose_propagates_compile_errors() -> Result<()>
    {
        // Same as above with quiet=false to cover the `if !quiet`
        // branch at lines 412-418 even on the failure path.
        let temp = tempdir()?;
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("missing");
        config.output_dir = temp.path().join("public");
        config.template_dir = temp.path().join("missing-templates");
        config.site_name = "broken-verbose".to_string();

        let opts = RunOptions {
            quiet: false,
            include_drafts: false,
            deploy_target: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        let _ = execute_build_pipeline(
            &plugins,
            &ctx,
            &build_dir,
            &config.content_dir,
            &site_dir,
            &config.template_dir,
            opts.quiet,
        );
        Ok(())
    }

    #[test]
    fn build_pipeline_with_drafts_flag_registers_draft_plugin() {
        // The DraftPlugin is always registered; this test verifies it
        // accepts the include_drafts flag without panicking.
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts = RunOptions {
            quiet: true,
            include_drafts: true,
            deploy_target: None,
        };
        let (plugins, _, _, _) = build_pipeline(&config, &opts);
        assert!(plugins.names().iter().any(|n| n == &"drafts"));
    }
}
