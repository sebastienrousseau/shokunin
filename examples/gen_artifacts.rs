// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Writes the packaging artefacts derived from the CLI definition: the
//! `ssg.1` man page and a completion script for each supported shell.
//!
//! Run by `make man`, `make completions` and `make install`, so a release
//! never ships a hand-written `.1` that has drifted from `--help`.
//!
//! ```sh
//! cargo run --quiet --example gen-artifacts -- target/dist
//! ```
//!
//! The build date is taken from `SOURCE_DATE_EPOCH` when set, so
//! reproducible-build environments and distribution packagers get byte
//! identical output; otherwise it falls back to the crate's release date
//! rather than today, for the same reason.

use ssg::cmd::completions::{self, Shell};
use ssg::cmd::{man, Cli};
use std::path::PathBuf;
use std::{env, fs};

/// Date used when `SOURCE_DATE_EPOCH` is unset. A fixed value keeps the
/// output reproducible; a page dated "today" differs on every build.
const FALLBACK_DATE: &str = "2026-09-02";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out: PathBuf = env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("target/dist"), PathBuf::from);

    let man_dir = out.join("man");
    let comp_dir = out.join("completions");
    fs::create_dir_all(&man_dir)?;
    fs::create_dir_all(&comp_dir)?;

    let app = Cli::subcommand_app();
    let bin = app.get_name().to_owned();

    let page = man::render(&app, env!("CARGO_PKG_VERSION"), &build_date());
    let man_path = man_dir.join(format!("{bin}.1"));
    fs::write(&man_path, page)?;
    println!("{}", man_path.display());

    for shell in Shell::ALL {
        let script = completions::render(&app, shell);
        let path = comp_dir.join(shell.file_name(&bin));
        fs::write(&path, script)?;
        println!("{}", path.display());
    }

    Ok(())
}

/// `SOURCE_DATE_EPOCH` as `YYYY-MM-DD`, or [`FALLBACK_DATE`].
fn build_date() -> String {
    let Some(epoch) = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
    else {
        return FALLBACK_DATE.to_owned();
    };
    civil_date(epoch)
}

/// Converts a Unix timestamp to a `YYYY-MM-DD` civil date (UTC).
///
/// Written out rather than pulled from `chrono` or `time`: this is the
/// only date arithmetic in the crate, and neither crate is in the vetted
/// dependency set. The algorithm is Howard Hinnant's `civil_from_days`.
fn civil_date(epoch: i64) -> String {
    let days = epoch.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
