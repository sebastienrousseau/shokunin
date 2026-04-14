// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dev server infrastructure for the static site generator.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use http_handle::Server;

use crate::cmd;
use crate::Paths;

/// Pluggable transport that drives the dev server.
///
/// Production code uses [`HttpTransport`] (a thin wrapper around
/// `http_handle::Server`); tests use a test-only `NoopTransport` which
/// records the call without actually binding a port. The trait exists
/// so every line of `serve_site` is unit-testable.
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
            anyhow!(
                "Site directory path contains invalid UTF-8: {}",
                site_dir.display()
            )
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
///
/// fn main() -> anyhow::Result<()> {
///     let mut log_file = create_log_file("./server.log")?;
///     let date = ssg::now_iso();
///     let paths = Paths {
///         site: PathBuf::from("public"),
///         content: PathBuf::from("content"),
///         build: PathBuf::from("build"),
///         template: PathBuf::from("templates"),
///     };
///     let serve_dir = PathBuf::from("serve");
///
///     handle_server(&mut log_file, &date, &paths, &serve_dir)?;
///     Ok(())
/// }
/// ```
///
/// # Server Configuration
///
/// * Default port: 8000
/// * Host: 127.0.0.1 (localhost)
/// * Serves static files from the specified directory
pub fn handle_server(
    log_file: &mut fs::File,
    date: &str,
    paths: &Paths,
    serve_dir: &PathBuf,
) -> Result<()> {
    // Log server initialization
    writeln!(log_file, "[{date}] INFO process: Server initialization")?;

    prepare_serve_dir(paths, serve_dir)?;

    let host = cmd::resolve_host();
    let port = cmd::resolve_port();
    let addr = format!("{host}:{port}");

    println!("\nStarting server at http://{addr}");
    println!("Serving content from: {}", serve_dir.display());

    let dir = serve_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("serve dir contains invalid UTF-8"))?
        .to_string();
    let bind = addr;

    let server = Server::new(&bind, &dir);
    let _ = server.start();
    Ok(())
}

/// Generates a root index.html that reads the browser's language
/// preference and redirects to the best matching locale directory.
///
/// The file is written at `site_dir/index.html`. If it already exists
/// and was not generated by this function, it is left untouched.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn generate_locale_redirect(
    site_dir: &Path,
    available_locales: &[String],
    default_locale: &str,
) -> Result<()> {
    let index_path = site_dir.join("index.html");

    // If an index.html already exists and wasn't generated by us, leave it.
    if index_path.exists() {
        let existing = fs::read_to_string(&index_path).unwrap_or_default();
        if !existing.contains("<!-- ssg-locale-redirect -->") {
            return Ok(());
        }
    }

    let locales_js: Vec<String> = available_locales
        .iter()
        .map(|l| format!("\"{l}\""))
        .collect();
    let locales_array = locales_js.join(",");
    let default_url = format!("/{default_locale}/");

    let html = format!(
        r#"<!DOCTYPE html>
<!-- ssg-locale-redirect -->
<html>
<head>
<meta charset="utf-8">
<script>
(function() {{
  var locales = [{locales_array}];
  var defaultLocale = "{default_locale}";
  var langs = navigator.languages || [navigator.language || defaultLocale];
  for (var i = 0; i < langs.length; i++) {{
    var lang = langs[i].toLowerCase();
    for (var j = 0; j < locales.length; j++) {{
      if (lang === locales[j] || lang.startsWith(locales[j] + "-")) {{
        window.location.replace("/" + locales[j] + "/");
        return;
      }}
    }}
    var prefix = lang.split("-")[0];
    for (var j = 0; j < locales.length; j++) {{
      if (prefix === locales[j]) {{
        window.location.replace("/" + locales[j] + "/");
        return;
      }}
    }}
  }}
  window.location.replace("/" + defaultLocale + "/");
}})();
</script>
<noscript>
<meta http-equiv="refresh" content="0; url={default_url}">
</noscript>
</head>
<body></body>
</html>
"#
    );

    fs::write(&index_path, &html)
        .with_context(|| format!("Failed to write {}", index_path.display()))?;

    println!(
        "[i18n] Generated locale redirect at {}",
        index_path.display()
    );
    Ok(())
}

/// Prepares the serve directory by creating it and copying site files.
pub fn prepare_serve_dir(paths: &Paths, serve_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(serve_dir)
        .context("Failed to create serve directory")?;

    println!("Setting up server...");
    println!("Source: {}", paths.site.display());
    println!("Serving from: {}", serve_dir.display());

    if serve_dir != &paths.site {
        crate::fs_ops::verify_and_copy_files_async(&paths.site, serve_dir)?;
    }
    Ok(())
}
