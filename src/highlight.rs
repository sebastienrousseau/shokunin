// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Syntax highlighting plugin.
//!
//! Post-processes compiled HTML to add syntax highlighting to code
//! blocks. Uses class-based highlighting with a generated CSS file,
//! avoiding inline styles for better performance and cacheability.

use crate::plugin::{Plugin, PluginContext};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Plugin that adds syntax highlighting CSS classes to code blocks.
///
/// Runs in `after_compile`. Finds `<pre><code class="language-X">`
/// blocks and wraps them with a highlight container. Generates a
/// `highlight.css` file with the color theme.
#[derive(Debug)]
pub struct HighlightPlugin {
    /// CSS theme name. Default themes are generated inline.
    theme: String,
}

impl Default for HighlightPlugin {
    fn default() -> Self {
        Self {
            theme: "github".to_string(),
        }
    }
}

impl HighlightPlugin {
    /// Creates a highlight plugin with the given theme name.
    pub fn with_theme(theme: impl Into<String>) -> Self {
        Self {
            theme: theme.into(),
        }
    }
}

impl Plugin for HighlightPlugin {
    fn name(&self) -> &'static str {
        "highlight"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        // Generate highlight.css
        let css = generate_highlight_css(&self.theme);
        fs::write(ctx.site_dir.join("highlight.css"), &css)?;

        // Process HTML files
        let html_files = collect_html_files(&ctx.site_dir)?;
        let mut highlighted = 0usize;

        for path in &html_files {
            let html = fs::read_to_string(path)?;
            let result = add_highlight_markup(&html);
            if result != html {
                // Inject CSS link if not present
                let output = if result.contains("highlight.css") {
                    result
                } else {
                    inject_css_link(&result)
                };
                fs::write(path, output)?;
                highlighted += 1;
            }
        }

        if highlighted > 0 {
            log::info!(
                "[highlight] Processed {} file(s), theme: {}",
                highlighted,
                self.theme
            );
        }

        Ok(())
    }
}

/// Adds highlight markup to code blocks.
///
/// Transforms `<pre><code class="language-X">` into
/// `<pre class="highlight"><code class="language-X" data-lang="X">`.
fn add_highlight_markup(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut pos = 0;

    while pos < html.len() {
        if let Some(pre_start) = html[pos..].find("<pre>") {
            let abs_pre = pos + pre_start;
            let after_pre = abs_pre + 5; // len("<pre>")

            // Check if next element is <code class="language-
            let remaining = &html[after_pre..];
            if remaining.starts_with("<code class=\"language-") {
                // Extract language name
                let lang_start = "language-".len();
                let code_attr = &remaining["<code class=\"".len()..];
                let lang_end = code_attr.find('"').unwrap_or(0);
                let lang = &code_attr[lang_start..lang_end];

                // Write the enhanced pre tag
                result.push_str(&html[pos..abs_pre]);
                result.push_str(&format!(
                    "<pre class=\"highlight language-{lang}\">"
                ));
                result.push_str(&format!(
                    "<code class=\"language-{lang}\" data-lang=\"{lang}\">"
                ));

                // Skip past the original <pre><code class="language-X">
                let code_tag_end = remaining.find('>').unwrap_or(0);
                pos = after_pre + code_tag_end + 1;
                continue;
            }
        }

        // No match — copy rest and break
        result.push_str(&html[pos..]);
        break;
    }

    result
}

/// Injects a `<link>` to highlight.css before `</head>`.
fn inject_css_link(html: &str) -> String {
    if let Some(pos) = html.find("</head>") {
        format!(
            "{}<link rel=\"stylesheet\" href=\"/highlight.css\">\n{}",
            &html[..pos],
            &html[pos..]
        )
    } else {
        html.to_string()
    }
}

/// Generates a CSS theme for syntax highlighting.
fn generate_highlight_css(theme: &str) -> String {
    match theme {
        "github" | _ => {
            r#"/* Syntax highlighting — GitHub-inspired theme */
pre.highlight {
  background: #f6f8fa;
  border: 1px solid #d0d7de;
  border-radius: 6px;
  padding: 1em;
  overflow-x: auto;
  font-size: 0.875em;
  line-height: 1.45;
}
pre.highlight code {
  background: none;
  padding: 0;
  border: none;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
}
@media (prefers-color-scheme: dark) {
  pre.highlight {
    background: #161b22;
    border-color: #30363d;
    color: #e6edf3;
  }
}
"#
            .to_string()
        }
    }
}

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
    use tempfile::tempdir;

    #[test]
    fn test_add_highlight_markup() {
        let html =
            r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let result = add_highlight_markup(html);
        assert!(result.contains("class=\"highlight language-rust\""));
        assert!(result.contains("data-lang=\"rust\""));
    }

    #[test]
    fn test_no_code_block_unchanged() {
        let html = "<pre>plain text</pre>";
        let result = add_highlight_markup(html);
        assert_eq!(result, html);
    }

    #[test]
    fn test_inject_css_link() {
        let html = "<html><head><title>X</title></head><body></body></html>";
        let result = inject_css_link(html);
        assert!(result.contains("highlight.css"));
    }

    #[test]
    fn test_generate_css() {
        let css = generate_highlight_css("github");
        assert!(css.contains("pre.highlight"));
        assert!(css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn test_plugin_generates_css() {
        let dir = tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();

        let html = r#"<html><head><title>X</title></head><body><pre><code class="language-js">let x = 1;</code></pre></body></html>"#;
        fs::write(site.join("index.html"), html).unwrap();

        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        HighlightPlugin::default().after_compile(&ctx).unwrap();

        assert!(site.join("highlight.css").exists());
        let output = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(output.contains("highlight.css"));
        assert!(output.contains("highlight language-js"));
    }
}
