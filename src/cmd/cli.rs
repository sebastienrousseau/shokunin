// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CLI argument parsing and banner display.

use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default)]
/// A simple CLI struct for building the SSG command.
pub struct Cli;

impl Cli {
    /// Creates the command-line interface.
    #[must_use]
    pub fn build() -> Command {
        Command::new(env!("CARGO_PKG_NAME"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::new("config")
                    .help("Configuration file path")
                    .long("config")
                    .short('f')
                    .value_name("FILE")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("new")
                    .help("Create new project")
                    .long("new")
                    .short('n')
                    .value_name("NAME")
                    .value_parser(clap::value_parser!(String)), // Change from PathBuf to String
            )
            .arg(
                Arg::new("content")
                    .help("Content directory")
                    .long("content")
                    .short('c')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("output")
                    .help("Output directory")
                    .long("output")
                    .short('o')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("template")
                    .help("Template directory")
                    .long("template")
                    .short('t')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("serve")
                    .help("Development server directory")
                    .long("serve")
                    .short('s')
                    .value_name("DIR")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("watch")
                    .help("Watch for changes")
                    .long("watch")
                    .short('w')
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("drafts")
                    .help("Include draft pages in the build")
                    .long("drafts")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("deploy")
                    .help("Generate deployment config (netlify, vercel, cloudflare, github)")
                    .long("deploy")
                    .value_name("TARGET")
                    .value_parser(clap::value_parser!(String)),
            )
            .arg(
                Arg::new("validate")
                    .help("Validate content schemas without building")
                    .long("validate")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("quiet")
                    .help("Suppress non-error output")
                    .long("quiet")
                    .short('q')
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("verbose")
                    .help("Show detailed build information")
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("jobs")
                    .help("Number of parallel threads (default: num CPUs)")
                    .long("jobs")
                    .short('j')
                    .value_name("N")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                Arg::new("max-memory")
                    .help("Peak memory budget in MB for streaming compilation (default: 512)")
                    .long("max-memory")
                    .value_name("MB")
                    .value_parser(clap::value_parser!(usize)),
            )
    }

    /// Displays the application banner
    pub fn print_banner() {
        let version = env!("CARGO_PKG_VERSION");
        let mut title = String::with_capacity(16 + version.len());
        title.push_str("SSG \u{1f980} v");
        title.push_str(version);

        let description =
            "A Fast and Flexible Static Site Generator written in Rust";
        let width = title.len().max(description.len()) + 4;
        let line = "\u{2500}".repeat(width - 2);

        println!("\n\u{250c}{line}\u{2510}");
        println!(
            "\u{2502}{:^width$}\u{2502}",
            format!("\x1b[1;32m{title}\x1b[0m"),
            width = width - 3
        );
        println!("\u{251c}{line}\u{2524}");
        println!(
            "\u{2502}{:^width$}\u{2502}",
            format!("\x1b[1;34m{description}\x1b[0m"),
            width = width - 2
        );
        println!("\u{2514}{line}\u{2518}\n");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {

    use super::*;

    #[test]
    fn test_banner_display() {
        let version = env!("CARGO_PKG_VERSION");
        let title = format!("SSG \u{1f980} v{version}");
        let description =
            "A Fast and Flexible Static Site Generator written in Rust";
        let width = title.len().max(description.len()) + 4;
        let line = "\u{2500}".repeat(width - 2);

        Cli::print_banner();

        assert!(!line.is_empty());
        assert!(title.contains("SSG"));
        assert!(title.contains(version));
    }

    #[test]
    fn build_returns_valid_command() {
        let cmd = Cli::build();
        assert_eq!(cmd.get_name(), env!("CARGO_PKG_NAME"));
        // Ensure all expected arguments are registered
        let arg_names: Vec<&str> =
            cmd.get_arguments().map(|a| a.get_id().as_str()).collect();
        for expected in [
            "config", "new", "content", "output", "template", "serve", "watch",
            "drafts", "deploy", "validate", "quiet", "verbose", "jobs",
        ] {
            assert!(
                arg_names.contains(&expected),
                "missing expected arg: {expected}"
            );
        }
    }

    #[test]
    fn parse_minimal_args() {
        let cmd = Cli::build();
        let matches = cmd.try_get_matches_from(["ssg"]).unwrap();
        // No arguments supplied — all should be absent / false
        assert!(matches.get_one::<PathBuf>("config").is_none());
        assert!(matches.get_one::<PathBuf>("output").is_none());
        assert!(!matches.get_flag("watch"));
        assert!(!matches.get_flag("drafts"));
    }

    #[test]
    fn parse_quiet_flag() {
        let cmd = Cli::build();
        let matches = cmd.try_get_matches_from(["ssg", "--quiet"]).unwrap();
        assert!(matches.get_flag("quiet"));
    }

    #[test]
    fn parse_verbose_flag() {
        let cmd = Cli::build();
        let matches = cmd.try_get_matches_from(["ssg", "--verbose"]).unwrap();
        assert!(matches.get_flag("verbose"));
    }

    #[test]
    fn parse_drafts_flag() {
        let cmd = Cli::build();
        let matches = cmd.try_get_matches_from(["ssg", "--drafts"]).unwrap();
        assert!(matches.get_flag("drafts"));
    }

    #[test]
    fn parse_combined_flags_and_values() {
        let cmd = Cli::build();
        let matches = cmd
            .try_get_matches_from([
                "ssg", "--quiet", "--drafts", "--output", "/tmp/out", "--jobs",
                "4",
            ])
            .unwrap();
        assert!(matches.get_flag("quiet"));
        assert!(matches.get_flag("drafts"));
        assert_eq!(
            matches.get_one::<PathBuf>("output").unwrap(),
            &PathBuf::from("/tmp/out")
        );
        assert_eq!(*matches.get_one::<usize>("jobs").unwrap(), 4);
    }

    #[test]
    fn cli_default_is_unit_struct() {
        let _cli = Cli;
        // Cli is a ZST — just ensure Default works.
    }
}
