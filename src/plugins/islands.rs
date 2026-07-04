// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Resumable hydration — interactive islands (Web Components).
//!
//! Provides an `<ssg-island>` custom element that lazily loads JavaScript
//! component bundles based on configurable hydration strategies:
//! `visible` (`IntersectionObserver`), `idle` (requestIdleCallback), or
//! `interaction` (click/focus/hover).
//!
//! ## Architecture
//!
//! 1. Content authors use `{{< island component="counter" hydrate="visible" >}}`
//! 2. The shortcode expands to `<ssg-island component="counter" hydrate="visible">`
//! 3. This plugin scans HTML for `<ssg-island>` elements and:
//!    - Copies user-provided island bundles from `islands/` to `_islands/`
//!    - Generates `_islands/manifest.json` listing all referenced components
//!    - Injects the `ssg-island.js` custom element loader into pages

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::{collections::BTreeSet, fs, path::Path};

/// Plugin that enables interactive islands via Web Components.
#[derive(Debug, Clone, Copy)]
pub struct IslandPlugin;

impl Default for IslandPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandPlugin {
    /// Creates a new `IslandPlugin`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::islands::IslandPlugin;
    /// use ssg::plugin::Plugin;
    ///
    /// let p = IslandPlugin::new();
    /// assert_eq!(p.name(), "islands");
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for IslandPlugin {
    fn name(&self) -> &'static str {
        "islands"
    }

    fn has_transform(&self) -> bool {
        true
    }

    fn transform_html(
        &self,
        html: &str,
        _path: &Path,
        _ctx: &PluginContext,
    ) -> Result<String, SsgError> {
        if !html.contains("<ssg-island") {
            return Ok(html.to_string());
        }

        if html.contains("ssg-island.js") {
            return Ok(html.to_string()); // Already injected
        }

        let script =
            "\n<script type=\"module\" src=\"/_islands/ssg-island.js\"></script>\n";

        let output = if let Some(pos) = html.rfind("</body>") {
            format!("{}{script}{}", &html[..pos], &html[pos..])
        } else {
            format!("{html}{script}")
        };

        Ok(output)
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = ctx.get_html_files();

        // Scan all HTML files for <ssg-island component="..."> references
        let mut components = BTreeSet::new();

        for path in &html_files {
            let html = fs::read_to_string(path).with_path(path)?;
            let page_components = extract_island_components(&html);
            components.extend(page_components);
        }

        if components.is_empty() {
            return Ok(());
        }

        let islands_dir = ctx.site_dir.join("_islands");
        fs::create_dir_all(&islands_dir).with_path(&islands_dir)?;

        // Copy user-provided island bundles from source islands/ dir
        let source_islands = ctx
            .content_dir
            .parent()
            .unwrap_or(&ctx.content_dir)
            .join("islands");

        if source_islands.exists() {
            for component in &components {
                let src = source_islands.join(format!("{component}.js"));
                if src.exists() {
                    let dst = islands_dir.join(format!("{component}.js"));
                    let _ = fs::copy(&src, &dst).with_path(&dst)?;
                }
            }
        }

        // Write manifest
        let manifest: Vec<_> = components.iter().collect();
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .unwrap_or_else(|_| "[]".to_string());
        let manifest_path = islands_dir.join("manifest.json");
        fs::write(&manifest_path, manifest_json).with_path(&manifest_path)?;

        // Write the ssg-island.js custom element loader
        let loader_path = islands_dir.join("ssg-island.js");
        fs::write(&loader_path, ISLAND_LOADER_JS).with_path(&loader_path)?;

        log::info!("[islands] {} component(s) bundled", components.len());
        Ok(())
    }
}

/// Extracts component names from `<ssg-island component="...">` elements.
fn extract_island_components(html: &str) -> BTreeSet<String> {
    let mut components = BTreeSet::new();
    let pattern = "component=\"";

    let mut search_from = 0;
    while let Some(tag_start) = html[search_from..].find("<ssg-island") {
        let abs_start = search_from + tag_start;
        let rest = &html[abs_start..];

        if let Some(tag_end) = rest.find('>') {
            let tag = &rest[..tag_end];
            if let Some(comp_start) = tag.find(pattern) {
                let value_start = comp_start + pattern.len();
                if let Some(value_end) = tag[value_start..].find('"') {
                    let component = &tag[value_start..value_start + value_end];
                    if !component.is_empty() {
                        let _ = components.insert(component.to_string());
                    }
                }
            }
            search_from = abs_start + tag_end;
        } else {
            break;
        }
    }

    components
}

#[cfg(test)]
fn inject_island_loader(path: &Path) -> Result<(), SsgError> {
    let html = fs::read_to_string(path).with_path(path)?;

    if html.contains("ssg-island.js") {
        return Ok(()); // Already injected
    }

    let script =
        "\n<script type=\"module\" src=\"/_islands/ssg-island.js\"></script>\n";

    let output = if let Some(pos) = html.rfind("</body>") {
        format!("{}{script}{}", &html[..pos], &html[pos..])
    } else {
        format!("{html}{script}")
    };

    fs::write(path, output).with_path(path)?;
    Ok(())
}

/// The `<ssg-island>` custom element loader.
///
/// Lazy-hydration strategies:
///
/// - `hydrate="visible"` (default): loads when the element enters the
///   viewport via `IntersectionObserver` — used for below-the-fold
///   widgets so they don't compete for first-paint bandwidth (AC4).
/// - `hydrate="idle"`: loads during browser idle time
///   (`requestIdleCallback`) — for non-critical widgets that should
///   still warm up shortly after first paint.
/// - `hydrate="interaction"`: loads on the first click/focus/hover —
///   for components that have zero value until the user touches them.
///
/// The element exposes a `detach()` method called by the view
/// transitions client (issue #547) before removing the outgoing page's
/// `<main>` from the DOM. It also listens for an `ssg:detach`
/// `CustomEvent` for userland code that prefers an event-based API.
/// Both paths cancel any pending `IntersectionObserver` /
/// `requestIdleCallback` / interaction listeners so the outgoing
/// page leaves no dangling event subscriptions (AC5).
const ISLAND_LOADER_JS: &str = r#"/**
 * SSG Island — lazy-hydrating Web Component loader.
 * Each <ssg-island> loads its component bundle on demand.
 *
 * Hydration strategies: visible | idle | interaction (default visible)
 * Lifecycle:
 *   connectedCallback → arm strategy
 *   detach() / disconnectedCallback / ssg:detach → tear down
 */
class SsgIsland extends HTMLElement {
  constructor() {
    super();
    this._cleanup = [];
    this._hydrated = false;
    this.addEventListener('ssg:detach', () => this.detach());
  }

  connectedCallback() {
    const strategy = this.getAttribute('hydrate') || 'visible';
    const component = this.getAttribute('component');
    if (!component) return;

    const load = () => this._hydrate(component);

    if (strategy === 'idle') {
      let handle;
      if ('requestIdleCallback' in window) {
        handle = requestIdleCallback(load);
        this._cleanup.push(() => {
          if ('cancelIdleCallback' in window) cancelIdleCallback(handle);
        });
      } else {
        handle = setTimeout(load, 200);
        this._cleanup.push(() => clearTimeout(handle));
      }
    } else if (strategy === 'interaction') {
      const events = ['click', 'focusin', 'pointerover'];
      const once = () => {
        events.forEach(e => this.removeEventListener(e, once));
        load();
      };
      events.forEach(e => this.addEventListener(e, once, { once: true }));
      this._cleanup.push(() => {
        events.forEach(e => this.removeEventListener(e, once));
      });
    } else {
      // Default: visible (IntersectionObserver, AC4)
      const io = new IntersectionObserver((entries, obs) => {
        if (entries[0] && entries[0].isIntersecting) {
          obs.disconnect();
          load();
        }
      });
      io.observe(this);
      this._cleanup.push(() => io.disconnect());
    }
  }

  disconnectedCallback() {
    this.detach();
  }

  /**
   * Tear down any pending hydration triggers and notify the loaded
   * component (if it exposed `detach`). Idempotent and safe to call
   * multiple times — used by the view-transitions client (#547) to
   * clean up before swapping <main>.
   */
  detach() {
    while (this._cleanup.length) {
      const fn = this._cleanup.pop();
      try { fn(); } catch (e) {}
    }
    if (this._module && typeof this._module.detach === 'function') {
      try { this._module.detach(this); } catch (e) {}
    }
    this._module = null;
  }

  async _hydrate(component) {
    if (this._hydrated) return;
    this._hydrated = true;
    try {
      const props = JSON.parse(this.getAttribute('props') || '{}');
      const mod = await import(`/_islands/${component}.js`);
      this._module = mod;
      if (mod.default) mod.default(this, props);
      else if (mod.hydrate) mod.hydrate(this, props);
    } catch (e) {
      console.error(`[ssg-island] Failed to hydrate "${component}":`, e);
    }
  }
}

customElements.define('ssg-island', SsgIsland);

// Re-arm islands on view-transition page swaps (issue #547).
// The transitions client dispatches `ssg:after-swap` after each
// successful navigation; the new <main> is fresh DOM, so the
// browser's own connectedCallback fires automatically. We only
// need to re-confirm any island whose connectedCallback may have
// raced with the swap.
document.addEventListener('ssg:after-swap', () => {
  document.querySelectorAll('ssg-island').forEach(el => {
    if (el.isConnected && !el._hydrated && el._cleanup.length === 0) {
      try { el.connectedCallback(); } catch (e) {}
    }
  });
});
"#;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extract_components_finds_all() {
        let html = r#"
            <ssg-island component="counter" hydrate="visible"></ssg-island>
            <p>Some text</p>
            <ssg-island component="search" hydrate="idle"></ssg-island>
        "#;
        let components = extract_island_components(html);
        assert_eq!(components.len(), 2);
        assert!(components.contains("counter"));
        assert!(components.contains("search"));
    }

    #[test]
    fn extract_components_deduplicates() {
        let html = r#"
            <ssg-island component="counter" hydrate="visible"></ssg-island>
            <ssg-island component="counter" hydrate="idle"></ssg-island>
        "#;
        let components = extract_island_components(html);
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn extract_components_empty_html() {
        let components =
            extract_island_components("<html><body></body></html>");
        assert!(components.is_empty());
    }

    #[test]
    fn inject_loader_adds_script() {
        let dir = tempdir().unwrap();
        let html_path = dir.path().join("index.html");
        fs::write(&html_path, "<html><body></body></html>").unwrap();

        inject_island_loader(&html_path).unwrap();

        let output = fs::read_to_string(&html_path).unwrap();
        assert!(output.contains("ssg-island.js"));
    }

    #[test]
    fn inject_loader_idempotent() {
        let dir = tempdir().unwrap();
        let html_path = dir.path().join("index.html");
        fs::write(&html_path, "<html><body><script type=\"module\" src=\"/_islands/ssg-island.js\"></script></body></html>").unwrap();

        inject_island_loader(&html_path).unwrap();

        let output = fs::read_to_string(&html_path).unwrap();
        // Should appear exactly once
        assert_eq!(output.matches("ssg-island.js").count(), 1);
    }

    #[test]
    fn island_plugin_name() {
        assert_eq!(IslandPlugin.name(), "islands");
    }

    #[test]
    fn island_plugin_skips_missing_site_dir() {
        let ctx = PluginContext::new(
            Path::new("/tmp/c"),
            Path::new("/tmp/b"),
            Path::new("/nonexistent/site"),
            Path::new("/tmp/t"),
        );
        assert!(IslandPlugin.after_compile(&ctx).is_ok());
    }

    #[test]
    fn island_plugin_processes_pages_with_islands() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let content = dir.path().join("content");
        let islands_src = dir.path().join("islands");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(&islands_src).unwrap();

        // Write a user island bundle
        fs::write(
            islands_src.join("counter.js"),
            "export default (el, props) => {};",
        )
        .unwrap();

        // Write HTML with an island
        let html_content = "<html><body><ssg-island component=\"counter\" hydrate=\"visible\"></ssg-island></body></html>";
        fs::write(site.join("index.html"), html_content).unwrap();

        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());
        IslandPlugin.after_compile(&ctx).unwrap();

        // Check manifest was created
        assert!(site.join("_islands/manifest.json").exists());
        // Check loader was created
        assert!(site.join("_islands/ssg-island.js").exists());
        // Check user bundle was copied
        assert!(site.join("_islands/counter.js").exists());
        // Check loader was injected into HTML via transform_html
        let output = IslandPlugin
            .transform_html(html_content, &site.join("index.html"), &ctx)
            .unwrap();
        assert!(output.contains("ssg-island.js"));
    }

    #[test]
    fn island_plugin_no_islands_in_html() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            "<html><body><p>No islands here</p></body></html>",
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        IslandPlugin.after_compile(&ctx).unwrap();

        // No _islands dir should be created
        assert!(!site.join("_islands").exists());
    }

    #[test]
    fn island_shortcode_expansion() {
        let input = r#"{{< island component="counter" hydrate="visible" >}}"#;
        let result = crate::shortcodes::expand_shortcodes(input);
        assert!(result.contains("<ssg-island"));
        assert!(result.contains("component=\"counter\""));
        assert!(result.contains("hydrate=\"visible\""));
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn island_plugin_new_and_default_yield_same_unit() {
        // Plugin is a unit struct — Default and new() are
        // interchangeable. Exercise both so the function-coverage
        // counter records them (this is the point of the test).
        let a = IslandPlugin::new();
        let b = IslandPlugin::default();
        assert_eq!(a.name(), b.name());
        assert!(a.has_transform());
        assert!(b.has_transform());
    }

    #[test]
    fn island_after_compile_with_html_file_but_zero_island_refs_returns_early()
    {
        // Site dir exists, contains an HTML file with no <ssg-island>,
        // so the components set is empty and the early-return arm fires
        // (the closure that maps over the empty set).
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("page.html"),
            "<html><body><p>nothing</p></body></html>",
        )
        .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        IslandPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("_islands").exists());
    }

    // -------------------------------------------------------------------
    // transform_html — remaining branches
    // -------------------------------------------------------------------

    #[test]
    fn transform_html_skips_when_loader_already_injected() {
        let dir = tempdir().unwrap();
        let ctx =
            PluginContext::new(dir.path(), dir.path(), dir.path(), dir.path());
        let html = "<body><ssg-island component=\"c\"></ssg-island>\
                    <script src=\"/_islands/ssg-island.js\"></script></body>";
        let out = IslandPlugin
            .transform_html(html, Path::new("i.html"), &ctx)
            .unwrap();
        assert_eq!(out, html);
    }

    #[test]
    fn transform_html_appends_loader_when_body_close_missing() {
        let dir = tempdir().unwrap();
        let ctx =
            PluginContext::new(dir.path(), dir.path(), dir.path(), dir.path());
        let html = "<ssg-island component=\"c\"></ssg-island>";
        let out = IslandPlugin
            .transform_html(html, Path::new("i.html"), &ctx)
            .unwrap();
        assert!(out.ends_with("</script>\n"));
        assert!(out.starts_with(html));
    }

    // -------------------------------------------------------------------
    // extract_island_components — parser edges
    // -------------------------------------------------------------------

    #[test]
    fn extract_ignores_empty_component_name() {
        let html = "<ssg-island component=\"\"></ssg-island>";
        assert!(extract_island_components(html).is_empty());
    }

    #[test]
    fn extract_ignores_unterminated_component_value() {
        // No closing quote before the tag's `>` — the value never
        // terminates within the tag.
        let html = "<ssg-island component=\"counter></ssg-island>";
        assert!(extract_island_components(html).is_empty());
    }

    #[test]
    fn extract_stops_on_unterminated_tag() {
        let html = "<ssg-island component=\"counter\"";
        assert!(extract_island_components(html).is_empty());
    }

    // -------------------------------------------------------------------
    // after_compile — copy branches + IO errors
    // -------------------------------------------------------------------

    /// Site layout with one page referencing `counter`.
    fn island_site(dir: &Path) -> std::path::PathBuf {
        let site = dir.join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(
            site.join("index.html"),
            "<html><body><ssg-island component=\"counter\"></ssg-island></body></html>",
        )
        .unwrap();
        site
    }

    #[test]
    fn after_compile_without_source_islands_dir_still_writes_loader() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        // content_dir has no sibling islands/ directory.
        let content = dir.path().join("nested").join("content");
        fs::create_dir_all(&content).unwrap();
        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());

        IslandPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("_islands/ssg-island.js").exists());
        assert!(!site.join("_islands/counter.js").exists());
    }

    #[test]
    fn after_compile_skips_component_without_source_bundle() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        // islands/ exists, but has no counter.js.
        fs::create_dir_all(dir.path().join("islands")).unwrap();
        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());

        IslandPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("_islands/counter.js").exists());
        assert!(site.join("_islands/manifest.json").exists());
    }

    #[test]
    fn after_compile_fails_when_islands_dir_squatted_by_file() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        fs::write(site.join("_islands"), "not a dir").unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err = IslandPlugin.after_compile(&ctx).unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn after_compile_fails_when_bundle_dst_squatted_by_dir() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(dir.path().join("islands")).unwrap();
        fs::write(dir.path().join("islands/counter.js"), "export {}").unwrap();
        // A directory squats the copy destination.
        fs::create_dir_all(site.join("_islands/counter.js")).unwrap();
        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());

        let err = IslandPlugin.after_compile(&ctx).unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn after_compile_fails_when_manifest_squatted_by_dir() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(site.join("_islands/manifest.json")).unwrap();
        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());

        let err = IslandPlugin.after_compile(&ctx).unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    fn after_compile_fails_when_loader_squatted_by_dir() {
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::create_dir_all(site.join("_islands/ssg-island.js")).unwrap();
        let ctx = PluginContext::new(&content, dir.path(), &site, dir.path());

        let err = IslandPlugin.after_compile(&ctx).unwrap_err();
        assert!(!format!("{err}").is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn after_compile_fails_when_html_is_unreadable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let site = island_site(dir.path());
        let html = site.join("index.html");
        fs::set_permissions(&html, fs::Permissions::from_mode(0o000)).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let res = IslandPlugin.after_compile(&ctx);

        let _ = fs::set_permissions(&html, fs::Permissions::from_mode(0o644));
        // Root CI runners bypass perms; only assert when it errored.
        if let Err(e) = res {
            assert!(!format!("{e}").is_empty());
        }
    }

    // -------------------------------------------------------------------
    // inject_island_loader (test-only helper) — remaining branches
    // -------------------------------------------------------------------

    #[test]
    fn inject_island_loader_errors_on_missing_file() {
        let dir = tempdir().unwrap();
        assert!(inject_island_loader(&dir.path().join("nope.html")).is_err());
    }

    #[test]
    fn inject_island_loader_appends_without_body_close() {
        let dir = tempdir().unwrap();
        let page = dir.path().join("p.html");
        fs::write(&page, "<p>no body close</p>").unwrap();
        inject_island_loader(&page).unwrap();
        let out = fs::read_to_string(&page).unwrap();
        assert!(out.ends_with("</script>\n"));
    }

    #[test]
    #[cfg(unix)]
    fn inject_island_loader_write_error_on_readonly_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let page = dir.path().join("p.html");
        fs::write(&page, "<body></body>").unwrap();
        fs::set_permissions(&page, fs::Permissions::from_mode(0o444)).unwrap();

        let res = inject_island_loader(&page);

        let _ = fs::set_permissions(&page, fs::Permissions::from_mode(0o644));
        if let Err(e) = res {
            assert!(!format!("{e}").is_empty());
        }
    }
}
