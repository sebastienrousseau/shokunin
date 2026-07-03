// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Markdown linting and formatting gate (native Rust, no shell-out).
//!
//! This is the MVP rule-set tracked under issue #549. It covers the
//! `markdownlint`-style rules that catch the bulk of authoring drift
//! without requiring a full upstream `markdownlint-rs` integration —
//! the depth-of-coverage roadmap calls out adding the upstream port
//! once it stabilises for 2026 toolchains.
//!
//! Rules enforced:
//! - **MD009** — no trailing whitespace at end of line.
//! - **MD010** — no hard tabs (use spaces).
//! - **MD025** — at most one top-level `#` heading per file.
//! - **MD034** — no bare `http://` / `https://` URLs (must be in
//!   `[text](url)` or `<url>` form).
//! - **MD041** — file must start with a top-level heading (or YAML
//!   frontmatter followed by one).
//!
//! Scans the **content** directory (`<site>/../content` or the sibling
//! `content/` dir). When neither exists the gate emits an info note
//! and skips — matches the behaviour of the docs gates.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};
use crate::walk::walk_files;
use std::path::Path;

const NAME: &str = "markdownlint";

/// Markdown linting + formatting gate.
///
/// # Examples
///
/// ```
/// use ssg::audit::AuditGate;
/// use ssg::audit::gates::markdownlint::MarkdownlintGate;
/// assert_eq!(MarkdownlintGate.name(), "markdownlint");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MarkdownlintGate;

impl AuditGate for MarkdownlintGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Lints Markdown source under content/ for: trailing whitespace \
         (MD009), hard tabs (MD010), multiple H1s (MD025), bare URLs \
         (MD034), and missing top-level heading (MD041). When content/ \
         is absent the gate skips with an info note."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content_dir = locate_content_dir(&site.root);
        let Some(dir) = content_dir else {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Info,
                    "No content/ directory found alongside site root; gate skipped",
                )
                .with_code("MD-INPUT-MISSING"),
            );
            return findings;
        };

        let files = walk_files(&dir, "md").unwrap_or_default();
        for path in &files {
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            let rel = path
                .strip_prefix(&dir)
                .unwrap_or(path)
                .to_string_lossy()
                .into_owned();
            lint_markdown(&text, &rel, &mut findings);
        }
        findings
    }
}

fn locate_content_dir(root: &Path) -> Option<std::path::PathBuf> {
    let direct = root.join("content");
    if direct.is_dir() {
        return Some(direct);
    }
    let sibling = root.parent()?.join("content");
    sibling.is_dir().then_some(sibling)
}

fn lint_markdown(text: &str, rel: &str, findings: &mut Vec<Finding>) {
    let mut h1_count = 0usize;
    let mut in_code_block = false;
    let fm_lines = frontmatter_line_count(text);

    // MD041: first non-frontmatter, non-blank line must be `# `.
    // A frontmatter `title:` satisfies the requirement too — mirrors
    // upstream markdownlint's `front_matter_title` behaviour, since
    // the H1 is template-provided from the title on such pages.
    let first_content_line = first_heading_candidate(text);
    if let Some(line) = first_content_line {
        if !line.starts_with("# ") && !frontmatter_has_title(text, fm_lines) {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    "File does not begin with a top-level (#) heading",
                )
                .with_code("MD041")
                .with_path(rel.to_string()),
            );
        }
    }

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        // YAML frontmatter is not Markdown — never lint it (URL values
        // like `permalink:` are not bare-URL prose, `#` is a comment).
        if idx < fm_lines {
            continue;
        }
        if raw_line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if raw_line.contains('\t') {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    format!("L{line_no}: hard tab character"),
                )
                .with_code("MD010")
                .with_path(rel.to_string()),
            );
        }
        if raw_line.ends_with(' ') && !raw_line.trim().is_empty() {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    format!("L{line_no}: trailing whitespace"),
                )
                .with_code("MD009")
                .with_path(rel.to_string()),
            );
        }
        if raw_line.starts_with("# ") {
            h1_count += 1;
        }
        // MD034: bare URL not inside (...) or <...>
        if let Some(idx2) = raw_line
            .find("http://")
            .or_else(|| raw_line.find("https://"))
        {
            let before = &raw_line[..idx2];
            let after_url =
                raw_line[idx2..].split_whitespace().next().unwrap_or("");
            let in_bracket = before.contains('(')
                && raw_line[idx2 + after_url.len()..].starts_with(')');
            let in_lt = before.ends_with('<')
                && raw_line[idx2 + after_url.len()..].starts_with('>');
            let in_link = before.ends_with("](");
            if !in_bracket && !in_lt && !in_link {
                findings.push(
                    Finding::new(
                        NAME,
                        Severity::Warn,
                        format!("L{line_no}: bare URL — wrap in <…> or []()"),
                    )
                    .with_code("MD034")
                    .with_path(rel.to_string()),
                );
            }
        }
    }

    if h1_count > 1 {
        findings.push(
            Finding::new(
                NAME,
                Severity::Warn,
                format!(
                    "File has {h1_count} top-level (#) headings; expected 1"
                ),
            )
            .with_code("MD025")
            .with_path(rel.to_string()),
        );
    }
}

/// Returns the number of leading lines occupied by YAML frontmatter
/// (opening `---`, body, closing `---` inclusive), or `0` when the
/// file has none or the fence never closes.
fn frontmatter_line_count(text: &str) -> usize {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return 0;
    }
    let mut count = 1;
    for line in lines {
        count += 1;
        if line.trim() == "---" {
            return count;
        }
    }
    0
}

/// `true` when the frontmatter block declares a `title:` (or `title=`)
/// key — upstream markdownlint's `front_matter_title` default.
fn frontmatter_has_title(text: &str, fm_lines: usize) -> bool {
    fm_lines > 0
        && text.lines().take(fm_lines).skip(1).any(|line| {
            let lower = line.trim_start().to_ascii_lowercase();
            lower.starts_with("title:") || lower.starts_with("title=")
        })
}

fn first_heading_candidate(text: &str) -> Option<&str> {
    let mut lines = text.lines();
    if let Some(first) = lines.next() {
        if first.trim() == "---" {
            // Skip frontmatter block
            for line in &mut lines {
                if line.trim() == "---" {
                    break;
                }
            }
        } else if !first.trim().is_empty() {
            return Some(first);
        }
    }
    lines.find(|line| !line.trim().is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn site_with_content(files: &[(&str, &str)]) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let site_root = tmp.path().join("public");
        let content = tmp.path().join("content");
        std::fs::create_dir_all(&site_root).unwrap();
        std::fs::create_dir_all(&content).unwrap();
        for (rel, body) in files {
            let p = content.join(rel);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&p, body).unwrap();
        }
        std::mem::forget(tmp);
        Site {
            root: site_root,
            html_files: Vec::new(),
        }
    }

    #[test]
    fn passing_markdown_is_clean() {
        let s = site_with_content(&[(
            "index.md",
            "# Title\n\nThis is a paragraph.\n\n[link](https://example.com)\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn bad_markdown_is_flagged() {
        let s = site_with_content(&[(
            "bad.md",
            "## not h1\n\nhttps://bare-url.test\n\ttab line\ntrailing ws \n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        let codes: Vec<_> =
            f.iter().filter_map(|x| x.code.as_deref()).collect();
        assert!(codes.contains(&"MD034"));
        assert!(codes.contains(&"MD010"));
        assert!(codes.contains(&"MD009"));
        assert!(codes.contains(&"MD041"));
    }

    #[test]
    fn absent_content_dir_emits_info_skip() {
        let s = Site {
            root: PathBuf::from("/nonexistent/dir/that/does/not/exist"),
            html_files: Vec::new(),
        };
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code.as_deref(), Some("MD-INPUT-MISSING"));
    }

    #[test]
    fn multiple_h1_headings_trip_md025() {
        let s =
            site_with_content(&[("doc.md", "# first\n\n# second\n\nbody\n")]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("MD025")));
    }

    #[test]
    fn frontmatter_then_heading_is_clean() {
        let s = site_with_content(&[(
            "doc.md",
            "---\ntitle: x\n---\n\n# Heading\n\nbody.\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "frontmatter wrapper should be clean: {f:?}");
    }

    #[test]
    fn bare_url_inside_markdown_link_is_silent() {
        let s = site_with_content(&[(
            "doc.md",
            "# title\n\n[click](https://example.com)\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.iter().all(|x| x.code.as_deref() != Some("MD034")));
    }

    #[test]
    fn trailing_whitespace_outside_code_block_trips_md009() {
        let s = site_with_content(&[(
            "doc.md",
            "# title\n\nline with trailing space \n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("MD009")));
    }

    #[test]
    fn code_block_contents_are_not_linted() {
        let s = site_with_content(&[(
            "doc.md",
            "# title\n\n```\n\ttab in code\nhttps://bare.in-code\n```\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        // Inside the fenced block: MD010 and MD034 should NOT fire.
        assert!(
            f.iter().all(|x| x.code.as_deref() != Some("MD010")
                && x.code.as_deref() != Some("MD034")),
            "code-block contents should be exempt: {f:?}"
        );
    }

    #[test]
    fn missing_top_heading_flagged_md041() {
        let s = site_with_content(&[("doc.md", "Just a paragraph.\n")]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.iter().any(|x| x.code.as_deref() == Some("MD041")));
    }

    #[test]
    fn empty_file_produces_no_md041() {
        // first_heading_candidate returns None on a fully empty file.
        let s = site_with_content(&[("empty.md", "")]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(f.iter().all(|x| x.code.as_deref() != Some("MD041")));
    }

    #[test]
    fn sibling_content_dir_layout_is_discovered() {
        // Use a `<root>/../content` layout, mimicking real ssg sites.
        let s = site_with_content(&[("doc.md", "# ok\n")]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter()
                .all(|x| x.code.as_deref() != Some("MD-INPUT-MISSING")),
            "sibling content/ should be discovered: {f:?}"
        );
    }

    #[test]
    fn frontmatter_url_values_are_not_bare_urls() {
        // Regression: ~110 false MD034 on frontmatter values like
        // `permalink: "https://…"` — YAML is not Markdown prose.
        let s = site_with_content(&[(
            "doc.md",
            "---\ntitle: \"X\"\npermalink: \"https://example.com/x/\"\n\
             url: https://example.com\natom: \"https://example.com/atom.xml\"\n\
             ---\n\n# Heading\n\nbody.\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().all(|x| x.code.as_deref() != Some("MD034")),
            "frontmatter URLs must not trip MD034: {f:?}"
        );
    }

    #[test]
    fn bare_url_in_body_still_flagged_after_frontmatter() {
        // True positive preserved: bare URLs in body text still fire.
        let s = site_with_content(&[(
            "doc.md",
            "---\ntitle: \"X\"\n---\n\n# Heading\n\nSee https://bare.example\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().any(|x| x.code.as_deref() == Some("MD034")),
            "body bare URL must still trip MD034: {f:?}"
        );
    }

    #[test]
    fn frontmatter_hard_tabs_and_trailing_ws_are_exempt() {
        let s = site_with_content(&[(
            "doc.md",
            "---\ntitle: \"X\"\nkey:\t\"tabbed\"   \n---\n\n# Heading\n\nbody\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().all(|x| x.code.as_deref() != Some("MD010")
                && x.code.as_deref() != Some("MD009")),
            "frontmatter must be exempt from MD009/MD010: {f:?}"
        );
    }

    #[test]
    fn frontmatter_yaml_comment_is_not_an_h1() {
        // A YAML `# comment` inside frontmatter must not count toward
        // MD025's H1 tally.
        let s = site_with_content(&[(
            "doc.md",
            "---\n# yaml comment\ntitle: \"X\"\n---\n\n# Only H1\n\nbody\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().all(|x| x.code.as_deref() != Some("MD025")),
            "yaml comments are not headings: {f:?}"
        );
    }

    #[test]
    fn frontmatter_title_satisfies_md041() {
        // Upstream `front_matter_title` behaviour: the H1 is
        // template-provided from `title:`, so no MD041.
        let s = site_with_content(&[(
            "doc.md",
            "---\ntitle: \"Threshold\"\n---\n\n## Section heading\n\nbody\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().all(|x| x.code.as_deref() != Some("MD041")),
            "frontmatter title: must satisfy MD041: {f:?}"
        );
    }

    #[test]
    fn missing_title_and_h1_still_trips_md041() {
        // True positive preserved: no `title:` and no leading `# `.
        let s = site_with_content(&[(
            "doc.md",
            "---\nauthor: \"A\"\n---\n\n## Not an H1\n\nbody\n",
        )]);
        let f = MarkdownlintGate.run(&s, &AuditOptions::default());
        assert!(
            f.iter().any(|x| x.code.as_deref() == Some("MD041")),
            "no title + no H1 must still trip MD041: {f:?}"
        );
    }

    #[test]
    fn frontmatter_helpers_edge_cases() {
        assert_eq!(frontmatter_line_count("no frontmatter\n"), 0);
        assert_eq!(frontmatter_line_count("---\ntitle: x\n---\n"), 3);
        assert_eq!(
            frontmatter_line_count("---\nnever closed\n"),
            0,
            "unterminated fence is not frontmatter"
        );
        assert!(frontmatter_has_title("---\nTitle: \"X\"\n---\n", 3));
        assert!(!frontmatter_has_title("---\nsubtitle: \"X\"\n---\n", 3));
        assert!(!frontmatter_has_title("body only\n", 0));
    }

    #[test]
    fn metadata_methods_exposed() {
        let g = MarkdownlintGate;
        assert_eq!(g.name(), "markdownlint");
        assert!(g.explain().contains("MD0"));
        let _copy: MarkdownlintGate = g;
        let _clone = g;
        assert!(format!("{g:?}").contains("MarkdownlintGate"));
    }
}
