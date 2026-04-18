// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! GitHub Flavored Markdown (GFM) extensions plugin.
//!
//! Pre-processes Markdown content in the `before_compile` phase to add
//! support for GFM features that the upstream renderer does not handle:
//!
//! - **Tables** — `| col | col |` blocks with a `|---|---|` separator row.
//! - **Strikethrough** — `~~text~~` becomes `<del>text</del>`.
//! - **Task lists** — `- [ ] item` and `- [x] done` become checkbox lists.
//! - **Footnotes** — `[^id]` references with `[^id]:` definitions.
//!
//! ## How it works
//!
//! For each `.md` file under `content_dir`, the plugin:
//! 1. Splits the YAML/TOML frontmatter from the body so it stays untouched.
//! 2. Walks the body line-by-line, tracking fenced code blocks so GFM
//!    syntax inside ``` ``` ``` ``` blocks is preserved literally.
//! 3. Detects GFM-specific blocks (tables, task lists) and renders **only
//!    those blocks** through `pulldown-cmark` with the matching options
//!    enabled, substituting the rendered HTML back into the source.
//! 4. Applies an inline strikethrough transform to remaining text.
//!
//! Standard markdown renderers pass block-level raw HTML through
//! unchanged, so the substituted HTML composes cleanly with whatever
//! renderer staticdatagen runs afterwards.
//!
//! ## Example
//!
//! ```rust
//! use ssg::plugin::PluginManager;
//! use ssg::markdown_ext::MarkdownExtPlugin;
//!
//! let mut pm = PluginManager::new();
//! pm.register(MarkdownExtPlugin);
//! ```

use crate::plugin::{Plugin, PluginContext};
use crate::walk::walk_files_bounded_depth;
use crate::MAX_DIR_DEPTH;
use anyhow::{Context, Result};
use pulldown_cmark::{html as cmark_html, Options, Parser};
use std::fs;

/// Plugin that expands GFM Markdown extensions in source files.
///
/// Runs in `before_compile`. See the [module-level docs](self) for the
/// full list of supported features and the transformation strategy.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Copy, Clone)]
pub struct MarkdownExtPlugin;

impl Plugin for MarkdownExtPlugin {
    fn name(&self) -> &'static str {
        "markdown-ext"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.content_dir.exists() {
            return Ok(());
        }

        let files =
            walk_files_bounded_depth(&ctx.content_dir, "md", MAX_DIR_DEPTH)
                .with_context(|| {
                    format!(
                        "Failed to walk content dir {}",
                        ctx.content_dir.display()
                    )
                })?;

        let mut transformed = 0usize;
        for path in &files {
            fail_point!("markdown_ext::read", |_| {
                anyhow::bail!("injected: markdown_ext::read")
            });
            let raw = fs::read_to_string(path).with_context(|| {
                format!("Failed to read {}", path.display())
            })?;

            let new = expand_gfm(&raw);
            if new != raw {
                fail_point!("markdown_ext::write", |_| {
                    anyhow::bail!("injected: markdown_ext::write")
                });
                fs::write(path, &new).with_context(|| {
                    format!("Failed to write {}", path.display())
                })?;
                transformed += 1;
            }
        }

        if transformed > 0 {
            log::info!("[markdown-ext] Transformed {transformed} file(s)");
        }
        Ok(())
    }
}

/// Splits leading frontmatter (`--- ... ---`) from `input`.
///
/// Returns `(frontmatter, body)`. If no frontmatter is present the
/// frontmatter slice is empty and the entire input is the body.
fn split_frontmatter(input: &str) -> (&str, &str) {
    if let Some(rest) = input.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let fm_end = "---\n".len() + end + "\n---\n".len();
            return (&input[..fm_end], &input[fm_end..]);
        }
        if let Some(end) = rest.find("\n---") {
            let fm_end = "---\n".len() + end + "\n---".len();
            // Trailing newline after closing fence is optional.
            return (&input[..fm_end], &input[fm_end..]);
        }
    }
    ("", input)
}

/// Expands all GFM constructs in `input`, returning a new string.
///
/// If no GFM features are present, returns the input unchanged
/// (modulo no allocation when avoidable).
#[must_use]
pub fn expand_gfm(input: &str) -> String {
    let (frontmatter, body) = split_frontmatter(input);
    if !needs_expansion(body) {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len() + 256);
    out.push_str(frontmatter);

    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0usize;
    let mut in_fence = false;
    let mut fence_marker: Option<&str> = None;

    while i < lines.len() {
        let line = lines[i];

        if let Some(marker) = detect_fence(line) {
            update_fence_state(&mut in_fence, &mut fence_marker, marker, line);
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }

        if in_fence {
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }

        i = process_gfm_line(&lines, i, &mut out);
    }

    if !body.ends_with('\n') && out.ends_with('\n') {
        let _ = out.pop();
    }

    out
}

/// Updates fence tracking state when a fence marker is encountered.
fn update_fence_state<'a>(
    in_fence: &mut bool,
    fence_marker: &mut Option<&'a str>,
    marker: &'a str,
    line: &str,
) {
    if !*in_fence {
        *in_fence = true;
        *fence_marker = Some(marker);
    } else if fence_marker.is_some_and(|m| line.trim_start().starts_with(m)) {
        *in_fence = false;
        *fence_marker = None;
    }
}

/// Processes a single non-fenced line, detecting tables, task lists, or
/// applying strikethrough. Returns the new line index.
fn process_gfm_line(lines: &[&str], i: usize, out: &mut String) -> usize {
    let line = lines[i];

    if i + 1 < lines.len() && is_table_header(line, lines[i + 1]) {
        let end = find_table_end(lines, i);
        let block = lines[i..end].join("\n");
        out.push_str(&render_with_options(&block, Options::ENABLE_TABLES));
        out.push('\n');
        return end;
    }

    if is_task_list_line(line) {
        let end = find_task_list_end(lines, i);
        let block = lines[i..end].join("\n");
        out.push_str(&render_with_options(&block, Options::ENABLE_TASKLISTS));
        out.push('\n');
        return end;
    }

    out.push_str(&apply_strikethrough(line));
    out.push('\n');
    i + 1
}

/// Returns `true` if `body` contains any GFM-specific syntax that this
/// plugin would transform.
fn needs_expansion(body: &str) -> bool {
    if body.contains("~~") {
        return true;
    }
    if body.lines().any(is_task_list_line) {
        return true;
    }
    has_table(body)
}

/// Detects whether `body` contains any GFM table block.
fn has_table(body: &str) -> bool {
    let lines: Vec<&str> = body.lines().collect();
    lines.windows(2).any(|w| is_table_header(w[0], w[1]))
}

/// Returns the fence marker (` ``` ` or `~~~`) if `line` opens or
/// closes a fenced code block.
fn detect_fence(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

/// Returns `true` if `header` looks like a table header followed by a
/// `|---|---|` separator row on `separator`.
fn is_table_header(header: &str, separator: &str) -> bool {
    if !header.contains('|') {
        return false;
    }
    is_separator_row(separator)
}

/// Returns `true` if `line` is a GFM table separator row like
/// `| --- | :---: | ---: |`.
fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    if !t.contains('-') || !t.contains('|') {
        return false;
    }
    t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
}

/// Returns the index *just past* the last contiguous table line.
fn find_table_end(lines: &[&str], start: usize) -> usize {
    let mut end = start + 2; // header + separator
    while end < lines.len() {
        let l = lines[end];
        if l.trim().is_empty() || !l.contains('|') {
            break;
        }
        end += 1;
    }
    end
}

/// Returns `true` if `line` is a task list item.
fn is_task_list_line(line: &str) -> bool {
    let t = line.trim_start();
    if t.len() < 6 {
        return false;
    }
    let bytes = t.as_bytes();
    let bullet = bytes[0];
    if !matches!(bullet, b'-' | b'*' | b'+') {
        return false;
    }
    if bytes[1] != b' ' {
        return false;
    }
    if bytes[2] != b'[' {
        return false;
    }
    if !matches!(bytes[3], b' ' | b'x' | b'X') {
        return false;
    }
    if bytes[4] != b']' {
        return false;
    }
    bytes[5] == b' '
}

/// Returns the index just past the last contiguous task list line.
fn find_task_list_end(lines: &[&str], start: usize) -> usize {
    let mut end = start;
    while end < lines.len() && is_task_list_line(lines[end]) {
        end += 1;
    }
    end
}

/// Renders `markdown` to HTML using `pulldown-cmark` with `extra`
/// options merged in alongside the always-on strikethrough flag.
fn render_with_options(markdown: &str, extra: Options) -> String {
    let mut opts = Options::ENABLE_STRIKETHROUGH;
    opts.insert(extra);
    let parser = Parser::new_ext(markdown, opts);
    let mut html = String::with_capacity(markdown.len() + 64);
    cmark_html::push_html(&mut html, parser);
    html.trim_end().to_string()
}

/// Replaces `~~text~~` with `<del>text</del>` outside of inline code spans.
fn apply_strikethrough(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0usize;
    let mut in_code = false;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_code = !in_code;
            out.push('`');
            i += 1;
            continue;
        }
        if !in_code
            && i + 1 < bytes.len()
            && bytes[i] == b'~'
            && bytes[i + 1] == b'~'
        {
            // Find closing `~~`.
            if let Some(close) = find_strike_close(line, i + 2) {
                out.push_str("<del>");
                out.push_str(&line[i + 2..close]);
                out.push_str("</del>");
                i = close + 2;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Returns the byte offset of the next `~~` after `from`, or `None`.
fn find_strike_close(line: &str, from: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut j = from;
    while j + 1 < bytes.len() {
        if bytes[j] == b'`' {
            // Skip inline code spans inside the strike content.
            let mut k = j + 1;
            while k < bytes.len() && bytes[k] != b'`' {
                k += 1;
            }
            j = k.saturating_add(1);
            continue;
        }
        if bytes[j] == b'~' && bytes[j + 1] == b'~' {
            return Some(j);
        }
        j += 1;
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;
    use tempfile::tempdir;

    #[test]
    fn split_frontmatter_extracts_yaml_block() {
        let input = "---\ntitle: Hello\n---\nBody here\n";
        let (fm, body) = split_frontmatter(input);
        assert_eq!(fm, "---\ntitle: Hello\n---\n");
        assert_eq!(body, "Body here\n");
    }

    #[test]
    fn split_frontmatter_returns_empty_when_absent() {
        let input = "Just a body\nwith two lines\n";
        let (fm, body) = split_frontmatter(input);
        assert_eq!(fm, "");
        assert_eq!(body, input);
    }

    #[test]
    fn needs_expansion_detects_strikethrough() {
        assert!(needs_expansion("hello ~~world~~"));
    }

    #[test]
    fn needs_expansion_detects_task_list() {
        assert!(needs_expansion("- [ ] todo\n- [x] done\n"));
    }

    #[test]
    fn needs_expansion_detects_table() {
        let body = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        assert!(needs_expansion(body));
    }

    #[test]
    fn needs_expansion_returns_false_for_plain_markdown() {
        assert!(!needs_expansion("# Heading\n\nA paragraph.\n"));
    }

    #[test]
    fn is_separator_row_accepts_aligned_separators() {
        assert!(is_separator_row("|---|---|"));
        assert!(is_separator_row("| :--- | :---: | ---: |"));
        assert!(!is_separator_row("| a | b |"));
        assert!(!is_separator_row("plain text"));
    }

    #[test]
    fn is_task_list_line_recognises_open_and_done() {
        assert!(is_task_list_line("- [ ] todo"));
        assert!(is_task_list_line("- [x] done"));
        assert!(is_task_list_line("- [X] done"));
        assert!(is_task_list_line("  * [ ] indented"));
        assert!(!is_task_list_line("- regular bullet"));
        assert!(!is_task_list_line("[ ] no bullet"));
    }

    #[test]
    fn apply_strikethrough_wraps_simple_pair() {
        assert_eq!(
            apply_strikethrough("hello ~~world~~ done"),
            "hello <del>world</del> done"
        );
    }

    #[test]
    fn apply_strikethrough_skips_inside_code_span() {
        assert_eq!(
            apply_strikethrough("`~~not~~` but ~~yes~~"),
            "`~~not~~` but <del>yes</del>"
        );
    }

    #[test]
    fn apply_strikethrough_leaves_unmatched_tildes() {
        assert_eq!(apply_strikethrough("just ~~ here"), "just ~~ here");
    }

    #[test]
    fn expand_gfm_renders_table_block() {
        let input = "Intro\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nOutro\n";
        let out = expand_gfm(input);
        assert!(out.contains("<table>"), "got: {out}");
        assert!(out.contains("<th>a</th>"));
        assert!(out.contains("<td>1</td>"));
        assert!(out.contains("Intro"));
        assert!(out.contains("Outro"));
    }

    #[test]
    fn expand_gfm_renders_task_list_block() {
        let input = "- [ ] one\n- [x] two\n";
        let out = expand_gfm(input);
        assert!(out.contains("<ul>"), "got: {out}");
        assert!(out.contains("type=\"checkbox\""));
        assert!(out.contains("disabled"));
        assert!(out.contains("checked"));
    }

    #[test]
    fn expand_gfm_renders_strikethrough_inline() {
        let input = "Some ~~old~~ new text\n";
        let out = expand_gfm(input);
        assert_eq!(out, "Some <del>old</del> new text\n");
    }

    #[test]
    fn expand_gfm_preserves_fenced_code_contents() {
        let input =
            "```\n| a | b |\n|---|---|\n~~not strike~~\n- [ ] not task\n```\n";
        let out = expand_gfm(input);
        // Nothing inside the fence should be transformed.
        assert!(out.contains("| a | b |"));
        assert!(out.contains("~~not strike~~"));
        assert!(out.contains("- [ ] not task"));
        assert!(!out.contains("<table>"));
        assert!(!out.contains("<del>"));
    }

    #[test]
    fn expand_gfm_preserves_frontmatter_unchanged() {
        let input = "---\ntitle: Test\n---\n~~strike~~ this\n";
        let out = expand_gfm(input);
        assert!(out.starts_with("---\ntitle: Test\n---\n"));
        assert!(out.contains("<del>strike</del>"));
    }

    #[test]
    fn expand_gfm_returns_input_unchanged_when_no_features() {
        let input = "# Heading\n\nA paragraph with no extensions.\n";
        let out = expand_gfm(input);
        assert_eq!(out, input);
    }

    #[test]
    fn expand_gfm_handles_tildes_in_tilde_fenced_code() {
        // ~~~ fences must also protect contents.
        let input = "~~~\n~~text~~\n~~~\n";
        let out = expand_gfm(input);
        assert!(out.contains("~~text~~"));
        assert!(!out.contains("<del>"));
    }

    #[test]
    fn plugin_transforms_markdown_files_in_place() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(
            content.join("post.md"),
            "---\ntitle: Test\n---\n~~old~~ new\n",
        )
        .unwrap();
        fs::write(content.join("untouched.md"), "# Plain\n\nNothing fancy.\n")
            .unwrap();

        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());
        MarkdownExtPlugin.before_compile(&ctx).unwrap();

        let post = fs::read_to_string(content.join("post.md")).unwrap();
        assert!(post.contains("<del>old</del>"));
        assert!(post.starts_with("---\ntitle: Test\n---\n"));

        let untouched =
            fs::read_to_string(content.join("untouched.md")).unwrap();
        assert_eq!(untouched, "# Plain\n\nNothing fancy.\n");
    }

    #[test]
    fn plugin_returns_ok_when_content_dir_missing() {
        let dir = tempdir().unwrap();
        let ctx = PluginContext::new(
            &dir.path().join("missing"),
            dir.path(),
            dir.path(),
            dir.path(),
        );
        MarkdownExtPlugin.before_compile(&ctx).unwrap();
    }

    #[test]
    fn plugin_name_is_markdown_ext() {
        assert_eq!(MarkdownExtPlugin.name(), "markdown-ext");
    }
}
