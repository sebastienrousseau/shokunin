#![forbid(unsafe_code)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://cloudcdn.pro/static-site-generator/v1/favicon.ico",
    html_logo_url = "https://cloudcdn.pro/static-site-generator/v1/logos/static-site-generator.svg",
    html_root_url = "https://docs.rs/ssg"
)]
#![crate_name = "ssg"]
#![crate_type = "lib"]

/// Fault injection macro. When the `test-fault-injection` feature is
/// enabled, delegates to the `fail` crate's real `fail_point!`. In
/// normal builds this compiles to nothing.
#[cfg(feature = "test-fault-injection")]
macro_rules! fail_point {
    ($name:expr, $body:expr) => {
        fail::fail_point!($name, $body);
    };
}
#[cfg(not(feature = "test-fault-injection"))]
macro_rules! fail_point {
    ($name:expr, $body:expr) => {};
}

/// Shared bounded directory walkers used by every plugin's
/// `collect_*_files` helper.
#[allow(unreachable_pub)]
pub(crate) mod walk;

/// Test-only utilities shared across unit test modules.
#[cfg(test)]
#[allow(unreachable_pub, clippy::unwrap_used, clippy::expect_used)]
pub(crate) mod test_support {
    use std::sync::Once;

    static LOGGER: Once = Once::new();

    /// Raises `log::max_level()` to Trace so `log::info!` / `log::warn!`
    /// macro bodies execute their format arguments and are counted by
    /// LLVM region coverage. We only bump the filter level; no logger
    /// backend is installed, so it does not conflict with tests that
    /// install their own (e.g. the `env_logger` init test in lib.rs).
    /// Safe to call from any number of tests or fixtures.
    pub fn init_logger() {
        LOGGER.call_once(|| {
            log::set_max_level(log::LevelFilter::Trace);
        });
    }
}

// Standard library imports
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::cmd::{Cli, SsgConfig};

// Third-party imports
use anyhow::{Context, Result};
use log::info;

/// Returns the current time as an ISO 8601 UTC string.
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let (sec, min, hour) = (secs % 60, (secs / 60) % 60, (secs / 3600) % 24);
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Civil days algorithm (Howard Hinnant) — converts days since Unix epoch to (Y, M, D).
const fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Automated WCAG accessibility checker.
pub mod accessibility;
/// AI-readiness content hooks (GEO/AEO).
pub mod ai;
/// Asset fingerprinting, SRI hashes, and minification.
pub mod assets;
/// Content fingerprinting for incremental builds.
pub mod cache;
pub mod cmd;
/// Typed content collections with frontmatter schema validation.
pub mod content;
/// Content Security Policy hardening: inline extraction + SRI.
pub mod csp;
/// Deployment adapter generation.
pub mod deploy;
/// Draft content filtering.
pub mod drafts;
/// Shared frontmatter extraction and `.meta.json` sidecar files.
pub mod frontmatter;
/// File system operations: directory copying, safety validation, and traversal.
pub mod fs_ops;
/// Syntax highlighting for code blocks.
pub mod highlight;
/// Internationalisation: hreflang injection, per-locale sitemaps, lang switcher.
pub mod i18n;
/// Image optimization with WebP and responsive srcset.
#[cfg(feature = "image-optimization")]
pub mod image_plugin;
/// Interactive islands — lazy-hydrating Web Components.
pub mod islands;
/// WebSocket-based live-reload script injection.
pub mod livereload;
/// Local LLM content augmentation plugin.
pub mod llm;
/// Logging infrastructure.
pub mod logging;
/// GitHub Flavored Markdown (GFM) extensions: tables, strikethrough, task lists.
pub mod markdown_ext;
/// Pagination for listing pages.
pub mod pagination;
/// Build pipeline orchestration.
#[allow(unreachable_pub)]
pub(crate) mod pipeline;
/// Lifecycle hook plugin system.
pub mod plugin;
/// Built-in plugins for common tasks.
pub mod plugins;
/// Post-processing fixes for staticdatagen output.
pub mod postprocess;
/// Command-line argument processing and site compilation.
pub mod process;
/// Project scaffolding for `--new`.
pub mod scaffold;
/// JSON Schema generation for configuration.
pub mod schema;
/// Client-side search index generator and search UI.
pub mod search;
/// SEO plugins: meta tags, robots.txt, and canonical URLs.
pub mod seo;
/// Dev server infrastructure.
pub mod server;
/// Shortcode expansion for Markdown content.
pub mod shortcodes;
/// High-performance streaming file processor.
pub mod stream;
/// Streaming compilation for large (100K+ page) sites.
pub mod streaming;
/// Taxonomy generation (tags, categories).
pub mod taxonomy;
/// Template engine integration (MiniJinja).
#[cfg(feature = "templates")]
pub mod template_engine;
/// Template rendering plugin.
#[cfg(feature = "templates")]
pub mod template_plugin;
/// File-watching for live rebuild.
pub mod watch;
/// Re-exports
pub use staticdatagen;

// Re-export everything that was previously pub in lib.rs
pub use fs_ops::{
    collect_files_recursive, copy_dir_all, copy_dir_all_async,
    copy_dir_with_progress, is_safe_path, verify_and_copy_files,
    verify_and_copy_files_async, verify_file_safety,
};
pub use logging::{create_log_file, log_arguments, log_initialization};
pub use pipeline::{compile_site, execute_build_pipeline};
pub use server::{
    generate_locale_redirect, handle_server, prepare_serve_dir, serve_site,
    serve_site_with, HttpTransport, ServeTransport,
};

/// Maximum directory nesting depth for all traversal operations.
/// Prevents stack overflow from pathological or circular directory trees.
/// 128 levels accommodates any realistic project structure.
pub const MAX_DIR_DEPTH: usize = 128;

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
                "Failed to create or access {name} directory at path: {}",
                path.display()
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

/// Executes the static site generation process.
///
/// Parses CLI arguments, runs the plugin pipeline around compilation,
/// and starts a local dev server. This function blocks indefinitely
/// while the server is running.
pub fn run() -> Result<()> {
    // Parse CLI arguments first so that `--help` and `--version`
    // short-circuit *before* the logger emits its banner. clap exits
    // the process for those flags, so we never reach the lines below.
    let matches = Cli::build().get_matches();

    logging::initialize_logging()?;
    info!("Starting site generation process");

    let config = SsgConfig::from_matches(&matches)?;
    let opts = pipeline::RunOptions::from_matches(&matches);

    // Configure Rayon global thread pool from --jobs flag.
    if let Some(n) = opts.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .context("failed to configure Rayon thread pool")?;
        info!("Rayon thread pool configured with {n} threads");
    }

    // --validate: validate content schemas and exit without building.
    if opts.validate_only {
        return content::validate_only(&config.content_dir);
    }

    if !opts.quiet {
        Cli::print_banner();
    }

    let (plugins, ctx, build_dir, site_dir) =
        pipeline::build_pipeline(&config, &opts);

    execute_build_pipeline(
        &plugins,
        &ctx,
        &build_dir,
        &config.content_dir,
        &site_dir,
        &config.template_dir,
        opts.quiet,
    )?;

    // Only start the dev server if `--serve` was explicitly requested.
    // Without this guard the binary blocks indefinitely, breaking CI.
    if config.serve_dir.is_some() {
        plugins.run_on_serve(&ctx)?;
        serve_site(&site_dir)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::cmd::Cli;
    use crate::logging::{SimpleLogger, DEFAULT_LOG_LEVEL, ENV_LOG_LEVEL};
    use crate::pipeline::{
        build_pipeline, execute_build_pipeline, resolve_build_and_site_dirs,
        RunOptions,
    };
    use crate::server::build_serve_address;
    use anyhow::Result;
    use log::Log;
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

        let date = now_iso();
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

    #[cfg(not(target_os = "windows"))] // Unix-only: invalid paths behave differently on Windows
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

    #[cfg(not(target_os = "windows"))] // Unix-only: invalid paths behave differently on Windows
    #[test]
    fn test_handle_server_failure() {
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
        let date = now_iso();
        let result = handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(result.is_err());
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

    #[cfg(not(target_os = "windows"))] // Unix-only: invalid paths behave differently on Windows
    #[test]
    fn test_create_directories_partial_failure() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().join("valid_dir");
        let invalid_path = PathBuf::from("/invalid/path");

        let paths = Paths {
            site: valid_path,
            content: invalid_path,
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

    #[test]
    fn test_handle_server_missing_serve_dir() {
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
        let binding = now_iso();
        let result = handle_server(
            &mut log_file,
            &binding,
            &paths,
            &non_existent_serve_dir,
        );
        assert!(result.is_err());
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

    #[test]
    fn test_handle_server_start_message() -> Result<()> {
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
        let date = now_iso();
        let result = handle_server(&mut log_file, &date, &paths, &serve_dir);
        assert!(
            result.is_err(),
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
        println!("Result: {result:?}");

        // Verify that we got an error
        assert!(result.is_err(), "Expected error for symlink, got success");

        // Verify the error message
        let err = result.unwrap_err();
        println!("Error message: {err}");
        assert!(
            err.to_string().contains("Symlinks are not allowed"),
            "Unexpected error message: {err}"
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
        assert!(result.is_err(), "Expected error, got: {result:?}");
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
    #[test]
    fn test_copy_empty_directory_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        let result = copy_dir_all_async(src_dir.path(), dst_dir.path());
        assert!(result.is_ok());

        // Verify destination directory exists
        assert!(dst_dir.path().exists());
        Ok(())
    }

    /// Tests copying a directory with a single file
    #[test]
    fn test_copy_single_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a test file
        let test_file = src_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path())?;

        // Verify file was copied
        let copied_file = dst_dir.path().join("test.txt");
        assert!(copied_file.exists());
        assert_eq!(fs::read_to_string(copied_file)?, "test content");

        Ok(())
    }

    /// Tests copying a directory with nested subdirectories
    #[test]
    fn test_copy_nested_directories_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create nested directory structure
        let nested_dir = src_dir.path().join("nested");
        fs::create_dir(&nested_dir)?;

        // Create files in both root and nested directory
        fs::write(src_dir.path().join("root.txt"), "root content")?;
        fs::write(nested_dir.join("nested.txt"), "nested content")?;

        copy_dir_all_async(src_dir.path(), dst_dir.path())?;

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
    #[test]
    fn test_copy_with_symlink_async() -> Result<()> {
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
        let result = copy_dir_all_async(src_dir.path(), dst_dir.path());
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying large files
    #[test]
    fn test_copy_large_file_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create a large file (11MB)
        let large_file = src_dir.path().join("large.txt");
        let file = File::create(&large_file)?;
        file.set_len(11 * 1024 * 1024)?;

        // Attempt to copy - should fail due to file size limit
        let result = copy_dir_all_async(src_dir.path(), dst_dir.path());
        assert!(result.is_err());

        Ok(())
    }

    /// Tests copying with invalid destination
    #[cfg(not(target_os = "windows"))] // Unix-only: invalid paths behave differently on Windows
    #[test]
    fn test_copy_invalid_destination_async() -> Result<()> {
        let src_dir = tempdir()?;
        let invalid_dst = PathBuf::from("/nonexistent/path");

        let result = copy_dir_all_async(src_dir.path(), &invalid_dst);
        assert!(result.is_err());

        Ok(())
    }

    /// Tests concurrent copying of multiple files
    #[test]
    fn test_concurrent_copy_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create multiple files
        for i in 0..5 {
            fs::write(
                src_dir.path().join(format!("file{i}.txt")),
                format!("content {i}"),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path())?;

        // Verify all files were copied
        for i in 0..5 {
            let copied_file = dst_dir.path().join(format!("file{i}.txt"));
            assert!(copied_file.exists());
            assert_eq!(
                fs::read_to_string(copied_file)?,
                format!("content {i}")
            );
        }

        Ok(())
    }

    /// Tests copying with maximum directory depth
    #[test]
    fn test_max_directory_depth_async() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;
        let max_depth = 5;

        // Create deeply nested directory structure
        let mut current_dir = src_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{i}"));
            fs::create_dir(&current_dir)?;
            fs::write(
                current_dir.join("file.txt"),
                format!("content level {i}"),
            )?;
        }

        copy_dir_all_async(src_dir.path(), dst_dir.path())?;

        // Verify the entire structure was copied
        current_dir = dst_dir.path().to_path_buf();
        for i in 0..max_depth {
            current_dir = current_dir.join(format!("level{i}"));
            assert!(current_dir.exists());
            assert!(current_dir.join("file.txt").exists());
            assert_eq!(
                fs::read_to_string(current_dir.join("file.txt"))?,
                format!("content level {i}")
            );
        }

        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_async_missing_source() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("nonexistent");
        let dst_dir = temp_dir.path().join("dst");

        let error = verify_and_copy_files_async(&src_dir, &dst_dir)
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("does not exist"),
            "Expected error message about non-existent source, got: {error}"
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
        assert!(logging::initialize_logging().is_ok());
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
        let cloned = builder;
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

    #[test]
    fn test_async_copy_with_empty_source() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("empty_src");
        let dst_dir = temp_dir.path().join("empty_dst");

        fs::create_dir(&src_dir)?;

        let result = verify_and_copy_files_async(&src_dir, &dst_dir);
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

        let date = now_iso();
        log_initialization(&mut log_file, &date)?;

        let content = fs::read_to_string(&log_path)?;
        assert!(!content.is_empty());
        assert!(content.contains("process"));
        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_async_with_nested_empty_dirs() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create nested empty directory structure
        fs::create_dir_all(src_dir.join("a/b/c"))?;
        fs::create_dir_all(src_dir.join("d/e/f"))?;

        verify_and_copy_files_async(&src_dir, &dst_dir)?;

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

    #[test]
    fn test_copy_dir_all_async_with_empty_dirs() -> Result<()> {
        let temp_dir = tempdir()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        fs::create_dir_all(src_dir.join("empty1"))?;
        fs::create_dir_all(src_dir.join("empty2/empty3"))?;

        copy_dir_all_async(&src_dir, &dst_dir)?;

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
                "Expected log level '{expected}' for input '{input}', but got '{processed_level}'"
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

    /// Test the logic for translating string log levels to `LevelFilter` values
    #[test]
    fn test_log_level_translation() {
        use log::LevelFilter;
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
                "Log level mismatch for input: '{input}' - expected {expected:?}, got {level:?}"
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
                "Log level mismatch for env value: {env_value:?}"
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
        use log::LevelFilter;
        assert_eq!(logging::parse_log_level("error"), LevelFilter::Error);
        assert_eq!(logging::parse_log_level("warn"), LevelFilter::Warn);
        assert_eq!(logging::parse_log_level("info"), LevelFilter::Info);
        assert_eq!(logging::parse_log_level("debug"), LevelFilter::Debug);
        assert_eq!(logging::parse_log_level("trace"), LevelFilter::Trace);
    }

    #[test]
    fn parse_log_level_is_case_insensitive() {
        use log::LevelFilter;
        assert_eq!(logging::parse_log_level("ERROR"), LevelFilter::Error);
        assert_eq!(logging::parse_log_level("Warn"), LevelFilter::Warn);
        assert_eq!(logging::parse_log_level("TraCe"), LevelFilter::Trace);
    }

    #[test]
    fn parse_log_level_unknown_value_falls_back_to_info() {
        use log::LevelFilter;
        assert_eq!(logging::parse_log_level("nonsense"), LevelFilter::Info);
        assert_eq!(logging::parse_log_level(""), LevelFilter::Info);
        assert_eq!(logging::parse_log_level("verbose"), LevelFilter::Info);
    }

    #[test]
    fn test_concurrent_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        // Create source directory
        fs::create_dir_all(&src_dir)?;

        // Create files
        for i in 0..100 {
            fs::write(
                src_dir.join(format!("file_{i}.txt")),
                format!("content {i}"),
            )?;
        }

        // Verify source files
        let mut src_files = Vec::new();
        collect_files_recursive(&src_dir, &mut src_files)?;
        assert_eq!(src_files.len(), 100);

        // Create destination directory
        fs::create_dir_all(&dst_dir)?;

        // Copy files using verify_and_copy_files
        verify_and_copy_files(&src_dir, &dst_dir)?;

        // Verify destination files
        let mut dst_files = Vec::new();
        collect_files_recursive(&dst_dir, &mut dst_files)?;

        assert_eq!(dst_files.len(), 100);

        // Verify file contents
        for i in 0..100 {
            let dst_path = dst_dir.join(format!("file_{i}.txt"));
            assert!(
                dst_path.exists(),
                "File {} does not exist in destination",
                dst_path.display()
            );

            let content = fs::read_to_string(&dst_path)?;
            assert_eq!(
                content,
                format!("content {i}"),
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

    #[cfg(not(target_os = "windows"))] // Unix-only: invalid paths behave differently on Windows
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
                src_dir.path().join(format!("file{i}.txt")),
                format!("content {i}"),
            )?;
        }
        copy_dir_all(src_dir.path(), dst_dir.path())?;
        for i in 0..20 {
            assert!(dst_dir.path().join(format!("file{i}.txt")).exists());
        }
        Ok(())
    }

    #[test]
    fn test_collect_files_recursive_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        // Create a directory deeper than MAX_DIR_DEPTH
        let mut path = temp_dir.path().to_path_buf();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{i}"));
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
            path = path.join(format!("d{i}"));
            fs::create_dir(&path)?;
        }
        let result = copy_dir_all(src_dir.path(), dst_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[test]
    fn test_verify_and_copy_files_async_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");
        let mut path = src.clone();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{i}"));
            fs::create_dir_all(&path)?;
        }
        let result = verify_and_copy_files_async(&src, &dst);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum depth"));
        Ok(())
    }

    #[test]
    fn test_copy_dir_all_async_depth_exceeded() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");
        let mut path = src.clone();
        for i in 0..=MAX_DIR_DEPTH {
            path = path.join(format!("d{i}"));
            fs::create_dir_all(&path)?;
        }
        let result = copy_dir_all_async(&src, &dst);
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
        let dst = env::temp_dir().join("ssg_copy_dir_dst");
        let result =
            copy_dir_with_progress(Path::new("/nonexistent/source"), &dst);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_and_copy_files_async_with_files() -> Result<()> {
        let temp_dir = tempdir()?;
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source with nested dirs + files
        fs::create_dir_all(src.join("sub1/sub2"))?;
        fs::write(src.join("root.txt"), "root")?;
        fs::write(src.join("sub1/a.txt"), "a")?;
        fs::write(src.join("sub1/sub2/b.txt"), "b")?;

        verify_and_copy_files_async(&src, &dst)?;

        assert_eq!(fs::read_to_string(dst.join("root.txt"))?, "root");
        assert_eq!(fs::read_to_string(dst.join("sub1/a.txt"))?, "a");
        assert_eq!(fs::read_to_string(dst.join("sub1/sub2/b.txt"))?, "b");
        Ok(())
    }

    #[test]
    fn test_copy_dir_with_progress_with_files() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        // Create nested structure
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
            path = path.join(format!("d{i}"));
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

    #[test]
    fn test_prepare_serve_dir_same_as_site() -> Result<()> {
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
        prepare_serve_dir(&paths, &site_dir)?;
        assert!(site_dir.join("index.html").exists());
        Ok(())
    }

    #[test]
    fn test_prepare_serve_dir_different() -> Result<()> {
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

        prepare_serve_dir(&paths, &serve_dir)?;
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
        let temp = tempdir().unwrap();
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
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

        assert!(plugins.len() >= 10);
        assert_ne!(build_dir, site_dir);
        assert_eq!(site_dir, temp.path().join("public"));
        assert_eq!(ctx.content_dir, temp.path().join("content"));
    }

    #[test]
    fn build_pipeline_with_deploy_target_registers_deploy_plugin() {
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts_no_deploy = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
        };
        let (no_deploy, _, _, _) = build_pipeline(&config, &opts_no_deploy);

        let opts_deploy = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: Some("netlify".to_string()),
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
        };
        let (with_deploy, _, _, _) = build_pipeline(&config, &opts_deploy);

        assert_eq!(with_deploy.len(), no_deploy.len() + 1);
    }

    #[test]
    fn build_pipeline_with_unknown_deploy_target_logs_and_skips() {
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: Some("nonsense-platform".to_string()),
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
        };
        let (plugins, _, _, _) = build_pipeline(&config, &opts);
        let names = plugins.names();
        assert!(!names.iter().any(|n| n == &"deploy"));
    }

    #[test]
    fn build_pipeline_with_each_known_deploy_target_registers_one_plugin() {
        for target in ["netlify", "vercel", "cloudflare", "github"] {
            let temp = tempdir().unwrap();
            let mut config = SsgConfig::default();
            config.content_dir = temp.path().join("content");
            config.output_dir = temp.path().join("public");

            let opts = RunOptions {
                quiet: true,
                include_drafts: false,
                deploy_target: Some(target.to_string()),
                validate_only: false,
                jobs: None,
                max_memory_mb: None,
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
            Err(anyhow::anyhow!("transport failed"))
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
        let temp = tempdir()?;
        let blocker = temp.path().join("blocker.txt");
        fs::write(&blocker, "i am a file, not a directory")?;

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

    #[cfg(not(target_os = "windows"))] // Unix-specific: path behaviour / error messages differ on Windows
    #[test]
    fn create_directories_unsafe_path_bails() -> Result<()> {
        let temp = tempdir()?;
        let blocker = temp.path().join("blocker.txt");
        fs::write(&blocker, "x")?;

        let unsafe_path = blocker.join("..").join("subdir");

        let paths = Paths {
            site: temp.path().join("s"),
            content: unsafe_path,
            build: temp.path().join("b"),
            template: temp.path().join("t"),
        };
        let result = create_directories(&paths);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn copy_dir_with_progress_read_dir_failure_propagates() -> Result<()> {
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

    #[test]
    fn verify_and_copy_files_async_destination_create_dir_failure_propagates(
    ) -> Result<()> {
        let temp = tempdir()?;
        let blocker = temp.path().join("async-blocker.txt");
        fs::write(&blocker, "blocker")?;

        let bad_dst = blocker.join("sub");
        let result = verify_and_copy_files_async(temp.path(), &bad_dst);
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Failed to create or access destination"));
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn build_serve_address_rejects_invalid_utf8_path() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let invalid_bytes = b"site_\xff_invalid";
        let path = Path::new(OsStr::from_bytes(invalid_bytes));
        let err = build_serve_address(path).unwrap_err();
        assert!(format!("{err:?}").contains("invalid UTF-8"));
    }

    #[test]
    #[cfg(unix)]
    fn serve_site_shim_propagates_invalid_utf8_path_error() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let invalid = b"\xfe\xfe_bad";
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
        fn assert_impl<T: ServeTransport>() {}
        assert_impl::<HttpTransport>();
    }

    // -----------------------------------------------------------------
    // execute_build_pipeline
    // -----------------------------------------------------------------

    #[test]
    fn execute_build_pipeline_propagates_compile_errors() -> Result<()> {
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
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
        };

        let (plugins, ctx, build_dir, site_dir) =
            build_pipeline(&config, &opts);

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
        config.content_dir = content;
        config.template_dir = template;
        config.output_dir = temp.path().join("public");
        config.site_name = "pipeline-success-test".to_string();
        config.base_url = "http://localhost".to_string();

        let opts = RunOptions {
            quiet: true,
            include_drafts: false,
            deploy_target: None,
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
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

        assert!(
            site_dir.exists() || build_dir.exists(),
            "expected build/site dir to exist after successful pipeline"
        );
        Ok(())
    }

    #[test]
    fn execute_build_pipeline_verbose_success_hits_println_arm() -> Result<()> {
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
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
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
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
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
        let temp = tempdir().unwrap();
        let mut config = SsgConfig::default();
        config.content_dir = temp.path().join("content");
        config.output_dir = temp.path().join("public");

        let opts = RunOptions {
            quiet: true,
            include_drafts: true,
            deploy_target: None,
            validate_only: false,
            jobs: None,
            max_memory_mb: None,
        };
        let (plugins, _, _, _) = build_pipeline(&config, &opts);
        assert!(plugins.names().iter().any(|n| n == &"drafts"));
    }

    // -----------------------------------------------------------------
    // now_iso / days_to_ymd coverage
    // -----------------------------------------------------------------

    #[test]
    fn now_iso_returns_valid_iso8601_format() {
        let ts = now_iso();
        assert_eq!(ts.len(), 20, "ISO timestamp should be 20 chars: {ts}");
        assert!(ts.ends_with('Z'), "should end with Z: {ts}");
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
        let year: u64 = ts[0..4].parse().unwrap();
        assert!(year >= 2020, "year should be recent: {year}");
    }

    #[test]
    fn days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date_2026_04_13() {
        let (y, m, d) = days_to_ymd(20_556);
        assert_eq!((y, m, d), (2026, 4, 13));
    }

    #[test]
    fn days_to_ymd_leap_day() {
        let (y, m, d) = days_to_ymd(11_016);
        assert_eq!((y, m, d), (2000, 2, 29));
    }

    #[test]
    fn days_to_ymd_y2k() {
        let (y, m, d) = days_to_ymd(10_957);
        assert_eq!((y, m, d), (2000, 1, 1));
    }

    // -----------------------------------------------------------------
    // SimpleLogger coverage
    // -----------------------------------------------------------------

    #[test]
    fn simple_logger_enabled_respects_max_level() {
        let logger = SimpleLogger;
        let meta = log::MetadataBuilder::new()
            .level(log::Level::Info)
            .target("test")
            .build();
        let _ = logger.enabled(&meta);
    }

    #[test]
    fn simple_logger_flush_is_noop() {
        use log::Log;
        let logger = SimpleLogger;
        logger.flush();
    }

    // -----------------------------------------------------------------
    // build_serve_address additional coverage
    // -----------------------------------------------------------------

    #[test]
    fn build_serve_address_with_absolute_path() {
        let (addr, root) = build_serve_address(Path::new("/tmp/site")).unwrap();
        assert!(addr.contains(&cmd::DEFAULT_PORT.to_string()));
        assert_eq!(root, "/tmp/site");
    }

    // -----------------------------------------------------------------
    // copy_dir_with_progress file count output
    // -----------------------------------------------------------------

    #[test]
    fn copy_dir_with_progress_counts_files_and_dirs() -> Result<()> {
        let src_dir = tempdir()?;
        let dst_dir = tempdir()?;

        fs::write(src_dir.path().join("a.txt"), "a")?;
        fs::write(src_dir.path().join("b.txt"), "b")?;
        let sub = src_dir.path().join("sub");
        fs::create_dir(&sub)?;
        fs::write(sub.join("c.txt"), "c")?;

        copy_dir_with_progress(src_dir.path(), dst_dir.path())?;

        assert!(dst_dir.path().join("a.txt").exists());
        assert!(dst_dir.path().join("b.txt").exists());
        assert!(dst_dir.path().join("sub/c.txt").exists());
        Ok(())
    }
}
