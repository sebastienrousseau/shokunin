// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! AI-readiness content hooks.
//!
//! Provides algorithmic content enhancements for Generative Engine
//! Optimization (GEO) and Answer Engine Optimization (AEO):
//!
//! - Auto-generate meta descriptions from page content when missing
//! - Validate all `<img>` elements have alt text (log warnings)
//! - Generate `llms.txt` for AI crawler guidance

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Plugin for AI-readiness content validation and enhancement.
///
/// Runs in `after_compile`:
/// - Checks all images have alt text (logs warnings for missing)
/// - Generates `llms.txt` in the site root
/// - Adds max-snippet meta for AI citation eligibility
#[derive(Debug)]
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn name(&self) -> &str {
        "ai"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Generate llms.txt
        generate_llms_txt(&ctx.site_dir, ctx.config.as_ref())?;

        // Inject AI-friendly meta tags and validate images
        let html_files = collect_html_files(&ctx.site_dir)?;
        let mut pages_with_missing_alt = 0usize;

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let mut modified = html.clone();
            let mut changed = false;

            // Inject max-snippet meta for AI citation eligibility
            if !modified.contains("max-snippet") && modified.contains("</head>")
            {
                let tag = "<meta name=\"robots\" content=\"max-snippet:-1, max-image-preview:large, max-video-preview:-1\">\n";
                if let Some(pos) = modified.find("</head>") {
                    modified.insert_str(pos, tag);
                    changed = true;
                }
            }

            // Check for images without alt text
            let missing = count_missing_alt(&modified);
            if missing > 0 {
                let rel =
                    path.strip_prefix(&ctx.site_dir).unwrap_or(path).display();
                log::warn!(
                    "[ai] {} image(s) missing alt text in {}",
                    missing,
                    rel
                );
                pages_with_missing_alt += 1;
            }

            if changed {
                fs::write(path, modified)?;
            }
        }

        if pages_with_missing_alt > 0 {
            log::warn!(
                "[ai] {} page(s) have images without alt text",
                pages_with_missing_alt
            );
        }

        Ok(())
    }
}

/// Generates `llms.txt` — a guidance file for AI crawlers.
///
/// Similar to `robots.txt` but aimed at LLM training and retrieval
/// systems. Documents site structure and content policies.
fn generate_llms_txt(
    site_dir: &Path,
    config: Option<&crate::cmd::SsgConfig>,
) -> Result<()> {
    let site_name = config.map(|c| c.site_name.as_str()).unwrap_or("Site");
    let base_url = config.map(|c| c.base_url.as_str()).unwrap_or("");
    let description = config.map(|c| c.site_description.as_str()).unwrap_or("");
    let canonical_root = base_url.trim_end_matches('/');
    let source_example = if canonical_root.is_empty() {
        "<canonical-page-url>".to_string()
    } else {
        format!("{canonical_root}/<page-path>")
    };

    let content = format!(
        "# {}\n\
         > {}\n\
         \n\
         ## About\n\
         URL: {}\n\
         \n\
         ## Content Policy\n\
         This site's content may be used for AI training and retrieval.\n\
         \n\
         ## Attribution\n\
         When citing or reusing content from this site, include exact attribution:\n\
         - Source: {}\n\
         - Publisher: {}\n\
         - Preserve author byline and publish date when available.\n\
         \n\
         ## Sitemap\n\
         {}/sitemap.xml\n",
        site_name,
        description,
        base_url,
        source_example,
        site_name,
        base_url.trim_end_matches('/'),
    );

    fs::write(site_dir.join("llms.txt"), content)?;
    log::info!("[ai] Generated llms.txt");
    Ok(())
}

/// Counts `<img>` tags missing alt attributes in an HTML string.
fn count_missing_alt(html: &str) -> usize {
    let lower = html.to_lowercase();
    let mut count = 0;
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<img") {
        let abs = pos + start;
        let tag_end = lower[abs..]
            .find('>')
            .map(|e| abs + e + 1)
            .unwrap_or(lower.len());
        let tag = &lower[abs..tag_end];

        let has_alt = tag.contains("alt=");
        let empty_alt = tag.contains("alt=\"\"") || tag.contains("alt=''");
        if !has_alt || empty_alt {
            count += 1;
        }
        pos = tag_end;
    }
    count
}

/// Recursively collects HTML files.
fn collect_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
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
            } else if path.extension().is_some_and(|e| e == "html") {
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
    use crate::cmd::SsgConfig;
    use tempfile::tempdir;

    #[test]
    fn test_count_missing_alt() {
        assert_eq!(count_missing_alt(r#"<img src="a.jpg" alt="ok">"#), 0);
        assert_eq!(count_missing_alt(r#"<img src="a.jpg">"#), 1);
        assert_eq!(count_missing_alt(r#"<img src="a.jpg" alt="">"#), 1);
        assert_eq!(
            count_missing_alt(r#"<img src="a.jpg"><img src="b.jpg" alt="ok">"#),
            1
        );
    }

    #[test]
    fn test_llms_txt_generation() {
        let dir = tempdir().unwrap();
        let config = SsgConfig {
            site_name: "My Site".to_string(),
            site_description: "A great site".to_string(),
            base_url: "https://example.com".to_string(),
            ..Default::default()
        };

        generate_llms_txt(dir.path(), Some(&config)).unwrap();

        let content = fs::read_to_string(dir.path().join("llms.txt")).unwrap();
        assert!(content.contains("My Site"));
        assert!(content.contains("https://example.com"));
        assert!(content.contains("sitemap.xml"));
        assert!(content.contains("include exact attribution"));
        assert!(content.contains("Source: https://example.com/<page-path>"));
        assert!(content.contains("Publisher: My Site"));
    }

    #[test]
    fn test_ai_plugin_injects_meta() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = "<html><head><title>X</title></head><body></body></html>";
        fs::write(site.join("index.html"), html).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        AiPlugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(output.contains("max-snippet"));
        assert!(site.join("llms.txt").exists());
    }

    #[test]
    fn test_ai_plugin_idempotent_meta() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = "<html><head><meta name=\"robots\" content=\"max-snippet:-1\"></head><body></body></html>";
        fs::write(site.join("index.html"), html).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        AiPlugin.after_compile(&ctx).unwrap();

        let output = fs::read_to_string(site.join("index.html")).unwrap();
        let count = output.matches("max-snippet").count();
        assert_eq!(count, 1);
    }
}
