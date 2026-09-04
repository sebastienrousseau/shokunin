//! Asserts the man page cannot fall behind the parser.
//!
//! SYNOPSIS and OPTIONS are generated from the clap definition, so a flag
//! that exists appears in the page by construction. That is only half the
//! guarantee: the prose half is hand-written, and a page that *mentions* a
//! flag the parser no longer has is as misleading as one that omits a flag
//! the parser gained.
//!
//! `man ssg` reads as authoritative and nobody diffs it against `--help`, so
//! it is worth more here than a README section. Every documentation drift
//! found in this repository — a module table reading 38 against 63 modules,
//! "33 plugins" against 32, an install snippet eleven releases stale — was
//! prose restating something the code already knew. This is the same class,
//! gated the same way: derive the inventory from the code, never restate it.

use ssg::cmd::man;
use ssg::cmd::Cli;
use std::collections::BTreeSet;

fn page() -> String {
    man::render(
        &Cli::subcommand_app(),
        env!("CARGO_PKG_VERSION"),
        "2026-09-02",
    )
}

/// Every long flag the parser accepts, across the top level and every
/// subcommand.
fn parser_long_flags() -> BTreeSet<String> {
    let app = Cli::subcommand_app();
    let mut out = BTreeSet::new();
    let mut collect = |cmd: &clap::Command| {
        for arg in cmd.get_arguments() {
            if arg.is_hide_set() {
                continue;
            }
            if let Some(long) = arg.get_long() {
                let _ = out.insert(long.to_string());
            }
        }
    };
    collect(&app);
    for sub in app.get_subcommands() {
        collect(sub);
    }
    out
}

#[test]
fn every_parser_flag_appears_in_the_page() {
    let page = page();
    let missing: Vec<String> = parser_long_flags()
        .into_iter()
        // The page writes `--x` as `\-\-x`, matching roff's dash escaping.
        .filter(|f| {
            let escaped = f.replace('-', r"\-");
            !page.contains(&format!(r"\fB\-\-{escaped}\fR"))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "the parser accepts these flags but the man page does not document \
         them:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn every_subcommand_appears_in_the_page() {
    let page = page();
    let app = Cli::subcommand_app();
    let missing: Vec<&str> = app
        .get_subcommands()
        .map(clap::Command::get_name)
        .filter(|name| !page.contains(&format!("ssg {name}")))
        .collect();

    assert!(
        missing.is_empty(),
        "these subcommands are missing from the man page:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn the_page_names_no_flag_the_parser_lacks() {
    // The direction that catches a removed flag. A page advertising an option
    // that no longer exists sends the reader to an error message.
    let page = page();
    let known = parser_long_flags();

    let mut phantom = Vec::new();
    let mut rest = page.as_str();
    while let Some(i) = rest.find(r"\fB\-\-") {
        let after = &rest[i + r"\fB\-\-".len()..];
        let Some(end) = after.find(r"\fR") else { break };
        let flag = after[..end].replace(r"\-", "-");
        if !known.contains(&flag) {
            phantom.push(flag);
        }
        rest = &after[end..];
    }
    phantom.sort_unstable();
    phantom.dedup();

    assert!(
        phantom.is_empty(),
        "the man page documents flags the parser does not accept:\n  {}",
        phantom.join("\n  ")
    );
}

#[test]
fn examples_only_invoke_real_subcommands() {
    // A worked example that no longer runs is worse than no example.
    let page = page();
    let app = Cli::subcommand_app();
    let known: BTreeSet<&str> =
        app.get_subcommands().map(clap::Command::get_name).collect();

    // Only `.nf` blocks hold examples. Scanning every line that begins with
    // "ssg " also catches DESCRIPTION prose — "ssg compiles a directory of
    // Markdown…" — and asserting against that would be a false positive, so
    // the block boundaries are tracked rather than guessed at.
    let mut in_literal = false;
    for line in page.lines() {
        let t = line.trim();
        match t {
            ".nf" => {
                in_literal = true;
                continue;
            }
            ".fi" => {
                in_literal = false;
                continue;
            }
            _ => {}
        }
        if !in_literal {
            continue;
        }
        let Some(rest) = t.strip_prefix("ssg ") else {
            continue;
        };
        let Some(word) = rest.split_whitespace().next() else {
            continue;
        };
        assert!(
            known.contains(word),
            "an example invokes `ssg {word}`, which is not a subcommand"
        );
    }
}

#[test]
fn page_version_matches_the_crate() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        page().contains(&format!("ssg {version}")),
        "the man page's .TH source field does not name version {version}"
    );
}
