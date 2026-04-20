// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Auto-generates Open Graph social card images from page metadata.
//!
//! For each HTML page, generates a branded SVG social card containing
//! the page title and site name. Injects the `og:image` meta tag
//! pointing to the generated image.
//!
//! No external dependencies — uses inline SVG generation.

use crate::plugin::{Plugin, PluginContext};
use crate::seo::helpers::{extract_title, has_meta_tag};
use anyhow::Result;
use std::{fs, path::Path};

/// Plugin that auto-generates Open Graph social card images.
#[derive(Debug, Clone)]
pub struct OgImagePlugin {
    /// Base URL for the site (used in og:image URLs).
    base_url: String,
    /// Background colour for the card (CSS hex).
    brand_color: String,
    /// Text colour (CSS hex).
    text_color: String,
}

impl OgImagePlugin {
    /// Creates a new `OgImagePlugin` with default branding.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            brand_color: "#1a1a2e".to_string(),
            text_color: "#ffffff".to_string(),
        }
    }

    /// Creates a plugin with custom brand colours.
    #[must_use]
    pub fn with_colors(
        base_url: impl Into<String>,
        brand_color: impl Into<String>,
        text_color: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            brand_color: brand_color.into(),
            text_color: text_color.into(),
        }
    }
}

/// Generates an SVG social card with the given title and site name.
///
/// The card is 1200x630 pixels (standard OG image dimensions).
#[must_use]
pub fn generate_og_svg(
    title: &str,
    site_name: &str,
    brand_color: &str,
    text_color: &str,
) -> String {
    let escaped_title = escape_svg(title);
    let escaped_site = escape_svg(site_name);

    // Wrap long titles across multiple lines
    let lines = wrap_text(&escaped_title, 30);
    let title_y_start = if lines.len() == 1 { 300 } else { 260 };

    let mut title_elements = String::new();
    for (i, line) in lines.iter().enumerate() {
        let y = title_y_start + i * 60;
        title_elements.push_str(&format!(
            r#"    <text x="600" y="{y}" font-family="system-ui, -apple-system, sans-serif" font-size="48" font-weight="bold" fill="{text_color}" text-anchor="middle">{line}</text>
"#
        ));
    }

    let site_y = title_y_start + lines.len() * 60 + 60;
    let divider_y = title_y_start + lines.len() * 60 + 20;

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <rect width="1200" height="630" fill="{brand_color}"/>
  <rect x="40" y="40" width="1120" height="550" rx="16" fill="none" stroke="{text_color}" stroke-opacity="0.15" stroke-width="2"/>
{title_elements}  <text x="600" y="{site_y}" font-family="system-ui, -apple-system, sans-serif" font-size="24" fill="{text_color}" fill-opacity="0.7" text-anchor="middle">{escaped_site}</text>
  <rect x="520" y="{divider_y}" width="160" height="3" rx="2" fill="{text_color}" fill-opacity="0.3"/>
</svg>"#
    )
}

/// Wraps text into lines of approximately `max_chars` characters.
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in words {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > max_chars {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    // Limit to 4 lines to stay within the card
    lines.truncate(4);
    lines
}

/// Escapes text for safe inclusion in SVG.
fn escape_svg(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Derives a URL-safe slug from a file path relative to the site directory.
fn slug_from_path(path: &Path, site_dir: &Path) -> String {
    let rel = path.strip_prefix(site_dir).unwrap_or(path);
    let stem = rel.with_extension("");
    stem.to_string_lossy()
        .replace(['/', '\\'], "-")
        .trim_matches('-')
        .to_string()
}

impl Plugin for OgImagePlugin {
    fn name(&self) -> &'static str {
        "og-image"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = ctx.get_html_files();
        let base = self.base_url.trim_end_matches('/');
        let site_name =
            ctx.config.as_ref().map_or("", |c| c.site_name.as_str());
        let mut generated = 0usize;

        for path in &html_files {
            let Ok(html) = fs::read_to_string(path) else {
                continue;
            };

            // Skip pages that already have an og:image
            if has_meta_tag(&html, "og:image") {
                continue;
            }

            let title = extract_title(&html);
            if title.is_empty() {
                continue;
            }

            let slug = slug_from_path(path, &ctx.site_dir);
            let svg_filename = format!("og-{slug}.svg");
            let svg_path = ctx.site_dir.join(&svg_filename);

            // Generate SVG
            let svg = generate_og_svg(
                &title,
                site_name,
                &self.brand_color,
                &self.text_color,
            );
            fs::write(&svg_path, &svg)?;

            // Inject og:image meta tag
            let og_url = format!("{base}/{svg_filename}");
            let meta = format!(
                "<meta property=\"og:image\" content=\"{og_url}\">\n\
                 <meta property=\"og:image:width\" content=\"1200\">\n\
                 <meta property=\"og:image:height\" content=\"630\">\n"
            );

            if let Some(pos) = html.find("</head>") {
                let mut modified = html.clone();
                modified.insert_str(pos, &meta);
                fs::write(path, &modified)?;
                generated += 1;
            }
        }

        if generated > 0 {
            log::info!("[og-image] Generated {generated} social card(s)");
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn generate_og_svg_basic() {
        let svg =
            generate_og_svg("Hello World", "My Site", "#1a1a2e", "#ffffff");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Hello World"));
        assert!(svg.contains("My Site"));
        assert!(svg.contains("#1a1a2e"));
        assert!(svg.contains("1200"));
        assert!(svg.contains("630"));
    }

    #[test]
    fn generate_og_svg_escapes_html() {
        let svg = generate_og_svg("A <B> & C", "Site \"X\"", "#000", "#fff");
        assert!(svg.contains("A &lt;B&gt; &amp; C"));
        assert!(svg.contains("Site &quot;X&quot;"));
    }

    #[test]
    fn generate_og_svg_wraps_long_title() {
        let title = "This Is A Very Long Title That Should Be Wrapped Across Multiple Lines";
        let svg = generate_og_svg(title, "Site", "#000", "#fff");
        // Should contain multiple <text> elements for the title
        let text_count = svg.matches("<text").count();
        assert!(
            text_count >= 3,
            "Long title should wrap, got {text_count} text elements"
        );
    }

    #[test]
    fn wrap_text_short() {
        let lines = wrap_text("Hello", 30);
        assert_eq!(lines, vec!["Hello"]);
    }

    #[test]
    fn wrap_text_long() {
        let lines =
            wrap_text("one two three four five six seven eight nine ten", 15);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= 20, "Line too long: {line}");
        }
    }

    #[test]
    fn wrap_text_empty() {
        let lines = wrap_text("", 30);
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn wrap_text_truncates_at_4_lines() {
        let long = "a b c d e f g h i j k l m n o p q r s t u v w x y z";
        let lines = wrap_text(long, 5);
        assert!(lines.len() <= 4);
    }

    #[test]
    fn escape_svg_special_chars() {
        assert_eq!(escape_svg("a & b"), "a &amp; b");
        assert_eq!(escape_svg("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_svg("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn slug_from_path_basic() {
        let slug = slug_from_path(
            Path::new("/site/about/index.html"),
            Path::new("/site"),
        );
        assert_eq!(slug, "about-index");
    }

    #[test]
    fn slug_from_path_root() {
        let slug =
            slug_from_path(Path::new("/site/index.html"), Path::new("/site"));
        assert_eq!(slug, "index");
    }

    #[test]
    fn og_image_plugin_name() {
        let plugin = OgImagePlugin::new("https://example.com");
        assert_eq!(plugin.name(), "og-image");
    }

    #[test]
    fn og_image_plugin_skips_missing_site_dir() {
        let plugin = OgImagePlugin::new("https://example.com");
        let ctx = PluginContext::new(
            Path::new("/tmp/c"),
            Path::new("/tmp/b"),
            Path::new("/nonexistent/site"),
            Path::new("/tmp/t"),
        );
        assert!(plugin.after_compile(&ctx).is_ok());
    }

    #[test]
    fn og_image_plugin_generates_svg_and_injects_meta() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html =
            "<html><head><title>Test Page</title></head><body></body></html>";
        fs::write(site.join("index.html"), html).unwrap();

        let plugin = OgImagePlugin::new("https://example.com");
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        plugin.after_compile(&ctx).unwrap();

        // Check SVG was created
        let svg_path = site.join("og-index.svg");
        assert!(svg_path.exists(), "SVG file should be created");
        let svg = fs::read_to_string(&svg_path).unwrap();
        assert!(svg.contains("Test Page"));

        // Check meta tag was injected
        let modified = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(modified.contains("og:image"));
        assert!(modified.contains("og-index.svg"));
    }

    #[test]
    fn og_image_plugin_skips_existing_og_image() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = r#"<html><head><title>T</title><meta property="og:image" content="existing.jpg"></head><body></body></html>"#;
        fs::write(site.join("index.html"), html).unwrap();

        let plugin = OgImagePlugin::new("https://example.com");
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        plugin.after_compile(&ctx).unwrap();

        // SVG should NOT be created
        assert!(!site.join("og-index.svg").exists());
    }

    #[test]
    fn og_image_with_custom_colors() {
        let plugin = OgImagePlugin::with_colors(
            "https://example.com",
            "#ff0000",
            "#00ff00",
        );
        assert_eq!(plugin.brand_color, "#ff0000");
        assert_eq!(plugin.text_color, "#00ff00");
    }
}
