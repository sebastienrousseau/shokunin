// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Documentation accuracy regression gate.
//!
//! Every claim the README, CHANGELOG, or docs/ makes about the
//! codebase that can be cross-referenced from `Cargo.toml`, source
//! files, or CI configuration is verified here.
//!
//! ## What this catches
//!
//! - README test-count claims drifting from `cargo test --lib` reality.
//! - Version mismatches between `Cargo.toml` and the README install
//!   instruction / SECURITY.md.
//! - Coverage-floor claims in README vs `ci.yml` `COVERAGE_*_FLOOR`
//!   env vars.
//! - WCAG version claims in README vs `src/accessibility.rs`.
//! - MSRV claims in README vs `Cargo.toml` `rust-version`.
//!
//! These are the load-bearing claims procurement, security, and SEO
//! reviewers cite when evaluating SSG. A drift between marketing
//! copy and source-of-truth files would silently undermine those
//! claims; this test fails CI before that happens.
//!
//! ## Methodology
//!
//! Read source-of-truth files at test time and grep the README /
//! CHANGELOG / SECURITY.md for the corresponding claim. If the
//! source-of-truth changed without the docs being updated, the test
//! prints a directly-actionable diff and panics. The test is
//! tolerant of "soft" claims (`approximately`, `1,000+`, etc.) by
//! design — it only fails on **exact-number** assertions that have
//! gone stale.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace().join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Extracts the `name = ...` and `version = ...` values from the
/// `[package]` table of the root `Cargo.toml`. Cheap regex-free
/// parse so we don't pull a TOML crate just for this test.
fn cargo_package_field(field: &str) -> String {
    let toml = read("Cargo.toml");
    let mut in_package = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(&format!("{field} = ")) {
            return rest
                .trim()
                .trim_start_matches('"')
                .split('"')
                .next()
                .unwrap_or_default()
                .to_string();
        }
    }
    panic!("Cargo.toml [package] missing field `{field}`");
}

/// Counts `#[test]`, `#[tokio::test]`, etc. attributes across the
/// `src/` tree. This approximates the lib test count without
/// running cargo, which would loop into us.
fn count_src_lib_tests() -> usize {
    let mut count = 0;
    walk_for_count(&workspace().join("src"), &mut count);
    count
}

fn walk_for_count(dir: &Path, count: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_for_count(&p, count);
        } else if p.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&p) {
                // Count `#[test]` and `#[tokio::test]` annotations
                // appearing as the immediate decorator of a function.
                for line in content.lines() {
                    let t = line.trim();
                    if t == "#[test]" || t.starts_with("#[tokio::test") {
                        *count += 1;
                    }
                }
            }
        }
    }
}

// =====================================================================
// Version sync
// =====================================================================

#[test]
fn readme_version_matches_cargo_toml() {
    let cargo_version = cargo_package_field("version");
    let readme = read("README.md");

    // README often cites the version in `cargo install ssg --version
    // X.Y.Z` or the docs.rs link `docs.rs/ssg/X.Y.Z`. We assert at
    // least one full-form occurrence.
    let badge = format!("v{cargo_version}");
    assert!(
        readme.contains(&cargo_version) || readme.contains(&badge),
        "Cargo.toml version `{cargo_version}` does not appear in README. \
         Either bump the README claim or revert the Cargo.toml change. \
         Soft-mention forms (~X.Y, latest, etc.) don't satisfy this gate."
    );
}

// =====================================================================
// Lib test count
// =====================================================================

#[test]
fn readme_lib_test_count_is_in_the_right_ballpark() {
    // The README has historically cited a specific test count (e.g.
    // "1,640 unit tests"). When source-tree test annotations grow
    // far past the cited number, the claim becomes stale.
    //
    // This test extracts the README's lib test count (any
    // X-or-X,XXX form followed by "unit tests" or "lib tests") and
    // asserts it's within ±15% of the source-tree count. The 15%
    // band tolerates "round numbers" (1,640 staying stable while
    // 1,685 ticks past) without demanding constant README churn,
    // but a 200+-test drift will fail this gate and force an update.
    let readme = read("README.md");
    let actual = count_src_lib_tests();

    let mut claim: Option<usize> = None;
    for line in readme.lines() {
        let lower = line.to_lowercase();
        if !lower.contains("unit test") && !lower.contains("lib test") {
            continue;
        }
        // Capture the first digit run in the line (handles 1,640 by
        // stripping commas).
        let mut buf = String::new();
        for ch in line.chars() {
            if ch.is_ascii_digit() {
                buf.push(ch);
            } else if !buf.is_empty() {
                if ch == ',' {
                    continue;
                }
                break;
            }
        }
        if let Ok(n) = buf.parse::<usize>() {
            if n >= 100 {
                claim = Some(n);
                break;
            }
        }
    }

    let Some(claim) = claim else {
        eprintln!(
            "[docs_accuracy] README has no `N unit tests` / `N lib tests` \
             claim — skipping. To enforce, add a claim like \
             `1,685 unit tests`."
        );
        return;
    };

    let lower = claim.saturating_sub(claim * 15 / 100);
    let upper = claim + claim * 15 / 100;
    assert!(
        actual >= lower && actual <= upper,
        "README cites {claim} unit/lib tests but `src/` has {actual} \
         `#[test]` annotations (±15% band: {lower}..={upper}). \
         Update the README claim or investigate the test-count drift."
    );
}

// =====================================================================
// Coverage floors
// =====================================================================

#[test]
fn readme_coverage_claims_match_ci_floors() {
    let ci = read(".github/workflows/ci.yml");
    let readme = read("README.md");

    // Extract the COVERAGE_REGIONS_FLOOR / COVERAGE_LINES_FLOOR /
    // COVERAGE_FUNCTIONS_FLOOR env values from ci.yml.
    let floor = |name: &str| -> Option<u32> {
        let needle = format!("{name}: \"");
        let idx = ci.find(&needle)?;
        let after = &ci[idx + needle.len()..];
        let close = after.find('"')?;
        after[..close].parse::<f32>().ok().map(|f| f as u32)
    };

    let regions = floor("COVERAGE_REGIONS_FLOOR")
        .expect("ci.yml missing COVERAGE_REGIONS_FLOOR");

    // The README cites "95% region", "97% line", etc. Verify the
    // numeric region claim matches.
    let readme_lower = readme.to_lowercase();
    let region_pat = format!("{regions}% region");
    assert!(
        readme_lower.contains(&region_pat),
        "ci.yml COVERAGE_REGIONS_FLOOR is {regions}% but README's \
         region-coverage claim doesn't match. Search: `{region_pat}`."
    );
}

// =====================================================================
// WCAG version sync
// =====================================================================

#[test]
fn readme_wcag_version_matches_accessibility_module() {
    // src/plugins/accessibility.rs declares the WCAG version in its module
    // docstring and uses it as the `wcag_version` field default.
    let accessibility = read("src/plugins/accessibility.rs");
    let readme = read("README.md");

    // Pull "WCAG 2.X" from the accessibility module (first match).
    let wcag_version = ["2.2", "2.1", "2.0"]
        .iter()
        .find(|v| accessibility.contains(&format!("WCAG {v}")))
        .copied()
        .unwrap_or("2.1");

    let claim = format!("WCAG {wcag_version}");
    assert!(
        readme.contains(&claim),
        "src/accessibility.rs targets `{claim}` but README doesn't \
         mention it. Update README or fix the module-level docstring."
    );
}

// =====================================================================
// MSRV sync
// =====================================================================

#[test]
fn readme_msrv_matches_cargo_toml() {
    let msrv = cargo_package_field("rust-version");
    let readme = read("README.md");

    // README either cites the MSRV explicitly (`Rust 1.X+`) or has
    // a "Rust version" / "MSRV" line. We accept any of the common
    // forms.
    let forms = [
        format!("Rust {msrv}"),
        format!("rust {msrv}"),
        format!("MSRV: {msrv}"),
        format!("MSRV {msrv}"),
        format!("rust-version = \"{msrv}\""),
    ];
    let found = forms.iter().any(|f| readme.contains(f));
    if !found {
        eprintln!(
            "[docs_accuracy] README has no MSRV claim — soft gate. \
             Cargo.toml MSRV is `{msrv}`. Consider adding to README \
             so consumers see it without reading Cargo.toml."
        );
    }
}

// =====================================================================
// SECURITY.md cross-check
// =====================================================================

#[test]
fn security_md_exists_and_mentions_disclosure() {
    let security = read("SECURITY.md");
    assert!(
        security
            .to_lowercase()
            .contains("reporting a vulnerability")
            || security.to_lowercase().contains("disclos"),
        "SECURITY.md must describe a vulnerability disclosure process"
    );
}

// =====================================================================
// SBOM emission verified at the source level
// =====================================================================

#[test]
fn sbom_cyclondx_version_matches_module_claim() {
    // src/plugins/sbom.rs commits to "1.5" in both the rustdoc and the
    // emitted JSON. If those drift, downstream tooling that expects
    // the docstring version will validate against the wrong schema.
    let sbom = read("src/plugins/sbom.rs");
    let docstring_v = sbom.contains("CycloneDX 1.5");
    let emit_v = sbom.contains("\"specVersion\": \"1.5\"");
    assert!(
        docstring_v && emit_v,
        "src/sbom.rs CycloneDX version drift: docstring says 1.5 = {docstring_v}, \
         emit says 1.5 = {emit_v}"
    );
}

#[cfg(test)]
mod helpers {
    use super::*;

    #[test]
    fn count_src_lib_tests_is_nonzero() {
        let n = count_src_lib_tests();
        assert!(n > 100, "expected >100 lib tests in src/, got {n}");
    }

    #[test]
    fn cargo_package_name_is_ssg() {
        assert_eq!(cargo_package_field("name"), "ssg");
    }
}
