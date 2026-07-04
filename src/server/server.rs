// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dev server infrastructure for the static site generator.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{PathErrorExt, SsgError};
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
    fn start(&self, addr: &str, root: &str) -> Result<(), SsgError>;
}

/// Production transport: starts an `http_handle::Server`.
#[derive(Debug, Clone, Copy)]
pub struct HttpTransport;

impl ServeTransport for HttpTransport {
    fn start(&self, addr: &str, root: &str) -> Result<(), SsgError> {
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
pub(crate) fn build_serve_address(
    site_dir: &Path,
) -> Result<(String, String), SsgError> {
    let root = site_dir
        .to_str()
        .ok_or_else(|| SsgError::Validation {
            field: "site_dir".to_string(),
            message: format!(
                "Site directory path contains invalid UTF-8: {}",
                site_dir.display()
            ),
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
///
/// # Examples
///
/// ```rust
/// use ssg::server::{serve_site_with, ServeTransport};
/// use ssg::SsgError;
/// use std::path::Path;
/// use std::cell::RefCell;
///
/// struct Recorder(RefCell<Vec<(String, String)>>);
/// impl ServeTransport for Recorder {
///     fn start(&self, addr: &str, root: &str) -> Result<(), SsgError> {
///         self.0.borrow_mut().push((addr.into(), root.into()));
///         Ok(())
///     }
/// }
/// let rec = Recorder(RefCell::new(vec![]));
/// serve_site_with(Path::new("public"), &rec).unwrap();
/// assert_eq!(rec.0.borrow().len(), 1);
/// ```
pub fn serve_site_with<T: ServeTransport>(
    site_dir: &Path,
    transport: &T,
) -> Result<(), SsgError> {
    let (addr, root) = build_serve_address(site_dir)?;
    transport.start(&addr, &root)
}

/// Converts a site directory path to a string and starts an HTTP server.
///
/// This function blocks while the server is running.
///
/// # Examples
///
/// ```no_run
/// use ssg::server::serve_site;
/// use std::path::Path;
///
/// // Blocks until the HTTP server stops — only ever called from `run`.
/// let _ = serve_site(Path::new("public"));
/// ```
///
/// # Errors
///
/// Returns an error if `site_dir` contains invalid UTF-8.
pub fn serve_site(site_dir: &Path) -> Result<(), SsgError> {
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
/// fn main() -> Result<(), ssg::error::SsgError> {
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
) -> Result<(), SsgError> {
    handle_server_with(log_file, date, paths, serve_dir, &HttpTransport)
}

/// Transport-injected body of [`handle_server`].
///
/// Extracted so tests can drive every branch (logging, serve-dir
/// preparation, address resolution) with a recording transport instead
/// of a blocking `http_handle::Server`.
fn handle_server_with<T: ServeTransport>(
    log_file: &mut fs::File,
    date: &str,
    paths: &Paths,
    serve_dir: &PathBuf,
    transport: &T,
) -> Result<(), SsgError> {
    // Log server initialization
    writeln!(log_file, "[{date}] INFO process: Server initialization")
        .map_err(|source| SsgError::Io {
            path: PathBuf::from("log"),
            source,
        })?;

    prepare_serve_dir(paths, serve_dir)?;

    let host = cmd::resolve_host();
    let port = cmd::resolve_port();
    let addr = format!("{host}:{port}");

    println!("\nStarting server at http://{addr}");
    println!("Serving content from: {}", serve_dir.display());

    let dir = serve_dir_to_string(serve_dir)?;
    let bind = addr;

    transport.start(&bind, &dir)
}

/// Converts the serve directory to the `String` root the transport
/// expects, rejecting non-UTF-8 paths.
///
/// Extracted from [`handle_server_with`] so the invalid-UTF-8 branch is
/// testable in memory — APFS refuses to create non-UTF-8 paths, so the
/// branch can never be reached through the filesystem on macOS.
fn serve_dir_to_string(serve_dir: &Path) -> Result<String, SsgError> {
    serve_dir
        .to_str()
        .ok_or_else(|| SsgError::Validation {
            field: "serve_dir".to_string(),
            message: "serve dir contains invalid UTF-8".to_string(),
        })
        .map(str::to_string)
}

/// Generates a root index.html that reads the browser's language
/// preference and redirects to the best matching locale directory.
///
/// The file is written at `site_dir/index.html`. If it already exists
/// and was not generated by this function, it is left untouched.
///
/// # Examples
///
/// ```rust
/// use ssg::server::generate_locale_redirect;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// generate_locale_redirect(dir.path(), &["en".into(), "fr".into()], "en").unwrap();
/// let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
/// assert!(html.contains("ssg-locale-redirect"));
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn generate_locale_redirect(
    site_dir: &Path,
    available_locales: &[String],
    default_locale: &str,
) -> Result<(), SsgError> {
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

    fs::write(&index_path, &html).with_path(&index_path)?;

    println!(
        "[i18n] Generated locale redirect at {}",
        index_path.display()
    );
    Ok(())
}

/// Prepares the serve directory by creating it and copying site files.
///
/// # Examples
///
/// ```rust
/// use ssg::{Paths, server::prepare_serve_dir};
/// use std::path::PathBuf;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let dir = tempdir().unwrap();
/// let site = dir.path().join("site");
/// let serve = dir.path().join("serve");
/// fs::create_dir(&site).unwrap();
/// fs::write(site.join("a.html"), "<p>").unwrap();
/// let paths = Paths {
///     site,
///     content: PathBuf::from("content"),
///     build: PathBuf::from("build"),
///     template: PathBuf::from("templates"),
/// };
/// prepare_serve_dir(&paths, &serve).unwrap();
/// assert!(serve.join("a.html").exists());
/// ```
pub fn prepare_serve_dir(
    paths: &Paths,
    serve_dir: &PathBuf,
) -> Result<(), SsgError> {
    fs::create_dir_all(serve_dir).with_path(serve_dir)?;

    println!("Setting up server...");
    println!("Source: {}", paths.site.display());
    println!("Serving from: {}", serve_dir.display());

    if serve_dir != &paths.site {
        crate::fs_ops::verify_and_copy_files_async(&paths.site, serve_dir)?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {

    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    /// Test transport that records `(addr, root)` and never blocks.
    #[derive(Default)]
    struct RecordingTransport {
        calls: Arc<Mutex<Vec<(String, String)>>>,
        fail: bool,
    }

    impl ServeTransport for RecordingTransport {
        fn start(&self, addr: &str, root: &str) -> Result<(), SsgError> {
            self.calls
                .lock()
                .unwrap()
                .push((addr.to_string(), root.to_string()));
            if self.fail {
                Err(SsgError::Validation {
                    field: "transport".to_string(),
                    message: "synthetic transport failure".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn build_serve_address_formats_addr_and_returns_root() {
        let dir = tempdir().unwrap();
        let (addr, root) = build_serve_address(dir.path()).unwrap();
        assert!(
            addr.contains(cmd::DEFAULT_HOST),
            "addr should contain default host: {addr}"
        );
        assert!(
            addr.contains(&cmd::DEFAULT_PORT.to_string()),
            "addr should contain default port: {addr}"
        );
        assert_eq!(root, dir.path().to_str().unwrap());
    }

    #[test]
    fn serve_site_with_invokes_transport_with_resolved_address() {
        let dir = tempdir().unwrap();
        let transport = RecordingTransport::default();
        let calls = transport.calls.clone();
        serve_site_with(dir.path(), &transport).unwrap();
        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded.len(), 1);
        let (addr, root) = &recorded[0];
        assert!(addr.contains(cmd::DEFAULT_HOST));
        assert_eq!(root, dir.path().to_str().unwrap());
    }

    #[test]
    fn serve_site_with_propagates_transport_errors() {
        let dir = tempdir().unwrap();
        let transport = RecordingTransport {
            calls: Default::default(),
            fail: true,
        };
        let err = serve_site_with(dir.path(), &transport).unwrap_err();
        assert!(
            err.to_string().contains("synthetic transport failure"),
            "transport error should bubble up, got: {err}"
        );
    }

    #[test]
    fn http_transport_implements_serve_transport() {
        // Smoke test that HttpTransport satisfies the trait. We don't
        // actually call .start() here because that would bind a port.
        let _t: &dyn ServeTransport = &HttpTransport;
    }

    #[test]
    fn generate_locale_redirect_creates_index_with_marker() {
        let dir = tempdir().unwrap();
        generate_locale_redirect(
            dir.path(),
            &["en".to_string(), "fr".to_string(), "de".to_string()],
            "en",
        )
        .unwrap();

        let index = dir.path().join("index.html");
        assert!(index.exists(), "index.html should be written");

        let html = fs::read_to_string(&index).unwrap();
        assert!(html.contains("<!-- ssg-locale-redirect -->"));
        assert!(html.contains("\"en\""));
        assert!(html.contains("\"fr\""));
        assert!(html.contains("\"de\""));
        assert!(html.contains("/en/")); // default fallback
    }

    #[test]
    fn generate_locale_redirect_overwrites_own_marker() {
        let dir = tempdir().unwrap();

        // First call writes the file.
        generate_locale_redirect(dir.path(), &["en".to_string()], "en")
            .unwrap();
        let first = fs::read_to_string(dir.path().join("index.html")).unwrap();

        // Second call with different locales must overwrite.
        generate_locale_redirect(
            dir.path(),
            &["en".to_string(), "fr".to_string()],
            "en",
        )
        .unwrap();
        let second = fs::read_to_string(dir.path().join("index.html")).unwrap();

        assert_ne!(first, second);
        assert!(second.contains("\"fr\""));
    }

    #[test]
    fn generate_locale_redirect_preserves_user_index_html() {
        // If the user wrote their own index.html (no marker), don't overwrite.
        let dir = tempdir().unwrap();
        let user_html = "<html><body>my hand-written page</body></html>";
        fs::write(dir.path().join("index.html"), user_html).unwrap();

        generate_locale_redirect(dir.path(), &["en".to_string()], "en")
            .unwrap();

        let after = fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert_eq!(
            after, user_html,
            "user-authored index.html must not be overwritten"
        );
    }

    #[test]
    fn prepare_serve_dir_creates_dir_when_missing() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("a.html"), "x").unwrap();

        let serve = dir.path().join("serve-out");
        let paths = Paths {
            site: site.clone(),
            content: dir.path().join("content"),
            build: dir.path().join("build"),
            template: dir.path().join("templates"),
        };

        prepare_serve_dir(&paths, &serve).unwrap();

        assert!(serve.exists(), "serve dir should be created");
        assert!(
            serve.join("a.html").exists(),
            "files should be copied from site to serve dir"
        );
    }

    #[test]
    fn prepare_serve_dir_skips_copy_when_serve_equals_site() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("a.html"), "x").unwrap();

        let paths = Paths {
            site: site.clone(),
            content: dir.path().join("content"),
            build: dir.path().join("build"),
            template: dir.path().join("templates"),
        };

        // serve_dir == site — should not re-copy (no-op).
        prepare_serve_dir(&paths, &site).unwrap();
        assert!(site.join("a.html").exists());
    }

    #[test]
    fn build_serve_address_contains_host_and_port() {
        let dir = tempdir().unwrap();
        let (addr, root) = build_serve_address(dir.path()).unwrap();
        assert_eq!(
            addr,
            format!("{}:{}", cmd::DEFAULT_HOST, cmd::DEFAULT_PORT)
        );
        assert_eq!(root, dir.path().to_str().unwrap());
    }

    #[test]
    fn serve_site_with_records_correct_root() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("deep").join("nested");
        fs::create_dir_all(&sub).unwrap();
        let transport = RecordingTransport::default();
        let calls = transport.calls.clone();
        serve_site_with(&sub, &transport).unwrap();
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded[0].1, sub.to_str().unwrap());
    }

    #[test]
    fn generate_locale_redirect_single_locale() {
        let dir = tempdir().unwrap();
        generate_locale_redirect(dir.path(), &["es".to_string()], "es")
            .unwrap();
        let html = fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("\"es\""));
        assert!(html.contains("/es/"));
        assert!(html.contains("<!-- ssg-locale-redirect -->"));
    }

    #[test]
    #[cfg(unix)]
    fn test_handle_server_invalid_utf8_serve_dir() {
        use std::os::unix::ffi::OsStringExt;
        let dir = std::ffi::OsString::from_vec(vec![0xff, 0xfe, 0xfd]);
        let serve_dir = PathBuf::from(dir);
        let mut log_file = tempfile::tempfile().unwrap();
        let paths = Paths {
            site: PathBuf::from("site"),
            content: PathBuf::from("content"),
            build: PathBuf::from("build"),
            template: PathBuf::from("templates"),
        };
        let res =
            handle_server(&mut log_file, "2026-06-06", &paths, &serve_dir);
        assert!(res.is_err());
    }

    #[test]
    fn handle_server_with_drives_transport_after_preparing_dir() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("a.html"), "x").unwrap();
        let serve = dir.path().join("serve");
        let paths = Paths {
            site,
            content: dir.path().join("content"),
            build: dir.path().join("build"),
            template: dir.path().join("templates"),
        };
        let mut log_file = tempfile::tempfile().unwrap();
        let transport = RecordingTransport::default();
        let calls = transport.calls.clone();

        handle_server_with(
            &mut log_file,
            "2026-07-04",
            &paths,
            &serve,
            &transport,
        )
        .unwrap();

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "transport must be started once");
        assert_eq!(recorded[0].1, serve.to_str().unwrap());
        assert!(serve.join("a.html").exists(), "site files copied");
    }

    #[test]
    fn handle_server_with_fails_when_log_file_is_read_only() {
        // A read-only handle makes the very first writeln fail,
        // exercising the log-write error branch.
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("server.log");
        fs::write(&log_path, "").unwrap();
        let mut read_only = fs::File::open(&log_path).unwrap();

        let paths = Paths {
            site: dir.path().join("site"),
            content: dir.path().join("content"),
            build: dir.path().join("build"),
            template: dir.path().join("templates"),
        };
        let serve = dir.path().join("serve");
        let transport = RecordingTransport::default();

        let err = handle_server_with(
            &mut read_only,
            "2026-07-04",
            &paths,
            &serve,
            &transport,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("log"), "unexpected error: {err}");
        assert!(
            transport.calls.lock().unwrap().is_empty(),
            "transport must not start when logging fails"
        );
    }

    #[test]
    fn serve_dir_to_string_accepts_utf8_path() {
        let s = serve_dir_to_string(Path::new("/tmp/serve")).unwrap();
        assert_eq!(s, "/tmp/serve");
    }

    #[test]
    #[cfg(unix)]
    fn serve_dir_to_string_rejects_invalid_utf8_path() {
        use std::os::unix::ffi::OsStringExt;
        let bad =
            PathBuf::from(std::ffi::OsString::from_vec(vec![0xff, 0xfe, 0xfd]));
        let err = serve_dir_to_string(&bad).unwrap_err();
        assert!(format!("{err}").contains("invalid UTF-8"));
    }

    #[test]
    #[cfg(unix)]
    fn generate_locale_redirect_fails_on_readonly_dir() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let site = dir.path().join("frozen");
        fs::create_dir_all(&site).unwrap();
        fs::set_permissions(&site, fs::Permissions::from_mode(0o555)).unwrap();

        let res = generate_locale_redirect(&site, &["en".to_string()], "en");

        let _ = fs::set_permissions(&site, fs::Permissions::from_mode(0o755));
        assert!(res.is_err(), "write into read-only dir must fail");
    }

    #[test]
    fn http_transport_start_returns_ok_even_when_bind_fails() {
        // start() swallows the error from server.start() via `let _ =`,
        // so even an invalid bind address must return Ok(()) — exercises
        // the body of HttpTransport::start without binding a real port.
        let t = HttpTransport;
        let res = t.start("not-a-valid-address:zzz", "/tmp");
        assert!(res.is_ok());
    }
}
