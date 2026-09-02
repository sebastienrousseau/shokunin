//! Keeps the README's numeric claims tied to the code that produces them.
//!
//! Every figure checked here had drifted at least once. At v0.0.57 the README
//! simultaneously advertised a "33-plugin pipeline" in three places and "38
//! plugins" in the capability matrix, while the pipeline registered 32 — and
//! the capability matrix was headed "v0.0.47" against a 0.0.57 crate. None of
//! it was wrong when written; it went stale because nothing checked it.
//!
//! A number a reader can verify in ten seconds, that turns out to be wrong, is
//! more expensive than no number at all: it invites the question of what else
//! is stale. So the counts are asserted against their sources rather than
//! maintained by hand.
//!
//! When one of these fails, fix the README — do not relax the test. The code is
//! the source of truth; the prose is the copy.

use ssg::audit::{AuditConfig, AuditRunner};
use ssg::cmd::SsgConfig;
use ssg::plugin::PluginManager;
use std::fs;

/// Reads the README from the crate root, independent of the test's cwd.
fn readme() -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The number of plugins the default pipeline registers.
fn registered_plugin_count() -> usize {
    let config = SsgConfig::default();
    let mut plugins = PluginManager::new();
    ssg::pipeline::register_default_plugins(&mut plugins, &config, false, None);
    plugins.inventory().len()
}

#[test]
fn readme_plugin_count_matches_the_pipeline() {
    let actual = registered_plugin_count();
    let text = readme();

    // Any "<n> plugin" / "<n>-plugin" claim must name the real number.
    let claimed: Vec<usize> = regex_like_counts(&text, "plugin");
    assert!(
        !claimed.is_empty(),
        "README no longer states a plugin count; the claim is what this guards"
    );
    for n in &claimed {
        assert_eq!(
            *n, actual,
            "README claims {n} plugins; the pipeline registers {actual}. \
             Run `ssg plugins list` and correct the README."
        );
    }
}

#[test]
fn readme_audit_gate_count_matches_the_runner() {
    let actual = AuditRunner::new(AuditConfig::new()).gate_names().len();
    let text = readme();

    let claimed: Vec<usize> = regex_like_counts(&text, "gate");
    assert!(
        !claimed.is_empty(),
        "README no longer states an audit gate count"
    );
    for n in &claimed {
        assert_eq!(
            *n, actual,
            "README claims {n} audit gates; the runner registers {actual}"
        );
    }
}

#[test]
fn readme_version_matches_the_crate() {
    let version = env!("CARGO_PKG_VERSION");
    let text = readme();

    // The dependency snippet is the one a reader copies, so it has to be right.
    let dep = format!("ssg = \"{version}\"");
    assert!(
        text.contains(&dep),
        "README's dependency snippet does not say `{dep}`; the crate is at \
         {version}"
    );

    // And the badge, which is the first thing on the page.
    let badge = format!("lib.rs-v{version}");
    assert!(
        text.contains(&badge),
        "README lib.rs badge does not show v{version}"
    );
}

/// Collects every `<number> <unit>` / `<number>-<unit>` claim in the text.
///
/// Deliberately hand-rolled: pulling `regex` into dev-dependencies for three
/// assertions is a worse trade than twenty lines here, and this crate keeps its
/// dependency budget deliberately small.
fn regex_like_counts(text: &str, unit: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for (idx, _) in text.match_indices(unit) {
        // Walk back over the separator (space or hyphen) then the digits.
        let before = &text[..idx];
        let before = before.strip_suffix([' ', '-']).unwrap_or(before);
        let digits: String = before
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if digits.is_empty() {
            continue;
        }
        // Skip version-like contexts ("v0.0.47 capability"), where the digits
        // are part of a version string rather than a count.
        let head = &before[..before.len() - digits.len()];
        if head.ends_with('.') || head.ends_with('v') {
            continue;
        }
        if let Ok(n) = digits.parse::<usize>() {
            out.push(n);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

#[test]
fn benchmarks_doc_states_the_current_version() {
    // BENCHMARKS.md described "SSG v0.0.45" while the crate was at v0.0.58 —
    // thirteen releases of drift in the document whose entire purpose is to
    // make performance claims checkable.
    //
    // Only the present-tense claims are checked. Historical references (for
    // instance "PR #583 for the v0.0.45 trajectory") are correct as written
    // and must not be rewritten.
    let version = env!("CARGO_PKG_VERSION");
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("BENCHMARKS.md");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));

    let claims = [
        format!("CI gates for SSG **v{version}**"),
        format!("| Capability | SSG v{version} |"),
        format!("| Metric | Floor | Current (v{version}) | Headroom |"),
    ];
    for claim in &claims {
        assert!(
            text.contains(claim.as_str()),
            "BENCHMARKS.md is missing the current-version claim: {claim}"
        );
    }
}

#[test]
fn readme_module_table_lists_every_public_module() {
    // The table was headed "All 38 modules" while listing 49 rows against 63
    // public modules — three numbers, none of them matching. Same class of
    // drift as the plugin count, and the reason a reader cannot use the table
    // as an index: fourteen modules were simply absent.
    //
    // `bench_corpus` is excluded: it is `#[cfg(any(test, feature =
    // "benchmark"))]` and so is not part of the surface a normal build
    // exposes.
    let lib = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"),
    )
    .expect("src/lib.rs is readable");

    let mut public: Vec<String> = Vec::new();
    let mut gated = false;
    for line in lib.lines() {
        let t = line.trim();
        if t.starts_with("#[cfg(") {
            gated = true;
            continue;
        }
        // `pub use crate::a::{b, c};` re-exports items, not modules, and
        // `pub use crate::a::b as c;` renames one. Neither names a module the
        // table should index, so both are skipped rather than mis-parsed.
        let name = t
            .strip_prefix("pub mod ")
            .and_then(|r| r.strip_suffix(';'))
            .map(str::to_string)
            .or_else(|| {
                let rest =
                    t.strip_prefix("pub use crate::")?.strip_suffix(';')?;
                if rest.contains('{') || rest.contains(" as ") {
                    return None;
                }
                rest.rsplit("::").next().map(str::to_string)
            });
        if let Some(name) = name {
            if !gated {
                public.push(name);
            }
        }
        if !t.is_empty() {
            gated = false;
        }
    }
    assert!(
        !public.is_empty(),
        "parsed no public modules from src/lib.rs"
    );

    let text = readme();
    let missing: Vec<&String> = public
        .iter()
        .filter(|m| !text.contains(&format!("| `{m}` |")))
        .collect();

    assert!(
        missing.is_empty(),
        "these public modules are absent from the README module table:\n  {}\n\
         The table is presented as an index; a module missing from it is \
         undiscoverable.",
        missing
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn readme_module_count_matches_the_table() {
    // The heading is a claim about the table directly beneath it.
    let text = readme();
    // Count only inside the collapsed module table; the README has several
    // other tables whose rows open the same way.
    let start = text
        .find("modules</b></summary>")
        .expect("README has a module-table heading");
    let end = text[start..]
        .find("</details>")
        .map_or(text.len(), |i| start + i);
    let rows = text[start..end]
        .lines()
        .filter(|l| l.starts_with("| `"))
        .count();

    let heading = text
        .lines()
        .find(|l| l.contains("modules</b></summary>"))
        .expect("README has a module-table heading");
    let claimed: usize = heading
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .expect("heading states a number");

    assert_eq!(
        claimed, rows,
        "the module table is headed {claimed} but lists {rows} rows"
    );
}

#[test]
fn readme_install_snippets_name_the_current_version() {
    // The `.deb` snippet said 0.0.47 against a 0.0.58 crate — eleven releases
    // stale, in the one line a reader copies and runs. The elevation plan
    // calls this out by name; the repo standard requires install-snippet
    // versions to be CI-checked against the manifest.
    //
    // Any `x.y.z` appearing in an install command must be the crate version.
    let version = env!("CARGO_PKG_VERSION");
    let text = readme();

    let mut stale = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let is_install = t.starts_with("sudo dpkg")
            || t.starts_with("sudo rpm")
            || t.starts_with("cargo install")
            || t.starts_with("brew install")
            || t.contains("releases/download/");
        if !is_install {
            continue;
        }
        // Pull out anything shaped like a version and compare.
        for tok in t.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
            if tok.matches('.').count() == 2
                && tok.split('.').all(|p| {
                    !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())
                })
                && tok != version
            {
                stale.push(format!("{t}  (found {tok}, crate is {version})"));
            }
        }
    }

    assert!(
        stale.is_empty(),
        "install snippets name a version other than the crate's:\n  {}",
        stale.join("\n  ")
    );
}
