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

/// Collect all `.html` files under `dir` (delegates to `crate::walk`).
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
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

/// Recursively collects HTML files (delegates to `crate::walk`).
fn collect_html_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files(dir, "html")
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
        crate::test_support::init_logger();
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

    // -----------------------------------------------------------------
    // extract_title — edge cases
    // -----------------------------------------------------------------

    #[test]
    fn extract_title_empty_tag_returns_empty_string() {
        // Lines 29-31: the `if !trimmed.is_empty()` branch is FALSE
        // when the title is empty or only whitespace. Falls through
        // to `String::new()` at line 34.
        assert_eq!(extract_title("<title></title>"), "");
        assert_eq!(extract_title("<title>   </title>"), "");
        assert_eq!(extract_title("<title>\n\t </title>"), "");
    }

    #[test]
    fn extract_title_without_closing_tag_returns_empty() {
        // The `if let Some(end)` at line 26 takes the None branch
        // when `</title>` is missing.
        assert_eq!(extract_title("<title>Unterminated"), "");
    }

    #[test]
    fn extract_title_strips_inner_html_tags() {
        // Inner tags are stripped by `strip_tags` call at line 27.
        // strip_tags collapses successive whitespace per its own
        // rules; the .trim() in extract_title collapses leading and
        // trailing whitespace but runs between words are preserved.
        let out = extract_title("<title>Hello <em>World</em></title>");
        assert!(out.contains("Hello"));
        assert!(out.contains("World"));
    }

    // -----------------------------------------------------------------
    // extract_description — every branch
    // -----------------------------------------------------------------

    #[test]
    fn extract_description_prefers_main_over_body() {
        let html = r#"<html><head></head><body>
            <nav>menu</nav>
            <main>The primary content.</main>
            <footer>Bottom</footer>
        </body></html>"#;
        let desc = extract_description(html, 200);
        assert!(desc.contains("primary content"));
        assert!(!desc.contains("menu"));
    }

    #[test]
    fn extract_description_main_without_closing_tag_takes_rest() {
        // Line 51: the `else { inner.to_string() }` branch of the
        // `</main>` search — taken when the closing tag is missing.
        let html = r#"<html><body><main>content without close"#;
        let desc = extract_description(html, 200);
        assert!(desc.contains("content without close"));
    }

    #[test]
    fn extract_description_main_without_angle_bracket_returns_empty_fallback() {
        // Line 54: `String::new()` branch when `<main` exists but
        // the tag never closes with `>`.
        let html = "<html><body><main";
        let desc = extract_description(html, 200);
        assert_eq!(desc, "");
    }

    #[test]
    fn extract_description_fallback_to_body_strips_script_and_style() {
        // No <main>, but a <body> with script/style/nav/header/footer
        // — all of those must be stripped. Covers lines 74-85.
        let html = r#"<html><head></head><body>
            <script>alert('skip');</script>
            <style>body { color: red; }</style>
            <nav>menu items here</nav>
            <header>site title</header>
            <p>The body text.</p>
            <footer>copyright</footer>
        </body></html>"#;
        let desc = extract_description(html, 200);
        assert!(desc.contains("body text"));
        assert!(!desc.contains("alert"));
        assert!(!desc.contains("color: red"));
        assert!(!desc.contains("menu items"));
        assert!(!desc.contains("site title"));
        assert!(!desc.contains("copyright"));
    }

    #[test]
    fn extract_description_body_without_closing_tag_uses_rest() {
        // Line 65: `else { inner.to_string() }` branch of the
        // `</body>` search.
        let html = "<html><body><p>open-ended body paragraph";
        let desc = extract_description(html, 200);
        assert!(desc.contains("open-ended body paragraph"));
    }

    #[test]
    fn extract_description_body_without_angle_bracket_returns_empty() {
        // Line 68: `String::new()` branch when `<body` doesn't close.
        let html = "<html><body";
        let desc = extract_description(html, 200);
        assert_eq!(desc, "");
    }

    #[test]
    fn extract_description_no_body_no_main_uses_entire_html() {
        // The `else { html.to_string() }` fallback at line 71 —
        // taken when there's no <body> tag at all.
        let html = "just plain text no tags here";
        let desc = extract_description(html, 200);
        assert!(desc.contains("just plain text"));
    }

    #[test]
    fn extract_description_unterminated_script_breaks_out() {
        // Line 98: the `else { break }` branch in the script-
        // stripping loop — taken when a `<script` exists but no
        // corresponding `</script>` is found.
        let html = "<html><body><main><script>unterminated<p>x</p>";
        let desc = extract_description(html, 200);
        // Function terminates without panic.
        let _ = desc;
    }

    #[test]
    fn extract_description_truncates_at_word_boundary() {
        // Lines 105-118: the `len > max_len` branch with word-
        // boundary truncation via rfind(' ').
        let html = "<html><body><main>one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty twenty-one twenty-two twenty-three twenty-four twenty-five</main></body></html>";
        let desc = extract_description(html, 80);
        assert!(desc.len() <= 80);
        // Should end on a complete word (not mid-word).
        assert!(!desc.ends_with('-'));
    }

    #[test]
    fn extract_description_truncates_without_space_falls_to_byte_cut() {
        // Line 118: the `else { truncated.to_string() }` branch
        // when no space is found within max_len.
        let html =
            "<html><body><main>oneverylongwordwithnospacesanywherehere</main></body></html>";
        let desc = extract_description(html, 10);
        assert!(desc.len() <= 10);
    }

    #[test]
    fn extract_description_respects_char_boundary_on_truncation() {
        // Lines 110-112: the char-boundary walk-back loop for
        // multibyte UTF-8 characters.
        let html = "<html><body><main>Rust programming — é ñ ü characters everywhere in this text that we want to truncate mid-char</main></body></html>";
        let desc = extract_description(html, 30);
        // Must produce a valid UTF-8 string (would panic otherwise).
        assert!(desc.is_ascii() || !desc.is_empty());
    }

    #[test]
    fn extract_description_truncation_walks_back_multiple_bytes() {
        // Forces the `end -= 1` loop at lines 110-112 to iterate
        // at least once. A 4-byte emoji positioned so that `max_len`
        // lands in the middle of its bytes forces the walk-back.
        let mut input = String::from("<html><body><main>");
        // pad with ASCII to near the max, then insert the emoji
        input.push_str(&"a".repeat(20));
        input.push('🎉'); // 4 bytes
        input.push_str(&"b".repeat(20));
        input.push_str("</main></body></html>");
        // max_len=22 should land inside the emoji bytes.
        let desc = extract_description(&input, 22);
        assert!(!desc.is_empty(), "expected non-empty desc");
        // Result must be valid UTF-8 (guaranteed by String type).
        let _ = desc.len();
    }

    #[test]
    fn extract_description_body_fallback_unterminated_nav_breaks() {
        // Line 82: the body-fallback strip loop hits `break` when
        // a `<nav` exists but no `</nav>` is found. Requires a
        // <body> with NO <main>.
        let html = "<html><body><nav>unterminated nav block<p>visible</p>";
        let desc = extract_description(html, 200);
        // Function terminates without panic; nav content may or
        // may not survive depending on the exact strip semantics.
        let _ = desc;
    }

    // -----------------------------------------------------------------
    // SeoPlugin.after_compile — no </head> tag
    // -----------------------------------------------------------------

    #[test]
    fn seo_plugin_file_without_head_tag_is_unchanged() {
        // The `else { html }` branch in inject_seo_tags at line 297.
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("fragment.html"),
            "<p>no html/head/body structure</p>",
        )
        .unwrap();
        let ctx = test_ctx(dir.path());
        SeoPlugin.after_compile(&ctx).unwrap();
        let out = fs::read_to_string(dir.path().join("fragment.html")).unwrap();
        assert_eq!(out, "<p>no html/head/body structure</p>");
    }

    #[test]
    fn seo_plugin_missing_site_dir_returns_ok() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let ctx = test_ctx(&missing);
        SeoPlugin.after_compile(&ctx).unwrap();
    }

    // -----------------------------------------------------------------
    // RobotsPlugin — idempotency + missing dir
    // -----------------------------------------------------------------

    #[test]
    fn robots_plugin_skips_existing_robots_txt() {
        // Line 350: the `if robots_path.exists() { return Ok(()) }`
        // branch.
        let dir = tempdir().unwrap();
        let existing = dir.path().join("robots.txt");
        fs::write(&existing, "USER: existing").unwrap();

        let plugin = RobotsPlugin::new("https://example.com");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        // Existing content preserved.
        assert_eq!(fs::read_to_string(&existing).unwrap(), "USER: existing");
    }

    #[test]
    fn robots_plugin_writes_user_agent_and_sitemap() {
        let dir = tempdir().unwrap();
        let plugin = RobotsPlugin::new("https://example.com/");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        let body = fs::read_to_string(dir.path().join("robots.txt")).unwrap();
        assert!(body.contains("User-agent: *"));
        // Trailing slash stripped via trim_end_matches.
        assert!(body.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn robots_plugin_missing_site_dir_returns_ok() {
        // Line 345: `!ctx.site_dir.exists()` early return.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let plugin = RobotsPlugin::new("https://example.com");
        let ctx = test_ctx(&missing);
        plugin.after_compile(&ctx).unwrap();
    }

    #[test]
    fn robots_plugin_name_returns_static_identifier() {
        assert_eq!(RobotsPlugin::new("").name(), "robots");
    }

    // -----------------------------------------------------------------
    // CanonicalPlugin — skip path, missing head, already-canonical
    // -----------------------------------------------------------------

    #[test]
    fn canonical_plugin_missing_site_dir_returns_ok() {
        // Line 409.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let plugin = CanonicalPlugin::new("https://example.com");
        let ctx = test_ctx(&missing);
        plugin.after_compile(&ctx).unwrap();
    }

    #[test]
    fn canonical_plugin_skips_pages_with_existing_canonical_link() {
        // Lines 419-422: the `continue` branch.
        let dir = tempdir().unwrap();
        let html = r#"<html><head><link rel="canonical" href="/original"></head><body></body></html>"#;
        fs::write(dir.path().join("p.html"), html).unwrap();

        let plugin = CanonicalPlugin::new("https://example.com");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(dir.path().join("p.html")).unwrap();
        assert_eq!(out.matches(r#"rel="canonical""#).count(), 1);
        assert!(out.contains("/original"));
    }

    #[test]
    fn canonical_plugin_skips_pages_with_single_quoted_canonical() {
        // Same `continue` branch via single-quoted variant.
        let dir = tempdir().unwrap();
        let html =
            r"<html><head><link rel='canonical' href='/x'></head></html>";
        fs::write(dir.path().join("p.html"), html).unwrap();

        let plugin = CanonicalPlugin::new("https://example.com");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(dir.path().join("p.html")).unwrap();
        assert_eq!(out.matches("canonical").count(), 1);
    }

    #[test]
    fn canonical_plugin_page_without_head_is_left_unchanged() {
        // Line 440: `else { html }` branch when no `</head>` exists.
        let dir = tempdir().unwrap();
        let html = "<p>no structure</p>";
        fs::write(dir.path().join("frag.html"), html).unwrap();

        let plugin = CanonicalPlugin::new("https://example.com");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(dir.path().join("frag.html")).unwrap();
        assert_eq!(out, html);
    }

    #[test]
    fn canonical_plugin_injects_canonical_link_before_head_close() {
        let dir = tempdir().unwrap();
        let html = "<html><head><title>T</title></head><body></body></html>";
        fs::write(dir.path().join("a.html"), html).unwrap();

        let plugin = CanonicalPlugin::new("https://example.com/");
        let ctx = test_ctx(dir.path());
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(dir.path().join("a.html")).unwrap();
        assert!(out.contains(r#"rel="canonical""#));
        assert!(out.contains("https://example.com/a.html"));
    }

    #[test]
    fn canonical_plugin_name_returns_static_identifier() {
        assert_eq!(CanonicalPlugin::new("").name(), "canonical");
    }

    // -----------------------------------------------------------------
    // JsonLdPlugin — WebPage branch + no-head skip
    // -----------------------------------------------------------------

    #[test]
    fn jsonld_plugin_missing_site_dir_returns_ok() {
        // Line 506.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        let ctx = test_ctx(&missing);
        plugin.after_compile(&ctx).unwrap();
    }

    #[test]
    fn jsonld_plugin_skips_pages_without_head_tag() {
        // Line 523: the `None => continue` branch.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("frag.html"), "<p>no head</p>").unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        plugin.after_compile(&ctx).unwrap();
        let out = fs::read_to_string(site.join("frag.html")).unwrap();
        assert_eq!(out, "<p>no head</p>");
    }

    #[test]
    fn jsonld_plugin_generates_webpage_when_no_article_element() {
        // The `else` branch of `if html.contains("<article")` —
        // pages without an <article> get a WebPage schema.
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let html = "<html><head><title>Hello</title></head><body><p>content</p></body></html>";
        fs::write(site.join("index.html"), html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(out.contains("application/ld+json"));
        assert!(out.contains("WebPage"));
    }

    #[test]
    fn jsonld_plugin_generates_article_when_article_element_present() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let html = "<html><head><title>Post</title></head><body><article><h1>Post</h1></article></body></html>";
        fs::write(site.join("post.html"), html).unwrap();

        let ctx = test_ctx(&site);
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        plugin.after_compile(&ctx).unwrap();

        let out = fs::read_to_string(site.join("post.html")).unwrap();
        assert!(out.contains("application/ld+json"));
        assert!(out.contains(r#""Article""#));
    }

    #[test]
    fn jsonld_plugin_new_stores_supplied_config() {
        let cfg = JsonLdConfig {
            base_url: "https://a".to_string(),
            org_name: "Org".to_string(),
            breadcrumbs: false,
        };
        let plugin = JsonLdPlugin::new(cfg.clone());
        assert_eq!(plugin.config.base_url, "https://a");
        assert_eq!(plugin.config.org_name, "Org");
        assert!(!plugin.config.breadcrumbs);
    }

    #[test]
    fn jsonld_plugin_name_returns_static_identifier() {
        let plugin = JsonLdPlugin::from_site("https://example.com", "Org");
        assert_eq!(plugin.name(), "json-ld");
    }

    // -----------------------------------------------------------------
    // collect_html_files_recursive
    // -----------------------------------------------------------------

    #[test]
    fn collect_html_files_recursive_filters_and_sorts() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(dir.path().join("z.html"), "").unwrap();
        fs::write(dir.path().join("a.html"), "").unwrap();
        fs::write(sub.join("m.html"), "").unwrap();
        fs::write(dir.path().join("ignore.css"), "").unwrap();

        let files = collect_html_files_recursive(dir.path()).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn collect_html_files_recursive_missing_dir_returns_empty() {
        let dir = tempdir().unwrap();
        let result =
            collect_html_files_recursive(&dir.path().join("missing")).unwrap();
        assert!(result.is_empty());
    }
}
