// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Theme manifest compatibility check.
//!
//! A theme declares the oldest generator it works with:
//!
//! ```toml
//! # themes/atlas/theme.toml
//! min_version = "0.0.50"
//! ```
//!
//! Nothing used to read it. That mattered because the ways a too-old
//! generator breaks a theme are all *silent*: before v0.0.50 the `layout`
//! named in front matter was ignored and every page rendered through
//! `page.html`, a bundled `content.schema.toml` aborted the compile with an
//! unrelated message, and extracted CSS 404'd under a sub-path. A user on
//! an older release got a build that succeeded and a site that was wrong.
//!
//! This turns that into one clear error at the start of the build.

use crate::error::SsgError;
use std::path::Path;

/// A semantic version reduced to the three numeric components ssg uses.
///
/// Pre-release and build metadata are ignored: `0.0.50-rc.1` compares equal
/// to `0.0.50`. Themes pin a floor, not an exact build, so treating a
/// release candidate as satisfying its own floor is the useful behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u64, u64, u64);

impl Version {
    fn parse(raw: &str) -> Option<Self> {
        let core = raw
            .trim()
            .trim_start_matches('v')
            .split(['-', '+'])
            .next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        // A theme may pin `0.1` or even `1`; absent components are zero.
        let minor = parts.next().map_or(Some(0), |p| p.parse().ok())?;
        let patch = parts.next().map_or(Some(0), |p| p.parse().ok())?;
        Some(Self(major, minor, patch))
    }
}

/// Reads `min_version` from a theme manifest beside `template_dir`.
///
/// Layouts conventionally live in `<theme>/_layouts`, so the manifest is
/// looked for in the template directory itself and then in its parent.
/// `theme.toml` wins over `theme.json`; a theme with neither, or with no
/// `min_version`, imposes no floor.
fn declared_min_version(template_dir: &Path) -> Option<(String, String)> {
    let candidates = [
        template_dir.join("theme.toml"),
        template_dir.parent()?.join("theme.toml"),
        template_dir.join("theme.json"),
        template_dir.parent()?.join("theme.json"),
    ];

    for path in candidates {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let key = if path.extension().is_some_and(|e| e == "json") {
            "min_ssg_version"
        } else {
            "min_version"
        };
        if let Some(v) = scan_for_key(&text, key) {
            return Some((v, path.display().to_string()));
        }
    }
    None
}

/// Pulls `key = "value"` / `"key": "value"` out of a manifest.
///
/// Deliberately not a TOML/JSON parse: this runs before anything else in
/// the build, and a malformed manifest should not be able to abort a build
/// that would otherwise succeed. A key it cannot find imposes no floor.
fn scan_for_key(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with('#') || !line.contains(key) {
            return None;
        }
        let (lhs, rhs) = line.split_once(['=', ':'])?;
        if lhs.trim().trim_matches(['"', '\''].as_ref()) != key {
            return None;
        }
        let value = rhs
            .trim()
            .trim_end_matches(',')
            .trim()
            .trim_matches(['"', '\''].as_ref());
        (!value.is_empty()).then(|| value.to_string())
    })
}

/// Fails the build when the theme requires a newer generator than this one.
///
/// # Errors
///
/// Returns [`SsgError::Validation`] naming both versions and the manifest
/// that declared the floor.
pub fn check_theme_compatibility(template_dir: &Path) -> Result<(), SsgError> {
    let Some((declared, manifest)) = declared_min_version(template_dir) else {
        return Ok(());
    };
    let (Some(required), Some(current)) = (
        Version::parse(&declared),
        Version::parse(env!("CARGO_PKG_VERSION")),
    ) else {
        // An unparseable version is the theme author's typo, not a reason to
        // refuse to build. Warn and continue.
        log::warn!(
            "[theme] could not parse min_version {declared:?} in {manifest}; skipping compatibility check"
        );
        return Ok(());
    };

    if current < required {
        return Err(SsgError::Validation {
            field: "theme min_version".to_string(),
            message: format!(
            "this theme requires ssg {declared} or later, but this is {current_v}.\n\
             \n\
             Declared by {manifest}.\n\
             \n\
             Older releases fail silently rather than loudly: the layout named in\n\
             front matter may be ignored so every page renders through page.html,\n\
             a bundled content.schema.toml may abort the compile, and extracted\n\
             CSS may 404 under a sub-path. Upgrade with `cargo install ssg`.",
            current_v = env!("CARGO_PKG_VERSION"),
            ),
        });
    }

    log::debug!(
        "[theme] {manifest} requires ssg {declared}; running {current:?}"
    );
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn version_parses_partial_and_prefixed_forms() {
        assert_eq!(Version::parse("0.0.50"), Some(Version(0, 0, 50)));
        assert_eq!(Version::parse("v1.2.3"), Some(Version(1, 2, 3)));
        assert_eq!(Version::parse("0.1"), Some(Version(0, 1, 0)));
        assert_eq!(Version::parse("2"), Some(Version(2, 0, 0)));
        // Pre-release satisfies its own floor.
        assert_eq!(Version::parse("0.0.50-rc.1"), Some(Version(0, 0, 50)));
        assert_eq!(Version::parse("nonsense"), None);
    }

    #[test]
    fn version_ordering_is_numeric_not_lexical() {
        // The bug a string compare would introduce: "0.0.9" > "0.0.50".
        assert!(
            Version::parse("0.0.9").unwrap()
                < Version::parse("0.0.50").unwrap()
        );
    }

    #[test]
    fn no_manifest_imposes_no_floor() {
        let dir = tempdir().unwrap();
        assert!(check_theme_compatibility(dir.path()).is_ok());
    }

    #[test]
    fn manifest_without_min_version_imposes_no_floor() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("theme.toml"), "name = \"x\"\n").unwrap();
        assert!(check_theme_compatibility(dir.path()).is_ok());
    }

    #[test]
    fn a_future_min_version_fails_with_both_versions_named() {
        let dir = tempdir().unwrap();
        let layouts = dir.path().join("_layouts");
        fs::create_dir_all(&layouts).unwrap();
        // Manifest sits beside _layouts, as themes ship it.
        fs::write(dir.path().join("theme.toml"), "min_version = \"999.0.0\"\n")
            .unwrap();

        let err = check_theme_compatibility(&layouts).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("999.0.0"), "{msg}");
        assert!(msg.contains(env!("CARGO_PKG_VERSION")), "{msg}");
        assert!(msg.contains("theme.toml"), "{msg}");
    }

    #[test]
    fn the_current_version_satisfies_its_own_floor() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("theme.toml"),
            format!("min_version = \"{}\"\n", env!("CARGO_PKG_VERSION")),
        )
        .unwrap();
        assert!(check_theme_compatibility(dir.path()).is_ok());
    }

    #[test]
    fn theme_json_min_ssg_version_is_honoured() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("theme.json"),
            "{\n  \"min_ssg_version\": \"999.0.0\"\n}\n",
        )
        .unwrap();
        assert!(check_theme_compatibility(dir.path()).is_err());
    }

    #[test]
    fn a_malformed_version_warns_rather_than_failing_the_build() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("theme.toml"), "min_version = \"latest\"\n")
            .unwrap();
        assert!(check_theme_compatibility(dir.path()).is_ok());
    }

    #[test]
    fn a_commented_out_key_is_not_read() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("theme.toml"),
            "# min_version = \"999.0.0\"\nname = \"x\"\n",
        )
        .unwrap();
        assert!(check_theme_compatibility(dir.path()).is_ok());
    }
}
