// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Live-reload script injection plugin.
//!
//! Injects a WebSocket-based live-reload client into all HTML files in
//! the site directory when the development server starts.
//!
//! # How it works
//!
//! 1. The `LiveReloadPlugin` hooks into the `on_serve` lifecycle event.
//! 2. It walks all HTML files in the site directory.
//! 3. It injects a `<script>` tag before `</body>` that opens a WebSocket
//!    connection to a configurable port (default 35729).
//! 4. When the server sends a `"reload"` message, the page reloads.
//! 5. On disconnect, the script auto-reconnects with exponential backoff
//!    (1s, 2s, 4s, capped at 10s) and shows a small "Connecting..."
//!    indicator in the bottom-right corner.

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Default WebSocket port for the live-reload server.
pub const DEFAULT_PORT: u16 = 35729;

/// Maximum number of HTML files to process.
const MAX_FILES: usize = 50_000;

/// Marker attribute used to detect whether the script has already been injected.
const MARKER: &str = "ssg-livereload";

/// Plugin that injects a live-reload script into all HTML files.
///
/// The injected script opens a WebSocket connection and reloads the page
/// when it receives a `"reload"` message. It reconnects automatically
/// with exponential backoff on disconnect.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::livereload::LiveReloadPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(LiveReloadPlugin::new());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LiveReloadPlugin {
    /// WebSocket port the live-reload client connects to.
    port: u16,
}

impl LiveReloadPlugin {
    /// Creates a new `LiveReloadPlugin` with the default port (35729).
    pub fn new() -> Self {
        Self {
            port: DEFAULT_PORT,
        }
    }

    /// Creates a new `LiveReloadPlugin` with a custom WebSocket port.
    pub fn with_port(port: u16) -> Self {
        Self { port }
    }

    /// Returns the configured port.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Default for LiveReloadPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LiveReloadPlugin {
    fn name(&self) -> &str {
        "livereload"
    }

    fn on_serve(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        if html_files.is_empty() {
            return Ok(());
        }

        for path in &html_files {
            inject_livereload(path, self.port)?;
        }

        println!(
            "[livereload] Injected live-reload script into {} HTML file(s) (port {})",
            html_files.len(),
            self.port,
        );
        Ok(())
    }
}

/// Collect all `.html` files under `dir` (iterative, bounded).
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if files.len() >= MAX_FILES {
            break;
        }
        let entries = fs::read_dir(&current)
            .with_context(|| format!("cannot read {}", current.display()))?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |e| e == "html") {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Inject the live-reload script into a single HTML file.
///
/// Inserts a `<script>` block before `</body>`. The script:
/// 1. Opens a WebSocket to `ws://localhost:{port}`
/// 2. Reloads on receiving a `"reload"` message
/// 3. Reconnects with exponential backoff (1s, 2s, 4s, max 10s)
/// 4. Shows a "Connecting..." indicator during reconnection
///
/// The injection is idempotent — if the marker is already present,
/// the file is left unchanged.
fn inject_livereload(path: &Path, port: u16) -> Result<()> {
    let html = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    if html.contains(MARKER) {
        return Ok(()); // Already injected
    }

    let script = livereload_script(port);

    let injected = if let Some(pos) = html.rfind("</body>") {
        format!("{}{}{}", &html[..pos], script, &html[pos..])
    } else {
        format!("{html}{script}")
    };

    fs::write(path, injected)
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}

/// Generate the live-reload script tag for a given port.
fn livereload_script(port: u16) -> String {
    format!(
        r##"
<!-- SSG Live-Reload -->
<script data-ssg-livereload>
(function(){{
  var url='ws://localhost:{port}',delay=1000,maxDelay=10000,indicator=null;
  function showIndicator(){{
    if(indicator)return;
    indicator=document.createElement('div');
    indicator.id='ssg-livereload';
    indicator.textContent='Connecting\u2026';
    indicator.style.cssText='position:fixed;bottom:8px;right:8px;z-index:99999;'
      +'background:rgba(0,0,0,0.75);color:#fff;padding:6px 12px;border-radius:6px;'
      +'font:13px/1 -apple-system,system-ui,sans-serif;pointer-events:none';
    document.body.appendChild(indicator);
  }}
  function hideIndicator(){{
    if(indicator){{indicator.remove();indicator=null;}}
  }}
  function connect(){{
    var ws=new WebSocket(url);
    ws.onopen=function(){{delay=1000;hideIndicator();}};
    ws.onmessage=function(e){{if(e.data==='reload')location.reload();}};
    ws.onclose=function(){{
      showIndicator();
      var d=delay;
      delay=Math.min(delay*2,maxDelay);
      setTimeout(connect,d);
    }};
    ws.onerror=function(){{ws.close();}};
  }}
  if(document.readyState==='loading'){{
    document.addEventListener('DOMContentLoaded',connect);
  }}else{{
    connect();
  }}
}})();
</script>
"##,
        port = port
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_html(body: &str) -> String {
        format!(
            "<html><head><title>Test</title></head>\
             <body>{body}</body></html>"
        )
    }

    #[test]
    fn inject_adds_script() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, make_html("<p>Hello</p>"))?;

        inject_livereload(&path, DEFAULT_PORT)?;

        let result = fs::read_to_string(&path)?;
        assert!(result.contains(MARKER));
        assert!(result.contains("WebSocket"));
        assert!(result.contains("35729"));
        assert!(result.contains("location.reload()"));
        Ok(())
    }

    #[test]
    fn inject_before_closing_body() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, make_html("<p>Hi</p>"))?;

        inject_livereload(&path, DEFAULT_PORT)?;

        let result = fs::read_to_string(&path)?;
        let script_pos = result.find(MARKER).unwrap();
        let body_pos = result.rfind("</body>").unwrap();
        assert!(script_pos < body_pos);
        Ok(())
    }

    #[test]
    fn inject_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, make_html("<p>Hi</p>"))?;

        inject_livereload(&path, DEFAULT_PORT)?;
        let first = fs::read_to_string(&path)?;

        inject_livereload(&path, DEFAULT_PORT)?;
        let second = fs::read_to_string(&path)?;

        assert_eq!(first, second);
        Ok(())
    }

    #[test]
    fn inject_custom_port() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, make_html("<p>Hi</p>"))?;

        inject_livereload(&path, 9999)?;

        let result = fs::read_to_string(&path)?;
        assert!(result.contains("9999"));
        assert!(!result.contains("35729"));
        Ok(())
    }

    #[test]
    fn inject_no_body_tag() -> Result<()> {
        let tmp = tempdir()?;
        let path = tmp.path().join("page.html");
        fs::write(&path, "<html><p>No body tag</p></html>")?;

        inject_livereload(&path, DEFAULT_PORT)?;

        let result = fs::read_to_string(&path)?;
        assert!(result.contains(MARKER));
        Ok(())
    }

    #[test]
    fn skip_non_html_files() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(tmp.path().join("style.css"), "body{}")?;
        fs::write(tmp.path().join("data.json"), "{}")?;
        fs::write(tmp.path().join("readme.txt"), "hello")?;

        let files = collect_html_files(tmp.path())?;
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn empty_directory() -> Result<()> {
        let tmp = tempdir()?;
        let files = collect_html_files(tmp.path())?;
        assert!(files.is_empty());
        Ok(())
    }

    #[test]
    fn nonexistent_directory() {
        let ctx = PluginContext::new(
            Path::new("c"),
            Path::new("b"),
            Path::new("/nonexistent_dir_ssg_test"),
            Path::new("t"),
        );
        let plugin = LiveReloadPlugin::new();
        assert!(plugin.on_serve(&ctx).is_ok());
    }

    #[test]
    fn plugin_name() {
        assert_eq!(LiveReloadPlugin::new().name(), "livereload");
    }

    #[test]
    fn plugin_registration() {
        use crate::plugin::PluginManager;
        let mut pm = PluginManager::new();
        pm.register(LiveReloadPlugin::new());
        assert_eq!(pm.names(), vec!["livereload"]);
    }

    #[test]
    fn with_port_constructor() {
        let plugin = LiveReloadPlugin::with_port(8080);
        assert_eq!(plugin.port(), 8080);
    }

    #[test]
    fn default_port_value() {
        let plugin = LiveReloadPlugin::new();
        assert_eq!(plugin.port(), 35729);
    }

    #[test]
    fn default_trait_impl() {
        let plugin = LiveReloadPlugin::default();
        assert_eq!(plugin.port(), DEFAULT_PORT);
    }

    #[test]
    fn on_serve_injects_all_html_files() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(tmp.path().join("index.html"), make_html("<p>Home</p>"))?;
        fs::write(tmp.path().join("about.html"), make_html("<p>About</p>"))?;
        fs::write(tmp.path().join("style.css"), "body{}")?;

        let ctx = PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            tmp.path(),
            Path::new("templates"),
        );
        LiveReloadPlugin::new().on_serve(&ctx)?;

        let index = fs::read_to_string(tmp.path().join("index.html"))?;
        let about = fs::read_to_string(tmp.path().join("about.html"))?;
        let css = fs::read_to_string(tmp.path().join("style.css"))?;

        assert!(index.contains(MARKER));
        assert!(about.contains(MARKER));
        assert!(!css.contains(MARKER));
        Ok(())
    }

    #[test]
    fn script_contains_reconnect_backoff() {
        let script = livereload_script(DEFAULT_PORT);
        assert!(script.contains("delay*2"));
        assert!(script.contains("maxDelay"));
        assert!(script.contains("10000"));
    }

    #[test]
    fn script_contains_connecting_indicator() {
        let script = livereload_script(DEFAULT_PORT);
        assert!(script.contains("Connecting"));
        assert!(script.contains("showIndicator"));
        assert!(script.contains("hideIndicator"));
        assert!(script.contains("bottom"));
        assert!(script.contains("right"));
    }
}
