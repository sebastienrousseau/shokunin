// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Man-page generation from the live clap definition.
//!
//! # Why this is written here rather than taken from a crate
//!
//! The obvious options were `clap_mangen`, `help2man`, or the `roff` crate
//! that `clap_mangen` is built on. All three were considered:
//!
//! * `help2man` builds one page from one `--help`. This CLI has eight
//!   subcommands, so it would need eight invocations plus `--include` files
//!   of hand-written prose — reintroducing the second source of truth it was
//!   meant to avoid, and adding a perl build dependency.
//! * `clap_mangen` cannot express the prose a good page needs: a real
//!   DESCRIPTION, worked EXAMPLES, EXIT STATUS.
//! * `roff` is small (456 lines, no runtime dependencies, no `unsafe`, no
//!   I/O) and would have served, but it is unaudited against this
//!   repository's `cargo vet` policy, whose exemption ratchet forbids adding
//!   an unreviewed crate. We use a narrow subset of roff — `.TH`, `.SH`,
//!   `.TP`, `.B`, `.nf` — so emitting it directly costs less than the
//!   supply-chain review it avoids.
//!
//! # What cannot drift
//!
//! SYNOPSIS and OPTIONS are walked out of [`crate::cmd::Cli`]'s own
//! `clap::Command`, so a flag that exists in the parser appears in the page
//! by construction. Only the prose sections are written by hand, and
//! `tests/man_page.rs` asserts that every flag and subcommand the parser
//! defines is present in the rendered output — so prose cannot fall behind
//! the parser either.
//!
//! # Escaping
//!
//! A leading `.` or `'` on a line is a roff control line, and `\` and `-`
//! are meaningful mid-text. [`escape_text`] neutralises all four. This is
//! the same set the `roff` crate handles, and for the same reason.

use clap::Command;
use std::fmt::Write as _;

/// Escapes text for inclusion in a roff text line.
///
/// Neutralises the four sequences a roff processor would otherwise act on:
/// a backslash, a hyphen (which becomes a typographic dash), and a leading
/// period or apostrophe on a line, either of which starts a control line and
/// would silently swallow the rest of it.
///
/// # Examples
///
/// ```
/// use ssg::cmd::man::escape_text;
/// assert_eq!(escape_text("a-b"), r"a\-b");
/// assert_eq!(escape_text("x\n.y"), "x\n\\&.y");
/// ```
#[must_use]
pub fn escape_text(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('-', r"\-")
        .replace("\n.", "\n\\&.")
        .replace("\n'", "\n\\&'")
}

/// Quotes a roff macro argument when it contains whitespace.
fn quote_arg(s: &str) -> String {
    if s.contains(char::is_whitespace) {
        format!("\"{}\"", s.replace('"', "'"))
    } else {
        s.to_owned()
    }
}

/// Renders the full `ssg.1` man page.
///
/// `version` and `date` are passed in rather than read from the environment
/// so the output is a pure function of its inputs — the determinism gate
/// compares builds across machines, and a page carrying today's date would
/// differ on every run.
///
/// # Examples
///
/// ```
/// use ssg::cmd::{man, Cli};
/// let page = man::render(&Cli::subcommand_app(), "0.0.58", "2026-09-02");
/// assert!(page.starts_with(".TH "));
/// assert!(page.contains(".SH NAME"));
/// ```
#[must_use]
pub fn render(app: &Command, version: &str, date: &str) -> String {
    let mut out = String::with_capacity(8192);

    // .TH title section date source manual
    let _ = writeln!(
        out,
        ".TH {} 1 {} {} {}",
        quote_arg("SSG"),
        quote_arg(date),
        quote_arg(&format!("ssg {version}")),
        quote_arg("User Commands"),
    );

    section(&mut out, "NAME");
    let _ = writeln!(
        out,
        "ssg \\- {}",
        escape_text(
            app.get_about()
                .map_or_else(
                    || "static site generator".into(),
                    ToString::to_string,
                )
                .as_str()
        )
    );

    section(&mut out, "SYNOPSIS");
    let _ = writeln!(out, "\\fBssg\\fR [\\fIOPTIONS\\fR]");
    for sub in app.get_subcommands() {
        let _ = writeln!(
            out,
            ".br\n\\fBssg {}\\fR [\\fIOPTIONS\\fR]",
            escape_text(sub.get_name())
        );
    }

    section(&mut out, "DESCRIPTION");
    for para in prose::DESCRIPTION {
        paragraph(&mut out, para);
    }

    // Top-level options, then one subsection per subcommand. Walking the
    // parser means a flag cannot be missing from the page.
    section(&mut out, "OPTIONS");
    write_args(&mut out, app);

    for sub in app.get_subcommands() {
        subsection(&mut out, &format!("ssg {}", sub.get_name()));
        if let Some(about) = sub.get_about() {
            paragraph(&mut out, &about.to_string());
        }
        write_args(&mut out, sub);
    }

    section(&mut out, "EXIT STATUS");
    for (code, meaning) in prose::EXIT_STATUS {
        tagged(&mut out, code, meaning);
    }

    section(&mut out, "ENVIRONMENT");
    for (var, meaning) in prose::ENVIRONMENT {
        tagged(&mut out, var, meaning);
    }

    section(&mut out, "EXAMPLES");
    for (caption, cmd) in prose::EXAMPLES {
        paragraph(&mut out, caption);
        literal(&mut out, cmd);
    }

    section(&mut out, "SEE ALSO");
    paragraph(&mut out, prose::SEE_ALSO);

    out
}

/// Writes every argument of `cmd` as a `.TP` tagged paragraph.
fn write_args(out: &mut String, cmd: &Command) {
    for arg in cmd.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }
        let mut tag = String::new();
        if let Some(short) = arg.get_short() {
            let _ = write!(tag, "\\fB\\-{short}\\fR");
        }
        if let Some(long) = arg.get_long() {
            if !tag.is_empty() {
                tag.push_str(", ");
            }
            let _ = write!(tag, "\\fB\\-\\-{}\\fR", escape_text(long));
        }
        if tag.is_empty() {
            // A positional argument.
            let _ =
                write!(tag, "\\fI{}\\fR", escape_text(arg.get_id().as_str()));
        }
        if let Some(names) = arg.get_value_names() {
            for n in names {
                let _ = write!(tag, " \\fI{}\\fR", escape_text(n));
            }
        }

        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(|h| h.to_string())
            .unwrap_or_default();

        let _ = writeln!(out, ".TP\n{tag}\n{}", escape_text(&help));
    }
}

fn section(out: &mut String, name: &str) {
    let _ = writeln!(out, ".SH {}", quote_arg(name));
}

fn subsection(out: &mut String, name: &str) {
    let _ = writeln!(out, ".SS {}", quote_arg(name));
}

fn paragraph(out: &mut String, text: &str) {
    let _ = writeln!(out, ".PP\n{}", escape_text(text));
}

fn tagged(out: &mut String, tag: &str, text: &str) {
    let _ = writeln!(
        out,
        ".TP\n\\fB{}\\fR\n{}",
        escape_text(tag),
        escape_text(text)
    );
}

/// An unfilled, indented block — used for command examples, where roff must
/// not reflow the text.
fn literal(out: &mut String, text: &str) {
    let _ = writeln!(out, ".nf\n.RS 4\n{}\n.RE\n.fi", escape_text(text));
}

/// The hand-written half of the page.
///
/// Everything here is prose a parser cannot know: why a flag exists, what a
/// workflow looks like, what an exit code means. The generated half covers
/// what the parser does know, so these two never restate each other.
mod prose {
    pub(super) const DESCRIPTION: &[&str] = &[
        "ssg compiles a directory of Markdown content and templates into a \
         static website. Unlike generators that render and stop, it runs its \
         accessibility, security and metadata checks during the build: a page \
         that fails is reported with its file and line, rather than \
         discovered after deployment.",
        "Configuration is read from ssg.toml or config.toml in the working \
         directory, or from the path given to --config. When no configuration \
         is found the built-in defaults are used and a warning naming the \
         fallback canonical host is printed, because that host ends up in \
         every canonical URL, sitemap entry and JSON-LD identifier the build \
         emits.",
    ];

    pub(super) const EXIT_STATUS: &[(&str, &str)] = &[
        ("0", "The build completed and every enabled gate passed."),
        (
            "1",
            "The build failed, or a gate reported a violation at or above the \
             configured --fail-on severity.",
        ),
        ("101", "An internal error. Please report this with the input that caused it."),
    ];

    pub(super) const ENVIRONMENT: &[(&str, &str)] = &[
        (
            "SSG_CONFIG",
            "Path to a configuration file. Consulted only when neither \
             --config nor a configuration file in the working directory is \
             found, so a stale value cannot override a project's own file.",
        ),
        (
            "STRICT_A11Y",
            "When set, accessibility violations fail the build instead of \
             warning.",
        ),
        (
            "SSG_REQUIRE_EXAMPLES",
            "Used by this project's own CI. Makes the example-output gates \
             fail rather than skip when they find nothing to inspect.",
        ),
    ];

    pub(super) const EXAMPLES: &[(&str, &str)] = &[
        (
            "Build the site described by ssg.toml in the current directory:",
            "ssg build",
        ),
        ("Serve the site with live reload while editing:", "ssg dev"),
        (
            "Run every validator without writing any output:",
            "ssg check",
        ),
        (
            "Report the plugin pipeline, including the deploy stage for a \
             chosen target:",
            "ssg plugins list --target netlify",
        ),
    ];

    pub(super) const SEE_ALSO: &str =
        "Full documentation at https://static-site-generator.com. Source, \
         issue tracker and the architecture decision records at \
         https://github.com/sebastienrousseau/static-site-generator.";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Cli;

    #[test]
    fn escapes_roff_control_sequences() {
        assert_eq!(escape_text("a-b"), r"a\-b");
        assert_eq!(escape_text(r"a\b"), r"a\\b");
        // A leading period on a line would otherwise start a control line and
        // swallow the rest of it.
        assert_eq!(escape_text("x\n.SH EVIL"), "x\n\\&.SH EVIL");
        assert_eq!(escape_text("x\n'tis"), "x\n\\&'tis");
    }

    #[test]
    fn quotes_only_arguments_containing_whitespace() {
        assert_eq!(quote_arg("SSG"), "SSG");
        assert_eq!(quote_arg("User Commands"), "\"User Commands\"");
    }

    #[test]
    fn render_is_deterministic() {
        // The determinism gate compares whole trees across machines, so the
        // page must not depend on the clock.
        let app = Cli::subcommand_app();
        let a = render(&app, "0.0.58", "2026-09-02");
        let b = render(&app, "0.0.58", "2026-09-02");
        assert_eq!(a, b);
    }

    #[test]
    fn page_has_the_mandatory_sections() {
        let page = render(&Cli::subcommand_app(), "0.0.58", "2026-09-02");
        for s in [
            ".TH ",
            ".SH NAME",
            ".SH SYNOPSIS",
            ".SH DESCRIPTION",
            ".SH OPTIONS",
            ".SH \"EXIT STATUS\"",
            ".SH ENVIRONMENT",
            ".SH EXAMPLES",
            ".SH \"SEE ALSO\"",
        ] {
            assert!(page.contains(s), "missing section {s}\n{page}");
        }
    }

    #[test]
    fn every_line_is_valid_roff_structure() {
        // A line starting with `.` must be a macro we actually emit; anything
        // else means escaping failed and prose is being read as markup.
        let page = render(&Cli::subcommand_app(), "0.0.58", "2026-09-02");
        let known = [
            ".TH", ".SH", ".SS", ".PP", ".TP", ".br", ".nf", ".fi", ".RS",
            ".RE",
        ];
        for (n, line) in page.lines().enumerate() {
            if !line.starts_with('.') {
                continue;
            }
            let macro_name = line.split_whitespace().next().unwrap_or_default();
            assert!(
                known.contains(&macro_name),
                "line {}: unrecognised roff macro {macro_name:?}\n  {line}",
                n + 1
            );
        }
    }
}
