// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Content staging — residual upstream-gap workarounds for the
//! `staticdatagen` → `staticweaver` → `metadata-gen` pipeline.
//!
//! ## v0.0.46 residual scope
//!
//! Most of v0.0.45's shim layer was retired in v0.0.46 by upstream
//! fixes:
//!
//! - `staticdatagen 0.0.10` (closes upstream `#67`, `#68`, `#69`,
//!   `#70`, `#71`) handles missing `layout:` keys, absent
//!   `main.js`/`sw.js`, absent tags-page templates, nested-locale
//!   walk (`_posts/<lang>/`), and success-log ordering natively.
//! - `staticweaver 0.0.3` (closes upstream `#28`) added the
//!   `Engine::with_lax_undefined(true)` opt-in.
//! - `staticweaver 0.0.3` also made `escape_html_into` idempotent
//!   (closes [ssg #589](https://github.com/sebastienrousseau/static-site-generator/issues/589)).
//! - `rss-gen 0.0.6` (closes upstream `#34`) prefixed validation
//!   errors with `channel.` / `item.` context and accepts
//!   relative URLs at item level.
//! - `metadata-gen 0.0.5` (closes upstream `#20`) collapses
//!   multi-line double-quoted YAML scalars internally.
//!
//! What remains here are **two** narrow gaps the upstreams haven't
//! closed yet:
//!
//! 1. **Template-default injection.** `staticdatagen 0.0.10` doesn't
//!    yet opt the staticweaver Engine into `lax_undefined` (tracked at
//!    [staticdatagen #99](https://github.com/sebastienrousseau/staticdatagen/issues/99)),
//!    so unresolved `{{ var }}` tags still abort the build.
//!    [`collect_template_vars`] + [`stage_content_with_template_defaults`]
//!    pre-fill an empty `var: ""` for every key the templates reference
//!    but the content omits.
//!
//! 2. **Multi-line quoted-scalar collapse.** `staticdatagen 0.0.10`
//!    pins `metadata-gen = "0.0.4"` (the pre-#20 release; tracked at
//!    [staticdatagen #100](https://github.com/sebastienrousseau/staticdatagen/issues/100)).
//!    [`copy_tree`] applies the same collapse pass that's now upstream
//!    in `metadata-gen 0.0.5`, so the user's content sees consistent
//!    behaviour regardless of which `metadata-gen` is transitively
//!    resolved.
//!
//! Both shims auto-retire when the corresponding staticdatagen follow-up
//! releases — the residual module shrinks to nothing.
//!
//! ## Why staging instead of editing in-place?
//!
//! The user's checkout is sacred:
//!
//! - the build runs from a CI checkout the user expects to be read-only;
//! - reruns of the build would re-inject defaults, doubling lines.
//!
//! Instead we operate on a fresh directory under
//! [`std::env::temp_dir()`] (FNV-1a-keyed by `build_dir` + pid) that's
//! recreated on every build.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Stages a copy of `content_dir` and injects empty defaults for every
/// `{{ var }}` reference the templates make.
///
/// Works around the staticweaver "Unresolved template tag" crash that
/// fires when user content omits a key their template references —
/// `staticdatagen 0.0.10` still calls `Engine::new(...)` without
/// `.with_lax_undefined(true)`. Tracked at [staticdatagen #99].
///
/// `template_var_keys` is the result of [`collect_template_vars`] run
/// over the user's template directory.
///
/// [staticdatagen #99]: https://github.com/sebastienrousseau/staticdatagen/issues/99
///
/// # Errors
///
/// Returns [`io::Error`] when the staging directory cannot be created
/// or a source file cannot be read or written.
pub fn stage_content_with_template_defaults(
    content_dir: &Path,
    build_dir: &Path,
    template_var_keys: &[String],
) -> Result<PathBuf, io::Error> {
    let staging_dir = staging_root_for("content", build_dir);

    // Recreate the staging directory each run so a previous build's
    // layout injection doesn't leak into this build's inputs.
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    fs::create_dir_all(&staging_dir)?;

    copy_tree(content_dir, &staging_dir)?;

    // staticdatagen 0.0.10 closes upstream #69 — the tags-page generator
    // is now a no-op when no `tags.md` / `tags/index.md` template is
    // present. The v0.0.45 `ensure_tags_stub` shim was retired in this
    // release.

    if !template_var_keys.is_empty() {
        inject_template_defaults_recursive(&staging_dir, template_var_keys)?;
    }

    Ok(staging_dir)
}

/// Picks a staging-directory location *outside* `build_dir` so the
/// staged tree never gets swept into staticdatagen's output.
///
/// Key derivation includes:
/// - the OS temp dir (`std::env::temp_dir()`)
/// - the process id (disambiguates concurrent `ssg` invocations)
/// - a stable FNV-1a hash of `build_dir`'s absolute path
///   (disambiguates concurrent in-process callers — load-bearing for
///   the parallel test runner, where every test shares the same pid)
/// - a per-purpose suffix (`content` / `templates`)
///
/// The hash is FNV-1a 64-bit rather than `DefaultHasher` to keep the
/// path deterministic across runs (good for debugging) while staying
/// well-distributed across the `usize` space.
fn staging_root_for(suffix: &str, build_dir: &Path) -> PathBuf {
    // FNV-1a 64-bit over the build_dir bytes — deterministic and
    // adequate for path disambiguation.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in build_dir.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    std::env::temp_dir().join(format!(
        "ssg-staging-{}-{hash:016x}-{suffix}",
        std::process::id()
    ))
}

/// Recursively mirrors `src` into `dst`, applying
/// [`collapse_multiline_quoted_scalars`] to `.md` files and copying
/// everything else verbatim. Per-file work runs in parallel via Rayon
/// — `read + transform + write` is the hot path that determined
/// whether the staging shim added meaningful build-time overhead.
/// With ~100 pages the parallel pass keeps the staging cost under
/// ~50 ms on a 4-core runner, well inside the perf-budget gate.
fn copy_tree(src: &Path, dst: &Path) -> Result<(), io::Error> {
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    // Collect entries up front so the directory-creation step is
    // serial (cheap) and the per-file work is parallel.
    let mut files: Vec<(PathBuf, PathBuf, bool)> = Vec::new();
    let mut dirs: Vec<(PathBuf, PathBuf)> = Vec::new();
    collect_entries(src, dst, &mut files, &mut dirs)?;

    // Create every directory serially (cheap; preserves order so
    // children can be written without races).
    for (_src_dir, dst_dir) in &dirs {
        fs::create_dir_all(dst_dir)?;
    }

    let errors: Vec<io::Error> = files
        .into_par_iter()
        .filter_map(|(src_path, dst_path, is_md)| {
            let r = if is_md {
                // staticdatagen 0.0.10 closes upstream #67 — missing
                // `layout:` keys default to "page" inside the compiler
                // itself. Per-file work is now ONLY the multi-line
                // quoted-scalar collapse (still needed until
                // staticdatagen bumps `metadata-gen` to 0.0.5;
                // tracked: staticdatagen#100).
                fs::read_to_string(&src_path).and_then(|body| {
                    let staged = collapse_multiline_quoted_scalars(&body);
                    fs::write(&dst_path, staged)
                })
            } else {
                fs::copy(&src_path, &dst_path).map(|_| ())
            };
            r.err()
        })
        .collect();

    if let Some(e) = errors.into_iter().next() {
        return Err(e);
    }
    Ok(())
}

/// Walks `src` and partitions entries into directories (to create
/// before parallel writes) and files (with their destination path
/// and a markdown flag).
fn collect_entries(
    src: &Path,
    dst: &Path,
    files: &mut Vec<(PathBuf, PathBuf, bool)>,
    dirs: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            dirs.push((src_path.clone(), dst_path.clone()));
            collect_entries(&src_path, &dst_path, files, dirs)?;
        } else if file_type.is_file() {
            let is_md = is_markdown(&src_path);
            files.push((src_path, dst_path, is_md));
        }
        // Symlinks and other special files are skipped — staticdatagen
        // wouldn't follow them safely anyway.
    }
    Ok(())
}

/// Walks `template_dir` recursively and returns the sorted, deduped
/// set of every `{{ <var> }}` reference found in the template files.
///
/// Filtered references (`{{ var | filter }}`), dotted paths (`{{ a.b }}`),
/// helpers (`{{#each ...}}`), and the `{{!...}}` raw-emit form are all
/// skipped — only bare top-level keys end up in the result, because
/// those are the ones staticweaver actually looks up against the
/// frontmatter `metadata` `HashMap`.
///
/// Returns an empty Vec if `template_dir` doesn't exist.
///
/// # Errors
///
/// Returns [`io::Error`] only for unexpected failures reading the
/// template tree; a missing directory is treated as "no templates",
/// not as an error.
///
/// # Examples
///
/// ```rust
/// use ssg::core_group::content_stager::collect_template_vars;
/// use std::fs;
///
/// let tmp = tempfile::tempdir().unwrap();
/// let t = tmp.path().join("t");
/// fs::create_dir_all(&t).unwrap();
/// fs::write(t.join("page.html"), "<title>{{ title }}</title>{{ author }}").unwrap();
///
/// let vars = collect_template_vars(&t).unwrap();
/// assert!(vars.contains(&"title".to_string()));
/// assert!(vars.contains(&"author".to_string()));
/// ```
pub fn collect_template_vars(
    template_dir: &Path,
) -> Result<Vec<String>, io::Error> {
    let mut out = std::collections::BTreeSet::new();
    if !template_dir.exists() {
        return Ok(Vec::new());
    }
    walk_collect_vars(template_dir, &mut out)?;
    Ok(out.into_iter().collect())
}

fn walk_collect_vars(
    dir: &Path,
    out: &mut std::collections::BTreeSet<String>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            walk_collect_vars(&p, out)?;
        } else if ft.is_file() {
            // Only scan templating-eligible files; ignore .js, .css,
            // etc. that happen to live under template_dir.
            let is_template = matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("html" | "htm" | "xml" | "txt" | "rss")
            );
            if is_template {
                if let Ok(body) = fs::read_to_string(&p) {
                    extract_simple_vars(&body, out);
                }
            }
        }
    }
    Ok(())
}

/// Extracts bare `{{ key }}` references from `body` into `out`.
/// Skips refs that include filters (`|`), dotted paths (`.`), or
/// staticweaver helpers (`#`, `/`, `!`, `>`).
fn extract_simple_vars(
    body: &str,
    out: &mut std::collections::BTreeSet<String>,
) {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Walk to the matching `}}`.
            if let Some(end) = find_closing_braces(&body[i + 2..]) {
                let inner = body[i + 2..i + 2 + end].trim();
                if let Some(name) = simple_var_name(inner) {
                    let _ = out.insert(name.to_string());
                }
                i += 2 + end + 2;
                continue;
            }
        }
        i += 1;
    }
}

fn find_closing_braces(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut j = 0;
    while j + 1 < bytes.len() {
        if bytes[j] == b'}' && bytes[j + 1] == b'}' {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Returns Some(name) when `inner` is a simple `<key>` lookup —
/// no filter (`|`), no dotted path (`.`), no helper prefix
/// (`#`, `/`, `!`, `>`), no whitespace inside the name.
fn simple_var_name(inner: &str) -> Option<&str> {
    let s = inner.trim();
    if s.is_empty() {
        return None;
    }
    let first = s.as_bytes()[0];
    if matches!(first, b'#' | b'/' | b'!' | b'>') {
        return None;
    }
    if s.contains('|') || s.contains('.') {
        return None;
    }
    if s.bytes().any(|b| b.is_ascii_whitespace()) {
        return None;
    }
    Some(s)
}

/// Walks every `.md` file under `dir` and injects empty defaults for
/// every key in `keys` not already present in that file's frontmatter.
/// Per-file work is parallelised via Rayon so the staging cost on a
/// 100-page corpus stays inside the perf-budget gate.
fn inject_template_defaults_recursive(
    dir: &Path,
    keys: &[String],
) -> Result<(), io::Error> {
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    let mut md_files: Vec<PathBuf> = Vec::new();
    collect_markdown_files(dir, &mut md_files)?;

    let errors: Vec<io::Error> = md_files
        .into_par_iter()
        .filter_map(|p| {
            let body = match fs::read_to_string(&p) {
                Ok(b) => b,
                Err(e) => return Some(e),
            };
            let staged = inject_missing_keys(&body, keys);
            if staged == body {
                return None;
            }
            fs::write(&p, staged).err()
        })
        .collect();
    if let Some(e) = errors.into_iter().next() {
        return Err(e);
    }
    Ok(())
}

fn collect_markdown_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_markdown_files(&p, out)?;
        } else if ft.is_file() && is_markdown(&p) {
            out.push(p);
        }
    }
    Ok(())
}

/// Injects empty `key: ""` entries into the frontmatter block for any
/// `key` not already present. No-op for files without a frontmatter
/// block.
#[must_use]
pub fn inject_missing_keys(body: &str, keys: &[String]) -> String {
    let trimmed = body.trim_start_matches('\u{FEFF}');
    let Some((_lead, after_open)) = find_opening_fence(trimmed) else {
        return body.to_string();
    };
    let Some(close_rel) = find_closing_fence(after_open) else {
        return body.to_string();
    };
    let block = &after_open[..close_rel];
    let after_block = &after_open[close_rel..];

    let missing: Vec<&String> = keys
        .iter()
        .filter(|k| !frontmatter_has_key(block, k))
        .collect();
    if missing.is_empty() {
        return body.to_string();
    }

    let mut additions = String::with_capacity(missing.len() * 16);
    for k in missing {
        additions.push_str(&format!("{k}: \"\"\n"));
    }

    let mut out = String::with_capacity(body.len() + additions.len());
    out.push_str(&trimmed[..trimmed.len() - after_open.len()]);
    out.push_str(&additions);
    out.push_str(block);
    out.push_str(after_block);
    if body.starts_with('\u{FEFF}') {
        return format!("\u{FEFF}{out}");
    }
    out
}

/// Returns `true` if the frontmatter block declares `key` (any
/// quoting style, comments skipped).
fn frontmatter_has_key(block: &str, key: &str) -> bool {
    for raw in block.lines() {
        let line = raw.trim_start();
        if line.starts_with('#') {
            continue;
        }
        let prefixes = [
            format!("{key}:"),
            format!("{key} :"),
            format!("\"{key}\":"),
            format!("'{key}':"),
        ];
        if prefixes.iter().any(|p| line.starts_with(p)) {
            return true;
        }
    }
    false
}

fn is_markdown(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|s| s.to_str()),
        Some("md" | "markdown")
    )
}

/// Collapses YAML-spec-compliant multi-line quoted scalars onto a
/// single line so noyalib's stricter parser inside `metadata-gen`
/// can read them.
///
/// Pattern handled (the common one in the wild):
///
/// ```yaml
/// key: "
/// value-on-next-line"
/// ```
///
/// becomes
///
/// ```yaml
/// key: "value-on-next-line"
/// ```
///
/// The collapse joins lines until the matching closing quote is seen
/// on a subsequent line, replacing intervening newlines with a single
/// space (the YAML semantic for folded line breaks inside double-
/// quoted scalars).
fn collapse_multiline_quoted_scalars(block: &str) -> String {
    let mut out = String::with_capacity(block.len());
    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        // Detect `key: "` (opening quote, nothing after).
        if let Some(eq_pos) = line.find(": \"") {
            let after_quote = &line[eq_pos + 3..];
            // The quote sits at the very end of the line (only
            // whitespace allowed after) AND no closing `"` on this
            // line — multi-line case.
            if after_quote.trim().is_empty() {
                // Walk forward joining lines until we find the
                // closing `"`.
                let mut joined = String::from(&line[..eq_pos + 3]);
                let mut closed = false;
                i += 1;
                while i < lines.len() {
                    let next = lines[i];
                    if let Some(close) = next.find('"') {
                        joined.push_str(next[..close].trim_start());
                        joined.push_str(&next[close..]);
                        out.push_str(&joined);
                        out.push('\n');
                        i += 1;
                        closed = true;
                        break;
                    }
                    joined.push_str(next.trim_start());
                    joined.push(' ');
                    i += 1;
                }
                // Pathological case — no closing quote in the file.
                // Emit what we've accumulated so the downstream
                // extractor sees the same broken content rather than
                // silently swallowing it.
                if !closed {
                    out.push_str(joined.trim_end());
                    out.push('\n');
                }
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
        i += 1;
    }
    out
}

/// Returns `(prefix, after_open_fence)` where `prefix` is the bytes
/// before the first `---\n` (i.e. leading blank lines) and
/// `after_open_fence` starts at the first character past the fence.
fn find_opening_fence(s: &str) -> Option<(&str, &str)> {
    // Walk lines until we see a non-empty one. If it's exactly `---`
    // (with optional CR), the fence opens. Anything else means no
    // frontmatter.
    let mut byte_pos = 0;
    for line in s.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n').trim_end_matches('\r');
        if bare.trim().is_empty() {
            byte_pos += line.len();
            continue;
        }
        if bare == "---" {
            let lead = &s[..byte_pos];
            let after = &s[byte_pos + line.len()..];
            return Some((lead, after));
        }
        return None;
    }
    None
}

/// Returns the byte offset within `after_open` at which the closing
/// `---` fence begins (the offset is the start of the closing fence
/// line, not past it).
fn find_closing_fence(after_open: &str) -> Option<usize> {
    let mut byte_pos = 0;
    for line in after_open.split_inclusive('\n') {
        let bare = line.trim_end_matches('\n').trim_end_matches('\r');
        if bare.trim() == "---" {
            return Some(byte_pos);
        }
        byte_pos += line.len();
    }
    None
}

// ---------------------------------------------------------------------
// Template staging
// ---------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn inject_missing_keys_no_frontmatter_passthrough() {
        // Covers `let Some(...) = find_opening_fence(...) else { return body.to_string(); }`.
        let body = "no frontmatter here\nplain markdown";
        let out = inject_missing_keys(body, &["x".to_string()]);
        assert_eq!(out, body);
    }

    #[test]
    fn inject_missing_keys_unterminated_frontmatter_passthrough() {
        // Covers `let Some(...) = find_closing_fence(...) else { return body.to_string(); }`.
        let body = "---\ntitle: T\n# never closes";
        let out = inject_missing_keys(body, &["x".to_string()]);
        assert_eq!(out, body);
    }

    #[test]
    fn inject_missing_keys_with_bom_preserves_bom() {
        // Covers the `if body.starts_with('\u{FEFF}') { format!("\u{FEFF}{out}") }`
        // branch inside `inject_missing_keys` — distinct from the
        // existing `bom_is_preserved` test which only exercises the
        // sibling layout-injection function.
        let body = "\u{FEFF}---\ntitle: T\n---\nbody";
        let out = inject_missing_keys(body, &["author".to_string()]);
        assert!(out.starts_with('\u{FEFF}'));
        assert!(out.contains("author: \"\""));
    }

    #[test]
    fn find_closing_braces_returns_none_when_unterminated() {
        // Covers `find_closing_braces` walking past the end without
        // seeing `}}` — happens for malformed templates.
        let mut out = std::collections::BTreeSet::new();
        extract_simple_vars("{{ never_closes", &mut out);
        // No vars extracted because the closing braces weren't found.
        assert!(out.is_empty());
    }

    #[test]
    fn collapse_multiline_quoted_scalar_collapses_two_line() {
        let input = "url: \"\nhttps://example.com/x\"\n";
        let out = collapse_multiline_quoted_scalars(input);
        assert!(out.contains("url: \"https://example.com/x\""));
        assert!(!out.contains("\nhttps"));
    }

    #[test]
    fn collapse_multiline_quoted_scalar_preserves_single_line() {
        let input = "title: \"On one line\"\nauthor: \"Jane\"\n";
        let out = collapse_multiline_quoted_scalars(input);
        assert_eq!(out, input);
    }

    #[test]
    fn collapse_multiline_quoted_scalar_collapses_three_line() {
        let input = "blurb: \"\nline one\nline two\"\n";
        let out = collapse_multiline_quoted_scalars(input);
        assert!(out.contains("blurb: \"line one line two\""));
    }

    #[test]
    fn collapse_handles_unterminated_quote_gracefully() {
        // Pathological case — never close. We should not panic; we
        // just emit what we have so the downstream extractor surfaces
        // a clean error.
        let input = "x: \"\nstill open\n";
        let out = collapse_multiline_quoted_scalars(input);
        assert!(out.contains("still open"));
    }

    #[test]
    fn user_real_world_twitter_url_multiline_collapses() {
        // Exact shape from
        // _posts/2026-04-11-quantum-thresholds-are-moving-again.md
        // — the file that exposed the noyalib brittleness.
        let input = "twitter_url: \"\nhttps://sebastienrousseau.com/2026-04-11-quantum-thresholds-are-moving-again\"\n";
        let out = collapse_multiline_quoted_scalars(input);
        assert!(out.contains("twitter_url: \"https://"));
        assert_eq!(out.lines().count(), 1);
    }

}
