// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Internal documentation link integrity gate.
//!
//! Walks every `.md` file under `docs/`, the top-level `README.md`,
//! `SECURITY.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, and the four
//! `docs/compare/*.md`/`docs/guide/*.md` files. For each Markdown
//! link of the form `[text](relative/path.md)` or `[text](path.md#anchor)`,
//! verifies the target file exists.
//!
//! ## Scope
//!
//! - **In:** relative links (`../foo.md`, `./bar.md`, `subdir/baz.md`).
//! - **In:** anchored relative links (`foo.md#section`) — the file
//!   must exist; anchor existence is not checked (would require a
//!   full Markdown parse).
//! - **Out:** absolute URLs (`http://`, `https://`, `//cdn...`).
//! - **Out:** `mailto:`, `tel:`, `data:` URIs.
//! - **Out:** in-page anchors (`#section` with no path).
//!
//! ## Methodology
//!
//! Cheap regex-free walker: scans for `](` substrings and extracts
//! the parenthesised URL. Skips URLs that look external. Resolves
//! every remaining URL against the containing file's directory and
//! asserts the resolved path exists on disk.
//!
//! On failure, the panic message groups broken links by source file
//! and prints up to 10 entries per file with a `source:link → target`
//! line so reviewers can fix in one pass.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Returns every Markdown file under the inspection set.
fn collect_markdown() -> Vec<PathBuf> {
    let ws = workspace();
    let mut files = Vec::new();

    // Top-level fixed files.
    for top in [
        "README.md",
        "SECURITY.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "BENCHMARKS.md",
        "AUTHORS.md",
        "TEMPLATE.md",
    ] {
        let p = ws.join(top);
        if p.is_file() {
            files.push(p);
        }
    }

    // Recursive sweep of docs/.
    let docs = ws.join("docs");
    if docs.is_dir() {
        walk(&docs, &mut files);
    }

    files
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

/// Extracts every Markdown link target from `text`. Returns
/// `(line_number, raw_url)` pairs. Skips obvious external URLs and
/// pure anchors.
fn extract_links(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let bytes = line.as_bytes();
        let mut i = 0;
        while i + 1 < bytes.len() {
            if bytes[i] == b']' && bytes[i + 1] == b'(' {
                let after = &line[i + 2..];
                if let Some(end) = after.find(')') {
                    let url = &after[..end];
                    if !is_external(url) {
                        out.push((line_no, url.trim().to_string()));
                    }
                    i += 2 + end;
                    continue;
                }
            }
            i += 1;
        }
    }
    out
}

/// Returns `true` for URLs we don't validate (full URLs, mailto, etc.).
fn is_external(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.starts_with('#') {
        return true; // in-page anchor
    }
    let lower = trimmed.to_lowercase();
    for prefix in [
        "http://", "https://", "//", "mailto:", "tel:", "data:", "ftp://",
    ] {
        if lower.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Splits a link URL into `(path, anchor)`. Anchor is `None` for
/// plain file links.
fn split_anchor(url: &str) -> (&str, Option<&str>) {
    if let Some(hash) = url.find('#') {
        (&url[..hash], Some(&url[hash + 1..]))
    } else {
        (url, None)
    }
}

/// Resolves `link` against the directory containing `source`, then
/// logically canonicalises `..` and `.` without touching the
/// filesystem. Preserves absolute-vs-relative status of the input.
fn resolve(source: &Path, link: &str) -> PathBuf {
    let base = source.parent().unwrap_or(source);
    let joined = base.join(link);
    let was_absolute = joined.is_absolute();

    let mut components: Vec<&std::ffi::OsStr> = Vec::new();
    for c in joined.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _ = components.pop();
            }
            std::path::Component::Normal(s) => components.push(s),
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                // These get re-added below via `was_absolute`.
            }
        }
    }

    let mut out = if was_absolute {
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };
    for c in components {
        out.push(c);
    }
    out
}

#[test]
fn every_markdown_link_in_docs_resolves() {
    let files = collect_markdown();
    if files.is_empty() {
        panic!("no markdown files found under workspace — broken test setup");
    }

    let ws = workspace();
    let mut broken: Vec<(PathBuf, usize, String, PathBuf)> = Vec::new();

    for source in &files {
        let Ok(text) = fs::read_to_string(source) else {
            continue;
        };
        for (line_no, raw) in extract_links(&text) {
            let (path_part, _anchor) = split_anchor(&raw);
            if path_part.is_empty() {
                continue; // pure anchor
            }
            let absolute = resolve(source, path_part);
            // `source` is already absolute (collected via walk()), so
            // `resolve()` returns an absolute path. No further join.
            let _ = ws; // retained for the workspace-root reference
            if !absolute.exists() {
                broken.push((
                    source.strip_prefix(ws).unwrap_or(source).to_path_buf(),
                    line_no,
                    raw,
                    absolute,
                ));
            }
        }
    }

    if !broken.is_empty() {
        let mut by_source: std::collections::BTreeMap<
            PathBuf,
            Vec<&(PathBuf, usize, String, PathBuf)>,
        > = std::collections::BTreeMap::new();
        for entry in &broken {
            by_source.entry(entry.0.clone()).or_default().push(entry);
        }
        let mut msg = format!(
            "{} broken link(s) across {} file(s):\n",
            broken.len(),
            by_source.len()
        );
        for (source, list) in &by_source {
            msg.push_str(&format!(
                "\n  ── {} ({} broken) ──\n",
                source.display(),
                list.len()
            ));
            for entry in list.iter().take(10) {
                msg.push_str(&format!(
                    "    line {}: `{}` → missing {}\n",
                    entry.1,
                    entry.2,
                    entry.3.display(),
                ));
            }
            if list.len() > 10 {
                msg.push_str(&format!(
                    "    ... and {} more\n",
                    list.len() - 10
                ));
            }
        }
        panic!("{msg}");
    }

    eprintln!(
        "[doc_links] {} markdown file(s) scanned, every internal link \
         resolves to an existing file",
        files.len()
    );
}

#[test]
fn extract_links_skips_external_urls() {
    let text = "Hello [a](https://example.com) and [b](/abs.md) and \
                [c](./rel.md) plus [d](mailto:x@y.com)";
    let links = extract_links(text);
    let urls: Vec<&str> = links.iter().map(|(_, u)| u.as_str()).collect();
    assert!(!urls.iter().any(|u| u.starts_with("https://")));
    assert!(!urls.iter().any(|u| u.starts_with("mailto:")));
    assert!(urls.contains(&"/abs.md"));
    assert!(urls.contains(&"./rel.md"));
}

#[test]
fn split_anchor_handles_both_forms() {
    assert_eq!(split_anchor("foo.md"), ("foo.md", None));
    assert_eq!(split_anchor("foo.md#bar"), ("foo.md", Some("bar")));
    assert_eq!(split_anchor("#bar"), ("", Some("bar")));
}

#[test]
fn resolve_collapses_parent_segments() {
    let source = Path::new("docs/guide/wcag-compliance.md");
    let resolved = resolve(source, "../../README.md");
    assert_eq!(resolved.to_string_lossy(), "README.md");
}
