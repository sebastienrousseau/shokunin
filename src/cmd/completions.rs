// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shell-completion generation from the live clap definition.
//!
//! # Why this is written here rather than taken from a crate
//!
//! `clap_complete` is the obvious answer and was measured, not assumed:
//! adding it reports `1 unvetted dependencies: clap_complete:4.6.9 missing
//! ["safe-to-deploy"]`. This repository's `cargo vet` policy runs an
//! exemption ratchet whose count may only decrease, so the crate cannot be
//! added without either a real audit or breaking the gate. The same
//! reasoning retired the `roff` crate in [`crate::cmd::man`], and the same
//! trade applies: four narrow emitters cost less than the supply-chain
//! review they avoid.
//!
//! # What cannot drift
//!
//! Every completion is walked out of [`crate::cmd::Cli`]'s own
//! `clap::Command`. Nothing is transcribed, so a flag the parser gains
//! appears in all four shells by construction, and `tests/completions.rs`
//! asserts that — as well as feeding each script to the real shell's
//! syntax checker, since a completion script that fails to parse is worse
//! than none at all: it breaks the user's prompt on every tab.
//!
//! # Path arguments
//!
//! Which options complete filenames is taken from the argument's
//! `value_parser` type — `PathBuf` means a path — rather than from its
//! `value_name` reading `DIR` or `FILE`. A name is a display convention
//! that nothing enforces; the parser type is the property that actually
//! decides how the value is used.

use clap::builder::ValueParser;
use clap::{Arg, ArgAction, Command};
use std::fmt::Write as _;

/// A shell that completions can be generated for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shell {
    /// GNU Bash, via `complete -F`.
    Bash,
    /// Z shell, via an autoloaded `#compdef` function.
    Zsh,
    /// fish, via `complete -c`.
    Fish,
    /// PowerShell, via `Register-ArgumentCompleter -Native`.
    PowerShell,
}

impl Shell {
    /// Every supported shell, in a stable order.
    pub const ALL: [Self; 4] =
        [Self::Bash, Self::Zsh, Self::Fish, Self::PowerShell];

    /// The lowercase name used on the command line and in file paths.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::PowerShell => "powershell",
        }
    }

    /// The filename each shell expects to find in its completions
    /// directory.
    ///
    /// These are not cosmetic. Bash looks for a file named exactly after
    /// the command, zsh requires the leading underscore to autoload the
    /// function, and fish requires the `.fish` extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::cmd::completions::Shell;
    /// assert_eq!(Self::Bash.file_name("ssg"), "ssg");
    /// assert_eq!(Self::Zsh.file_name("ssg"), "_ssg");
    /// assert_eq!(Self::Fish.file_name("ssg"), "ssg.fish");
    /// ```
    #[must_use]
    pub fn file_name(self, bin: &str) -> String {
        match self {
            Self::Bash => bin.to_owned(),
            Self::Zsh => format!("_{bin}"),
            Self::Fish => format!("{bin}.fish"),
            Self::PowerShell => format!("_{bin}.ps1"),
        }
    }

    /// Parses a shell name, as accepted on the command line.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|sh| sh.name().eq_ignore_ascii_case(s))
    }
}

/// True when the argument's value is a filesystem path.
///
/// Derived from the `value_parser`, not from the value name — see the
/// module documentation.
fn takes_path(arg: &Arg) -> bool {
    arg.get_value_parser().type_id() == ValueParser::path_buf().type_id()
}

/// True when the argument consumes a value at all, as opposed to being a
/// bare flag such as `--drafts` or `--help`.
///
/// The signal is the argument's [`ArgAction`], not `get_num_args`, which
/// returns `None` for every argument that did not set it explicitly — none
/// of them here. Reading it as "takes no value" marked even `--content` as
/// a bare flag, which drops the `-r` from the fish spec and the
/// `:DIR:_files` from the zsh one, so the shell offers the next option
/// where it should offer a path.
fn takes_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Set | ArgAction::Append)
}

/// Every spelling of an argument: `-c` and `--content`.
fn spellings(arg: &Arg) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(short) = arg.get_short() {
        out.push(format!("-{short}"));
    }
    if let Some(long) = arg.get_long() {
        out.push(format!("--{long}"));
    }
    out
}

/// The one-line help for an argument, collapsed to a single line.
fn help_of(arg: &Arg) -> String {
    arg.get_help()
        .map(|h| h.to_string().replace('\n', " "))
        .unwrap_or_default()
}

/// Visible arguments only — a hidden flag should not be suggested.
fn visible_args(cmd: &Command) -> impl Iterator<Item = &Arg> {
    cmd.get_arguments().filter(|a| !a.is_hide_set())
}

/// Renders a completion script for `shell`.
///
/// # Examples
///
/// ```
/// use ssg::cmd::completions::{render, Shell};
/// use ssg::cmd::Cli;
///
/// let script = render(&Cli::subcommand_app(), Shell::Fish);
/// assert!(script.contains("complete -c ssg"));
/// ```
#[must_use]
pub fn render(app: &Command, shell: Shell) -> String {
    match shell {
        Shell::Bash => render_bash(app),
        Shell::Zsh => render_zsh(app),
        Shell::Fish => render_fish(app),
        Shell::PowerShell => render_powershell(app),
    }
}

// ---------------------------------------------------------------------------
// bash
// ---------------------------------------------------------------------------

fn render_bash(app: &Command) -> String {
    let bin = app.get_name();
    let subs: Vec<&str> =
        app.get_subcommands().map(Command::get_name).collect();

    let mut out = String::new();
    let _ =
        writeln!(out, "# {bin} completion for bash. Generated — do not edit.");
    let _ = writeln!(out, "_{bin}() {{");
    let _ = writeln!(out, "    local cur prev cmd opts paths i");
    let _ = writeln!(out, "    COMPREPLY=()");
    let _ = writeln!(out, r#"    cur="${{COMP_WORDS[COMP_CWORD]}}""#);
    let _ = writeln!(out, r#"    prev="${{COMP_WORDS[COMP_CWORD-1]}}""#);
    let _ = writeln!(out, r#"    cmd="""#);
    let _ = writeln!(out, "    for ((i = 1; i < COMP_CWORD; i++)); do");
    let _ = writeln!(out, r#"        case "${{COMP_WORDS[i]}}" in"#);
    let _ = writeln!(out, "            -*) ;;");
    let _ = writeln!(
        out,
        r#"            {}) cmd="${{COMP_WORDS[i]}}"; break ;;"#,
        subs.join("|")
    );
    let _ = writeln!(out, "        esac");
    let _ = writeln!(out, "    done");
    let _ = writeln!(out);
    let _ = writeln!(out, r#"    paths="""#);
    let _ = writeln!(out, r#"    case "$cmd" in"#);

    for sub in app.get_subcommands() {
        let opts = bash_word_list(sub, false);
        let paths = bash_word_list(sub, true);
        let _ = writeln!(out, r#"        {})"#, sub.get_name());
        let _ = writeln!(out, r#"            opts="{opts}""#);
        if !paths.is_empty() {
            let _ = writeln!(out, r#"            paths="{paths}""#);
        }
        let _ = writeln!(out, "            ;;");
    }

    // No subcommand seen yet: offer the subcommands and the global flags.
    let mut root = subs.join(" ");
    let root_opts = bash_word_list(app, false);
    if !root_opts.is_empty() {
        root.push(' ');
        root.push_str(&root_opts);
    }
    let root_paths = bash_word_list(app, true);
    let _ = writeln!(out, "        *)");
    let _ = writeln!(out, r#"            opts="{root}""#);
    if !root_paths.is_empty() {
        let _ = writeln!(out, r#"            paths="{root_paths}""#);
    }
    let _ = writeln!(out, "            ;;");
    let _ = writeln!(out, "    esac");
    let _ = writeln!(out);
    // A path-taking option was the previous word, so complete filenames
    // rather than repeating the option list.
    let _ = writeln!(
        out,
        r#"    if [[ -n "$paths" && " $paths " == *" $prev "* ]]; then"#
    );
    let _ = writeln!(
        out,
        r#"        mapfile -t COMPREPLY < <(compgen -f -- "$cur")"#
    );
    let _ = writeln!(out, "        return 0");
    let _ = writeln!(out, "    fi");
    let _ = writeln!(
        out,
        r#"    mapfile -t COMPREPLY < <(compgen -W "$opts" -- "$cur")"#
    );
    let _ = writeln!(out, "    return 0");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out, "complete -F _{bin} {bin}");
    out
}

/// Space-separated flag spellings for `cmd`; `paths_only` restricts the
/// list to options whose value is a filesystem path.
fn bash_word_list(cmd: &Command, paths_only: bool) -> String {
    let mut words = Vec::new();
    for arg in visible_args(cmd) {
        if paths_only && !takes_path(arg) {
            continue;
        }
        words.extend(spellings(arg));
    }
    words.join(" ")
}

// ---------------------------------------------------------------------------
// zsh
// ---------------------------------------------------------------------------

/// Escapes text for use inside a zsh `_arguments` specification.
///
/// `[`, `]` and `:` delimit the fields of a spec, so an unescaped one in a
/// help string silently truncates the entry or corrupts the one after it.
fn zsh_escape(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('\'', r"'\''")
        .replace('[', r"\[")
        .replace(']', r"\]")
        .replace(':', r"\:")
}

fn render_zsh(app: &Command) -> String {
    let bin = app.get_name();
    let mut out = String::new();
    let _ = writeln!(out, "#compdef {bin}");
    let _ =
        writeln!(out, "# {bin} completion for zsh. Generated — do not edit.");
    let _ = writeln!(out);
    let _ = writeln!(out, "_{bin}() {{");
    let _ = writeln!(out, "    local curcontext=\"$curcontext\" state line");
    let _ = writeln!(out, "    local -a commands");
    let _ = writeln!(out, "    commands=(");
    for sub in app.get_subcommands() {
        let about = sub
            .get_about()
            .map(|a| a.to_string().replace('\n', " "))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "        '{}:{}'",
            sub.get_name(),
            zsh_escape(&about)
        );
    }
    let _ = writeln!(out, "    )");
    let _ = writeln!(out);
    let _ = writeln!(out, "    _arguments -C \\");
    for arg in visible_args(app) {
        let _ = writeln!(out, "        {} \\", zsh_arg_spec(arg));
    }
    let _ = writeln!(out, "        '1: :->command' \\");
    let _ = writeln!(out, "        '*:: :->args' && return 0");
    let _ = writeln!(out);
    let _ = writeln!(out, "    case $state in");
    let _ = writeln!(out, "        command)");
    let _ = writeln!(
        out,
        "            _describe -t commands '{bin} command' commands && return 0"
    );
    let _ = writeln!(out, "            ;;");
    let _ = writeln!(out, "        args)");
    let _ = writeln!(out, "            case $words[1] in");
    for sub in app.get_subcommands() {
        let _ = writeln!(out, "                {})", sub.get_name());
        let _ = writeln!(out, "                    _arguments \\");
        for arg in visible_args(sub) {
            let _ = writeln!(
                out,
                "                        {} \\",
                zsh_arg_spec(arg)
            );
        }
        let _ = writeln!(out, "                        && return 0");
        let _ = writeln!(out, "                    ;;");
    }
    let _ = writeln!(out, "            esac");
    let _ = writeln!(out, "            ;;");
    let _ = writeln!(out, "    esac");
    let _ = writeln!(out, "    return 1");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
    let _ = writeln!(out, "_{bin} \"$@\"");
    out
}

/// One `_arguments` spec line for a single argument.
fn zsh_arg_spec(arg: &Arg) -> String {
    let names = spellings(arg);
    let help = zsh_escape(&help_of(arg));

    // The exclusion list stops zsh offering `--content` once `-c` is typed.
    let exclusion = if names.len() > 1 {
        format!("({})", names.join(" "))
    } else {
        String::new()
    };

    let action = if takes_value(arg) {
        let value = arg
            .get_value_names()
            .and_then(|n| n.first())
            .map_or_else(|| "VALUE".to_owned(), ToString::to_string);
        let completer = if takes_path(arg) { "_files" } else { " " };
        format!(":{}:{completer}", zsh_escape(&value))
    } else {
        String::new()
    };

    if names.len() > 1 {
        // `'(-c --content)'{-c,--content}'[help]:DIR:_files'`
        format!("'{exclusion}'{{{}}}'[{help}]{action}'", names.join(","))
    } else {
        format!("'{}[{help}]{action}'", names.join(""))
    }
}

// ---------------------------------------------------------------------------
// fish
// ---------------------------------------------------------------------------

/// Escapes text for a single-quoted fish string, where only `\` and `'`
/// are special.
fn fish_escape(s: &str) -> String {
    s.replace('\\', r"\\").replace('\'', r"\'")
}

fn render_fish(app: &Command) -> String {
    let bin = app.get_name();
    let mut out = String::new();
    let _ =
        writeln!(out, "# {bin} completion for fish. Generated — do not edit.");
    // Disable the default filename fallback; path options opt back in with
    // `-F` below, so a flag-only position never suggests the whole cwd.
    let _ = writeln!(out, "complete -c {bin} -f");
    let _ = writeln!(out);

    for arg in visible_args(app) {
        let _ = writeln!(
            out,
            "complete -c {bin} -n '__fish_use_subcommand' {}",
            fish_arg_spec(arg)
        );
    }
    for sub in app.get_subcommands() {
        let about = sub
            .get_about()
            .map(|a| a.to_string().replace('\n', " "))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "complete -c {bin} -n '__fish_use_subcommand' -a '{}' -d '{}'",
            sub.get_name(),
            fish_escape(&about)
        );
    }
    let _ = writeln!(out);
    for sub in app.get_subcommands() {
        let name = sub.get_name();
        for arg in visible_args(sub) {
            let _ = writeln!(
                out,
                "complete -c {bin} -n '__fish_seen_subcommand_from {name}' {}",
                fish_arg_spec(arg)
            );
        }
    }
    out
}

fn fish_arg_spec(arg: &Arg) -> String {
    let mut parts = Vec::new();
    if let Some(short) = arg.get_short() {
        parts.push(format!("-s {short}"));
    }
    if let Some(long) = arg.get_long() {
        parts.push(format!("-l {long}"));
    }
    if takes_value(arg) {
        // `-r` marks the option as requiring an argument; `-F` re-enables
        // the filename completion turned off by `complete -c ssg -f`.
        parts.push(if takes_path(arg) {
            "-r -F".to_owned()
        } else {
            "-r".to_owned()
        });
    }
    let help = help_of(arg);
    if !help.is_empty() {
        parts.push(format!("-d '{}'", fish_escape(&help)));
    }
    parts.join(" ")
}

// ---------------------------------------------------------------------------
// powershell
// ---------------------------------------------------------------------------

/// Escapes text for a single-quoted PowerShell string, where a literal
/// quote is written by doubling it.
fn ps_escape(s: &str) -> String {
    s.replace('\'', "''")
}

fn render_powershell(app: &Command) -> String {
    let bin = app.get_name();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# {bin} completion for PowerShell. Generated — do not edit."
    );
    let _ = writeln!(out, "using namespace System.Management.Automation");
    let _ =
        writeln!(out, "using namespace System.Management.Automation.Language");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Register-ArgumentCompleter -Native -CommandName '{bin}' -ScriptBlock {{"
    );
    let _ = writeln!(
        out,
        "    param($wordToComplete, $commandAst, $cursorPosition)"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "    $commandElements = $commandAst.CommandElements");
    let _ = writeln!(out, "    $command = @(");
    let _ = writeln!(out, "        '{bin}'");
    let _ = writeln!(
        out,
        "        for ($i = 1; $i -lt $commandElements.Count; $i++) {{"
    );
    let _ = writeln!(out, "            $element = $commandElements[$i]");
    let _ = writeln!(
        out,
        "            if ($element -isnot [StringConstantExpressionAst] -or"
    );
    let _ = writeln!(out, "                $element.StringConstantType -ne [StringConstantType]::BareWord -or");
    let _ = writeln!(out, "                $element.Value.StartsWith('-')) {{");
    let _ = writeln!(out, "                break");
    let _ = writeln!(out, "            }}");
    let _ = writeln!(out, "            $element.Value");
    let _ = writeln!(out, "        }}) -join ';'");
    let _ = writeln!(out);
    let _ = writeln!(out, "    $completions = @(switch ($command) {{");

    let _ = writeln!(out, "        '{bin}' {{");
    for arg in visible_args(app) {
        for line in ps_completion_results(arg) {
            let _ = writeln!(out, "            {line}");
        }
    }
    for sub in app.get_subcommands() {
        let about = sub
            .get_about()
            .map(|a| a.to_string().replace('\n', " "))
            .unwrap_or_default();
        let name = sub.get_name();
        let _ = writeln!(
            out,
            "            [CompletionResult]::new('{name}', '{name}', \
             [CompletionResultType]::ParameterValue, '{}')",
            ps_escape(&about)
        );
    }
    let _ = writeln!(out, "            break");
    let _ = writeln!(out, "        }}");

    for sub in app.get_subcommands() {
        let _ = writeln!(out, "        '{bin};{}' {{", sub.get_name());
        for arg in visible_args(sub) {
            for line in ps_completion_results(arg) {
                let _ = writeln!(out, "            {line}");
            }
        }
        let _ = writeln!(out, "            break");
        let _ = writeln!(out, "        }}");
    }

    let _ = writeln!(out, "    }})");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    $completions.Where{{ $_.CompletionText -like \"$wordToComplete*\" }} |"
    );
    let _ = writeln!(out, "        Sort-Object -Property ListItemText");
    let _ = writeln!(out, "}}");
    out
}

fn ps_completion_results(arg: &Arg) -> Vec<String> {
    let help = ps_escape(&help_of(arg));
    spellings(arg)
        .into_iter()
        .map(|name| {
            format!(
                "[CompletionResult]::new('{name}', '{name}', \
                 [CompletionResultType]::ParameterName, '{help}')"
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Cli;

    fn app() -> Command {
        Cli::subcommand_app()
    }

    #[test]
    fn shell_names_round_trip() {
        for sh in Shell::ALL {
            assert_eq!(Shell::parse(sh.name()), Some(sh));
        }
        assert_eq!(Shell::parse("BASH"), Some(Shell::Bash));
        assert_eq!(Shell::parse("tcsh"), None);
    }

    #[test]
    fn file_names_match_what_each_shell_looks_for() {
        assert_eq!(Shell::Bash.file_name("ssg"), "ssg");
        assert_eq!(Shell::Zsh.file_name("ssg"), "_ssg");
        assert_eq!(Shell::Fish.file_name("ssg"), "ssg.fish");
        assert_eq!(Shell::PowerShell.file_name("ssg"), "_ssg.ps1");
    }

    #[test]
    fn every_shell_produces_a_non_empty_script() {
        for sh in Shell::ALL {
            let script = render(&app(), sh);
            assert!(
                script.len() > 200,
                "{} produced a suspiciously short script",
                sh.name()
            );
        }
    }

    #[test]
    fn rendering_is_deterministic() {
        for sh in Shell::ALL {
            assert_eq!(
                render(&app(), sh),
                render(&app(), sh),
                "{} output varies between runs",
                sh.name()
            );
        }
    }

    #[test]
    fn zsh_escape_neutralises_every_spec_delimiter() {
        assert_eq!(zsh_escape("a[b]c:d"), r"a\[b\]c\:d");
        assert_eq!(zsh_escape(r"back\slash"), r"back\\slash");
        assert_eq!(zsh_escape("it's"), r"it'\''s");
    }

    #[test]
    fn fish_escape_handles_quotes_and_backslashes() {
        assert_eq!(fish_escape("it's"), r"it\'s");
        assert_eq!(fish_escape(r"a\b"), r"a\\b");
    }

    #[test]
    fn powershell_escape_doubles_quotes() {
        assert_eq!(ps_escape("it's"), "it''s");
    }

    #[test]
    fn path_arguments_are_detected_from_the_value_parser() {
        let app = app();
        let build = app
            .get_subcommands()
            .find(|c| c.get_name() == "build")
            .expect("build subcommand");
        let content = build
            .get_arguments()
            .find(|a| a.get_id() == "content")
            .expect("--content");
        assert!(
            takes_path(content),
            "--content takes a PathBuf and must complete filenames"
        );
    }

    /// The regression this pins: `get_num_args()` is `None` for every
    /// argument in this parser, so a `takes_values()` reading marked
    /// `--content` — an obvious value-taking option — as a bare flag.
    #[test]
    fn value_taking_and_bare_flags_are_told_apart() {
        let app = app();
        let build = app
            .get_subcommands()
            .find(|c| c.get_name() == "build")
            .expect("build subcommand");
        let arg = |id: &str| {
            build
                .get_arguments()
                .find(|a| a.get_id() == id)
                .unwrap_or_else(|| panic!("--{id}"))
        };
        assert!(takes_value(arg("content")), "--content takes a directory");
        assert!(takes_value(arg("output")), "--output takes a directory");
        assert!(!takes_value(arg("drafts")), "--drafts is a bare flag");
        assert!(!takes_value(arg("quiet")), "--quiet is a bare flag");
    }
}
