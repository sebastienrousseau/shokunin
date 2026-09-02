//! Asserts the shell completions cannot fall behind the parser, and that
//! each script is something its shell will actually accept.
//!
//! A broken completion script fails in the worst possible way: not at build
//! time, but on the user's next tab keypress, in a shell they have already
//! started, with an error that names our file. So there are two classes of
//! gate here.
//!
//! **Content.** Every flag and subcommand in the clap definition must reach
//! all four scripts, and no script may name a flag the parser lacks. This
//! is the same drift gate `tests/man_page.rs` applies to the man page.
//!
//! **Form.** Each script is fed to its real shell's syntax checker, when
//! that shell is installed. `zsh -n` is necessary but not sufficient for
//! zsh: an `_arguments` specification is a *string*, so a spec mangled by
//! bad escaping still parses as valid shell and only misbehaves at
//! completion time. The structural assertions below cover what `-n` cannot
//! see, and they are what catches an unescaped `:` or `[` in a help text.
//!
//! Driving a real completion through a pty was tried and abandoned: `zpty`
//! against an interactive zsh hangs more reliably than it completes, and a
//! flaky gate is worse than an honest narrower one.

use clap::{ArgAction, Command};
use ssg::cmd::completions::{render, Shell};
use ssg::cmd::Cli;
use std::collections::BTreeSet;
use std::process::Command as Proc;

fn script(shell: Shell) -> String {
    render(&Cli::subcommand_app(), shell)
}

/// Every visible long flag the parser accepts, across every subcommand.
fn parser_long_flags() -> BTreeSet<String> {
    let app = Cli::subcommand_app();
    let mut out = BTreeSet::new();
    let mut collect = |cmd: &Command| {
        for arg in cmd.get_arguments().filter(|a| !a.is_hide_set()) {
            if let Some(long) = arg.get_long() {
                let _ = out.insert(long.to_owned());
            }
        }
    };
    collect(&app);
    for sub in app.get_subcommands() {
        collect(sub);
    }
    out
}

/// The long flags a rendered script actually offers.
///
/// Each shell spells a long option differently — fish writes `-l content`
/// where the others write `--content` — so extraction has to be per-shell.
/// A single `--` scan looks like it works and silently finds nothing in the
/// fish script, turning both flag gates into no-ops for a quarter of the
/// output.
fn flags_offered_by(shell: Shell, script: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if shell == Shell::Fish {
        for line in script.lines() {
            // Stop at the description so a `-l` inside help text is not
            // mistaken for an option spelling.
            let spec = line.split(" -d '").next().unwrap_or(line);
            let mut rest = spec;
            while let Some(i) = rest.find("-l ") {
                let after = &rest[i + 3..];
                let flag: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                rest = &after[flag.len()..];
                if !flag.is_empty() {
                    let _ = out.insert(flag);
                }
            }
        }
        return out;
    }
    let mut rest = script;
    while let Some(i) = rest.find("--") {
        let after = &rest[i + 2..];
        let flag: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        rest = &after[flag.len()..];
        if !flag.is_empty() {
            let _ = out.insert(flag);
        }
    }
    out
}

#[test]
fn every_parser_flag_reaches_every_shell() {
    for shell in Shell::ALL {
        let offered = flags_offered_by(shell, &script(shell));
        let missing: Vec<String> = parser_long_flags()
            .into_iter()
            .filter(|f| !offered.contains(f))
            .collect();
        assert!(
            missing.is_empty(),
            "{} completions omit these flags:\n  {}",
            shell.name(),
            missing.join("\n  ")
        );
    }
}

#[test]
fn every_subcommand_reaches_every_shell() {
    let app = Cli::subcommand_app();
    let subs: Vec<&str> =
        app.get_subcommands().map(Command::get_name).collect();
    for shell in Shell::ALL {
        let script = script(shell);
        let missing: Vec<&&str> =
            subs.iter().filter(|s| !script.contains(**s)).collect();
        assert!(
            missing.is_empty(),
            "{} completions omit these subcommands: {missing:?}",
            shell.name()
        );
    }
}

#[test]
fn no_shell_names_a_flag_the_parser_lacks() {
    // The direction that catches a removed flag: a completion offering an
    // option that no longer parses sends the user straight to an error.
    let known = parser_long_flags();
    for shell in Shell::ALL {
        let offered = flags_offered_by(shell, &script(shell));
        // Every script must actually offer something, or the extraction is
        // broken and the assertion below passes without testing anything.
        assert!(
            !offered.is_empty(),
            "no flags were extracted from the {} script",
            shell.name()
        );
        let phantom: Vec<&String> =
            offered.iter().filter(|f| !known.contains(*f)).collect();
        assert!(
            phantom.is_empty(),
            "{} completions offer flags the parser does not accept: \
             {phantom:?}",
            shell.name()
        );
    }
}

/// Path-valued options must offer filename completion, in each shell's own
/// idiom. Without this the shell silently offers nothing where a path
/// belongs, which reads to the user as "this option takes no argument".
#[test]
fn path_options_offer_filename_completion() {
    assert!(
        script(Shell::Fish).contains("-l content -r -F"),
        "fish: --content must be marked -r (takes a value) and -F (files)"
    );
    assert!(
        script(Shell::Zsh).contains(":DIR:_files"),
        "zsh: path options must complete with _files"
    );
    assert!(
        script(Shell::Bash).contains("compgen -f"),
        "bash: path options must fall through to filename completion"
    );
}

/// Bare flags must *not* be marked as taking a value — the mirror of the
/// bug that `get_num_args()` hid, where every option looked like a flag.
#[test]
fn bare_flags_are_not_marked_as_taking_a_value() {
    let fish = script(Shell::Fish);
    assert!(
        fish.contains("-l drafts -d"),
        "fish: --drafts is a bare flag and must not be marked -r"
    );
    assert!(
        !fish.contains("-l drafts -r"),
        "fish: --drafts was marked as taking a value"
    );
}

/// An `_arguments` spec is a string, so `zsh -n` cannot see inside it.
/// These are the structural properties a mangled spec violates.
///
/// Counting brackets for *balance* is not enough, and that mistake was made
/// here first: `--config`'s help reads "used to load [audit] section", so
/// leaving it unescaped still yields two `[` and two `]` and a balance
/// check passes. zsh ends the description at the **first** unescaped `]`,
/// so the real property is exactly one of each — and likewise `:`, which
/// delimits the value and action fields, must appear exactly 0 times (a
/// bare flag) or 2 (an option with a value).
#[test]
fn zsh_argument_specs_are_structurally_sound() {
    /// Characters not preceded by a backslash.
    fn unescaped(spec: &str, want: char) -> usize {
        let mut n = 0;
        let mut chars = spec.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                let _ = chars.next();
            } else if c == want {
                n += 1;
            }
        }
        n
    }

    let script = script(Shell::Zsh);
    let mut checked = 0_usize;
    for line in script.lines() {
        let t = line.trim().trim_end_matches('\\').trim();
        // Spec lines are the ones carrying a bracketed description.
        if !t.starts_with('\'') || !t.contains('[') {
            continue;
        }
        checked += 1;
        assert_eq!(
            unescaped(t, '['),
            1,
            "a `[` from help text leaked into a zsh spec, which ends the \
             description early:\n  {t}"
        );
        assert_eq!(
            unescaped(t, ']'),
            1,
            "a `]` from help text leaked into a zsh spec, which ends the \
             description early:\n  {t}"
        );
        let colons = unescaped(t, ':');
        assert!(
            colons == 0 || colons == 2,
            "a zsh spec has {colons} unescaped colons; expected 0 (bare \
             flag) or 2 (value plus action):\n  {t}"
        );
    }
    assert!(
        checked > 20,
        "only {checked} zsh specs were examined — the line filter is \
         wrong and this gate is testing almost nothing"
    );
}

/// The generated scripts must be byte-identical run to run. A packager
/// that rebuilds gets the same file, and a diff in a release archive means
/// something really changed.
#[test]
fn every_shell_renders_deterministically() {
    for shell in Shell::ALL {
        assert_eq!(
            script(shell),
            script(shell),
            "{} output varies between runs",
            shell.name()
        );
    }
}

/// Feeds each script to the real shell's parser. Skipped, with a notice,
/// for any shell not installed on this machine — but never silently: a
/// gate that quietly tests nothing is the failure mode this whole file
/// exists to prevent.
#[test]
fn each_script_parses_in_its_own_shell() {
    let dir = std::env::temp_dir().join("ssg-completions-syntax");
    std::fs::create_dir_all(&dir).expect("temp dir");

    // (shell binary, args, script)
    let cases: [(&str, &[&str], Shell); 3] = [
        ("bash", &["-n"], Shell::Bash),
        ("zsh", &["-n"], Shell::Zsh),
        ("fish", &["--no-execute"], Shell::Fish),
    ];

    let mut checked = 0_usize;
    for (bin, args, shell) in cases {
        let path = dir.join(shell.file_name("ssg"));
        std::fs::write(&path, script(shell)).expect("write script");

        let Ok(out) = Proc::new(bin).args(args).arg(&path).output() else {
            eprintln!("note: {bin} is not installed; skipping its check");
            continue;
        };
        assert!(
            out.status.success(),
            "{bin} rejected the generated {} completion:\n{}",
            shell.name(),
            String::from_utf8_lossy(&out.stderr)
        );
        checked += 1;
    }

    // bash is present on every supported platform and every CI runner. If
    // even that was skipped, the environment is wrong and this test is
    // reporting a pass it did not earn.
    assert!(
        checked > 0,
        "no shell was available to syntax-check any script"
    );
}

/// The parser must expose the actions the emitter reads. If clap ever
/// stops reporting `ArgAction::Set` for value-taking options, every script
/// silently loses its argument markers — so this pins the assumption the
/// generator rests on rather than leaving it implicit.
#[test]
fn the_parser_still_reports_the_actions_the_emitter_reads() {
    let app = Cli::subcommand_app();
    let build = app
        .get_subcommands()
        .find(|c| c.get_name() == "build")
        .expect("build subcommand");
    let action = |id: &str| {
        build
            .get_arguments()
            .find(|a| a.get_id() == id)
            .map_or_else(|| panic!("--{id} is gone"), clap::Arg::get_action)
            .clone()
    };
    assert!(matches!(action("content"), ArgAction::Set));
    assert!(matches!(action("drafts"), ArgAction::SetTrue));
}
