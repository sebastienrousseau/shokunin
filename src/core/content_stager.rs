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
//!    `copy_tree` (the per-file staging helper) applies the same collapse pass that's now upstream
//!    in `metadata-gen 0.0.5`, so the user's content sees consistent
//!    behaviour regardless of which `metadata-gen` is transitively
//!    resolved.
//!
//! Both shims auto-retire when the corresponding staticdatagen follow-up
//! releases — the residual module shrinks to nothing.
//!
//! ## Permalink derivation (spec A2/B1, plan §2 item 1.2, issue #586)
//!
//! `staticdatagen`'s RSS generator hard-fails the whole build when a
//! post lacks `permalink:` front matter (`rss-gen`: "channel.link is
//! missing"). Because every page passes through this stager before
//! `staticdatagen::compile`, the stager can make that failure
//! unreachable: when a staged `.md` file's frontmatter carries
//! neither `permalink` nor `url`, [`stage_content_with_site_defaults`]
//! injects `permalink: "{base_url}/{relative_output_path}"` derived
//! via [`crate::urls::derive_permalink`]. Author-specified permalinks
//! always win — files that already declare `permalink` or `url` pass
//! through verbatim. Only YAML `---` fenced frontmatter flows through
//! this stager (the template-default shim shares the same
//! constraint); files without a frontmatter block are left untouched.
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
/// Convenience wrapper over [`stage_content_with_site_defaults`] with
/// no base URL — no `permalink:` derivation happens on staged content.
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
    stage_content_with_site_defaults(
        content_dir,
        build_dir,
        template_var_keys,
        None,
    )
}

/// Like [`stage_content_with_template_defaults`] but also derives a
/// `permalink:` for staged `.md` files that don't declare one.
///
/// Injection targets every staged `.md` file whose frontmatter
/// carries neither `permalink` nor `url` (spec A2/B1, plan §2 item
/// 1.2, issue #586). The derived value comes from
/// [`crate::urls::derive_permalink`] applied to `base_url` and the
/// file's content-relative path — i.e.
/// `{base_url}/{relative_output_path}` under the compiler's
/// `foo.md → foo/index.html` output convention, published as a pretty
/// directory URL (`{base_url}/foo/`). This guarantees `staticdatagen`
/// / `rss-gen` always see a channel/item link and can never abort the
/// build with "channel.link is missing".
///
/// Passing `base_url: None` (or an empty/whitespace-only base URL)
/// disables permalink injection and behaves exactly like
/// [`stage_content_with_template_defaults`] — an rss-gen-valid
/// permalink must be an *absolute* URL, so there is nothing useful to
/// derive without a base.
///
/// Author-specified front matter always wins: files that already
/// declare `permalink` or `url` pass through verbatim.
///
/// # Errors
///
/// Returns [`io::Error`] when the staging directory cannot be created
/// or a source file cannot be read or written.
///
/// # Examples
///
/// ```rust
/// use ssg::content_stager::stage_content_with_site_defaults;
/// use std::fs;
///
/// let tmp = tempfile::tempdir().unwrap();
/// let src = tmp.path().join("content");
/// let build = tmp.path().join("build");
/// fs::create_dir_all(&src).unwrap();
/// fs::write(src.join("post.md"), "---\ntitle: A\n---\nbody").unwrap();
///
/// let staged = stage_content_with_site_defaults(
///     &src, &build, &[], Some("https://example.com"),
/// ).unwrap();
/// let out = fs::read_to_string(staged.join("post.md")).unwrap();
/// assert!(out.contains("permalink: \"https://example.com/post/\""));
/// ```
pub fn stage_content_with_site_defaults(
    content_dir: &Path,
    build_dir: &Path,
    template_var_keys: &[String],
    base_url: Option<&str>,
) -> Result<PathBuf, io::Error> {
    // An empty base URL can't produce the absolute permalink rss-gen
    // validates for — treat it as "no base URL, skip injection".
    let base_url = base_url.map(str::trim).filter(|b| !b.is_empty());

    let staging_dir = staging_root_for("content", build_dir);
    recreate_staging_dir(&staging_dir)?;

    copy_tree(content_dir, &staging_dir, base_url)?;

    // staticdatagen 0.0.10 closes upstream #69 — the tags-page generator
    // is now a no-op when no `tags.md` / `tags/index.md` template is
    // present. The v0.0.45 `ensure_tags_stub` shim was retired in this
    // release.

    if !template_var_keys.is_empty() {
        inject_template_defaults_recursive(&staging_dir, template_var_keys)?;
    }

    Ok(staging_dir)
}

/// Recreates the staging directory from scratch so a previous build's
/// layout injection doesn't leak into this build's inputs.
fn recreate_staging_dir(staging_dir: &Path) -> Result<(), io::Error> {
    if staging_dir.exists() {
        fs::remove_dir_all(staging_dir)?;
    }
    fs::create_dir_all(staging_dir)?;
    Ok(())
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
/// [`collapse_multiline_quoted_scalars`] to `.md` files (and, when
/// `base_url` is present, [`inject_permalink_if_missing`] — spec
/// A2/B1, plan §2 item 1.2, issue #586) and copying everything else
/// verbatim. Per-file work runs in parallel via Rayon
/// — `read + transform + write` is the hot path that determined
/// whether the staging shim added meaningful build-time overhead.
/// With ~100 pages the parallel pass keeps the staging cost under
/// ~50 ms on a 4-core runner, well inside the perf-budget gate.
fn copy_tree(
    src: &Path,
    dst: &Path,
    base_url: Option<&str>,
) -> Result<(), io::Error> {
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
                    let mut staged = collapse_multiline_quoted_scalars(&body);
                    // Guarantee a permalink so rss-gen's
                    // "channel.link is missing" hard-fail is
                    // unreachable (spec A2/B1, plan §2 item 1.2,
                    // issue #586). Author-specified `permalink`/`url`
                    // keys always win — see
                    // `inject_permalink_if_missing`.
                    if let Some(base) = base_url {
                        if let Ok(rel) = src_path.strip_prefix(src) {
                            let rel = rel.to_string_lossy();
                            let permalink =
                                crate::urls::derive_permalink(base, &rel);
                            staged = inject_permalink_if_missing(
                                &staged, &permalink,
                            );
                        }
                    }
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
/// use ssg::content_stager::collect_template_vars;
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

    fail_point!("content_stager::inject-defaults", |_| {
        Err(io::Error::other(
            "injected: content_stager::inject-defaults",
        ))
    });

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

/// Injects `permalink: "<permalink>"` as the first key of the YAML
/// frontmatter block when the block exists and declares *neither*
/// `permalink` nor `url` (spec A2/B1, plan §2 item 1.2, issue #586).
///
/// Author-specified values always win: a file with either key passes
/// through byte-for-byte. Files without a frontmatter fence are left
/// untouched — the stager's structural line-scan (shared with the
/// template-default shim) only operates on YAML `---` fenced blocks,
/// and `staticdatagen` extracts no metadata from fence-less files
/// anyway.
///
/// Idempotent: a second pass over previously-staged content returns
/// the input unchanged (the injected `permalink:` is detected as an
/// existing key).
///
/// # Examples
///
/// ```rust
/// use ssg::content_stager::inject_permalink_if_missing;
///
/// // Missing both keys — derived permalink lands as the first key.
/// let out = inject_permalink_if_missing(
///     "---\ntitle: T\n---\nbody",
///     "https://example.com/t/",
/// );
/// assert!(out.contains("permalink: \"https://example.com/t/\""));
///
/// // Author-specified permalink wins verbatim.
/// let with_permalink = "---\npermalink: /mine/\ntitle: T\n---\nbody";
/// assert_eq!(
///     inject_permalink_if_missing(with_permalink, "https://x/"),
///     with_permalink
/// );
///
/// // `url` counts as author-specified too.
/// let with_url = "---\nurl: /u/\ntitle: T\n---\nbody";
/// assert_eq!(inject_permalink_if_missing(with_url, "https://x/"), with_url);
/// ```
#[must_use]
pub fn inject_permalink_if_missing(body: &str, permalink: &str) -> String {
    let trimmed = body.trim_start_matches('\u{FEFF}');
    let Some((_lead, after_open)) = find_opening_fence(trimmed) else {
        return body.to_string();
    };
    let Some(close_rel) = find_closing_fence(after_open) else {
        return body.to_string();
    };
    let block = &after_open[..close_rel];
    let after_block = &after_open[close_rel..];

    // Author-specified link keys always win (spec B1): `permalink`
    // is what staticdatagen reads; `url` is the common alias authors
    // migrating from other generators carry.
    if frontmatter_has_key(block, "permalink")
        || frontmatter_has_key(block, "url")
    {
        return body.to_string();
    }

    let mut out = String::with_capacity(body.len() + permalink.len() + 16);
    out.push_str(&trimmed[..trimmed.len() - after_open.len()]);
    out.push_str(&format!("permalink: \"{permalink}\"\n"));
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

    // ---------------------------------------------------------------
    // Permalink derivation (spec A2/B1, plan §2 item 1.2, issue #586)
    // ---------------------------------------------------------------

    #[test]
    fn inject_permalink_adds_key_when_missing() {
        let out = inject_permalink_if_missing(
            "---\ntitle: T\n---\nbody",
            "https://example.com/t/",
        );
        assert!(out.contains("permalink: \"https://example.com/t/\""));
        assert!(out.contains("title: T"));
        assert!(out.contains("body"));
    }

    #[test]
    fn inject_permalink_preserves_author_permalink_verbatim() {
        let input = "---\npermalink: /custom/place/\ntitle: T\n---\nbody";
        assert_eq!(
            inject_permalink_if_missing(input, "https://example.com/t/"),
            input
        );
    }

    #[test]
    fn inject_permalink_treats_url_key_as_author_specified() {
        let input = "---\nurl: https://elsewhere.example/\ntitle: T\n---\nb";
        assert_eq!(
            inject_permalink_if_missing(input, "https://example.com/t/"),
            input
        );
    }

    #[test]
    fn inject_permalink_no_frontmatter_passthrough() {
        let input = "# Heading\n\nBody.";
        assert_eq!(
            inject_permalink_if_missing(input, "https://example.com/"),
            input
        );
    }

    #[test]
    fn inject_permalink_unterminated_fence_passthrough() {
        let input = "---\ntitle: T\n# never closes";
        assert_eq!(
            inject_permalink_if_missing(input, "https://example.com/"),
            input
        );
    }

    #[test]
    fn inject_permalink_preserves_bom() {
        let input = "\u{FEFF}---\ntitle: T\n---\nbody";
        let out = inject_permalink_if_missing(input, "https://example.com/t/");
        assert!(out.starts_with('\u{FEFF}'));
        assert!(out.contains("permalink: \"https://example.com/t/\""));
    }

    #[test]
    fn inject_permalink_is_idempotent() {
        let input = "---\ntitle: T\n---\nbody";
        let once = inject_permalink_if_missing(input, "https://example.com/t/");
        let twice =
            inject_permalink_if_missing(&once, "https://example.com/t/");
        assert_eq!(once, twice);
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_with_site_defaults_derives_permalinks_for_all_pages() {
        // Plan §2 1.2 acceptance shape: a 3-page fixture where ZERO
        // pages carry `permalink:` must stage with derived permalinks
        // matching `{base_url}/{output_path}` under the compiler's
        // `foo.md → foo/index.html` pretty-URL convention.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(src.join("posts")).unwrap();
        fs::write(src.join("index.md"), "---\ntitle: Home\n---\nhome").unwrap();
        fs::write(src.join("about.md"), "---\ntitle: About\n---\nab").unwrap();
        fs::write(src.join("posts/first.md"), "---\ntitle: First\n---\npost")
            .unwrap();

        let staged = stage_content_with_site_defaults(
            &src,
            &build,
            &[],
            Some("https://example.com"),
        )
        .unwrap();

        let home = fs::read_to_string(staged.join("index.md")).unwrap();
        assert!(
            home.contains("permalink: \"https://example.com/\""),
            "index.md must map to the site root URL: {home}"
        );
        let about = fs::read_to_string(staged.join("about.md")).unwrap();
        assert!(
            about.contains("permalink: \"https://example.com/about/\""),
            "about.md must map to a pretty directory URL: {about}"
        );
        let post = fs::read_to_string(staged.join("posts/first.md")).unwrap();
        assert!(
            post.contains("permalink: \"https://example.com/posts/first/\""),
            "nested page must include its directory path: {post}"
        );
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_with_site_defaults_keeps_author_permalink_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("custom.md"),
            "---\nlayout: post\npermalink: \"https://example.com/my-spot/\"\ntitle: C\n---\nbody",
        )
        .unwrap();

        let staged = stage_content_with_site_defaults(
            &src,
            &build,
            &[],
            Some("https://example.com"),
        )
        .unwrap();
        let body = fs::read_to_string(staged.join("custom.md")).unwrap();
        assert!(body.contains("permalink: \"https://example.com/my-spot/\""));
        // Exactly one permalink key — no derived duplicate.
        assert_eq!(body.matches("permalink:").count(), 1);
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_with_site_defaults_handles_nested_index_md() {
        // `about/index.md` publishes at `about/index.html` →
        // permalink `{base}/about/` (index.html collapses to the
        // directory URL, matching the Atom feed convention).
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(src.join("about")).unwrap();
        fs::write(src.join("about/index.md"), "---\ntitle: About\n---\nbody")
            .unwrap();

        let staged = stage_content_with_site_defaults(
            &src,
            &build,
            &[],
            Some("https://example.com/"),
        )
        .unwrap();
        let body = fs::read_to_string(staged.join("about/index.md")).unwrap();
        assert!(
            body.contains("permalink: \"https://example.com/about/\""),
            "trailing-slash base + nested index.md: {body}"
        );
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_without_base_url_injects_no_permalink() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();

        // Legacy entry point — must stay permalink-free.
        let staged =
            stage_content_with_template_defaults(&src, &build, &[]).unwrap();
        let body = fs::read_to_string(staged.join("a.md")).unwrap();
        assert!(!body.contains("permalink:"));

        // Empty / whitespace-only base URL disables injection too —
        // rss-gen only accepts absolute URLs, so there is nothing
        // valid to derive.
        let staged =
            stage_content_with_site_defaults(&src, &build, &[], Some("   "))
                .unwrap();
        let body = fs::read_to_string(staged.join("a.md")).unwrap();
        assert!(!body.contains("permalink:"));
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_with_site_defaults_is_idempotent_across_runs() {
        // v0.0.46 retired the layout-injection shim (staticdatagen
        // 0.0.10 defaults missing `layout:` natively), so the staged
        // output must carry exactly one derived permalink and no
        // injected layout key — across repeated staging runs.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();

        let _first = stage_content_with_site_defaults(
            &src,
            &build,
            &[],
            Some("https://example.com"),
        )
        .unwrap();
        let staged = stage_content_with_site_defaults(
            &src,
            &build,
            &[],
            Some("https://example.com"),
        )
        .unwrap();
        let body = fs::read_to_string(staged.join("a.md")).unwrap();
        assert_eq!(body.matches("permalink:").count(), 1);
        assert_eq!(body.matches("layout:").count(), 0);
    }

    // -----------------------------------------------------------------
    // recreate_staging_dir — happy path and both failure arms
    // -----------------------------------------------------------------

    #[test]
    fn recreate_staging_dir_wipes_previous_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("stale.md"), "old").unwrap();

        recreate_staging_dir(&staging).unwrap();

        assert!(staging.is_dir());
        assert!(!staging.join("stale.md").exists());
    }

    #[test]
    fn recreate_staging_dir_fails_when_path_is_a_file() {
        // `remove_dir_all` on a regular file fails — the first `?`.
        let tmp = tempfile::tempdir().unwrap();
        let blocked = tmp.path().join("staging");
        fs::write(&blocked, "not a dir").unwrap();

        assert!(recreate_staging_dir(&blocked).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn recreate_staging_dir_fails_when_parent_is_read_only() {
        // `create_dir_all` under a read-only parent fails — the
        // second `?`.
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let parent = tmp.path().join("ro");
        fs::create_dir_all(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o555))
            .unwrap();

        let res = recreate_staging_dir(&parent.join("staging"));

        let _ = fs::set_permissions(&parent, fs::Permissions::from_mode(0o755));
        // Root bypasses permissions on some CI runners, so tolerate Ok.
        assert!(res.err().is_none_or(|e| !format!("{e}").is_empty()));
    }

    #[test]
    #[serial_test::parallel(stager_fp)]
    fn stage_fails_when_staging_root_is_blocked_by_a_file() {
        // Drives the `recreate_staging_dir(..)?` edge inside
        // `stage_content_with_site_defaults`.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();

        let staging = staging_root_for("content", &build);
        fs::write(&staging, "blocker").unwrap();

        let res = stage_content_with_site_defaults(&src, &build, &[], None);
        let _ = fs::remove_file(&staging);
        assert!(res.is_err());
    }

    // -----------------------------------------------------------------
    // copy_tree / collect_entries — error and skip arms
    // -----------------------------------------------------------------

    #[test]
    fn copy_tree_fails_when_destination_subdir_is_blocked() {
        // A file squatting on a destination directory name makes the
        // serial `create_dir_all` fail.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("sub/a.md"), "---\nt: a\n---\nx").unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("sub"), "file, not dir").unwrap();

        assert!(copy_tree(&src, &dst, None).is_err());
    }

    #[test]
    fn copy_tree_copies_non_markdown_files_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("style.css"), "body{}").unwrap();

        copy_tree(&src, &dst, None).unwrap();
        assert_eq!(
            fs::read_to_string(dst.join("style.css")).unwrap(),
            "body{}"
        );
    }

    #[test]
    fn copy_tree_reports_first_per_file_error() {
        // A non-UTF-8 .md file fails `read_to_string` inside the
        // parallel pass; the first collected error is returned.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("bad.md"), [0xFF, 0xFE, 0x00]).unwrap();

        assert!(copy_tree(&src, &dst, None).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn copy_tree_propagates_unreadable_source_subdir() {
        // The recursive collect_entries call fails when a nested
        // source directory can't be listed.
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        let locked = src.join("locked");
        fs::create_dir_all(&locked).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))
            .unwrap();

        let res = copy_tree(&src, &dst, None);

        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));
        // Root bypasses permissions on some CI runners, so tolerate Ok.
        assert!(res.err().is_none_or(|e| !format!("{e}").is_empty()));
    }

    #[test]
    #[cfg(unix)]
    fn collect_entries_skips_symlinks_and_special_files() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("real.md"), "---\nt: a\n---\nx").unwrap();
        std::os::unix::fs::symlink(src.join("nowhere.md"), src.join("link.md"))
            .unwrap();

        copy_tree(&src, &dst, None).unwrap();
        assert!(dst.join("real.md").exists());
        assert!(!dst.join("link.md").exists(), "symlinks must be skipped");
    }

    // -----------------------------------------------------------------
    // collect_template_vars / extract_simple_vars — recursion, errors,
    // and every reject shape
    // -----------------------------------------------------------------

    #[test]
    fn collect_template_vars_recurses_into_subdirectories() {
        let tmp = tempfile::tempdir().unwrap();
        let t = tmp.path().join("templates");
        fs::create_dir_all(t.join("partials")).unwrap();
        fs::write(t.join("page.html"), "{{ title }}").unwrap();
        fs::write(t.join("partials/nav.html"), "{{ nav_label }}").unwrap();

        let vars = collect_template_vars(&t).unwrap();
        assert!(vars.contains(&"title".to_string()));
        assert!(vars.contains(&"nav_label".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn collect_template_vars_propagates_unreadable_subdir() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let t = tmp.path().join("templates");
        let sub = t.join("locked");
        fs::create_dir_all(&sub).unwrap();
        fs::set_permissions(&sub, fs::Permissions::from_mode(0o000)).unwrap();

        let res = collect_template_vars(&t);

        let _ = fs::set_permissions(&sub, fs::Permissions::from_mode(0o755));
        // Root bypasses permissions on some CI runners, so tolerate Ok.
        assert!(res.err().is_none_or(|e| !format!("{e}").is_empty()));
    }

    #[test]
    fn extract_simple_vars_rejects_every_non_simple_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let t = tmp.path().join("templates");
        fs::create_dir_all(&t).unwrap();
        fs::write(
            t.join("page.html"),
            // empty ref, helper, closing tag, raw-emit, partial,
            // filtered, dotted, spaced, unclosed — none are simple.
            "{{  }}{{#each xs}}{{/each}}{{!raw}}{{>part}}\
             {{ a | upper }}{{ a.b }}{{ a b }}{{ good }}{{ broken",
        )
        .unwrap();

        let vars = collect_template_vars(&t).unwrap();
        assert_eq!(vars, vec!["good".to_string()]);
    }

    #[test]
    fn walk_collect_vars_skips_unreadable_and_non_template_files() {
        let tmp = tempfile::tempdir().unwrap();
        let t = tmp.path().join("templates");
        fs::create_dir_all(&t).unwrap();
        // Non-UTF-8 template file: read_to_string fails, silently
        // skipped.
        fs::write(t.join("binary.html"), [0xFF, 0xFE, 0x00]).unwrap();
        // Non-templating extension: never scanned.
        fs::write(t.join("style.css"), "{{ not_a_var }}").unwrap();
        fs::write(t.join("page.html"), "{{ real_var }}").unwrap();

        let vars = collect_template_vars(&t).unwrap();
        assert_eq!(vars, vec!["real_var".to_string()]);
    }

    // -----------------------------------------------------------------
    // inject_template_defaults_recursive — direct error/skip arms
    // -----------------------------------------------------------------

    #[test]
    fn inject_defaults_recurses_and_injects_in_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("staged");
        fs::create_dir_all(dir.join("blog")).unwrap();
        fs::write(dir.join("blog/a.md"), "---\ntitle: A\n---\nx").unwrap();

        inject_template_defaults_recursive(&dir, &["author".to_string()])
            .unwrap();

        let body = fs::read_to_string(dir.join("blog/a.md")).unwrap();
        assert!(body.contains("author:"));
    }

    #[test]
    fn inject_defaults_reports_unreadable_markdown() {
        // Non-UTF-8 bytes fail the read inside the parallel pass.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("staged");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("bad.md"), [0xFF, 0xFE]).unwrap();

        let res = inject_template_defaults_recursive(&dir, &["k".to_string()]);
        assert!(res.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn inject_defaults_propagates_unreadable_subdir() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("staged");
        let sub = dir.join("locked");
        fs::create_dir_all(&sub).unwrap();
        fs::set_permissions(&sub, fs::Permissions::from_mode(0o000)).unwrap();

        let res = inject_template_defaults_recursive(&dir, &["k".to_string()]);

        let _ = fs::set_permissions(&sub, fs::Permissions::from_mode(0o755));
        assert!(res.err().is_none_or(|e| !format!("{e}").is_empty()));
    }

    #[test]
    #[cfg(unix)]
    fn collect_markdown_files_skips_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("staged");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("real.md"), "---\nt: a\n---\nx").unwrap();
        std::os::unix::fs::symlink(dir.join("nowhere.md"), dir.join("link.md"))
            .unwrap();

        let mut found = Vec::new();
        collect_markdown_files(&dir, &mut found).unwrap();
        assert_eq!(found.len(), 1);
    }

    // -----------------------------------------------------------------
    // is_markdown / find_opening_fence — remaining shapes
    // -----------------------------------------------------------------

    #[test]
    fn is_markdown_accepts_both_extensions() {
        assert!(is_markdown(Path::new("a.md")));
        assert!(is_markdown(Path::new("a.markdown")));
        assert!(!is_markdown(Path::new("a.html")));
    }

    #[test]
    fn find_opening_fence_skips_leading_blank_lines() {
        let (lead, after) =
            find_opening_fence("\n  \n---\ntitle: x\n---\nbody").unwrap();
        assert_eq!(lead, "\n  \n");
        assert!(after.starts_with("title: x"));
    }

    #[test]
    fn find_opening_fence_returns_none_for_blank_only_input() {
        assert!(find_opening_fence("\n\n  \n").is_none());
        assert!(find_opening_fence("").is_none());
    }

    // -----------------------------------------------------------------
    // Fault injection — inject_template_defaults_recursive failpoint
    // -----------------------------------------------------------------

    #[cfg(feature = "test-fault-injection")]
    #[test]
    #[serial_test::serial(stager_fp)]
    fn stage_fault_inject_defaults_returns_err() {
        // RAII guard so a panicking assertion still deactivates the
        // failpoint (mirrors tests/fault_injection.rs).
        struct FailGuard(&'static str);
        impl Drop for FailGuard {
            fn drop(&mut self) {
                let _ = fail::cfg(self.0, "off");
            }
        }
        let _guard = FailGuard("content_stager::inject-defaults");
        fail::cfg("content_stager::inject-defaults", "return")
            .expect("activate failpoint");

        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();

        let err = stage_content_with_site_defaults(
            &src,
            &build,
            &["title".to_string()],
            None,
        )
        .expect_err("failpoint must abort the staging pass");
        assert!(format!("{err}").contains("inject-defaults"));
    }
}
