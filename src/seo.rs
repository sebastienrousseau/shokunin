// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SEO plugins for the static site generator.
//!
//! Provides three plugins that improve search engine optimization:
//!
//! - `SeoPlugin` — Injects missing meta tags (description, Open Graph,
//!   Twitter Card) into HTML files.
//! - `RobotsPlugin` — Generates a `robots.txt` file.
//! - `CanonicalPlugin` — Injects `<link rel="canonical">` tags.

use crate::plugin::{Plugin, PluginContext};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

// =====================================================================
// Helper functions
// =====================================================================

/// Extract the page title from the `<title>` tag.
fn extract_title(html: &str) -> String {
    if let Some(start) = html.find("<title>") {
        let after = &html[start + 7..];
        if let Some(end) = after.find("</title>") {
            let title = strip_tags(&after[..end]);
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

/// Extract plain text from the page content, strip tags, and truncate to
/// `max_len` characters.
///
/// Prefers `<main>` content if present. Falls back to `<body>` with nav,
/// header, footer, script, and style blocks removed.
fn extract_description(html: &str, max_len: usize) -> String {
    // Try to extract from <main> first — this is the actual page content
    let content = if let Some(start) = html.find("<main") {
        let after = &html[start..];
        if let Some(gt) = after.find('>') {
            let inner = &after[gt + 1..];
            if let Some(end) = inner.find("</main>") {
                inner[..end].to_string()
            } else {
                inner.to_string()
            }
        } else {
            String::new()
        }
    } else {
        // Fall back to <body> with non-content elements stripped
        let body = if let Some(start) = html.find("<body") {
            let after = &html[start..];
            if let Some(gt) = after.find('>') {
                let inner = &after[gt + 1..];
                if let Some(end) = inner.find("</body>") {
                    inner[..end].to_string()
                } else {
                    inner.to_string()
                }
            } else {
                String::new()
            }
        } else {
            html.to_string()
        };

        let mut clean = body;
        for tag in &["script", "style", "nav", "header", "footer"] {
            let open = format!("<{tag}");
            let close = format!("</{tag}>");
            while let Some(start) = clean.find(&open) {
                if let Some(end) = clean[start..].find(&close) {
                    clean.replace_range(start..start + end + close.len(), " ");
                } else {
                    break;
                }
            }
        }
        clean
    };

    // Strip remaining HTML tags from <main> content too
    let mut clean = content;
    for tag in &["script", "style"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(start) = clean.find(&open) {
            if let Some(end) = clean[start..].find(&close) {
                clean.replace_range(start..start + end + close.len(), " ");
            } else {
                break;
            }
        }
    }

    let text = strip_tags(&clean);
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        // Find a char boundary at or before max_len
        let mut end = max_len;
        while end > 0 && !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        let truncated = &trimmed[..end];
        // Truncate at word boundary
        if let Some(last_space) = truncated.rfind(' ') {
            truncated[..last_space].to_string()
        } else {
            truncated.to_string()
        }
    }
}

/// Remove all HTML tags and collapse whitespace.
fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_space = false;
        }
    }
    collapsed.trim().to_string()
}

/// Collect all `.html` files under `dir` using an iterative directory walk.
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .with_context(|| format!("cannot read {}", current.display()))?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Escape a string for safe inclusion in an HTML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// =====================================================================
// SeoPlugin
// =====================================================================

/// Injects missing SEO meta tags into HTML files.
///
/// After compilation, this plugin scans all HTML files in the site
/// directory and adds any missing meta tags for description, Open Graph
/// (title, description, type), and Twitter Card.
///
/// The plugin is idempotent — it checks for existing tags before
/// injecting and will not duplicate them.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::SeoPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(SeoPlugin);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SeoPlugin;

impl Plugin for SeoPlugin {
    fn name(&self) -> &'static str {
        "seo"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        for path in &html_files {
            inject_seo_tags(path)?;
        }

        Ok(())
    }
}

/// Inject missing SEO meta tags into a single HTML file.
fn inject_seo_tags(path: &Path) -> Result<()> {
    let html = fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    let mut tags = Vec::new();

    let title = extract_title(&html);
    let description = extract_description(&html, 160);

    // Meta description
    if !html.contains("<meta name=\"description\"")
        && !html.contains("<meta name='description'")
        && !description.is_empty()
    {
        tags.push(format!(
            "<meta name=\"description\" content=\"{}\">",
            escape_attr(&description)
        ));
    }

    // Open Graph title
    if !html.contains("<meta property=\"og:title\"")
        && !html.contains("<meta property='og:title'")
        && !title.is_empty()
    {
        tags.push(format!(
            "<meta property=\"og:title\" content=\"{}\">",
            escape_attr(&title)
        ));
    }

    // Open Graph description
    if !html.contains("<meta property=\"og:description\"")
        && !html.contains("<meta property='og:description'")
        && !description.is_empty()
    {
        tags.push(format!(
            "<meta property=\"og:description\" content=\"{}\">",
            escape_attr(&description)
        ));
    }

    // Open Graph type
    if !html.contains("<meta property=\"og:type\"")
        && !html.contains("<meta property='og:type'")
    {
        tags.push(
            "<meta property=\"og:type\" content=\"website\">".to_string(),
        );
    }

    // Twitter card
    if !html.contains("<meta name=\"twitter:card\"")
        && !html.contains("<meta name='twitter:card'")
    {
        tags.push(
            "<meta name=\"twitter:card\" content=\"summary\">".to_string(),
        );
    }

    if tags.is_empty() {
        return Ok(());
    }

    let injection = tags.join("\n");
    let result = if let Some(pos) = html.find("</head>") {
        format!("{}{}\n{}", &html[..pos], injection, &html[pos..])
    } else {
        html
    };

    fs::write(path, result)
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}

// =====================================================================
// RobotsPlugin
// =====================================================================

/// Generates a `robots.txt` file in the site directory.
///
/// The file allows all user agents and references the sitemap at
/// `{base_url}/sitemap.xml`. If a `robots.txt` already exists, it is
/// not overwritten.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::RobotsPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(RobotsPlugin::new("https://example.com"));
/// ```
#[derive(Debug, Clone)]
pub struct RobotsPlugin {
    base_url: String,
}

impl RobotsPlugin {
    /// Creates a new `RobotsPlugin` with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Plugin for RobotsPlugin {
    fn name(&self) -> &'static str {
        "robots"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let robots_path = ctx.site_dir.join("robots.txt");
        if robots_path.exists() {
            return Ok(());
        }

        let content = format!(
            "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
            self.base_url.trim_end_matches('/')
        );

        fs::write(&robots_path, content).with_context(|| {
            format!("cannot write {}", robots_path.display())
        })?;

        Ok(())
    }
}

// =====================================================================
// CanonicalPlugin
// =====================================================================

/// Injects `<link rel="canonical">` tags into HTML files.
///
/// For each HTML file missing a canonical link, this plugin computes
/// the canonical URL from the base URL and the file's relative path,
/// then injects the tag before `</head>`.
///
/// The plugin is idempotent — it will not add a duplicate canonical
/// link if one already exists.
///
/// # Example
///
/// ```rust
/// use ssg::plugin::PluginManager;
/// use ssg::seo::CanonicalPlugin;
///
/// let mut pm = PluginManager::new();
/// pm.register(CanonicalPlugin::new("https://example.com"));
/// ```
#[derive(Debug, Clone)]
pub struct CanonicalPlugin {
    base_url: String,
}

impl CanonicalPlugin {
    /// Creates a new `CanonicalPlugin` with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Plugin for CanonicalPlugin {
    fn name(&self) -> &'static str {
        "canonical"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files(&ctx.site_dir)?;
        let base = self.base_url.trim_end_matches('/');

        for path in &html_files {
            let html = fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;

            if html.contains("<link rel=\"canonical\"")
                || html.contains("<link rel='canonical'")
            {
                continue;
            }

            let rel_path = path
                .strip_prefix(&ctx.site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let canonical_url = format!("{base}/{rel_path}");
            let tag = format!(
                "<link rel=\"canonical\" href=\"{}\">",
                escape_attr(&canonical_url)
            );

            let result = if let Some(pos) = html.find("</head>") {
                format!("{}{}\n{}", &html[..pos], tag, &html[pos..])
            } else {
                html
            };

            fs::write(path, result)
                .with_context(|| format!("cannot write {}", path.display()))?;
        }

        Ok(())
    }
}

// =====================================================================
// JSON-LD Structured Data Plugin
// =====================================================================

/// Configuration for the JSON-LD structured data plugin.
#[derive(Debug, Clone)]
pub struct JsonLdConfig {
    /// Base URL of the site (for absolute URLs in JSON-LD).
    pub base_url: String,
    /// Organization name for Organization schema.
    pub org_name: String,
    /// Whether to generate `BreadcrumbList` for every page.
    pub breadcrumbs: bool,
}

/// Injects JSON-LD structured data into HTML files.
///
/// Auto-detects schema.org types from page metadata:
/// - Pages with `<article>` → `Article`
/// - All other pages → `WebPage`
/// - `BreadcrumbList` derived from URL path (opt-in)
///
/// Idempotent: skips files that already contain `application/ld+json`.
#[derive(Debug, Clone)]
pub struct JsonLdPlugin {
    config: JsonLdConfig,
}

impl JsonLdPlugin {
    /// Creates a new `JsonLdPlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: JsonLdConfig) -> Self {
        Self { config }
    }

    /// Creates a `JsonLdPlugin` from site config values.
    #[must_use]
    pub fn from_site(base_url: &str, site_name: &str) -> Self {
        Self {
            config: JsonLdConfig {
                base_url: base_url.to_string(),
                org_name: site_name.to_string(),
                breadcrumbs: true,
            },
        }
    }
}

impl Plugin for JsonLdPlugin {
    fn name(&self) -> &'static str {
        "json-ld"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let html_files = collect_html_files_recursive(&ctx.site_dir)?;
        let mut injected = 0usize;
        let base = self.config.base_url.trim_end_matches('/');

        for path in &html_files {
            let html = fs::read_to_string(path)?;

            // Skip if already has JSON-LD
            if html.contains("application/ld+json") {
                continue;
            }

            let head_pos = match html.find("</head>") {
                Some(p) => p,
                None => continue,
            };

            let title = extract_title(&html);
            let description = extract_description(&html, 160);
            let rel_path = path
                .strip_prefix(&ctx.site_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let page_url = format!("{base}/{rel_path}");

            let mut scripts = Vec::new();

            // Determine type: Article if <article> present, else WebPage
            if html.contains("<article") {
                let article = serde_json::json!({
                    "@context": "https://schema.org",
                    "@type": "Article",
                    "headline": title,
                    "description": description,
                    "url": page_url,
                    "mainEntityOfPage": {
                        "@type": "WebPage",
                        "@id": page_url
                    },
                    "publisher": {
                        "@type": "Organization",
                        "name": self.config.org_name
                    }
                });
                scripts.push(article);
            } else {
                let webpage = serde_json::json!({
                    "@context": "https://schema.org",
                    "@type": "WebPage",
                    "name": title,
                    "description": description,
                    "url": page_url
                });
                scripts.push(webpage);
            }

            // BreadcrumbList
            if self.config.breadcrumbs {
                let parts: Vec<&str> = rel_path
                    .trim_matches('/')
                    .split('/')
                    .filter(|p| !p.is_empty() && *p != "index.html")
                    .collect();

                if !parts.is_empty() {
                    let mut items = vec![serde_json::json!({
                        "@type": "ListItem",
                        "position": 1,
                        "name": "Home",
                        "item": format!("{}/", base)
                    })];

                    let mut accumulated = String::new();
                    for (i, part) in parts.iter().enumerate() {
                        accumulated = format!("{accumulated}/{part}");
                        let name =
                            part.trim_end_matches(".html").replace('-', " ");
                        items.push(serde_json::json!({
                            "@type": "ListItem",
                            "position": i + 2,
                            "name": name,
                            "item": format!("{}{}", base, accumulated)
                        }));
                    }

                    let breadcrumb = serde_json::json!({
                        "@context": "https://schema.org",
                        "@type": "BreadcrumbList",
                        "itemListElement": items
                    });
                    scripts.push(breadcrumb);
                }
            }

            // Inject all JSON-LD scripts before </head>
            let mut injection = String::new();
            for script in &scripts {
                let json = serde_json::to_string(script)?;
                injection.push_str(&format!(
                    "<script type=\"application/ld+json\">{json}</script>\n"
                ));
            }

            let result = format!(
                "{}{}{}",
                &html[..head_pos],
                injection,
                &html[head_pos..]
            );
            fs::write(path, result)?;
            injected += 1;
        }

        if injected > 0 {
            log::info!(
                "[json-ld] Injected structured data into {injected} page(s)"
            );
        }
        Ok(())
    }
}

/// Recursively collects HTML files (shared helper).
fn collect_html_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if !current.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "html") {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginManager;
    use tempfile::tempdir;

    fn make_html(title: &str, body: &str) -> String {
        format!(
            "<html><head><title>{title}</title></head>\
             <body>{body}</body></html>"
        )
    }

    fn test_ctx(site_dir: &Path) -> PluginContext {
        PluginContext::new(
            Path::new("content"),
            Path::new("build"),
            site_dir,
            Path::new("templates"),
        )
    }

    // -----------------------------------------------------------------
    // Helper function tests
    // -----------------------------------------------------------------

    #[test]
    fn test_extract_title_present() {
        let html = "<html><head><title>My Page</title></head></html>";
        assert_eq!(extract_title(html), "My Page");
    }

    #[test]
    fn test_extract_title_missing() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn test_extract_description_truncates() {
        let long = "word ".repeat(100);
        let html =
            format!("<html><head></head><body><p>{long}</p></body></html>");
        let desc = extract_description(&html, 160);
        assert!(desc.len() <= 160);
        assert!(!desc.is_empty());
    }

    // -----------------------------------------------------------------
    // SeoPlugin tests
    // -----------------------------------------------------------------

    #[test]
    fn test_seo_plugin_name() {
        assert_eq!(SeoPlugin.name(), "seo");
    }

    #[test]
    fn test_seo_plugin_injects_meta_tags() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Hello World", "<p>Some content here</p>"),
        )?;

        let ctx = test_ctx(tmp.path());
        SeoPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(tmp.path().join("index.html"))?;
        assert!(result.contains("<meta name=\"description\""));
        assert!(result.contains("<meta property=\"og:title\""));
        assert!(result.contains("Hello World"));
        assert!(result.contains("<meta property=\"og:description\""));
        assert!(
            result.contains("<meta property=\"og:type\" content=\"website\"")
        );
        assert!(
            result.contains("<meta name=\"twitter:card\" content=\"summary\"")
        );
        Ok(())
    }

    #[test]
    fn test_seo_plugin_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("page.html"),
            make_html("Test", "<p>Content</p>"),
        )?;

        let ctx = test_ctx(tmp.path());
        SeoPlugin.after_compile(&ctx)?;
        let first = fs::read_to_string(tmp.path().join("page.html"))?;

        SeoPlugin.after_compile(&ctx)?;
        let second = fs::read_to_string(tmp.path().join("page.html"))?;

        assert_eq!(first, second);
        Ok(())
    }

    #[test]
    fn test_extract_description_excludes_nav_header_footer() {
        let html = r##"<html><head></head><body>
            <a href="#main">Skip to content</a>
            <nav><ul><li>Home</li><li>About</li><li>Search</li></ul></nav>
            <header><h1>Site Header</h1></header>
            <main><p>This is the actual page content that should be extracted.</p></main>
            <footer><p>Copyright 2026</p></footer>
            </body></html>"##;
        let desc = extract_description(html, 160);
        assert!(
            desc.contains("actual page content"),
            "description should contain main content, got: {desc}"
        );
        assert!(
            !desc.contains("Skip to content"),
            "description should not contain skip link text"
        );
        assert!(
            !desc.contains("Site Header"),
            "description should not contain header text"
        );
        assert!(
            !desc.contains("Copyright"),
            "description should not contain footer text"
        );
    }

    #[test]
    fn test_seo_plugin_handles_missing_title() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("no-title.html"),
            "<html><head></head><body><p>No title here</p></body></html>",
        )?;

        let ctx = test_ctx(tmp.path());
        SeoPlugin.after_compile(&ctx)?;

        let result = fs::read_to_string(tmp.path().join("no-title.html"))?;
        // Should still inject og:type and twitter:card
        assert!(result.contains("<meta property=\"og:type\""));
        assert!(result.contains("<meta name=\"twitter:card\""));
        // Should not inject og:title (no title available)
        assert!(!result.contains("<meta property=\"og:title\""));
        Ok(())
    }

    #[test]
    fn test_seo_plugin_empty_dir() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        assert!(SeoPlugin.after_compile(&ctx).is_ok());
        Ok(())
    }

    #[test]
    fn test_seo_plugin_nonexistent_dir() -> Result<()> {
        let ctx = test_ctx(Path::new("/nonexistent/path"));
        assert!(SeoPlugin.after_compile(&ctx).is_ok());
        Ok(())
    }

    // -----------------------------------------------------------------
    // RobotsPlugin tests
    // -----------------------------------------------------------------

    #[test]
    fn test_robots_plugin_name() {
        let plugin = RobotsPlugin::new("https://example.com");
        assert_eq!(plugin.name(), "robots");
    }

    #[test]
    fn test_robots_plugin_creates_file() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;

        let path = tmp.path().join("robots.txt");
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn test_robots_plugin_correct_content() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("robots.txt"))?;
        assert!(content.contains("User-agent: *"));
        assert!(content.contains("Allow: /"));
        assert!(content.contains("Sitemap: https://example.com/sitemap.xml"));
        Ok(())
    }

    #[test]
    fn test_robots_plugin_does_not_overwrite() -> Result<()> {
        let tmp = tempdir()?;
        let robots_path = tmp.path().join("robots.txt");
        fs::write(&robots_path, "User-agent: *\nDisallow: /secret\n")?;

        let ctx = test_ctx(tmp.path());
        let plugin = RobotsPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;

        let content = fs::read_to_string(&robots_path)?;
        assert!(content.contains("Disallow: /secret"));
        assert!(!content.contains("Sitemap:"));
        Ok(())
    }

    #[test]
    fn test_robots_plugin_custom_base_url() -> Result<()> {
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        let plugin = RobotsPlugin::new("https://my-site.org");
        plugin.after_compile(&ctx)?;

        let content = fs::read_to_string(tmp.path().join("robots.txt"))?;
        assert!(content.contains("Sitemap: https://my-site.org/sitemap.xml"));
        Ok(())
    }

    // -----------------------------------------------------------------
    // CanonicalPlugin tests
    // -----------------------------------------------------------------

    #[test]
    fn test_canonical_plugin_name() {
        let plugin = CanonicalPlugin::new("https://example.com");
        assert_eq!(plugin.name(), "canonical");
    }

    #[test]
    fn test_canonical_plugin_injects_tag() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Home", "<p>Welcome</p>"),
        )?;

        let ctx = test_ctx(tmp.path());
        let plugin = CanonicalPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;

        let result = fs::read_to_string(tmp.path().join("index.html"))?;
        assert!(result.contains("<link rel=\"canonical\""));
        assert!(result.contains("https://example.com/index.html"));
        Ok(())
    }

    #[test]
    fn test_canonical_plugin_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("page.html"),
            make_html("Page", "<p>Content</p>"),
        )?;

        let ctx = test_ctx(tmp.path());
        let plugin = CanonicalPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;
        let first = fs::read_to_string(tmp.path().join("page.html"))?;

        plugin.after_compile(&ctx)?;
        let second = fs::read_to_string(tmp.path().join("page.html"))?;

        assert_eq!(first, second);
        Ok(())
    }

    #[test]
    fn test_canonical_plugin_nested_files() -> Result<()> {
        let tmp = tempdir()?;
        fs::create_dir_all(tmp.path().join("blog"))?;
        fs::write(
            tmp.path().join("blog/post.html"),
            make_html("Post", "<p>Blog post</p>"),
        )?;

        let ctx = test_ctx(tmp.path());
        let plugin = CanonicalPlugin::new("https://example.com");
        plugin.after_compile(&ctx)?;

        let result = fs::read_to_string(tmp.path().join("blog/post.html"))?;
        assert!(result.contains("https://example.com/blog/post.html"));
        Ok(())
    }

    // -----------------------------------------------------------------
    // Registration tests
    // -----------------------------------------------------------------

    #[test]
    fn test_all_plugins_register() {
        let mut pm = PluginManager::new();
        pm.register(SeoPlugin);
        pm.register(RobotsPlugin::new("https://example.com"));
        pm.register(CanonicalPlugin::new("https://example.com"));
        assert_eq!(pm.len(), 3);
        assert_eq!(pm.names(), vec!["seo", "robots", "canonical"]);
    }

    // -----------------------------------------------------------------
    // Additional edge-case tests
    // -----------------------------------------------------------------

    #[test]
    fn extract_description_unicode_truncation_respects_char_boundary() {
        // Arrange: multi-byte chars (é = 2 bytes, 日 = 3 bytes)
        let text = "café 日本語 ".repeat(30);
        let html =
            format!("<html><head></head><body><p>{text}</p></body></html>");

        // Act
        let desc = extract_description(&html, 50);

        // Assert: result is valid UTF-8 and within limit
        assert!(desc.len() <= 50);
        assert!(!desc.is_empty());
        // Verify it doesn't panic and is a valid string
        let _ = desc.chars().count();
    }

    #[test]
    fn extract_description_empty_main_falls_back_to_body() {
        // Arrange: <main> is present but empty
        let html = "<html><head></head><body>\
                     <main></main>\
                     <p>Body fallback text</p>\
                     </body></html>";

        // Act
        let desc = extract_description(html, 160);

        // Assert: empty main yields empty string (main takes priority)
        assert!(
            desc.is_empty(),
            "expected empty description from empty <main>, got: {desc}"
        );
    }

    #[test]
    fn extract_description_no_body_uses_raw_html() {
        // Arrange: no <body> tag at all
        let html = "<div><p>Raw content without body</p></div>";

        // Act
        let desc = extract_description(html, 160);

        // Assert: falls back to raw HTML content
        assert!(
            desc.contains("Raw content without body"),
            "expected raw content fallback, got: {desc}"
        );
    }

    #[test]
    fn extract_title_with_nested_tags() {
        // Arrange: title contains nested HTML tags
        let html = "<html><head><title><span>Foo</span></title></head></html>";

        // Act
        let title = extract_title(html);

        // Assert: nested tags are stripped, text is preserved
        assert_eq!(title, "Foo");
    }

    #[test]
    fn escape_attr_all_special_chars() {
        // Arrange
        let input = r#"Tom & "Jerry" <script>alert('xss')</script>"#;

        // Act
        let escaped = escape_attr(input);

        // Assert: all special chars are escaped
        assert!(escaped.contains("&amp;"), "& should be escaped");
        assert!(escaped.contains("&quot;"), "\" should be escaped");
        assert!(escaped.contains("&lt;"), "< should be escaped");
        assert!(escaped.contains("&gt;"), "> should be escaped");
        assert_eq!(
            escaped,
            "Tom &amp; &quot;Jerry&quot; &lt;script&gt;alert('xss')&lt;/script&gt;"
        );
    }

    #[test]
    fn seo_plugin_skips_existing_single_quote_meta() -> Result<()> {
        // Arrange: meta tags use single quotes
        let html = "<html><head>\
                     <meta name='description' content='Already set'>\
                     <meta property='og:title' content='Title'>\
                     <meta property='og:description' content='Desc'>\
                     <meta property='og:type' content='website'>\
                     <meta name='twitter:card' content='summary'>\
                     <title>Test</title></head>\
                     <body><p>Content</p></body></html>";
        let tmp = tempdir()?;
        let path = tmp.path().join("single-quote.html");
        fs::write(&path, html)?;

        // Act
        let ctx = test_ctx(tmp.path());
        SeoPlugin.after_compile(&ctx)?;

        // Assert: no duplicate meta tags injected
        let result = fs::read_to_string(&path)?;
        assert_eq!(
            result.matches("meta name=\"description\"").count()
                + result.matches("meta name='description'").count(),
            1,
            "description meta should not be duplicated"
        );
        assert_eq!(
            result.matches("og:title").count(),
            1,
            "og:title should not be duplicated"
        );
        Ok(())
    }

    #[test]
    fn canonical_plugin_trailing_slash_base_url() -> Result<()> {
        // Arrange: base_url has a trailing slash
        let tmp = tempdir()?;
        fs::write(
            tmp.path().join("index.html"),
            make_html("Home", "<p>Welcome</p>"),
        )?;

        // Act
        let ctx = test_ctx(tmp.path());
        let plugin = CanonicalPlugin::new("https://example.com/");
        plugin.after_compile(&ctx)?;

        // Assert: canonical URL has no double slash
        let result = fs::read_to_string(tmp.path().join("index.html"))?;
        assert!(
            result.contains("https://example.com/index.html"),
            "should produce clean URL without double slash"
        );
        assert!(
            !result.contains("https://example.com//"),
            "should not contain double slash in canonical URL"
        );
        Ok(())
    }

    #[test]
    fn robots_plugin_trailing_slash_base_url() -> Result<()> {
        // Arrange: base_url has a trailing slash
        let tmp = tempdir()?;
        let ctx = test_ctx(tmp.path());
        let plugin = RobotsPlugin::new("https://example.com/");

        // Act
        plugin.after_compile(&ctx)?;

        // Assert: sitemap URL has no double slash
        let content = fs::read_to_string(tmp.path().join("robots.txt"))?;
        assert!(
            content.contains("Sitemap: https://example.com/sitemap.xml"),
            "sitemap URL should not have double slash, got: {content}"
        );
        assert!(
            !content.contains("https://example.com//"),
            "should not contain double slash"
        );
        Ok(())
    }

    #[test]
    fn extract_description_nested_script_in_main() {
        // Arrange: <main> contains a <script> block alongside real content
        let html = "<html><head></head><body>\
                     <main>\
                     <script>var x = 'ignore me';</script>\
                     <p>Visible text after script</p>\
                     </main></body></html>";

        // Act
        let desc = extract_description(html, 160);

        // Assert: script content is stripped, visible text remains
        assert!(
            desc.contains("Visible text after script"),
            "should contain the paragraph text, got: {desc}"
        );
        assert!(
            !desc.contains("ignore me"),
            "should not contain script content, got: {desc}"
        );
    }

    // -----------------------------------------------------------------
    // JSON-LD Plugin tests
    // -----------------------------------------------------------------

    #[test]
    fn test_jsonld_injects_webpage() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = make_html("About", "<p>About us</p>");
        fs::write(site.join("about.html"), &html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Test Org");
        plugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("about.html")).unwrap();
        assert!(output.contains("application/ld+json"));
        assert!(output.contains("\"@type\":\"WebPage\""));
        assert!(output.contains("\"name\":\"About\""));
    }

    #[test]
    fn test_jsonld_injects_article() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = "<html><head><title>Post</title></head>\
                     <body><article><h1>Post</h1></article></body></html>";
        fs::write(site.join("post.html"), html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "My Org");
        plugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("post.html")).unwrap();
        assert!(output.contains("\"@type\":\"Article\""));
        assert!(output.contains("\"headline\":\"Post\""));
        assert!(output.contains("My Org"));
    }

    #[test]
    fn test_jsonld_breadcrumbs() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        let blog = site.join("blog");
        fs::create_dir_all(&blog).unwrap();

        let html = make_html("My Post", "<p>Content</p>");
        fs::write(blog.join("my-post.html"), &html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        plugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(blog.join("my-post.html")).unwrap();
        assert!(output.contains("BreadcrumbList"));
        assert!(output.contains("\"name\":\"Home\""));
        assert!(output.contains("\"name\":\"blog\""));
    }

    #[test]
    fn test_jsonld_idempotent() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = "<html><head><title>X</title>\
                     <script type=\"application/ld+json\">{}</script>\
                     </head><body></body></html>";
        fs::write(site.join("x.html"), html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        plugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("x.html")).unwrap();
        // Should have exactly one ld+json (the original), not two
        let count = output.matches("application/ld+json").count();
        assert_eq!(count, 1);
    }
}
