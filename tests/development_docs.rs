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
    // Every workflow, not just ci.yml. The table documents how to run
    // CI's gates locally, and some of those gates live elsewhere — the
    // fuzz regression replay is in fuzz.yml. Reading only ci.yml made
    // this reject a correctly documented command.
    //
    // The companion test below stays scoped to ci.yml on purpose: that
    // one asserts every *job* is documented, and the table covers the
    // per-push gates rather than release and scheduled machinery.
    let dir = format!("{}/.github/workflows", env!("CARGO_MANIFEST_DIR"));
    let mut all = String::new();
    let mut files = 0_usize;
    for entry in fs::read_dir(&dir).expect("workflows dir").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yml") {
            all.push_str(&fs::read_to_string(&path).unwrap_or_default());
            all.push('\n');
            files += 1;
        }
    }
    assert!(
        files >= 5,
        "only {files} workflow files were read — the glob is wrong and \
         this gate would accept almost anything"
    );
    let ci = normalise(&all);

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
        // CI runs these as pinned GitHub Actions
        // (`markdownlint-cli2-action`, `reuse-action`), which is what
        // keeps them pinned by SHA. These are the local equivalents,
        // reading the same `.markdownlint-cli2.jsonc` and `REUSE.toml`.
        "npx markdownlint-cli2",
        "reuse lint",
        // The CI replay loops over every `$OUT/*_seed_corpus.zip` the
        // OSS-Fuzz build produced. These are the single-target local
        // equivalents, with `<target>` as a placeholder — they cannot
        // appear literally in a workflow.
        "cargo +nightly fuzz build <target>",
        "./fuzz/target/*/release/<target> fuzz/corpus/<target> -runs=0",
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
    // Whole-word matching, not `contains`. A substring test passed the
    // job id `nix` because DEVELOPMENT.md says "Unix" — the gate
    // reported success while the job was genuinely undocumented. Short
    // job ids are exactly the ones a loose match lets through.
    let doc = read("DEVELOPMENT.md").to_lowercase();
    let words: BTreeSet<String> = doc
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect();

    let undocumented: Vec<&String> = jobs
        .iter()
        .filter(|j| {
            let id = j.to_lowercase();
            // Either the hyphenated id as one token ("docs-lint"), or
            // every word of it present ("docs" and "lint").
            !words.contains(&id)
                && !id.split('-').all(|part| words.contains(part))
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
        // Skip paths that are patterns rather than files: a glob for a
        // target triple, or a `<target>` placeholder the reader
        // substitutes. Those cannot resolve to one executable, and
        // asserting they do would reject a correctly written example.
        if path.contains('*') || path.contains('<') {
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

/// The workspace-layout table in `docs/ARCHITECTURE.md` must list every
/// crate the workspace actually contains.
///
/// REPO-STANDARD §8 asks for "a CI-checked table of which repo has
/// what, so the layout can't silently drift". Here the family is the
/// workspace rather than separate repositories, but the failure mode is
/// identical: a crate is added, the table is not updated, and the
/// document that tells a newcomer where things live quietly becomes
/// wrong. Deriving the check from `Cargo.toml` rather than restating
/// the list is the same discipline the plugin and gate counts follow.
#[test]
fn architecture_lists_every_workspace_crate() {
    let manifest = read("Cargo.toml");
    let doc = read("docs/ARCHITECTURE.md");

    let members: Vec<String> = manifest
        .split_once("members = [")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| {
            list.split(',')
                .map(|s| s.trim().trim_matches(['"', '\n', ' '].as_ref()))
                .filter(|s| s.starts_with("crates/"))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    assert!(
        members.len() >= 5,
        "only {} workspace members were parsed from Cargo.toml — the \
         parser is wrong and this gate is testing almost nothing",
        members.len()
    );

    // Scope the search to the workspace table, not the whole document.
    // Checking the document was the first version of this test, and it
    // passed a mutation that deleted a table row — because the crate was
    // also named in surrounding prose. A table gate has to read the
    // table.
    // Only the pipe-delimited rows, not the whole section. Taking the
    // section was the second version of this test, and it also passed
    // the mutation: the prose *after* the table names `ssg-mcp` too.
    let table: String = doc
        .split_once("## Workspace layout")
        .and_then(|(_, rest)| rest.split_once("\n## "))
        .map_or_else(String::new, |(t, _)| {
            t.lines()
                .filter(|l| l.trim_start().starts_with('|'))
                .collect::<Vec<_>>()
                .join("\n")
        });
    assert!(
        table.lines().filter(|l| l.starts_with('|')).count() >= 8,
        "the workspace table in docs/ARCHITECTURE.md could not be \
         located, so this gate would pass without reading anything"
    );

    let undocumented: Vec<&String> = members
        .iter()
        .filter(|m| {
            // Accept either the path (`crates/ssg-core`) or the crate
            // name (`ssg-core`), since the table names crates.
            let name = m.rsplit('/').next().unwrap_or(m);
            !table.contains(m.as_str()) && !table.contains(name)
        })
        .collect();

    assert!(
        undocumented.is_empty(),
        "docs/ARCHITECTURE.md's workspace table omits these crates: \
         {undocumented:?}\n\nA newcomer reading that table would not \
         know they exist."
    );
}

/// The prebuilt-target table in `docs/packaging.md` must match the
/// release workflow's build matrix.
///
/// REPO-STANDARD §4 asks for binaries across a target matrix, and §5
/// asks for packaging documentation addressed to distro maintainers. A
/// maintainer reading that table is deciding what to package; if the
/// workflow gains or loses a target and the table does not follow, they
/// are planning against a list that no longer exists.
///
/// Same discipline as every other inventory here: derive it from the
/// thing that produces the artefacts, never restate it.
#[test]
fn packaging_doc_lists_every_release_target() {
    let workflow = read(".github/workflows/release.yml");
    let doc = read("docs/packaging.md");

    let targets: BTreeSet<String> = workflow
        .lines()
        .filter_map(|l| l.trim().strip_prefix("- target: "))
        .map(str::to_owned)
        .collect();

    assert!(
        targets.len() >= 5,
        "only {} targets were parsed from release.yml — the parser is \
         wrong and this gate is testing almost nothing",
        targets.len()
    );

    let missing: Vec<&String> = targets
        .iter()
        .filter(|t| !doc.contains(t.as_str()))
        .collect();

    assert!(
        missing.is_empty(),
        "docs/packaging.md does not list these release targets: \
         {missing:?}\n\nA distro maintainer reads that table to decide \
         what to package."
    );
}
