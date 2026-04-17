// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Local LLM content plugin.
//!
//! Invokes a local LLM (Ollama, llama.cpp) at build time to auto-generate:
//! - `alt` text for images missing it
//! - `meta description` for pages where it's empty or < 50 chars
//! - JSON-LD `description` fields from page content
//!
//! Configured via the `[ai]` section in `ssg.toml`:
//! ```toml
//! [ai]
//! model = "llama3"
//! endpoint = "http://localhost:11434"
//! ```
//!
//! Graceful fallback: if no LLM is reachable, logs a warning and skips.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::{fs, path::Path, process::Command};

/// Configuration for the LLM plugin.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Model name (e.g., `"llama3"`, `"mistral"`).
    pub model: String,
    /// Ollama API endpoint.
    pub endpoint: String,
    /// If true, print generated text but don't write files.
    pub dry_run: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "llama3".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            dry_run: false,
        }
    }
}

/// Plugin that uses a local LLM to augment content at build time.
#[derive(Debug)]
pub struct LlmPlugin {
    config: LlmConfig,
}

impl LlmPlugin {
    /// Creates a new `LlmPlugin` with the given configuration.
    #[must_use]
    pub const fn new(config: LlmConfig) -> Self {
        Self { config }
    }
}

impl Plugin for LlmPlugin {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Check if Ollama is available
        if !is_ollama_available(&self.config.endpoint) {
            log::warn!(
                "[llm] Ollama not reachable at {}, skipping AI augmentation",
                self.config.endpoint
            );
            return Ok(());
        }

        let html_files = ctx.get_html_files();
        let mut augmented = 0usize;

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let mut modified = html.clone();

            // Auto-generate meta descriptions for pages with short/missing ones
            if needs_meta_description(&modified) {
                if let Some(desc) = generate_meta_description(
                    &modified,
                    &self.config.model,
                    &self.config.endpoint,
                ) {
                    if self.config.dry_run {
                        let rel = path
                            .strip_prefix(&ctx.site_dir)
                            .unwrap_or(path)
                            .display();
                        log::info!(
                            "[llm] [dry-run] {rel}: description = {desc}"
                        );
                    } else {
                        modified = inject_meta_description(&modified, &desc);
                    }
                }
            }

            // Auto-generate alt text for images missing it
            let alt_count = generate_missing_alt_text(
                &mut modified,
                &self.config.model,
                &self.config.endpoint,
                self.config.dry_run,
                path,
                &ctx.site_dir,
            );

            if !self.config.dry_run && modified != html {
                fs::write(path, &modified)?;
                augmented += 1;
            }

            if alt_count > 0 {
                augmented += 1;
            }
        }

        if augmented > 0 {
            log::info!(
                "[llm] Augmented {augmented} page(s) with model '{}'",
                self.config.model
            );
        }

        Ok(())
    }
}

/// Checks if Ollama is reachable at the given endpoint.
fn is_ollama_available(endpoint: &str) -> bool {
    // Try a simple HTTP health check via curl
    Command::new("curl")
        .args(["-sf", "--max-time", "2", endpoint])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Returns true if the page needs a meta description (missing or < 50 chars).
fn needs_meta_description(html: &str) -> bool {
    if let Some(start) = html.find("name=\"description\"") {
        if let Some(content_start) = html[start..].find("content=\"") {
            let abs = start + content_start + 9;
            if let Some(end) = html[abs..].find('"') {
                let desc = &html[abs..abs + end];
                return desc.len() < 50;
            }
        }
    }
    // No description meta tag found
    !html.contains("name=\"description\"")
}

/// Generates a meta description via LLM from page content.
fn generate_meta_description(
    html: &str,
    model: &str,
    endpoint: &str,
) -> Option<String> {
    let text = extract_page_text(html, 500);
    if text.len() < 20 {
        return None;
    }

    let prompt = format!(
        "Write a concise SEO meta description (120-155 characters) for this page content. \
         Return ONLY the description text, no quotes or explanation:\n\n{text}"
    );

    call_ollama(endpoint, model, &prompt)
}

/// Injects a meta description tag into the HTML head.
fn inject_meta_description(html: &str, description: &str) -> String {
    let escaped = description
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
    let tag = format!("<meta name=\"description\" content=\"{escaped}\">\n");

    if let Some(pos) = html.find("</head>") {
        let mut result = html.to_string();
        result.insert_str(pos, &tag);
        result
    } else {
        html.to_string()
    }
}

/// Generates alt text for images that are missing it.
fn generate_missing_alt_text(
    html: &mut String,
    model: &str,
    endpoint: &str,
    dry_run: bool,
    path: &Path,
    site_dir: &Path,
) -> usize {
    let mut count = 0;
    let mut search_from = 0;

    while let Some(start) = html[search_from..].find("<img") {
        let abs_start = search_from + start;
        let Some(tag_end) = html[abs_start..].find('>') else {
            break;
        };
        let tag_end_abs = abs_start + tag_end + 1;
        let tag = &html[abs_start..tag_end_abs];

        if !tag.contains("alt=") || tag.contains("alt=\"\"") {
            // Extract src for context
            let src = extract_attr(tag, "src").unwrap_or_default();
            let prompt = format!(
                "Describe this image for an alt text attribute. The image file is named '{}'. \
                 Return ONLY the alt text (max 125 characters), no quotes:\n",
                src
            );

            if let Some(alt) = call_ollama(endpoint, model, &prompt) {
                let alt = alt.trim().replace('"', "&quot;");
                if dry_run {
                    let rel =
                        path.strip_prefix(site_dir).unwrap_or(path).display();
                    log::info!(
                        "[llm] [dry-run] {rel}: alt=\"{alt}\" for {src}"
                    );
                } else {
                    // Replace the tag with one that has alt text
                    let new_tag = if tag.contains("alt=\"\"") {
                        tag.replace("alt=\"\"", &format!("alt=\"{alt}\""))
                    } else {
                        tag.replace("<img", &format!("<img alt=\"{alt}\""))
                    };
                    html.replace_range(abs_start..tag_end_abs, &new_tag);
                }
                count += 1;
            }
        }

        search_from = tag_end_abs;
    }

    count
}

/// Extracts plain text from HTML for LLM prompting.
fn extract_page_text(html: &str, max_chars: usize) -> String {
    let body_start = html
        .find("<main")
        .or_else(|| html.find("<body"))
        .unwrap_or(0);
    let body = &html[body_start..];

    let mut text = String::with_capacity(max_chars + 50);
    let mut in_tag = false;
    for ch in body.chars() {
        if text.len() >= max_chars {
            break;
        }
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && !ch.is_control() => text.push(ch),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts an attribute value from an HTML tag.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

/// Calls the Ollama API to generate text.
fn call_ollama(endpoint: &str, model: &str, prompt: &str) -> Option<String> {
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
    });

    let output = Command::new("curl")
        .args([
            "-sf",
            "--max-time",
            "30",
            "-X",
            "POST",
            &url,
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload.to_string(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).ok()?;
    response
        .get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_meta_description_missing() {
        assert!(needs_meta_description("<html><head></head></html>"));
    }

    #[test]
    fn needs_meta_description_short() {
        let html = r#"<html><head><meta name="description" content="Short"></head></html>"#;
        assert!(needs_meta_description(html));
    }

    #[test]
    fn needs_meta_description_adequate() {
        let html = r#"<html><head><meta name="description" content="This is a sufficiently long meta description that exceeds fifty characters easily"></head></html>"#;
        assert!(!needs_meta_description(html));
    }

    #[test]
    fn inject_meta_description_into_head() {
        let html = "<html><head><title>T</title></head><body></body></html>";
        let result = inject_meta_description(html, "Test description");
        assert!(result.contains("name=\"description\""));
        assert!(result.contains("Test description"));
    }

    #[test]
    fn extract_attr_basic() {
        assert_eq!(
            extract_attr(r#"<img src="photo.jpg" alt="x">"#, "src"),
            Some("photo.jpg".to_string())
        );
    }

    #[test]
    fn extract_attr_missing() {
        assert_eq!(extract_attr(r#"<img src="x.jpg">"#, "alt"), None);
    }

    #[test]
    fn extract_page_text_strips_tags() {
        let html = "<body><p>Hello <b>world</b></p></body>";
        let text = extract_page_text(html, 100);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn llm_plugin_name() {
        let plugin = LlmPlugin::new(LlmConfig::default());
        assert_eq!(plugin.name(), "llm");
    }

    #[test]
    fn llm_plugin_skips_when_ollama_unavailable() {
        let plugin = LlmPlugin::new(LlmConfig {
            endpoint: "http://localhost:99999".to_string(),
            ..LlmConfig::default()
        });

        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join("index.html"), "<html><body></body></html>")
            .unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        // Should succeed (graceful skip)
        plugin.after_compile(&ctx).unwrap();
    }
}
