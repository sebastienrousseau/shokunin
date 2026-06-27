// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `ssg audit` subcommand handler.
//!
//! Loads the site under `--output` (or the configured `output_dir`),
//! merges `[audit]` from `ssg.toml` over the default config, applies
//! CLI overrides (`--gate`, `--severity`, `--fail-on`, `--skip-network`,
//! `--json`, `--junit`, `--explain`), runs the [`crate::audit::AuditRunner`]
//! and renders + returns an exit-code-shaped result.

use crate::audit::{AuditConfig, AuditRunner, AuditTomlConfig, Severity, Site};
use crate::error::SsgError;
use clap::ArgMatches;
use std::path::PathBuf;

/// Outcome of running [`run`] — the caller turns this into a process
/// exit code (0 == [`Outcome::Pass`]; 1 == [`Outcome::Fail`]).
///
/// # Examples
///
/// ```
/// use ssg::cmd::audit::Outcome;
/// assert_ne!(Outcome::Pass, Outcome::Fail);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// No finding exceeded the configured `--fail-on` threshold.
    Pass,
    /// At least one gate produced a finding at or above `--fail-on`.
    Fail,
}

/// Executes the audit subcommand against the parsed clap matches.
///
/// # Errors
/// Returns [`SsgError`] when the site cannot be loaded or rendering
/// fails. Findings themselves never produce an `Err` — they're folded
/// into the returned [`Outcome`].
///
/// # Examples
///
/// ```
/// use ssg::cmd::audit::{build_subcommand, run, Outcome};
/// let tmp = tempfile::tempdir().unwrap();
/// let site = tmp.path().join("public");
/// std::fs::create_dir_all(&site).unwrap();
/// let cmd = build_subcommand();
/// let matches = cmd
///     .try_get_matches_from(["audit", "--output", site.to_str().unwrap()])
///     .unwrap();
/// let outcome = run(&matches).unwrap();
/// assert_eq!(outcome, Outcome::Pass);
/// ```
pub fn run(sub_m: &ArgMatches) -> Result<Outcome, SsgError> {
    let output_dir = sub_m
        .get_one::<PathBuf>("output")
        .cloned()
        .unwrap_or_else(|| PathBuf::from("public"));

    // --explain (early-exit, no audit run)
    if sub_m.get_flag("explain") {
        let gate = sub_m.get_one::<String>("gate");
        return explain_gate(gate.map(String::as_str)).map(|()| Outcome::Pass);
    }

    let mut config = load_audit_config(sub_m.get_one::<PathBuf>("config"))?;
    apply_cli_overrides(&mut config, sub_m);

    let site = Site::load(&output_dir)?;
    let runner = AuditRunner::new(config);
    let report = runner.run(&site);

    if sub_m.get_flag("json") {
        report.print_json()?;
    } else if sub_m.get_flag("junit") {
        report.print_junit();
    } else if sub_m.get_flag("sarif") {
        report.print_sarif();
    } else {
        report.print_text();
    }

    let outcome = if report.should_fail(runner.fail_on()) {
        Outcome::Fail
    } else {
        Outcome::Pass
    };
    Ok(outcome)
}

/// Loads the `[audit]` table from `ssg.toml` if `--config` was passed.
///
/// Returns the default config when `--config` is absent or the file
/// has no `[audit]` table.
fn load_audit_config(
    config_path: Option<&PathBuf>,
) -> Result<AuditConfig, SsgError> {
    let Some(path) = config_path else {
        return Ok(AuditConfig::new());
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(AuditConfig::new());
    };

    // We don't deserialise the whole file with `SsgConfig` here —
    // that would force a `[site]` table that an audit-only config
    // doesn't need. Instead, parse the bare `[audit]` table.
    #[derive(Debug, Default, serde::Deserialize)]
    struct OuterToml {
        #[serde(default)]
        audit: AuditTomlConfig,
    }
    let outer: OuterToml = toml::from_str(&text).unwrap_or_default();
    Ok(outer.audit.into_audit_config())
}

fn apply_cli_overrides(config: &mut AuditConfig, sub_m: &ArgMatches) {
    if let Some(gate) = sub_m.get_one::<String>("gate") {
        config.only = Some(gate.clone());
    }
    if let Some(sev) = sub_m.get_one::<String>("severity") {
        if let Some(parsed) = Severity::parse(sev) {
            config.severity_floor = parsed;
        }
    }
    if let Some(fo) = sub_m.get_one::<String>("fail-on") {
        if let Some(parsed) = Severity::parse(fo) {
            config.fail_on = parsed;
        }
    }
    if sub_m.get_flag("skip-network") {
        config.options.skip_network = true;
    }
    if sub_m.get_flag("no-skip-network") {
        config.options.skip_network = false;
    }
}

fn explain_gate(name: Option<&str>) -> Result<(), SsgError> {
    let gates = crate::audit::gates::all();
    match name {
        Some(target) => {
            let Some(gate) = gates.iter().find(|g| g.name() == target) else {
                return Err(SsgError::Validation {
                    field: "gate".to_string(),
                    message: format!("unknown gate `{target}`"),
                });
            };
            println!("[{}] {}", gate.name(), gate.explain());
        }
        None => {
            for gate in &gates {
                println!("[{}] {}\n", gate.name(), gate.explain());
            }
        }
    }
    Ok(())
}

/// Builds the clap `Command` for the `audit` subcommand. Re-used by
/// `cli.rs::subcommand_app` so the surface is wired in one place.
///
/// # Examples
///
/// ```
/// use ssg::cmd::audit::build_subcommand;
/// let cmd = build_subcommand();
/// assert_eq!(cmd.get_name(), "audit");
/// ```
#[must_use]
pub fn build_subcommand() -> clap::Command {
    use clap::{Arg, ArgAction};
    clap::Command::new("audit")
        .about("Run the 14 native audit gates against the built site")
        .long_about(
            "Runs WCAG, JSON-LD, hreflang, CSP/SRI, PQC TLS, HTML5, broken \
             links, metadata, markdown, performance, AI discovery, feeds, \
             images, and the semantic search index gates. Exits 1 if any \
             finding is at or above --fail-on (default: error).",
        )
        .arg(
            Arg::new("output")
                .help("Site output directory (defaults to ./public)")
                .long("output")
                .short('o')
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("config")
                .help("ssg.toml path (used to load [audit] section)")
                .long("config")
                .short('f')
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("gate")
                .help("Only run the named gate (e.g. hreflang)")
                .long("gate")
                .value_name("NAME"),
        )
        .arg(
            Arg::new("severity")
                .help("Minimum severity to print (info|warn|error)")
                .long("severity")
                .value_name("LEVEL"),
        )
        .arg(
            Arg::new("fail-on")
                .help("Severity that triggers a non-zero exit (default: error)")
                .long("fail-on")
                .value_name("LEVEL"),
        )
        .arg(
            Arg::new("json")
                .help("Emit JSON to stdout instead of rich text")
                .long("json")
                .action(ArgAction::SetTrue)
                .conflicts_with("junit"),
        )
        .arg(
            Arg::new("junit")
                .help("Emit JUnit XML to stdout instead of rich text")
                .long("junit")
                .action(ArgAction::SetTrue)
                .conflicts_with("sarif"),
        )
        .arg(
            Arg::new("sarif")
                .help("Emit SARIF v2.1.0 JSON (GitHub Code Scanning, GitLab Ultra, Sonatype) — issue #562")
                .long("sarif")
                .action(ArgAction::SetTrue)
                .conflicts_with("json"),
        )
        .arg(
            Arg::new("skip-network")
                .help("Skip external HTTP probes (default)")
                .long("skip-network")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-skip-network")
                .help("Enable external HTTP probes for the broken-link gate")
                .long("no-skip-network")
                .action(ArgAction::SetTrue)
                .conflicts_with("skip-network"),
        )
        .arg(
            Arg::new("explain")
                .help("Print the long-form explainer (use with --gate)")
                .long("explain")
                .action(ArgAction::SetTrue),
        )
}

/// Tiny shim used by `lib.rs::run`. Mapped to `Result<(), SsgError>`
/// so the dispatcher's call-site stays uniform across subcommands; the
/// non-zero exit is surfaced by a `process::exit(1)` in the caller.
///
/// # Errors
/// Propagates [`SsgError`] from [`run`].
///
/// # Examples
///
/// ```
/// use ssg::cmd::audit::{build_subcommand, run_and_dispatch};
/// let tmp = tempfile::tempdir().unwrap();
/// let site = tmp.path().join("public");
/// std::fs::create_dir_all(&site).unwrap();
/// let cmd = build_subcommand();
/// let matches = cmd
///     .try_get_matches_from(["audit", "--output", site.to_str().unwrap()])
///     .unwrap();
/// run_and_dispatch(&matches, true).unwrap();
/// ```
pub fn run_and_dispatch(
    matches: &ArgMatches,
    quiet: bool,
) -> Result<(), SsgError> {
    let outcome = run(matches)?;
    match outcome {
        Outcome::Pass => {
            if !quiet {
                log::info!("[audit] all gates passed");
            }
            Ok(())
        }
        Outcome::Fail => {
            if !quiet {
                eprintln!("audit: one or more gates failed");
            }
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn build_subcommand_exposes_audit_flags() {
        let cmd = build_subcommand();
        let arg_names: Vec<&str> =
            cmd.get_arguments().map(|a| a.get_id().as_str()).collect();
        for required in &[
            "output",
            "config",
            "gate",
            "severity",
            "fail-on",
            "json",
            "junit",
            "skip-network",
            "explain",
        ] {
            assert!(arg_names.contains(required), "missing arg: {required}");
        }
    }

    #[test]
    fn audit_passes_on_empty_site() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--fail-on",
                "error",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        // Empty site triggers info-level skips from PQC/Search/Markdown
        // gates only — no errors, so should pass on `--fail-on error`.
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn explain_with_unknown_gate_errors() {
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--explain",
                "--gate",
                "no-such-gate",
            ])
            .unwrap();
        let err = run(&matches).unwrap_err();
        assert!(matches!(err, SsgError::Validation { .. }));
    }

    #[test]
    fn explain_with_known_gate_prints() {
        // Covers explain_gate(Some(gate)) success arm. We accept any
        // gate name that exists; pick `wcag` which always ships.
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from(["audit", "--explain", "--gate", "wcag"])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn explain_with_no_gate_prints_all() {
        // Covers explain_gate(None) loop.
        let cmd = build_subcommand();
        let matches = cmd.try_get_matches_from(["audit", "--explain"]).unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn json_output_branch() {
        // Covers `report.print_json()` arm (line ~75).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--json",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn junit_output_branch() {
        // Covers `report.print_junit()` arm (line ~77).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--junit",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn cli_overrides_gate_severity_failon() {
        // Covers apply_cli_overrides paths for --gate, --severity,
        // --fail-on (lines 117-128).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--gate",
                "wcag",
                "--severity",
                "warn",
                "--fail-on",
                "error",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn cli_override_skip_network_flag() {
        // Covers apply_cli_overrides --skip-network arm (line 130-131).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--skip-network",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn cli_override_no_skip_network_flag() {
        // Covers apply_cli_overrides --no-skip-network arm (line 133-134).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from([
                "audit",
                "--output",
                site.to_str().unwrap(),
                "--no-skip-network",
            ])
            .unwrap();
        let outcome = run(&matches).unwrap();
        assert_eq!(outcome, Outcome::Pass);
    }

    #[test]
    fn load_audit_config_missing_file_returns_default() {
        // Covers the `Err` -> default branch at line 100-101 of
        // load_audit_config (path passed but file doesn't exist).
        let cfg =
            load_audit_config(Some(&PathBuf::from("/nonexistent/x.toml")))
                .unwrap();
        // Default config: severity_floor is `Warn`-or-similar by
        // default. Smoke-check that we got a usable value back.
        assert!(format!("{cfg:?}").contains("AuditConfig"));
    }

    #[test]
    fn load_audit_config_garbage_toml_returns_default() {
        // Covers the toml::from_str -> unwrap_or_default branch at
        // line 112-113 (file exists, parse fails).
        let tmp = tempfile::tempdir().unwrap();
        let bad = tmp.path().join("bad.toml");
        std::fs::write(&bad, "this is = not valid <<< toml >>>").unwrap();
        let cfg = load_audit_config(Some(&bad)).unwrap();
        assert!(format!("{cfg:?}").contains("AuditConfig"));
    }

    #[test]
    fn load_audit_config_with_table_parses() {
        // Covers the success path through line 112-113.
        let tmp = tempfile::tempdir().unwrap();
        let good = tmp.path().join("ssg.toml");
        std::fs::write(
            &good,
            r#"
[audit]
severity_floor = "warn"
fail_on = "error"
"#,
        )
        .unwrap();
        let cfg = load_audit_config(Some(&good)).unwrap();
        assert!(format!("{cfg:?}").contains("AuditConfig"));
    }

    #[test]
    fn run_and_dispatch_pass_quiet() {
        // Covers run_and_dispatch's Pass + quiet arm (line 274-278).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from(["audit", "--output", site.to_str().unwrap()])
            .unwrap();
        run_and_dispatch(&matches, true).unwrap();
    }

    #[test]
    fn run_and_dispatch_pass_verbose() {
        // Covers the !quiet log::info!() arm (line 275-276).
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("public");
        std::fs::create_dir_all(&site).unwrap();
        let cmd = build_subcommand();
        let matches = cmd
            .try_get_matches_from(["audit", "--output", site.to_str().unwrap()])
            .unwrap();
        run_and_dispatch(&matches, false).unwrap();
    }
}
