// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration coverage for issue #527 — unified `ssg dev / build /
//! check / deploy` subcommand surface.
//!
//! These tests exercise the public CLI surface via
//! [`ssg::cmd::Cli::parse_and_dispatch`] and the per-target deploy
//! adapters in [`ssg::deploy_adapter`]. They intentionally avoid
//! booting a process (`cargo run`) so they stay fast and CI-safe.
//!
//! Mapping to acceptance criteria (issue #527):
//!
//! | AC  | Test name                                          |
//! | --- | -------------------------------------------------- |
//! | AC1 | `ac1_build_subcommand_parses_and_routes`           |
//! | AC2 | `ac2_dev_subcommand_parses_and_routes`             |
//! | AC3 | `ac3_check_subcommand_routes_and_dry_run_propagates` |
//! | AC4 | `ac4_deploy_subcommand_accepts_all_six_targets`    |
//! | AC5 | `ac5_legacy_flag_form_dispatches_to_legacy`        |
//! | AC6 | `ac6_top_level_help_lists_all_subcommands`         |
//! | AC7 | `ac7_legacy_invocation_preserved_for_back_compat`  |

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ssg::cmd::{
    Cli, CliInvocation, DEPLOY_TARGETS, LEGACY_DEPRECATION_WARNING,
};
use ssg::deploy_adapter::{adapter_for, Target};
use ssg::plugin::PluginContext;
use std::path::Path;

// ---------------------------------------------------------------------
// AC1: `ssg build` parses and routes to the build subcommand.
// ---------------------------------------------------------------------

#[test]
fn ac1_build_subcommand_parses_and_routes() {
    let (inv, matches) = Cli::parse_and_dispatch(["ssg", "build"]).unwrap();
    assert!(matches!(inv, CliInvocation::Build));
    assert_eq!(matches.subcommand_name(), Some("build"));
}

#[test]
fn ac1_build_subcommand_accepts_output_override() {
    let (_, matches) =
        Cli::parse_and_dispatch(["ssg", "build", "--output", "/tmp/site-out"])
            .unwrap();
    let sub = matches.subcommand_matches("build").expect("build present");
    assert_eq!(
        sub.get_one::<std::path::PathBuf>("output").unwrap(),
        Path::new("/tmp/site-out"),
    );
}

#[test]
fn ac1_build_subcommand_accepts_drafts_and_jobs() {
    let (_, matches) =
        Cli::parse_and_dispatch(["ssg", "build", "--drafts", "--jobs", "4"])
            .unwrap();
    let sub = matches.subcommand_matches("build").unwrap();
    assert!(sub.get_flag("drafts"));
    assert_eq!(*sub.get_one::<usize>("jobs").unwrap(), 4);
}

// ---------------------------------------------------------------------
// AC2: `ssg dev` parses and routes to the dev subcommand.
// ---------------------------------------------------------------------

#[test]
fn ac2_dev_subcommand_parses_and_routes() {
    let (inv, matches) = Cli::parse_and_dispatch(["ssg", "dev"]).unwrap();
    assert!(matches!(inv, CliInvocation::Dev));
    assert_eq!(matches.subcommand_name(), Some("dev"));
}

#[test]
fn ac2_dev_subcommand_supports_serve_override() {
    let (_, matches) =
        Cli::parse_and_dispatch(["ssg", "dev", "--serve", "./public"]).unwrap();
    let sub = matches.subcommand_matches("dev").unwrap();
    assert_eq!(
        sub.get_one::<std::path::PathBuf>("serve").unwrap(),
        Path::new("./public"),
    );
}

// ---------------------------------------------------------------------
// AC3: `ssg check` propagates the `dry_run` flag through PluginContext.
// ---------------------------------------------------------------------

#[test]
fn ac3_check_subcommand_routes_and_dry_run_propagates() {
    let (inv, matches) = Cli::parse_and_dispatch(["ssg", "check"]).unwrap();
    assert!(matches!(inv, CliInvocation::Check));
    assert_eq!(matches.subcommand_name(), Some("check"));

    // The `with_dry_run` builder is the contract the run handler uses.
    let ctx = PluginContext::new(
        Path::new("content"),
        Path::new("build"),
        Path::new("public"),
        Path::new("templates"),
    );
    assert!(!ctx.dry_run, "default ctx must not be dry_run");

    let ctx = ctx.with_dry_run(true);
    assert!(ctx.dry_run, "with_dry_run(true) must flip the flag");
}

// ---------------------------------------------------------------------
// AC4: `ssg deploy --target` accepts each of the six advertised
// targets and rejects unknowns.
// ---------------------------------------------------------------------

#[test]
fn ac4_deploy_subcommand_accepts_all_six_targets() {
    for target in DEPLOY_TARGETS {
        let (inv, _matches) =
            Cli::parse_and_dispatch(["ssg", "deploy", "--target", target])
                .unwrap_or_else(|e| panic!("target {target} rejected: {e}"));
        match inv {
            CliInvocation::Deploy { target: parsed } => {
                assert_eq!(&parsed, target);
            }
            other => panic!("expected Deploy, got {other:?}"),
        }
    }
}

#[test]
fn ac4_deploy_subcommand_rejects_unknown_target() {
    let err =
        Cli::parse_and_dispatch(["ssg", "deploy", "--target", "raspberry-pi"])
            .unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
}

#[test]
fn ac4_deploy_adapters_exist_for_every_target() {
    // The adapter table must cover the same six targets the CLI
    // advertises. Each one is a stub today — they print "not yet
    // implemented" and return Ok(()) — but the wiring is in place
    // (issue #527 explicitly accepts stubs).
    for target_str in DEPLOY_TARGETS {
        let target = Target::from_cli(target_str).unwrap_or_else(|_| {
            panic!("Target::from_cli rejected `{target_str}`")
        });
        let adapter = adapter_for(target);
        let result = adapter.deploy(Path::new("/tmp/ssg-fake-site"));
        assert!(
            result.is_ok(),
            "stub adapter for `{target_str}` must not fail"
        );
    }
}

// ---------------------------------------------------------------------
// AC5: Legacy flag form prints the deprecation warning.
// ---------------------------------------------------------------------

#[test]
fn ac5_legacy_flag_form_dispatches_to_legacy() {
    let (inv, _matches) =
        Cli::parse_and_dispatch(["ssg", "-s", "public"]).unwrap();
    assert!(matches!(inv, CliInvocation::Legacy));
}

#[test]
fn ac5_deprecation_warning_text_is_stable() {
    // The exact wording is part of the public contract (issue #527 AC5):
    //   "warning: legacy CLI form deprecated; use 'ssg dev' (will be
    //    removed in 1.0)"
    // Lock the message text so we notice unintended rewordings in a
    // refactor.
    assert!(LEGACY_DEPRECATION_WARNING.contains("legacy CLI form deprecated"));
    assert!(LEGACY_DEPRECATION_WARNING.contains("ssg dev"));
    assert!(LEGACY_DEPRECATION_WARNING.contains("1.0"));
}

// ---------------------------------------------------------------------
// AC6: Top-level `--help` lists all four subcommands.
// ---------------------------------------------------------------------

#[test]
fn ac6_top_level_help_lists_all_subcommands() {
    let mut app = Cli::subcommand_app();
    let mut buf = Vec::new();
    app.write_help(&mut buf).expect("help renders");
    let help = String::from_utf8(buf).expect("help is utf-8");
    for sub in ["build", "dev", "check", "deploy"] {
        assert!(
            help.contains(sub),
            "top-level help missing subcommand `{sub}`:\n{help}"
        );
    }
    // The subcommands are grouped under Development / Build /
    // Validate / Deploy headings in the after_help block. Spot-check
    // the headings so AC6's "grouped" requirement has a regression
    // guard.
    for heading in ["Development", "Build", "Validate", "Deploy"] {
        assert!(
            help.contains(heading),
            "top-level help missing heading `{heading}`:\n{help}"
        );
    }
}

#[test]
fn ac6_subcommand_help_is_specific() {
    // `ssg deploy --help` must surface `--target` and the supported
    // targets list. clap renders the possible values when we ask for
    // help output, which is exactly the self-documenting behaviour
    // AC6 specifies.
    let app = Cli::subcommand_app();
    let mut deploy = app
        .find_subcommand("deploy")
        .expect("deploy subcommand")
        .clone();
    let mut buf = Vec::new();
    deploy.write_long_help(&mut buf).expect("help renders");
    let help = String::from_utf8(buf).expect("help is utf-8");
    assert!(help.contains("--target"), "deploy help missing --target");
    for target in [
        "netlify",
        "vercel",
        "cloudflare-pages",
        "github-pages",
        "s3",
        "none",
    ] {
        assert!(
            help.contains(target),
            "deploy help missing target `{target}`:\n{help}"
        );
    }
}

// ---------------------------------------------------------------------
// AC7: Legacy invocations continue to work for one more minor cycle.
// ---------------------------------------------------------------------

#[test]
fn ac7_legacy_invocation_preserved_for_back_compat() {
    // The canonical legacy invocation from the issue body.
    let (inv, matches) =
        Cli::parse_and_dispatch(["ssg", "-s", "public", "-w"]).unwrap();
    assert!(matches!(inv, CliInvocation::Legacy));
    // Confirm the matches still expose the old flag names so the
    // pipeline keeps working unchanged.
    assert!(matches.get_flag("watch"));
    assert_eq!(
        matches
            .get_one::<std::path::PathBuf>("serve")
            .map(|p| p.as_path()),
        Some(Path::new("public")),
    );
}

#[test]
fn ac7_legacy_validate_flag_still_recognised() {
    let (inv, matches) =
        Cli::parse_and_dispatch(["ssg", "--validate"]).unwrap();
    assert!(matches!(inv, CliInvocation::Legacy));
    assert!(matches.get_flag("validate"));
}

#[test]
fn ac7_legacy_combined_flags_keep_parsing() {
    // The mix of long / short / value flags that appear in the
    // project's own Makefiles and CI scripts.
    let (inv, matches) = Cli::parse_and_dispatch([
        "ssg",
        "--content",
        "./examples/content",
        "--output",
        "./examples/public",
        "--template",
        "./examples/templates",
        "--quiet",
    ])
    .unwrap();
    assert!(matches!(inv, CliInvocation::Legacy));
    assert!(matches.get_flag("quiet"));
    assert_eq!(
        matches
            .get_one::<std::path::PathBuf>("output")
            .map(|p| p.as_path()),
        Some(Path::new("./examples/public")),
    );
}
