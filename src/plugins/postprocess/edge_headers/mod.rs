// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! PQC-aware edge-runtime header emitter (issue #550).
//!
//! Emits per-platform header configuration files for Cloudflare
//! Workers (`wrangler-headers.toml`), Netlify (`_headers`), and Vercel
//! (`vercel-headers.json`) so the deployed site lands with TLS,
//! Permissions-Policy, X-Content-Type-Options, Referrer-Policy and the
//! site's computed Content-Security-Policy already locked in.
//!
//! ## Scope (locked-in)
//!
//! - **Static emit only.** No live TLS probing at build time.
//! - **Five baseline headers.** [`baseline_headers`] is the source of
//!   truth; per-target emitters render those keys/values into the
//!   platform-specific syntax. Anything else (cache-control,
//!   per-route headers) lives in the existing deploy adapter — the
//!   edge-headers emitter is intentionally orthogonal to deploy
//!   target generation.
//! - **CSP comes from the CSP plugin.** The `Content-Security-Policy`
//!   value is sourced from [`crate::csp::computed_policy`]; the
//!   emitter does **not** recompute or hardcode the string. AC7.
//! - **PQC documentation comment** in every emitted file naming the
//!   X25519+ML-KEM-768 hybrid key-exchange suite (CDN handles the
//!   actual negotiation — we just document and link). AC6.
//! - **Per-target overrides** via `[edge_headers.overrides]` in
//!   `ssg.toml`; case-insensitive on the header key, last-write-wins
//!   per target. AC5.
//!
//! ## File layout
//!
//! ```text
//! dist/
//! ├── _headers                           # Netlify (AC2)
//! └── .ssg/edge/
//!     ├── wrangler-headers.toml          # Cloudflare (AC1)
//!     └── vercel-headers.json            # Vercel (AC3)
//! ```
//!
//! Cloudflare and Vercel files live under `.ssg/edge/` so they don't
//! clash with any user-managed `wrangler.toml`/`vercel.json` at the
//! site root; the leading comment block on each file documents the
//! intended merge path.

pub(crate) mod cloudflare;
pub(crate) mod netlify;
pub(crate) mod vercel;

use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::collections::BTreeMap;
use std::fs;

/// Baseline header set emitted by every target.
///
/// Order matters: emitters render headers in iteration order so the
/// resulting files are deterministic across rebuilds (golden-test
/// friendly). The five baseline keys are:
///
/// 1. `Strict-Transport-Security` — 2-year `max-age`, preload-ready
/// 2. `Content-Security-Policy`   — sourced from [`crate::csp::computed_policy`]
/// 3. `X-Content-Type-Options`    — `nosniff`
/// 4. `Referrer-Policy`           — `strict-origin-when-cross-origin`
/// 5. `Permissions-Policy`        — camera/geolocation/microphone off
///
/// # Examples
///
/// ```
/// use ssg::postprocess::edge_headers::baseline_headers;
/// let baseline = baseline_headers();
/// assert_eq!(baseline.len(), 5);
/// assert_eq!(baseline[0].0, "Strict-Transport-Security");
/// ```
#[must_use]
pub fn baseline_headers() -> [(&'static str, String); 5] {
    [
        (
            "Strict-Transport-Security",
            "max-age=63072000; includeSubDomains; preload".to_string(),
        ),
        (
            "Content-Security-Policy",
            crate::csp::computed_policy().to_string(),
        ),
        ("X-Content-Type-Options", "nosniff".to_string()),
        (
            "Referrer-Policy",
            "strict-origin-when-cross-origin".to_string(),
        ),
        (
            "Permissions-Policy",
            "camera=(), geolocation=(), microphone=()".to_string(),
        ),
    ]
}

/// Merges baseline headers with case-insensitive overrides.
///
/// Returns an ordered `Vec<(String, String)>` so emitters render in
/// the same deterministic order as [`baseline_headers`]. Overrides are
/// matched by lowercased key; values replace the baseline verbatim.
/// Overrides referencing a header name **not** present in the baseline
/// are appended after the baseline so site authors can add e.g.
/// `Cross-Origin-Opener-Policy` without us hardcoding it.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
/// use ssg::postprocess::edge_headers::merged_headers;
/// let merged = merged_headers(&BTreeMap::new());
/// assert_eq!(merged.len(), 5);
/// assert_eq!(merged[0].0, "Strict-Transport-Security");
/// ```
#[must_use]
pub fn merged_headers(
    overrides: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let baseline = baseline_headers();
    let mut lower_overrides: BTreeMap<String, (String, String)> = overrides
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), (k.clone(), v.clone())))
        .collect();

    let mut out: Vec<(String, String)> = Vec::with_capacity(baseline.len());
    for (key, default_value) in baseline {
        let key_lc = key.to_ascii_lowercase();
        if let Some((_orig_key, override_value)) =
            lower_overrides.remove(&key_lc)
        {
            out.push((key.to_string(), override_value));
        } else {
            out.push((key.to_string(), default_value));
        }
    }
    // Append any extra (non-baseline) overrides in deterministic
    // (alphabetical) order.
    for (_lc, (orig, value)) in lower_overrides {
        out.push((orig, value));
    }
    out
}

/// PQC documentation snippet appended to every emitted file as a
/// platform-appropriate comment block (TOML `#`, plain-text `#`, JSON
/// `_pqc_note` key). Names the recommended hybrid key-exchange suite
/// and links each platform's TLS configuration page. AC6.
pub(crate) const PQC_NOTE_LINES: &[&str] = &[
    "PQC posture: TLS 1.3 with the X25519+ML-KEM-768 hybrid",
    "key-exchange suite (RFC 9420 / draft-ietf-tls-hybrid-design).",
    "Cloudflare auto-negotiates as of mid-2026; Netlify is",
    "behind an opt-in in the platform dashboard; Vercel surfaces",
    "the suite once the upstream CDN (Cloudflare/AWS) enables it.",
    "Configure at the platform level — this file is documentation",
    "of the recommended posture, not a runtime knob.",
    "Cloudflare:    https://developers.cloudflare.com/ssl/post-quantum-cryptography/",
    "Netlify:       https://docs.netlify.com/edge-functions/overview/",
    "Vercel:        https://vercel.com/docs/edge-network/headers",
];

/// Postprocess plugin that emits per-platform edge header config.
///
/// Reads [`crate::cmd::EdgeHeadersConfig`] off the plugin context's
/// `config.edge_headers` field; for each recognised entry in
/// `targets`, invokes the corresponding emitter. The plugin is a
/// no-op when `targets` is empty, when `config` is `None`, or when
/// `site_dir` does not yet exist.
///
/// # Examples
///
/// ```
/// use ssg::plugin::Plugin;
/// use ssg::postprocess::edge_headers::EdgeHeadersPlugin;
/// assert_eq!(EdgeHeadersPlugin::new().name(), "edge-headers");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeHeadersPlugin;

impl EdgeHeadersPlugin {
    /// Creates a new `EdgeHeadersPlugin`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::postprocess::edge_headers::EdgeHeadersPlugin;
    /// let plugin = EdgeHeadersPlugin::new();
    /// let _copy: EdgeHeadersPlugin = plugin;
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for EdgeHeadersPlugin {
    fn name(&self) -> &'static str {
        "edge-headers"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let Some(cfg) = ctx.config.as_ref() else {
            return Ok(());
        };
        let edge = &cfg.edge_headers;
        if !edge.is_enabled() {
            return Ok(());
        }

        let headers = merged_headers(&edge.overrides);

        // Cloudflare and Vercel artifacts go under `.ssg/edge/` so they
        // don't collide with any user-owned wrangler.toml / vercel.json
        // at the site root.
        let edge_dir = ctx.site_dir.join(".ssg").join("edge");

        for target in &edge.targets {
            match target.to_ascii_lowercase().as_str() {
                "cloudflare" => {
                    fs::create_dir_all(&edge_dir).with_path(&edge_dir)?;
                    let out_path = edge_dir.join("wrangler-headers.toml");
                    let body = cloudflare::render(&headers);
                    fs::write(&out_path, body).with_path(&out_path)?;
                    log::info!("[edge-headers] wrote {}", out_path.display());
                }
                "netlify" => {
                    let out_path = ctx.site_dir.join("_headers");
                    let body = netlify::render(&headers);
                    fs::write(&out_path, body).with_path(&out_path)?;
                    log::info!("[edge-headers] wrote {}", out_path.display());
                }
                "vercel" => {
                    fs::create_dir_all(&edge_dir).with_path(&edge_dir)?;
                    let out_path = edge_dir.join("vercel-headers.json");
                    let body = vercel::render(&headers).map_err(|e| {
                        SsgError::io(
                            std::io::Error::other(e.to_string()),
                            &out_path,
                        )
                    })?;
                    fs::write(&out_path, body).with_path(&out_path)?;
                    log::info!("[edge-headers] wrote {}", out_path.display());
                }
                other => {
                    log::warn!(
                        "[edge-headers] unknown target `{other}` — skipping"
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_overrides(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn baseline_headers_includes_five_canonical_entries() {
        let baseline = baseline_headers();
        let keys: Vec<&str> = baseline.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            keys,
            vec![
                "Strict-Transport-Security",
                "Content-Security-Policy",
                "X-Content-Type-Options",
                "Referrer-Policy",
                "Permissions-Policy",
            ]
        );
    }

    #[test]
    fn baseline_csp_comes_from_csp_plugin_not_hardcoded() {
        // AC7: CSP value must equal the value exposed by the CSP
        // plugin's computed_policy() function.
        let baseline = baseline_headers();
        let csp = baseline
            .iter()
            .find(|(k, _)| *k == "Content-Security-Policy")
            .map(|(_, v)| v.as_str())
            .expect("baseline must contain CSP");
        assert_eq!(csp, crate::csp::computed_policy());
    }

    #[test]
    fn baseline_hsts_is_2_year_preload_ready() {
        let baseline = baseline_headers();
        let hsts = baseline
            .iter()
            .find(|(k, _)| *k == "Strict-Transport-Security")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert!(hsts.contains("max-age=63072000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn merged_preserves_baseline_when_no_overrides() {
        let merged = merged_headers(&BTreeMap::new());
        assert_eq!(merged.len(), 5);
        assert_eq!(merged[0].0, "Strict-Transport-Security");
    }

    #[test]
    fn merged_applies_case_insensitive_override() {
        // AC5: override must replace a single header and preserve all
        // others — and the lookup is case-insensitive.
        let overrides =
            make_overrides(&[("permissions-policy", "geolocation=(self)")]);
        let merged = merged_headers(&overrides);
        let pp = merged
            .iter()
            .find(|(k, _)| k == "Permissions-Policy")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(pp, "geolocation=(self)");

        // Other defaults must be untouched.
        let hsts = merged
            .iter()
            .find(|(k, _)| k == "Strict-Transport-Security")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert!(hsts.contains("max-age=63072000"));
    }

    #[test]
    fn merged_uppercase_override_key_still_matches_baseline() {
        let overrides = make_overrides(&[("PERMISSIONS-POLICY", "camera=*")]);
        let merged = merged_headers(&overrides);
        let pp = merged
            .iter()
            .find(|(k, _)| k == "Permissions-Policy")
            .unwrap();
        assert_eq!(pp.1, "camera=*");
    }

    #[test]
    fn merged_appends_non_baseline_overrides() {
        let overrides =
            make_overrides(&[("Cross-Origin-Opener-Policy", "same-origin")]);
        let merged = merged_headers(&overrides);
        assert_eq!(merged.len(), 6);
        assert!(merged
            .iter()
            .any(|(k, v)| k == "Cross-Origin-Opener-Policy"
                && v == "same-origin"));
    }

    #[test]
    fn no_duplicate_csp_in_baseline() {
        // AC7: there should never be more than one Content-Security-Policy.
        let baseline = baseline_headers();
        let csp_count = baseline
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("Content-Security-Policy"))
            .count();
        assert_eq!(csp_count, 1);
    }

    #[test]
    fn plugin_name_is_stable() {
        assert_eq!(EdgeHeadersPlugin.name(), "edge-headers");
    }

    #[test]
    fn after_compile_is_noop_when_site_dir_missing() {
        let ctx = PluginContext::new(
            std::path::Path::new("/tmp/c"),
            std::path::Path::new("/tmp/b"),
            std::path::Path::new("/nonexistent/site-xyz"),
            std::path::Path::new("/tmp/t"),
        );
        assert!(EdgeHeadersPlugin.after_compile(&ctx).is_ok());
    }

    #[test]
    fn after_compile_is_noop_when_config_missing() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        // No config set → early-return.
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("_headers").exists());
        assert!(!site.join(".ssg/edge").exists());
    }

    #[test]
    fn after_compile_skips_when_no_targets_configured() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = crate::cmd::SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .build()
            .unwrap();
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("_headers").exists());
    }

    fn cfg_with_targets(targets: Vec<&str>) -> crate::cmd::SsgConfig {
        let mut edge = crate::cmd::EdgeHeadersConfig::default();
        edge.targets = targets.into_iter().map(String::from).collect();
        crate::cmd::SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .edge_headers(edge)
            .build()
            .unwrap()
    }

    #[test]
    fn after_compile_cloudflare_target_writes_wrangler_headers() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["cloudflare"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        let written = site.join(".ssg/edge/wrangler-headers.toml");
        assert!(written.exists(), "{}", written.display());
        let body = fs::read_to_string(&written).unwrap();
        assert!(body.contains("Strict-Transport-Security"));
    }

    #[test]
    fn after_compile_netlify_target_writes_underscore_headers() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        let written = site.join("_headers");
        assert!(written.exists());
        let body = fs::read_to_string(&written).unwrap();
        assert!(body.contains("Strict-Transport-Security"));
    }

    #[test]
    fn after_compile_vercel_target_writes_json() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        let written = site.join(".ssg/edge/vercel-headers.json");
        assert!(written.exists());
        let body = fs::read_to_string(&written).unwrap();
        assert!(body.contains("Strict-Transport-Security"));
    }

    #[test]
    fn after_compile_unknown_target_is_warned_and_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["unknown-cdn"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(!site.join("_headers").exists());
        assert!(!site.join(".ssg/edge").exists());
    }

    #[test]
    fn after_compile_target_name_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["CloudFlare"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(site.join(".ssg/edge/wrangler-headers.toml").exists());
    }

    #[test]
    fn after_compile_all_three_targets_emit_all_three_artefacts() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["cloudflare", "netlify", "vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(), dir.path(), &site, dir.path(), cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(site.join("_headers").exists());
        assert!(site.join(".ssg/edge/wrangler-headers.toml").exists());
        assert!(site.join(".ssg/edge/vercel-headers.json").exists());
    }

    #[test]
    fn new_constructs_unit() {
        let _ = EdgeHeadersPlugin::new();
    }
}
