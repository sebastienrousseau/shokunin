// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Canonical page-URL derivation shared by staging, feeds, and SEO
//! output (spec A2/B1, plan §2 item 1.2, issue #586).
//!
//! ## Why this module exists
//!
//! `staticdatagen`'s RSS generator hard-fails the whole build when a
//! page lacks a `permalink:` front-matter key (`rss-gen`:
//! "channel.link is missing"). ssg stages content through
//! [`content_stager`](crate::content_stager) before
//! `staticdatagen::compile` ever sees it, so ssg can guarantee a
//! permalink is *always* present by deriving one at staging time.
//! This module is that single derivation code path — the plan's goal
//! is that canonical `<link>`, feed `<link>`, and injected
//! `permalink:` all agree because they all call
//! [`derive_page_url`].
//!
//! ## The URL convention
//!
//! Published pages use *pretty* directory URLs with a trailing slash,
//! matching the Atom feed entry convention already shipped in
//! `src/plugins/postprocess/atom.rs` (`{base_url}/{rel_path}/`) and
//! the stager's own tags-stub permalink
//! (`https://example.invalid/tags/`):
//!
//! | Relative output path    | Derived URL                     |
//! | ----------------------- | ------------------------------- |
//! | `index.html`            | `{base_url}/`                   |
//! | `foo/index.html`        | `{base_url}/foo/`               |
//! | `posts/bar/index.html`  | `{base_url}/posts/bar/`         |
//! | `feed.xml` (non-index)  | `{base_url}/feed.xml`           |
//!
//! Windows path separators are normalised to `/`; percent-encoding is
//! left untouched (paths are already slugified upstream).

/// Joins a site `base_url` and a page's site-relative output path
/// into the canonical absolute URL for that page.
///
/// Conventions (see module docs for the full table):
///
/// - a trailing slash on `base_url` is tolerated (no `//` is emitted);
/// - `index.html` at any depth collapses to the enclosing directory
///   URL with a trailing slash — the pretty-URL form the Atom feed
///   (`src/plugins/postprocess/atom.rs`) already publishes;
/// - `\` separators are normalised to `/` (Windows builds);
/// - leading `./` / `/` on the relative path are ignored;
/// - percent-encoding is passed through as-is.
///
/// # Examples
///
/// ```rust
/// use ssg::urls::derive_page_url;
///
/// // Root page.
/// assert_eq!(
///     derive_page_url("https://example.com", "index.html"),
///     "https://example.com/"
/// );
/// // Pretty directory URL for a nested page.
/// assert_eq!(
///     derive_page_url("https://example.com/", "posts/foo/index.html"),
///     "https://example.com/posts/foo/"
/// );
/// // Non-index outputs keep their file name.
/// assert_eq!(
///     derive_page_url("https://example.com", "feed.xml"),
///     "https://example.com/feed.xml"
/// );
/// ```
#[must_use]
pub fn derive_page_url(base_url: &str, relative_output_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    // Normalise Windows separators, then drop any leading `./` or `/`
    // so callers can pass either bare or rooted relative paths.
    let normalised = relative_output_path.replace('\\', "/");
    let mut rel = normalised.as_str();
    loop {
        if let Some(stripped) = rel.strip_prefix("./") {
            rel = stripped;
        } else if let Some(stripped) = rel.strip_prefix('/') {
            rel = stripped;
        } else {
            break;
        }
    }

    if rel.is_empty() || rel == "index.html" {
        return format!("{base}/");
    }
    if let Some(dir) = rel.strip_suffix("/index.html") {
        return format!("{base}/{dir}/");
    }
    format!("{base}/{rel}")
}

/// Maps a content-relative markdown source path to the site-relative
/// output path `staticdatagen`'s compiler will write for it.
///
/// Mirrors the compiler's convention exactly (locked by tests below):
/// `staticdatagen 0.0.9` (`src/utilities/write.rs`,
/// `get_processed_file_name` + `write_files_to_build_directory`)
/// writes `foo.md` to `<build>/foo/index.html` and `index.md` to the
/// build root's `index.html`. The same mapping is documented on
/// `derive_url` in `src/plugins/isr_manifest.rs`
/// (`posts/foo.md → /posts/foo/index.html`, `index.md → /index.html`,
/// `about/index.md → /about/index.html`).
///
/// Only the `.md` extension is stripped — matching both the compiler
/// and the ISR manifest — so a `.markdown` file keeps its full name
/// as the directory segment, exactly as `staticdatagen` would emit it.
///
/// # Examples
///
/// ```rust
/// use ssg::urls::derive_output_rel_path;
///
/// assert_eq!(derive_output_rel_path("index.md"), "index.html");
/// assert_eq!(derive_output_rel_path("foo.md"), "foo/index.html");
/// assert_eq!(
///     derive_output_rel_path("posts/foo.md"),
///     "posts/foo/index.html"
/// );
/// assert_eq!(
///     derive_output_rel_path("about/index.md"),
///     "about/index.html"
/// );
/// ```
#[must_use]
pub fn derive_output_rel_path(content_rel_source_path: &str) -> String {
    let normalised = content_rel_source_path.replace('\\', "/");
    let rel = normalised.trim_start_matches("./").trim_start_matches('/');
    let stripped = rel.strip_suffix(".md").unwrap_or(rel);
    if stripped == "index" {
        return "index.html".to_string();
    }
    if let Some(dir) = stripped.strip_suffix("/index") {
        return format!("{dir}/index.html");
    }
    format!("{stripped}/index.html")
}

/// Derives the canonical permalink for a content-relative markdown
/// source path: [`derive_output_rel_path`] piped into
/// [`derive_page_url`].
///
/// This is the value the content stager injects as `permalink:` when
/// a page's front matter carries neither `permalink` nor `url`
/// (spec A2/B1, plan §2 item 1.2, issue #586).
///
/// # Examples
///
/// ```rust
/// use ssg::urls::derive_permalink;
///
/// assert_eq!(
///     derive_permalink("https://example.com", "posts/foo.md"),
///     "https://example.com/posts/foo/"
/// );
/// assert_eq!(
///     derive_permalink("https://example.com/", "index.md"),
///     "https://example.com/"
/// );
/// ```
#[must_use]
pub fn derive_permalink(
    base_url: &str,
    content_rel_source_path: &str,
) -> String {
    derive_page_url(base_url, &derive_output_rel_path(content_rel_source_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_index_becomes_bare_base_with_trailing_slash() {
        assert_eq!(
            derive_page_url("https://example.com", "index.html"),
            "https://example.com/"
        );
    }

    #[test]
    fn trailing_slash_base_url_does_not_double_slash() {
        assert_eq!(
            derive_page_url("https://example.com/", "foo/index.html"),
            "https://example.com/foo/"
        );
    }

    #[test]
    fn nested_index_html_collapses_to_directory_url() {
        assert_eq!(
            derive_page_url("https://example.com", "posts/bar/index.html"),
            "https://example.com/posts/bar/"
        );
    }

    #[test]
    fn non_index_output_keeps_file_name() {
        assert_eq!(
            derive_page_url("https://example.com", "rss.xml"),
            "https://example.com/rss.xml"
        );
    }

    #[test]
    fn windows_separators_are_normalised() {
        assert_eq!(
            derive_page_url("https://example.com", "posts\\bar\\index.html"),
            "https://example.com/posts/bar/"
        );
    }

    #[test]
    fn leading_dot_slash_and_slash_are_ignored() {
        assert_eq!(
            derive_page_url("https://example.com", "./a/index.html"),
            "https://example.com/a/"
        );
        assert_eq!(
            derive_page_url("https://example.com", "/a/index.html"),
            "https://example.com/a/"
        );
    }

    #[test]
    fn percent_encoding_is_passed_through() {
        assert_eq!(
            derive_page_url("https://example.com", "caf%C3%A9/index.html"),
            "https://example.com/caf%C3%A9/"
        );
    }

    #[test]
    fn empty_rel_path_yields_base_with_trailing_slash() {
        assert_eq!(
            derive_page_url("https://example.com", ""),
            "https://example.com/"
        );
    }

    /// Locks [`derive_output_rel_path`] to the compiler's mapping:
    /// the three documented shapes from `staticdatagen 0.0.9`
    /// `src/utilities/write.rs` and `isr_manifest::derive_url`
    /// (`posts/foo.md → /posts/foo/index.html`,
    /// `index.md → /index.html`,
    /// `about/index.md → /about/index.html`). If either side changes
    /// convention this test flags the disagreement.
    #[test]
    fn output_rel_path_agrees_with_compiler_convention() {
        assert_eq!(
            derive_output_rel_path("posts/foo.md"),
            "posts/foo/index.html"
        );
        assert_eq!(derive_output_rel_path("index.md"), "index.html");
        assert_eq!(
            derive_output_rel_path("about/index.md"),
            "about/index.html"
        );
    }

    #[test]
    fn markdown_extension_is_kept_when_not_md() {
        // staticdatagen's get_processed_file_name only strips a known
        // extension list ("md" among them, "markdown" not) — mirror
        // it exactly rather than being "helpfully" prettier.
        assert_eq!(
            derive_output_rel_path("foo.markdown"),
            "foo.markdown/index.html"
        );
    }

    #[test]
    fn windows_source_separator_is_normalised_in_output_path() {
        assert_eq!(
            derive_output_rel_path("posts\\foo.md"),
            "posts/foo/index.html"
        );
    }

    #[test]
    fn permalink_composes_both_derivations() {
        assert_eq!(
            derive_permalink("https://example.com/", "posts/foo.md"),
            "https://example.com/posts/foo/"
        );
        assert_eq!(
            derive_permalink("https://example.com", "index.md"),
            "https://example.com/"
        );
        assert_eq!(
            derive_permalink("https://example.com", "about/index.md"),
            "https://example.com/about/"
        );
    }
}
