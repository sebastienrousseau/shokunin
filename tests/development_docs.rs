//! Asserts DEVELOPMENT.md's "Reproducing CI locally" table matches
//! `.github/workflows/ci.yml`.
//!
//! The table's whole value is that a contributor can trust it verbatim.
//! An instruction to run `cargo clippy --lib` when CI runs
//! `--lib --tests --examples --all-features` is worse than no instruction,
//! because it produces a confident local green and a red pipeline — which
//! is exactly what happened repeatedly before this file existed.
//!
//! Two directions are checked, and both matter:
//!
//! * **No phantom commands.** Every command the table presents as a CI
//!   mirror must actually appear in the workflow. This catches the table
//!   going stale when a workflow step is edited.
//! * **No undocumented jobs.** Every job in the workflow must be named in
//!   the table. This catches a new gate being added with no local
//!   equivalent, which is how a job becomes folklore.
//!
//! The workflow is parsed as text rather than YAML so the test needs no
//! dependency: `cargo vet`'s exemption ratchet forbids adding one, and the
//! properties asserted here do not need a full parse.

use std::collections::BTreeSet;
use std::fs;

fn read(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    fs::read_to_string(format!("{root}/{path}"))
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// The commands in the "Reproducing CI locally" table: the contents of
/// every backtick span in the right-hand column.
fn documented_commands() -> Vec<String> {
    let doc = read("DEVELOPMENT.md");
    let start = doc
        .find("## Reproducing CI locally")
        .expect("DEVELOPMENT.md has no 'Reproducing CI locally' section");
    let rest = &doc[start..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 3);
    let table = &rest[..end];

    let mut out = Vec::new();
    for line in table.lines() {
        // Table rows only; skip the header and separator.
        if !line.starts_with('|') || line.contains("---") {
            continue;
        }
        let Some(cell) = line.split('|').nth(2) else {
            continue;
        };
        let mut rest = cell;
        while let Some(i) = rest.find('`') {
            let after = &rest[i + 1..];
            let Some(j) = after.find('`') else { break };
            out.push(after[..j].to_owned());
            rest = &after[j + 1..];
        }
    }
    assert!(
        out.len() > 10,
        "only {} commands were extracted from the CI table — the parser \
         is wrong and this gate is testing almost nothing",
        out.len()
    );
    out
}

/// Collapses whitespace so a command wrapped across YAML continuation
/// lines compares equal to the single-line form in the table.
fn normalise(s: &str) -> String {
    s.replace('\\', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn every_documented_command_appears_in_the_workflow() {
    let ci = normalise(&read(".github/workflows/ci.yml"));

    // Commands that are deliberately a local convenience wrapper rather
    // than a literal CI string. Each is justified, and the list is short
    // on purpose: it is an escape hatch, and a long one would defeat the
    // gate.
    let wrappers: BTreeSet<&str> = [
        // `make coverage` runs the same cargo-llvm-cov invocation the
        // workflow inlines, with LLVM_PROFILE_FILE pinned so profraw
        // files do not scatter (see scripts/repo-hygiene.sh).
        "make coverage",
        // The workflow passes an explicit stage directory; the script
        // defaults to a temporary one when run locally.
        "./scripts/install-smoke.sh",
    ]
    .into_iter()
    .collect();

    let missing: Vec<String> = documented_commands()
        .into_iter()
        .filter(|c| !wrappers.contains(c.as_str()))
        .filter(|c| !ci.contains(&normalise(c)))
        .collect();

    assert!(
        missing.is_empty(),
        "DEVELOPMENT.md documents these as CI commands, but they do not \
         appear in ci.yml:\n  {}\n\nEither the workflow changed and the \
         table is stale, or the command belongs in the wrapper list with \
         a reason.",
        missing.join("\n  ")
    );
}

#[test]
fn every_workflow_job_is_documented() {
    let ci = read(".github/workflows/ci.yml");

    // Job ids are the two-space-indented keys under `jobs:`.
    let mut in_jobs = false;
    let mut jobs = BTreeSet::new();
    for line in ci.lines() {
        if line.starts_with("jobs:") {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        if !line.starts_with("  ") || line.starts_with("   ") {
            continue;
        }
        let t = line.trim_end();
        if let Some(name) = t.strip_suffix(':') {
            let name = name.trim();
            if !name.is_empty() && !name.starts_with('#') {
                let _ = jobs.insert(name.to_owned());
            }
        }
    }
    assert!(
        jobs.len() >= 8,
        "only {} jobs were parsed from ci.yml — the parser is wrong",
        jobs.len()
    );

    // The table names jobs in prose ("fmt", "clippy (lib — strict)"), so
    // the check is that each job id's words are represented, not that the
    // id appears verbatim.
    let doc = read("DEVELOPMENT.md").to_lowercase();
    let undocumented: Vec<&String> = jobs
        .iter()
        .filter(|j| {
            let plain = j.replace('-', " ");
            !doc.contains(&j.to_lowercase())
                && !doc.contains(&plain.to_lowercase())
        })
        .collect();

    assert!(
        undocumented.is_empty(),
        "these CI jobs have no entry in DEVELOPMENT.md's table, so there \
         is no documented way to run them locally: {undocumented:?}"
    );
}

/// The scripts the table points at must exist and be executable —
/// a documented command that is not runnable is the same failure as a
/// wrong one.
#[test]
fn every_referenced_script_exists_and_is_executable() {
    let root = env!("CARGO_MANIFEST_DIR");
    for cmd in documented_commands() {
        let Some(path) = cmd.split_whitespace().next() else {
            continue;
        };
        if !path.starts_with("./") {
            continue;
        }
        let full = format!("{root}/{}", path.trim_start_matches("./"));
        let meta = fs::metadata(&full).unwrap_or_else(|e| {
            panic!("{path} is documented but missing: {e}")
        });

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            assert!(
                meta.permissions().mode() & 0o111 != 0,
                "{path} is documented as a command but is not executable"
            );
        }
        #[cfg(not(unix))]
        let _ = meta;
    }
}
