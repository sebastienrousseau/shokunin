// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Deploy adapter trait + per-target stubs for the `ssg deploy`
//! subcommand (issue #527 AC4).
//!
//! The legacy [`crate::deploy::DeployPlugin`] generates platform-specific
//! configuration files (`netlify.toml`, `vercel.json`, …) **at build
//! time**. This module is responsible for the *upload* half — taking a
//! freshly built `site_dir` and shipping it to the target platform.
//!
//! Each adapter is a stub at the moment: it prints
//! `"deploy adapter for <target>: not yet implemented; see #527"` to
//! stderr and exits cleanly. The wiring and CLI surface are in place
//! so we can land the rest of the implementation behind it without
//! breaking the `ssg deploy --target …` API.
//!
//! ## Stability
//!
//! [`Target`] is `#[non_exhaustive]` — new targets can be added in
//! minor releases. Downstream `match` arms must include a wildcard.

use crate::error::SsgError;
use std::path::Path;

/// Deploy targets accepted by `ssg deploy --target <TARGET>`.
///
/// Kept in lockstep with [`crate::cmd::DEPLOY_TARGETS`] (the CLI-facing
/// list); this enum is the typed view used inside the deploy pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Target {
    /// Netlify static-site hosting (`SSG_NETLIFY_TOKEN`).
    Netlify,
    /// Vercel static deployment (`SSG_VERCEL_TOKEN`).
    Vercel,
    /// Cloudflare Pages direct-upload (`SSG_CLOUDFLARE_TOKEN`).
    CloudflarePages,
    /// GitHub Pages branch push (`SSG_GITHUB_TOKEN`).
    GithubPages,
    /// Amazon S3 bucket sync (`SSG_S3_BUCKET`, `AWS_*`).
    S3,
    /// No-op: build only, do not upload. Useful for CI dry-runs.
    None,
}

impl Target {
    /// Parses a CLI string into a [`Target`].
    ///
    /// Unknown values are rejected by clap before reaching here (see
    /// `Cli::subcommand_app` -> `--target` `PossibleValuesParser`),
    /// but we treat an unknown input as a defensive validation error
    /// rather than panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::deploy_adapter::Target;
    ///
    /// assert_eq!(Target::from_cli("netlify").unwrap(), Target::Netlify);
    /// assert!(Target::from_cli("not-a-target").is_err());
    /// ```
    ///
    /// # Errors
    /// Returns [`SsgError::Validation`] if `name` is not one of the
    /// six supported targets.
    pub fn from_cli(name: &str) -> Result<Self, SsgError> {
        match name {
            "netlify" => Ok(Self::Netlify),
            "vercel" => Ok(Self::Vercel),
            "cloudflare-pages" => Ok(Self::CloudflarePages),
            "github-pages" => Ok(Self::GithubPages),
            "s3" => Ok(Self::S3),
            "none" => Ok(Self::None),
            other => Err(SsgError::Validation {
                field: "deploy.target".to_string(),
                message: format!("unknown deploy target: {other}"),
            }),
        }
    }

    /// Returns the canonical CLI name for this target.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::deploy_adapter::Target;
    ///
    /// assert_eq!(Target::Netlify.as_str(), "netlify");
    /// assert_eq!(Target::None.as_str(), "none");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Netlify => "netlify",
            Self::Vercel => "vercel",
            Self::CloudflarePages => "cloudflare-pages",
            Self::GithubPages => "github-pages",
            Self::S3 => "s3",
            Self::None => "none",
            // Defensive: future variants fall through to a placeholder
            // so we don't accidentally break the build when a new
            // target is added.
            #[allow(unreachable_patterns)]
            _ => "unknown",
        }
    }
}

/// Trait implemented by per-target deploy adapters.
///
/// All current implementations are stubs (issue #527) — they log
/// `"deploy adapter for <name>: not yet implemented; see #527"` and
/// return `Ok(())`. The trait shape is the stable contract.
pub trait DeployAdapter: Send + Sync {
    /// Short, human-readable name for logs (e.g. `"netlify"`).
    fn name(&self) -> &'static str;

    /// Performs the deploy. `site_dir` is the freshly built site root.
    ///
    /// Stubs print a "not yet implemented" message and exit cleanly so
    /// users can wire CI now and pick up the actual upload behaviour in
    /// a follow-up patch.
    ///
    /// # Errors
    /// Returns [`SsgError`] if the deploy fails. Stubs never error;
    /// they always succeed after printing the placeholder message.
    fn deploy(&self, site_dir: &Path) -> Result<(), SsgError>;
}

/// Returns the adapter implementation for a given [`Target`].
///
/// # Examples
///
/// ```rust
/// use ssg::deploy_adapter::{adapter_for, Target};
///
/// let a = adapter_for(Target::None);
/// assert_eq!(a.name(), "none");
/// ```
#[must_use]
pub fn adapter_for(target: Target) -> Box<dyn DeployAdapter> {
    match target {
        Target::Netlify => Box::new(NetlifyAdapter),
        Target::Vercel => Box::new(VercelAdapter),
        Target::CloudflarePages => Box::new(CloudflarePagesAdapter),
        Target::GithubPages => Box::new(GithubPagesAdapter),
        Target::S3 => Box::new(S3Adapter),
        Target::None => Box::new(NoneAdapter),
        // Defensive: future variants fall back to a no-op so we don't
        // accidentally panic on a new target before its adapter lands.
        #[allow(unreachable_patterns)]
        _ => Box::new(NoneAdapter),
    }
}

/// Helper used by every stub. Centralises the message text so tests
/// can assert on it without depending on string concatenation rules.
fn stub_message(target: &str) {
    eprintln!("deploy adapter for {target}: not yet implemented; see #527");
}

// ----- stubs ---------------------------------------------------------

/// Stub adapter for Netlify.
#[derive(Debug, Clone, Copy)]
pub struct NetlifyAdapter;
impl DeployAdapter for NetlifyAdapter {
    fn name(&self) -> &'static str {
        "netlify"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        stub_message(self.name());
        Ok(())
    }
}

/// Stub adapter for Vercel.
#[derive(Debug, Clone, Copy)]
pub struct VercelAdapter;
impl DeployAdapter for VercelAdapter {
    fn name(&self) -> &'static str {
        "vercel"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        stub_message(self.name());
        Ok(())
    }
}

/// Stub adapter for Cloudflare Pages.
#[derive(Debug, Clone, Copy)]
pub struct CloudflarePagesAdapter;
impl DeployAdapter for CloudflarePagesAdapter {
    fn name(&self) -> &'static str {
        "cloudflare-pages"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        stub_message(self.name());
        Ok(())
    }
}

/// Stub adapter for GitHub Pages.
#[derive(Debug, Clone, Copy)]
pub struct GithubPagesAdapter;
impl DeployAdapter for GithubPagesAdapter {
    fn name(&self) -> &'static str {
        "github-pages"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        stub_message(self.name());
        Ok(())
    }
}

/// Stub adapter for Amazon S3.
#[derive(Debug, Clone, Copy)]
pub struct S3Adapter;
impl DeployAdapter for S3Adapter {
    fn name(&self) -> &'static str {
        "s3"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        stub_message(self.name());
        Ok(())
    }
}

/// No-op adapter — build only, no upload.
#[derive(Debug, Clone, Copy)]
pub struct NoneAdapter;
impl DeployAdapter for NoneAdapter {
    fn name(&self) -> &'static str {
        "none"
    }
    fn deploy(&self, _site_dir: &Path) -> Result<(), SsgError> {
        // The `none` target is intentionally silent — it's the
        // documented "build only" path. No stub message.
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn target_from_cli_accepts_all_six() {
        for (name, expected) in [
            ("netlify", Target::Netlify),
            ("vercel", Target::Vercel),
            ("cloudflare-pages", Target::CloudflarePages),
            ("github-pages", Target::GithubPages),
            ("s3", Target::S3),
            ("none", Target::None),
        ] {
            assert_eq!(Target::from_cli(name).unwrap(), expected);
        }
    }

    #[test]
    fn target_from_cli_rejects_unknown() {
        assert!(Target::from_cli("moon").is_err());
    }

    #[test]
    fn target_as_str_round_trips() {
        for t in [
            Target::Netlify,
            Target::Vercel,
            Target::CloudflarePages,
            Target::GithubPages,
            Target::S3,
            Target::None,
        ] {
            let s = t.as_str();
            assert_eq!(Target::from_cli(s).unwrap(), t);
        }
    }

    #[test]
    fn adapter_for_returns_correct_name() {
        let pairs = [
            (Target::Netlify, "netlify"),
            (Target::Vercel, "vercel"),
            (Target::CloudflarePages, "cloudflare-pages"),
            (Target::GithubPages, "github-pages"),
            (Target::S3, "s3"),
            (Target::None, "none"),
        ];
        for (t, expected) in pairs {
            let a = adapter_for(t);
            assert_eq!(a.name(), expected);
        }
    }

    #[test]
    fn stub_adapters_succeed() {
        // Every stub returns Ok(()) so wiring downstream consumers
        // doesn't fail before the real implementations land.
        let dir = PathBuf::from("/tmp/ssg-test-site");
        for t in [
            Target::Netlify,
            Target::Vercel,
            Target::CloudflarePages,
            Target::GithubPages,
            Target::S3,
            Target::None,
        ] {
            let a = adapter_for(t);
            assert!(a.deploy(&dir).is_ok(), "adapter {} failed", a.name());
        }
    }
}
