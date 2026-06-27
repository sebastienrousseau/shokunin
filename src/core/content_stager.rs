// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Content staging — workaround for the `staticdatagen 0.0.9`
//! empty-layout regression that breaks any site whose markdown files
//! don't carry a `layout:` frontmatter key.
//!
//! ## The regression
//!
//! `staticdatagen 0.0.9` (the markdown → HTML compiler we ship) calls
//! the templating engine like this:
//!
//! ```ignore
//! engine.render_page(
//!     &context,
//!     metadata.get("layout").cloned().unwrap_or_default().as_str(),
//! )
//! ```
//!
//! When `layout` is absent from the frontmatter, `unwrap_or_default()`
//! produces an empty string.
//!
//! The `MiniJinja` layer then fails with
//! `invalid template or partial name: ""` and the whole build aborts.
//!
//! That's wrong shape — every prior SSG version silently fell back to
//! a `page` (or similar) layout for unannotated content. We can't ship
//! a patched `staticdatagen` from a branch PR, but we *can* shield it
//! from the bad inputs by pre-processing content into a staging
//! directory.
//!
//! ## What this module does
//!
//! Walks `content_dir` once, copies every file to a parallel
//! `staging_dir` tree, and for every `.md` file whose frontmatter
//! lacks a `layout:` key:
//!
//! 1. detects the missing key with a cheap line-scan over the YAML
//!    fence block (no full YAML parse — the bug is structural, not
//!    semantic);
//! 2. inserts `layout: "<default>"` as the first key in the
//!    frontmatter, preserving every other line verbatim;
//! 3. writes the augmented body to the staged path.
//!
//! Non-markdown files are byte-for-byte copies. Empty frontmatter
//! (no `---` fence) is left untouched — those files won't trigger the
//! bug because the `metadata.get("layout")` lookup never runs without
//! a parseable frontmatter block.
//!
//! ## Why not modify the source tree?
//!
//! The user's checkout is sacred. Two reasons we must not edit
//! `content_dir`:
//!
//! - the build runs from a CI checkout that the user expects to be
//!   read-only;
//! - reruns of the build would re-inject the default, doubling lines.
//!
//! Instead we operate on a fresh directory under `<build_dir>/.ssg-content-staged/`
//! that's recreated on every build.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default layout name injected into `.md` files that lack one.
///
/// Matches the scaffold's `templates/tera/page.html`, which is the
/// universally-available layout in every SSG-generated project.
pub const DEFAULT_LAYOUT: &str = "page";

/// Stages a copy of `content_dir` under `<build_dir>/.ssg-content-staged/`
/// and injects `layout: "<DEFAULT_LAYOUT>"` into every markdown file
/// whose frontmatter lacks one.
///
/// Returns the path to the staged directory. The caller passes that
/// path to `staticdatagen::compile` instead of the original
/// `content_dir`.
///
/// # Errors
///
/// Returns [`io::Error`] when the staging directory cannot be created
/// or a source file cannot be read or written.
///
/// # Examples
///
/// ```rust
/// use ssg::core_group::content_stager::stage_content_with_default_layout;
/// use std::fs;
///
/// let tmp = tempfile::tempdir().unwrap();
/// let src = tmp.path().join("content");
/// let build = tmp.path().join("build");
/// fs::create_dir_all(&src).unwrap();
/// fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();
///
/// let staged = stage_content_with_default_layout(&src, &build).unwrap();
/// let out = fs::read_to_string(staged.join("a.md")).unwrap();
/// assert!(out.contains("layout: \"page\""));
/// assert!(out.contains("title: A"));
/// ```
pub fn stage_content_with_default_layout(
    content_dir: &Path,
    build_dir: &Path,
) -> Result<PathBuf, io::Error> {
    stage_content_with_template_defaults(content_dir, build_dir, &[])
}

/// Like [`stage_content_with_default_layout`] but also injects empty
/// defaults for every `{{ var }}` reference the templates make.
///
/// Closes the staticweaver "Unresolved template tag" crash that
/// fires when user content omits a key their template references.
///
/// `template_var_keys` is the result of [`collect_template_vars`] run
/// over the user's template directory.
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

    // staticdatagen 0.0.9's tags-page generator (`write_tags_html_to_file`)
    // unconditionally opens `<build_dir>/tags/index.html` and replaces a
    // `[[content]]` placeholder. If the user hasn't shipped a tags page,
    // the open fails with "No such file or directory (os error 2)" and
    // the entire build aborts after producing all real outputs. Stub
    // one in so the open succeeds; the resulting tags page just won't
    // be linked anywhere.
    //
    // The stub is created BEFORE template-default injection so the
    // injector covers it too — every `{{ var }}` reference in the
    // user's templates lands in the staged tags.md as well.
    ensure_tags_stub(&staging_dir)?;

    if !template_var_keys.is_empty() {
        inject_template_defaults_recursive(&staging_dir, template_var_keys)?;
    }

    Ok(staging_dir)
}

fn ensure_tags_stub(staged_content_dir: &Path) -> Result<(), io::Error> {
    let candidates = [
        staged_content_dir.join("tags.md"),
        staged_content_dir.join("tags/index.md"),
    ];
    if candidates.iter().any(|p| p.exists()) {
        return Ok(());
    }
    // Minimal stub: layout + title + an absolute-URL permalink (rss-gen
    // 0.0.5 validates this as a real URL — relative paths trip
    // "Invalid link: Invalid URL provided") + the literal `[[content]]`
    // placeholder that staticdatagen's tags writer expects to find.
    //
    // The `https://example.invalid/tags/` host is reserved (RFC 2606
    // `.invalid` TLD) so it can never collide with a real site URL.
    // Users with their own tags page replace this stub by checking
    // their tags.md or tags/index.md into the content tree.
    let stub = "---\n\
                layout: \"page\"\n\
                title: \"Tags\"\n\
                description: \"Tag index\"\n\
                permalink: \"https://example.invalid/tags/\"\n\
                ---\n\
                \n\
                [[content]]\n";
    fs::write(staged_content_dir.join("tags.md"), stub)?;
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

/// Recursively mirrors `src` into `dst`, transforming `.md` files via
/// [`inject_default_layout_if_missing`] and copying everything else
/// verbatim. Per-file work runs in parallel via Rayon — `read +
/// transform + write` is the hot path that determined whether the
/// staging shim added meaningful build-time overhead. With ~100 pages
/// the parallel pass keeps the staging cost under ~50 ms on a 4-core
/// runner, well inside the perf-budget gate.
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
                fs::read_to_string(&src_path).and_then(|body| {
                    let staged =
                        inject_default_layout_if_missing(&body, DEFAULT_LAYOUT);
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

/// Returns `body` with `layout: "<default>"` inserted as the first key
/// in the YAML frontmatter block when the block exists and the key is
/// absent.
///
/// Idempotent: a second pass over a previously-staged
/// file returns the input unchanged.
///
/// Behaviour matrix:
///
/// | Input shape                                     | Output                       |
/// | ----------------------------------------------- | ---------------------------- |
/// | No frontmatter fence at all                     | unchanged                    |
/// | Frontmatter with existing `layout:` key         | unchanged                    |
/// | Frontmatter without `layout:` key               | `layout: "<default>"` injected as the first key inside the fence |
/// | Empty frontmatter (`---\n---`)                  | `layout: "<default>"` injected as the only key |
///
/// # Examples
///
/// ```rust
/// use ssg::core_group::content_stager::inject_default_layout_if_missing;
///
/// // Already has layout — untouched.
/// let with_layout = "---\nlayout: post\ntitle: T\n---\nbody";
/// assert_eq!(inject_default_layout_if_missing(with_layout, "page"), with_layout);
///
/// // Missing layout — gets one.
/// let no_layout = "---\ntitle: T\n---\nbody";
/// let out = inject_default_layout_if_missing(no_layout, "page");
/// assert!(out.contains("layout: \"page\""));
/// assert!(out.contains("title: T"));
///
/// // No frontmatter — passthrough.
/// let plain = "# heading\n\nbody";
/// assert_eq!(inject_default_layout_if_missing(plain, "page"), plain);
/// ```
#[must_use]
pub fn inject_default_layout_if_missing(
    body: &str,
    default_layout: &str,
) -> String {
    // Tolerate a UTF-8 BOM and leading whitespace before the fence —
    // both occur in editor-mangled exports.
    let trimmed = body.trim_start_matches('\u{FEFF}');

    // Find the opening fence `---\n` (or `---\r\n`). We rely on
    // the fence being on the first non-blank line; static-site
    // generators universally require this shape.
    let Some((lead, after_open)) = find_opening_fence(trimmed) else {
        return body.to_string();
    };

    // Locate the closing fence within `after_open`.
    let Some(close_rel) = find_closing_fence(after_open) else {
        return body.to_string();
    };

    let block = &after_open[..close_rel];
    let after_block = &after_open[close_rel..];

    // Normalise multi-line quoted scalars before checking for the
    // layout key. The user's content sometimes carries values like:
    //
    //     twitter_url: "
    //     https://example.com/x"
    //
    // which is YAML-spec-compliant (newline → space in the value)
    // but trips noyalib's stricter parser inside `metadata-gen`.
    // Collapsing them onto a single line is shape-preserving and
    // unblocks the downstream extractor.
    let normalised_block = collapse_multiline_quoted_scalars(block);

    if frontmatter_has_layout_key(&normalised_block) {
        // Even if no layout injection happens, write the normalised
        // block back so the downstream extractor sees a parseable
        // value.
        if normalised_block != block {
            let mut out = String::with_capacity(body.len());
            out.push_str(&trimmed[..trimmed.len() - after_open.len()]);
            out.push_str(&normalised_block);
            out.push_str(after_block);
            if body.starts_with('\u{FEFF}') {
                return format!("\u{FEFF}{out}");
            }
            return out;
        }
        return body.to_string();
    }

    let mut out = String::with_capacity(body.len() + 32);
    out.push_str(&trimmed[..trimmed.len() - after_open.len()]);
    let _ = lead;
    out.push_str(&format!("layout: \"{default_layout}\"\n"));
    out.push_str(&normalised_block);
    out.push_str(after_block);

    // Re-prepend any BOM we trimmed off.
    if body.starts_with('\u{FEFF}') {
        return format!("\u{FEFF}{out}");
    }
    out
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

/// Files that `staticdatagen 0.0.9` unconditionally copies from the
/// template directory in `copy_auxiliary_files`. Missing files abort
/// the build with `No such file or directory (os error 2)`.
const REQUIRED_TEMPLATE_FILES: &[&str] = &["main.js", "sw.js"];

/// Stages a copy of `template_dir` under
/// `<build_dir>/.ssg-templates-staged/` and writes empty-stub files
/// for any required-template-files entry (`main.js`, `sw.js`) the
/// user hasn't provided.
///
/// This shields `staticdatagen`'s hardcoded auxiliary-file copy step
/// from breaking the build on minimal template sets. The user's
/// original templates are read-only.
///
/// Returns the staged template directory path.
///
/// # Errors
///
/// Returns [`io::Error`] if the staging directory cannot be created
/// or a source file cannot be read or written.
///
/// # Examples
///
/// ```rust
/// use ssg::core_group::content_stager::stage_templates_with_required_stubs;
/// use std::fs;
///
/// let tmp = tempfile::tempdir().unwrap();
/// let src = tmp.path().join("templates");
/// let build = tmp.path().join("build");
/// fs::create_dir_all(&src).unwrap();
/// fs::write(src.join("page.html"), "<html/>").unwrap();
///
/// let staged = stage_templates_with_required_stubs(&src, &build).unwrap();
/// assert!(staged.join("page.html").exists());
/// // staticdatagen's hardcoded auxiliary files get stubbed in:
/// assert!(staged.join("main.js").exists());
/// assert!(staged.join("sw.js").exists());
/// ```
pub fn stage_templates_with_required_stubs(
    template_dir: &Path,
    build_dir: &Path,
) -> Result<PathBuf, io::Error> {
    let staging_dir = staging_root_for("templates", build_dir);

    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    fs::create_dir_all(&staging_dir)?;

    if template_dir.is_dir() {
        copy_templates_tree(template_dir, &staging_dir)?;
    }

    for name in REQUIRED_TEMPLATE_FILES {
        let dest = staging_dir.join(name);
        if !dest.exists() {
            // Empty stub — staticdatagen only checks the path
            // resolves, not the content. A 0-byte file satisfies
            // both copy_auxiliary_files and any browser that
            // requests the asset (it'll just be empty).
            fs::write(&dest, b"")?;
        }
    }
    Ok(staging_dir)
}

fn copy_templates_tree(src: &Path, dst: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_templates_tree(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            let _ = fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Returns `true` if any line inside the frontmatter block starts
/// with `layout:` (after optional whitespace; comments excluded).
fn frontmatter_has_layout_key(block: &str) -> bool {
    for raw in block.lines() {
        let line = raw.trim_start();
        // Skip YAML comments.
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with("layout:")
            || line.starts_with("layout :")
            || line.starts_with("\"layout\":")
            || line.starts_with("'layout':")
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_passthrough() {
        let input = "# Heading\n\nBody.";
        assert_eq!(inject_default_layout_if_missing(input, "page"), input);
    }

    #[test]
    fn existing_layout_passthrough() {
        let input = "---\nlayout: post\ntitle: T\n---\nbody";
        assert_eq!(inject_default_layout_if_missing(input, "page"), input);
    }

    #[test]
    fn quoted_layout_passthrough() {
        let input = "---\nlayout: \"report\"\ntitle: T\n---\nbody";
        assert_eq!(inject_default_layout_if_missing(input, "page"), input);
    }

    #[test]
    fn missing_layout_gets_injected() {
        let input = "---\ntitle: T\n---\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        assert!(out.contains("layout: \"page\""));
        assert!(out.contains("title: T"));
        assert!(out.contains("body"));
    }

    #[test]
    fn empty_frontmatter_gets_injection() {
        let input = "---\n---\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        assert!(out.contains("layout: \"page\""));
        assert!(out.contains("body"));
    }

    #[test]
    fn idempotent_double_pass() {
        let input = "---\ntitle: T\n---\nbody";
        let once = inject_default_layout_if_missing(input, "page");
        let twice = inject_default_layout_if_missing(&once, "page");
        assert_eq!(once, twice);
    }

    #[test]
    fn injection_preserves_user_keys_in_order() {
        // Real-world shape: heading-comment + many keys, no layout.
        let input = "---\n\n# Front Matter (YAML)\n\nauthor: \"x\"\ntitle: \"y\"\n---\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        // Layout lands as the FIRST key inside the fence.
        let layout_pos = out.find("layout:").unwrap();
        let author_pos = out.find("author:").unwrap();
        let title_pos = out.find("title:").unwrap();
        assert!(layout_pos < author_pos);
        assert!(author_pos < title_pos);
        // Other keys preserved verbatim.
        assert!(out.contains("# Front Matter (YAML)"));
        assert!(out.contains("author: \"x\""));
        assert!(out.contains("title: \"y\""));
    }

    #[test]
    fn comment_line_is_not_confused_for_layout() {
        // A comment that mentions "layout" must NOT block injection.
        let input = "---\n# layout: explained\ntitle: T\n---\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        assert!(out.contains("layout: \"page\""));
    }

    #[test]
    fn crlf_line_endings_are_handled() {
        let input = "---\r\ntitle: T\r\n---\r\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        assert!(out.contains("layout: \"page\""));
        assert!(out.contains("title: T"));
    }

    #[test]
    fn bom_is_preserved() {
        let input = "\u{FEFF}---\ntitle: T\n---\nbody";
        let out = inject_default_layout_if_missing(input, "page");
        assert!(out.starts_with('\u{FEFF}'));
        assert!(out.contains("layout: \"page\""));
    }

    #[test]
    fn stage_content_recreates_tree_with_injection() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("page.md"), "---\ntitle: P\n---\nbody").unwrap();
        fs::write(
            src.join("nested/with-layout.md"),
            "---\nlayout: post\ntitle: WL\n---\nbody",
        )
        .unwrap();
        fs::write(src.join("nested/data.json"), "{}").unwrap();

        let staged = stage_content_with_default_layout(&src, &build).unwrap();
        // Files are mirrored.
        assert!(staged.join("page.md").exists());
        assert!(staged.join("nested/with-layout.md").exists());
        assert!(staged.join("nested/data.json").exists());
        // Injection happened for the missing-layout file.
        let page = fs::read_to_string(staged.join("page.md")).unwrap();
        assert!(page.contains("layout: \"page\""));
        // Existing layout untouched.
        let wl =
            fs::read_to_string(staged.join("nested/with-layout.md")).unwrap();
        assert!(wl.contains("layout: post"));
        assert!(!wl.contains("layout: \"page\""));
        // Non-markdown file copied verbatim.
        assert_eq!(
            fs::read_to_string(staged.join("nested/data.json")).unwrap(),
            "{}"
        );
    }

    #[test]
    fn stage_templates_creates_missing_required_stubs() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("templates");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("page.html"), "<html/>").unwrap();
        // No main.js, no sw.js — staticdatagen would crash.

        let staged = stage_templates_with_required_stubs(&src, &build).unwrap();
        assert!(staged.join("page.html").exists());
        assert!(staged.join("main.js").exists());
        assert!(staged.join("sw.js").exists());
        // Stubs are zero-byte.
        assert_eq!(fs::metadata(staged.join("sw.js")).unwrap().len(), 0);
    }

    #[test]
    fn stage_templates_preserves_existing_required_files() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("templates");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.js"), "// real main").unwrap();
        // sw.js still missing.

        let staged = stage_templates_with_required_stubs(&src, &build).unwrap();
        // Existing file untouched.
        assert_eq!(
            fs::read_to_string(staged.join("main.js")).unwrap(),
            "// real main"
        );
        // Missing one stubbed.
        assert!(staged.join("sw.js").exists());
    }

    #[test]
    fn stage_templates_handles_nonexistent_template_dir() {
        // Some sites pass a template_dir that doesn't exist yet
        // (the legacy CLI used to scaffold one). The staging must
        // still produce a usable dir with the required stubs.
        let tmp = tempfile::tempdir().unwrap();
        let nope = tmp.path().join("nonexistent-templates");
        let build = tmp.path().join("build");

        let staged =
            stage_templates_with_required_stubs(&nope, &build).unwrap();
        assert!(staged.join("main.js").exists());
        assert!(staged.join("sw.js").exists());
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

    #[test]
    fn injection_via_main_function_normalises_multiline_then_injects_layout() {
        let body = "---\ntitle: T\nurl: \"\nhttps://x.com\"\n---\nbody";
        let out = inject_default_layout_if_missing(body, "page");
        // Layout was injected.
        assert!(out.contains("layout: \"page\""));
        // Multi-line value collapsed.
        assert!(out.contains("url: \"https://x.com\""));
        // Body preserved.
        assert!(out.contains("body"));
    }

    #[test]
    fn stage_content_creates_tags_stub_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();

        let staged = stage_content_with_default_layout(&src, &build).unwrap();
        let tags = staged.join("tags.md");
        assert!(tags.exists());
        let body = fs::read_to_string(&tags).unwrap();
        assert!(body.contains("layout:"));
        assert!(body.contains("[[content]]"));
    }

    #[test]
    fn stage_content_preserves_existing_tags_page() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tags.md"),
            "---\nlayout: post\ntitle: \"Real Tags\"\n---\nreal body",
        )
        .unwrap();

        let staged = stage_content_with_default_layout(&src, &build).unwrap();
        let body = fs::read_to_string(staged.join("tags.md")).unwrap();
        // User's body is preserved verbatim.
        assert!(body.contains("Real Tags"));
        assert!(body.contains("real body"));
    }

    #[test]
    fn stage_content_preserves_existing_tags_index_md() {
        // Some sites ship `tags/index.md` instead of `tags.md`.
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(src.join("tags")).unwrap();
        fs::write(
            src.join("tags/index.md"),
            "---\nlayout: post\ntitle: NestedTags\n---\nnested",
        )
        .unwrap();

        let staged = stage_content_with_default_layout(&src, &build).unwrap();
        // We do NOT create a sibling tags.md when tags/index.md
        // already exists.
        assert!(staged.join("tags/index.md").exists());
        assert!(!staged.join("tags.md").exists());
    }

    #[test]
    fn stage_content_is_idempotent_across_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("content");
        let build = tmp.path().join("build");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.md"), "---\ntitle: A\n---\nbody").unwrap();

        let _staged1 = stage_content_with_default_layout(&src, &build).unwrap();
        let staged2 = stage_content_with_default_layout(&src, &build).unwrap();
        let body = fs::read_to_string(staged2.join("a.md")).unwrap();
        // No double-injection on the second run.
        assert_eq!(body.matches("layout:").count(), 1);
    }
}
