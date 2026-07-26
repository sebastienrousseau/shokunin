// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Integration tests for `ssg::cmd::Cli`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use ssg::cmd::Cli;

#[test]
fn cli_build_returns_a_clap_command() {
    let cmd = Cli::build();
    assert_eq!(cmd.get_name(), "ssg");
}
