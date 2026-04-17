// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Resumable hydration — interactive islands (Web Components).
//!
//! Provides an `<ssg-island>` custom element that lazily loads JavaScript
//! component bundles based on configurable hydration strategies:
//! `visible` (IntersectionObserver), `idle` (requestIdleCallback), or
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

use crate::plugin::{Plugin, PluginContext};
use crate::walk;
use anyhow::Result;
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
};

/// Plugin that enables interactive islands via Web Components.
#[derive(Debug, Clone, Copy)]
pub struct IslandPlugin;

impl Plugin for IslandPlugin {
    fn name(&self) -> &'static str {
        "islands"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = walk::walk_files(&ctx.site_dir, "html")
            .unwrap_or_default();

        // Scan all HTML files for <ssg-island component="..."> references
        let mut components = BTreeSet::new();
        let mut pages_with_islands = Vec::new();

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let page_components = extract_island_components(&html);
            if !page_components.is_empty() {
                components.extend(page_components);
                pages_with_islands.push(path.clone());
            }
        }

        if components.is_empty() {
            return Ok(());
        }

        let islands_dir = ctx.site_dir.join("_islands");
        fs::create_dir_all(&islands_dir)?;

        // Copy user-provided island bundles from source islands/ dir
        let source_islands = ctx.content_dir
            .parent()
            .unwrap_or(&ctx.content_dir)
            .join("islands");

        if source_islands.exists() {
            for component in &components {
                let src = source_islands.join(format!("{component}.js"));
                if src.exists() {
                    let dst = islands_dir.join(format!("{component}.js"));
                    let _ = fs::copy(&src, &dst)?;
                }
            }
        }

        // Write manifest
        let manifest: Vec<_> = components.iter().collect();
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .unwrap_or_else(|_| "[]".to_string());
        fs::write(islands_dir.join("manifest.json"), manifest_json)?;

        // Write the ssg-island.js custom element loader
        fs::write(islands_dir.join("ssg-island.js"), ISLAND_LOADER_JS)?;

        // Inject the loader script into pages that contain islands
        for path in &pages_with_islands {
            inject_island_loader(path)?;
        }

        log::info!(
            "[islands] {} component(s), {} page(s) with islands",
            components.len(),
            pages_with_islands.len()
        );
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

/// Injects the island loader `<script>` before `</body>`.
fn inject_island_loader(path: &Path) -> Result<()> {
    let html = fs::read_to_string(path)?;

    if html.contains("ssg-island.js") {
        return Ok(()); // Already injected
    }

    let script = "\n<script type=\"module\" src=\"/_islands/ssg-island.js\"></script>\n";

    let output = if let Some(pos) = html.rfind("</body>") {
        format!("{}{script}{}", &html[..pos], &html[pos..])
    } else {
        format!("{html}{script}")
    };

    fs::write(path, output)?;
    Ok(())
}

/// The `<ssg-island>` custom element loader.
///
/// - `hydrate="visible"`: loads when element enters viewport (IntersectionObserver)
/// - `hydrate="idle"`: loads during browser idle time (requestIdleCallback)
/// - `hydrate="interaction"`: loads on first click/focus/hover
const ISLAND_LOADER_JS: &str = r#"/**
 * SSG Island — lazy-hydrating Web Component loader.
 * Each <ssg-island> loads its component bundle on demand.
 */
class SsgIsland extends HTMLElement {
  connectedCallback() {
    const strategy = this.getAttribute('hydrate') || 'visible';
    const component = this.getAttribute('component');
    if (!component) return;

    const load = () => this._hydrate(component);

    if (strategy === 'idle') {
      ('requestIdleCallback' in window)
        ? requestIdleCallback(load)
        : setTimeout(load, 200);
    } else if (strategy === 'interaction') {
      const events = ['click', 'focusin', 'pointerover'];
      const once = () => {
        events.forEach(e => this.removeEventListener(e, once));
        load();
      };
      events.forEach(e => this.addEventListener(e, once, { once: true }));
    } else {
      // Default: visible (IntersectionObserver)
      const io = new IntersectionObserver((entries, obs) => {
        if (entries[0].isIntersecting) {
          obs.disconnect();
          load();
        }
      });
      io.observe(this);
    }
  }

  async _hydrate(component) {
    try {
      const props = JSON.parse(this.getAttribute('props') || '{}');
      const mod = await import(`/_islands/${component}.js`);
      if (mod.default) mod.default(this, props);
      else if (mod.hydrate) mod.hydrate(this, props);
    } catch (e) {
      console.error(`[ssg-island] Failed to hydrate "${component}":`, e);
    }
  }
}

customElements.define('ssg-island', SsgIsland);
"#;

#[cfg(test)]
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
        let components = extract_island_components("<html><body></body></html>");
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
        fs::write(islands_src.join("counter.js"), "export default (el, props) => {};").unwrap();

        // Write HTML with an island
        fs::write(
            site.join("index.html"),
            "<html><body><ssg-island component=\"counter\" hydrate=\"visible\"></ssg-island></body></html>",
        ).unwrap();

        let ctx = PluginContext::new(
            &content,
            dir.path(),
            &site,
            dir.path(),
        );
        IslandPlugin.after_compile(&ctx).unwrap();

        // Check manifest was created
        assert!(site.join("_islands/manifest.json").exists());
        // Check loader was created
        assert!(site.join("_islands/ssg-island.js").exists());
        // Check user bundle was copied
        assert!(site.join("_islands/counter.js").exists());
        // Check loader was injected into HTML
        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(output.contains("ssg-island.js"));
    }

    #[test]
    fn island_shortcode_expansion() {
        let input = r#"{{< island component="counter" hydrate="visible" >}}"#;
        let result = crate::shortcodes::expand_shortcodes(input);
        assert!(result.contains("<ssg-island"));
        assert!(result.contains("component=\"counter\""));
        assert!(result.contains("hydrate=\"visible\""));
    }
}
