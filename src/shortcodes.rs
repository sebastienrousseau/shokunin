// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shortcode expansion plugin.
//!
//! Preprocesses Markdown content before compilation, expanding
//! `{{< shortcode args >}}` patterns into HTML fragments.

use crate::plugin::{Plugin, PluginContext};
use crate::MAX_DIR_DEPTH;
use anyhow::Result;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Plugin that expands shortcodes in Markdown content.
///
/// Runs in `before_compile` to transform content before staticdatagen
/// processes it.
///
/// Built-in shortcodes:
/// - `{{< youtube id="..." >}}` — responsive `YouTube` embed
/// - `{{< gist user="..." id="..." >}}` — GitHub gist embed
/// - `{{< figure src="..." alt="..." caption="..." >}}` — figure with caption
/// - `{{< warning >}}...{{< /warning >}}` — admonition blocks
/// - `{{< info >}}...{{< /info >}}`
/// - `{{< tip >}}...{{< /tip >}}`
/// - `{{< danger >}}...{{< /danger >}}`
#[derive(Debug, Clone, Copy)]
pub struct ShortcodePlugin;

impl Plugin for ShortcodePlugin {
    fn name(&self) -> &'static str {
        "shortcodes"
    }

    fn before_compile(&self, ctx: &PluginContext) -> Result<()> {
        if !ctx.content_dir.exists() {
            return Ok(());
        }

        let md_files = collect_md_files(&ctx.content_dir)?;
        let mut expanded = 0usize;

        for path in &md_files {
            let content = fs::read_to_string(path)?;
            let result = expand_shortcodes(&content);
            if result != content {
                fs::write(path, &result)?;
                expanded += 1;
            }
        }

        if expanded > 0 {
            log::info!(
                "[shortcodes] Expanded shortcodes in {expanded} file(s)"
            );
        }
        Ok(())
    }
}

/// Expands all shortcodes in a string.
#[must_use]
pub fn expand_shortcodes(input: &str) -> String {
    let mut result = input.to_string();

    // Block shortcodes: {{< name >}}...{{< /name >}}
    for name in &["warning", "info", "tip", "danger"] {
        result = expand_block_shortcode(&result, name);
    }

    // Inline shortcodes: {{< name key="value" >}}
    result = expand_inline_shortcodes(&result);

    result
}

/// Expands block shortcodes like `{{< warning >}}...{{< /warning >}}`.
fn expand_block_shortcode(input: &str, name: &str) -> String {
    let open = format!("{{{{< {name} >}}}}");
    let close = format!("{{{{< /{name} >}}}}");
    let mut result = input.to_string();

    while let Some(start) = result.find(&open) {
        let after_open = start + open.len();
        if let Some(end_offset) = result[after_open..].find(&close) {
            let end = after_open + end_offset;
            let inner = result[after_open..end].trim();
            let html = format!(
                "<div class=\"admonition admonition-{}\" role=\"note\">\n\
                 <p class=\"admonition-title\">{}</p>\n\
                 <div class=\"admonition-content\">\n{}\n</div>\n</div>",
                name,
                capitalize(name),
                inner
            );
            result = format!(
                "{}{}{}",
                &result[..start],
                html,
                &result[end + close.len()..]
            );
        } else {
            break;
        }
    }

    result
}

/// Expands inline shortcodes like `{{< youtube id="..." >}}`.
///
/// Safe for non-ASCII input: byte-level slicing is guarded by
/// `is_char_boundary` and fallthrough characters are iterated via
/// `char_indices()` so multi-byte codepoints (emoji, `©`, etc.) are
/// preserved verbatim rather than truncated mid-byte.
fn expand_inline_shortcodes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        // The opening marker "{{<" is pure ASCII so byte-level
        // comparison is safe *as long as pos lands on a char
        // boundary*. Guard with is_char_boundary to be explicit.
        if input.is_char_boundary(pos)
            && pos + 3 <= input.len()
            && input.as_bytes()[pos] == b'{'
            && input.as_bytes()[pos + 1] == b'{'
            && input.as_bytes()[pos + 2] == b'<'
        {
            if let Some(end) = input[pos..].find(">}}") {
                let tag = input[pos + 3..pos + end].trim();
                let html = render_inline_shortcode(tag);
                result.push_str(&html);
                pos += end + 3;
                continue;
            }
        }
        // Fallthrough: push the next full codepoint, not just one
        // byte — this handles multi-byte UTF-8 characters cleanly.
        // The `chars().next()` is guaranteed to be `Some` because
        // the loop guard `pos < input.len()` ensures the suffix is
        // non-empty, so an `expect` here cannot panic.
        #[allow(clippy::expect_used)]
        let c = input[pos..]
            .chars()
            .next()
            .expect("loop guard ensures pos < input.len()");
        result.push(c);
        pos += c.len_utf8();
    }

    result
}

/// Renders a single inline shortcode tag content.
fn render_inline_shortcode(tag: &str) -> String {
    let parts = parse_shortcode_attrs(tag);
    let name = parts.get("_name").map_or("", String::as_str);

    match name {
        "youtube" => {
            let id = parts.get("id").map_or("", String::as_str);
            if id.is_empty() {
                return "<!-- youtube: missing id -->".to_string();
            }
            format!(
                "<div class=\"video-container\" style=\"position:relative;padding-bottom:56.25%;height:0;overflow:hidden\">\
                 <iframe src=\"https://www.youtube-nocookie.com/embed/{id}\" \
                 style=\"position:absolute;top:0;left:0;width:100%;height:100%\" \
                 frameborder=\"0\" allowfullscreen loading=\"lazy\" \
                 title=\"YouTube video\"></iframe></div>"
            )
        }
        "gist" => {
            let user = parts.get("user").map_or("", String::as_str);
            let id = parts.get("id").map_or("", String::as_str);
            if user.is_empty() || id.is_empty() {
                return "<!-- gist: missing user or id -->".to_string();
            }
            format!(
                "<script src=\"https://gist.github.com/{user}/{id}.js\"></script>"
            )
        }
        "figure" => {
            let src = parts.get("src").map_or("", String::as_str);
            let alt = parts.get("alt").map_or("", String::as_str);
            let caption = parts.get("caption").map_or("", String::as_str);
            let mut html = format!(
                "<figure><img src=\"{src}\" alt=\"{alt}\" loading=\"lazy\">"
            );
            if !caption.is_empty() {
                html.push_str(&format!("<figcaption>{caption}</figcaption>"));
            }
            html.push_str("</figure>");
            html
        }
        _ => format!("<!-- unknown shortcode: {name} -->"),
    }
}

/// Parses shortcode attributes: `name key="value" key2="value2"`
fn parse_shortcode_attrs(tag: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let trimmed = tag.trim();

    // First token is the shortcode name
    let mut chars = trimmed.char_indices().peekable();
    let mut name_end = 0;
    while let Some(&(i, c)) = chars.peek() {
        if c.is_whitespace() {
            name_end = i;
            break;
        }
        name_end = i + c.len_utf8();
        let _ = chars.next();
    }
    let _ = attrs.insert("_name".to_string(), trimmed[..name_end].to_string());

    // Parse key="value" pairs
    let rest = &trimmed[name_end..];
    let mut pos = 0;
    while pos < rest.len() {
        // Skip whitespace
        while pos < rest.len() && rest.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= rest.len() {
            break;
        }

        // Find key
        let key_start = pos;
        while pos < rest.len() && rest.as_bytes()[pos] != b'=' {
            pos += 1;
        }
        if pos >= rest.len() {
            break;
        }
        let key = rest[key_start..pos].trim().to_string();
        pos += 1; // skip =

        // Find value (quoted)
        if pos < rest.len() && rest.as_bytes()[pos] == b'"' {
            pos += 1;
            let val_start = pos;
            while pos < rest.len() && rest.as_bytes()[pos] != b'"' {
                pos += 1;
            }
            let val = rest[val_start..pos].to_string();
            let _ = attrs.insert(key, val);
            pos += 1; // skip closing "
        }
    }

    attrs
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>> {
    crate::walk::walk_files_bounded_depth(dir, "md", MAX_DIR_DEPTH)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_shortcode() {
        let input = r#"Check this: {{< youtube id="abc123" >}}"#;
        let result = expand_shortcodes(input);
        assert!(result.contains("youtube-nocookie.com/embed/abc123"));
        assert!(result.contains("video-container"));
    }

    #[test]
    fn test_gist_shortcode() {
        let input = r#"{{< gist user="octocat" id="12345" >}}"#;
        let result = expand_shortcodes(input);
        assert!(result.contains("gist.github.com/octocat/12345.js"));
    }

    #[test]
    fn test_figure_shortcode() {
        let input = r#"{{< figure src="/img/photo.jpg" alt="A photo" caption="My photo" >}}"#;
        let result = expand_shortcodes(input);
        assert!(result.contains("<figure>"));
        assert!(result.contains("alt=\"A photo\""));
        assert!(result.contains("<figcaption>My photo</figcaption>"));
    }

    #[test]
    fn test_warning_block() {
        let input = "{{< warning >}}\nBe careful!\n{{< /warning >}}";
        let result = expand_shortcodes(input);
        assert!(result.contains("admonition-warning"));
        assert!(result.contains("Warning"));
        assert!(result.contains("Be careful!"));
    }

    #[test]
    fn test_info_block() {
        let input = "{{< info >}}\nNote this.\n{{< /info >}}";
        let result = expand_shortcodes(input);
        assert!(result.contains("admonition-info"));
        assert!(result.contains("Info"));
    }

    #[test]
    fn test_unknown_shortcode() {
        let input = r#"{{< unknown key="val" >}}"#;
        let result = expand_shortcodes(input);
        assert!(result.contains("<!-- unknown shortcode: unknown -->"));
    }

    #[test]
    fn test_no_shortcodes() {
        let input = "Regular markdown with no shortcodes.";
        let result = expand_shortcodes(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_parse_attrs() {
        let attrs = parse_shortcode_attrs(r#"youtube id="abc" "#);
        assert_eq!(attrs.get("_name").unwrap(), "youtube");
        assert_eq!(attrs.get("id").unwrap(), "abc");
    }

    // -------------------------------------------------------------------
    // Plugin surface + missing-param branches
    // -------------------------------------------------------------------

    #[test]
    fn name_returns_static_shortcodes_identifier() {
        assert_eq!(ShortcodePlugin.name(), "shortcodes");
    }

    #[test]
    fn before_compile_missing_content_dir_returns_ok() {
        // Line 41: `!ctx.content_dir.exists()` early return.
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        let ctx =
            PluginContext::new(&missing, dir.path(), dir.path(), dir.path());
        ShortcodePlugin.before_compile(&ctx).unwrap();
    }

    #[test]
    fn before_compile_no_markdown_files_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());
        ShortcodePlugin.before_compile(&ctx).unwrap();
    }

    #[test]
    fn before_compile_unchanged_file_is_not_rewritten() {
        // Line 50 else branch: file doesn't contain any shortcodes,
        // so result == content and the fs::write is skipped.
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        let body = "plain markdown no shortcodes";
        fs::write(content.join("p.md"), body).unwrap();
        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());
        ShortcodePlugin.before_compile(&ctx).unwrap();
        assert_eq!(fs::read_to_string(content.join("p.md")).unwrap(), body);
    }

    // -------------------------------------------------------------------
    // render_inline_shortcode — missing required-param branches
    // -------------------------------------------------------------------

    #[test]
    fn render_inline_shortcode_youtube_missing_id_emits_comment() {
        // Line 146: the `if id.is_empty()` branch.
        let result = expand_shortcodes(r"{{< youtube >}}");
        assert!(result.contains("<!-- youtube: missing id -->"));
    }

    #[test]
    fn render_inline_shortcode_gist_missing_user_emits_comment() {
        // Line 160: the `if user.is_empty() || id.is_empty()` branch.
        let result = expand_shortcodes(r#"{{< gist id="123" >}}"#);
        assert!(result.contains("<!-- gist: missing user or id -->"));
    }

    #[test]
    fn render_inline_shortcode_gist_missing_id_emits_comment() {
        let result = expand_shortcodes(r#"{{< gist user="octocat" >}}"#);
        assert!(result.contains("<!-- gist: missing user or id -->"));
    }

    #[test]
    fn render_inline_shortcode_figure_without_caption_omits_figcaption() {
        // Line 173 else branch: no caption argument means the
        // `<figcaption>` is NOT appended.
        let result =
            expand_shortcodes(r#"{{< figure src="/a.jpg" alt="A" >}}"#);
        assert!(result.contains("<figure>"));
        assert!(result.contains(r#"alt="A""#));
        assert!(!result.contains("<figcaption>"));
    }

    #[test]
    fn render_inline_shortcode_figure_with_caption_includes_figcaption() {
        let result = expand_shortcodes(
            r#"{{< figure src="/a.jpg" alt="A" caption="Hi" >}}"#,
        );
        assert!(result.contains("<figcaption>Hi</figcaption>"));
    }

    // -------------------------------------------------------------------
    // expand_block_shortcode — unterminated block break
    // -------------------------------------------------------------------

    #[test]
    fn expand_block_shortcode_unterminated_breaks_out_cleanly() {
        // Line 107: the `break` branch when the closing tag is
        // missing. `expand_block_shortcode` must return without
        // looping forever; the inline-shortcode pass then
        // subsequently processes the unterminated `{{< warning >}}`
        // as an unknown inline tag. What matters for coverage is
        // that the function terminates.
        let input = "{{< warning >}}\nno closing tag\n";
        let result = expand_block_shortcode(input, "warning");
        // Since there's no close tag, the block expander leaves
        // the input untouched.
        assert_eq!(result, input);
    }

    // -------------------------------------------------------------------
    // capitalize
    // -------------------------------------------------------------------

    #[test]
    fn capitalize_empty_string_returns_empty() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn capitalize_single_word_uppercases_first_letter() {
        assert_eq!(capitalize("warning"), "Warning");
        assert_eq!(capitalize("info"), "Info");
    }

    // -------------------------------------------------------------------
    // collect_md_files — depth guard + filter
    // -------------------------------------------------------------------

    #[test]
    fn shortcodes_collect_md_files_respects_max_dir_depth() {
        let dir = tempfile::tempdir().unwrap();
        let mut current = dir.path().to_path_buf();
        for i in 0..MAX_DIR_DEPTH + 2 {
            current = current.join(format!("d{i}"));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("p.md"), "").unwrap();
        }
        let files = collect_md_files(dir.path()).unwrap();
        assert!(files.len() <= MAX_DIR_DEPTH + 1);
    }

    #[test]
    fn parse_shortcode_attrs_trailing_whitespace_breaks_outer_loop() {
        // Line 210: `if pos >= rest.len() { break }` after the
        // whitespace-skip loop. Trigger by trailing whitespace
        // immediately after the shortcode name (no key=value).
        let attrs = parse_shortcode_attrs("name   ");
        assert_eq!(attrs.get("_name").unwrap(), "name");
        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn parse_shortcode_attrs_with_value_then_trailing_whitespace() {
        let attrs = parse_shortcode_attrs("youtube id=\"x\"   ");
        assert_eq!(attrs.get("_name").unwrap(), "youtube");
        assert_eq!(attrs.get("id").unwrap(), "x");
    }

    #[test]
    fn parse_shortcode_attrs_key_without_equals_breaks() {
        // Line 219: `if pos >= rest.len() { break }` when searching
        // for `=` falls off the end of the string.
        let attrs = parse_shortcode_attrs("youtube id=\"x\" trailingflag");
        assert_eq!(attrs.get("_name").unwrap(), "youtube");
        // The trailing token without `=` is silently dropped.
        assert!(!attrs.contains_key("trailingflag"));
    }

    #[test]
    fn parse_shortcode_attrs_unquoted_value_is_dropped() {
        // Line 234: the `if rest.as_bytes()[pos] == b'"'` is FALSE
        // for unquoted values, so the body is skipped.
        let attrs = parse_shortcode_attrs("name id=unquoted");
        assert_eq!(attrs.get("_name").unwrap(), "name");
        assert!(!attrs.contains_key("id"));
    }

    #[test]
    fn expand_inline_shortcodes_unterminated_tag_falls_through_to_pushchar() {
        // Line 128 path: when `{{<` is found but no `>}}` follows,
        // the if-let returns None and the byte-by-byte fallback at
        // line 130 takes over.
        let result = expand_shortcodes("text {{< unterminated");
        assert!(result.contains("text"));
        assert!(result.contains("unterminated"));
    }

    #[test]
    fn shortcodes_collect_md_files_filters_non_md_extensions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();
        let files = collect_md_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_plugin_expands_files() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(
            content.join("test.md"),
            r#"---
title: Test
---
{{< youtube id="xyz" >}}
"#,
        )
        .unwrap();

        let ctx =
            PluginContext::new(&content, dir.path(), dir.path(), dir.path());
        ShortcodePlugin.before_compile(&ctx).unwrap();

        let result = fs::read_to_string(content.join("test.md")).unwrap();
        assert!(result.contains("youtube-nocookie.com"));
    }
}
