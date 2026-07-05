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
//! ## Per-page CSP (spec B4, v0.0.47 plan §3 item 2.4)
//!
//! Pages that still carry inline blocks after the CSP plugin's
//! extraction pass (JSON-LD structured data, chiefly) get a
//! **per-path** `Content-Security-Policy` entry in the Netlify
//! `_headers` and Vercel `vercel-headers.json` outputs, built from
//! [`crate::csp::page_policy`] — the hash-strict rendering of
//! [`crate::csp::DEFAULT_CSP_POLICY_TEMPLATE`] with that page's
//! SHA-256 inline source hashes. Pages without inline blocks fall
//! back to the global `/*` policy.
//!
//! ### Ordering contract
//!
//! The build pipeline runs every plugin's `after_compile` hook
//! *before* the fused `transform_html` pass, so per-page hashes
//! cannot be computed in `after_compile` (the CSP plugin's inline
//! extraction and the minifier have not run yet). Instead:
//!
//! 1. `after_compile` emits the platform files with the **global**
//!    policy only (deterministic fallback, also the final state for
//!    sites with zero inline blocks) and resets this build's
//!    per-page registry.
//! 2. `transform_html` — registered **after** `CspPlugin` and
//!    `MinifyPlugin` in `register_default_plugins`, so it observes
//!    the final shipped bytes of each page — records the page's
//!    policy and re-emits the platform files. Any future transform
//!    plugin that injects inline `<script>`/`<style>` content must
//!    register *before* `edge-headers` or its blocks will not be
//!    hashed.
//!
//! ### Determinism
//!
//! Per-page policies accumulate in a `BTreeMap` keyed by URL path, so
//! rendered output is sorted regardless of rayon scheduling. Every
//! page inserts its entry before writing a full snapshot **under the
//! same mutex**, so the chronologically last write — the one that
//! survives on disk — contains every page's entry. No sidecar file is
//! written and nothing extra ships in the site output
//! (`determinism.yml` byte-hashes the result).
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

use crate::cmd::EdgeHeadersConfig;
use crate::error::{PathErrorExt, SsgError};
use crate::plugin::{Plugin, PluginContext};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, PoisonError};

/// Per-build registry of per-page CSP policies, keyed by `site_dir`
/// so concurrent builds (and the test suite's tempdir sites) never
/// observe each other's pages. Inner map: URL path → policy string.
///
/// Plugin instances are stateless unit structs and the pipeline
/// offers no post-transform hook, so this module-level registry is
/// the coordination point between the fused transform pass and the
/// emitted platform files (see the module docs' ordering contract).
/// `after_compile` clears the current site's entry at the start of
/// every build, so watch-mode rebuilds never accumulate stale pages.
static PAGE_CSP_REGISTRY: LazyLock<
    Mutex<BTreeMap<PathBuf, BTreeMap<String, String>>>,
> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

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
        // New build: forget the previous build's per-page policies
        // for this site before anything else (watch-mode rebuilds
        // must never accumulate pages that no longer exist).
        {
            let mut registry = PAGE_CSP_REGISTRY
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let _ = registry.remove(&ctx.site_dir);
        }

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

        // Global-policy emission (also the final state for sites with
        // zero inline blocks); the fused transform pass below re-emits
        // with per-page CSP entries as it discovers them.
        emit_targets(ctx, edge, &BTreeMap::new())
    }

    fn has_transform(&self) -> bool {
        true
    }

    /// Pass-through transform that records the page's hash-strict CSP
    /// (spec B4) and re-emits the platform files. Returns the input
    /// HTML unchanged — this hook is an observation point, not a
    /// rewrite; it runs last in the fused pass so it sees the final
    /// shipped bytes (post-CSP-extraction, post-minify).
    fn transform_html(
        &self,
        html: &str,
        path: &Path,
        ctx: &PluginContext,
    ) -> Result<String, SsgError> {
        let Some(cfg) = ctx.config.as_ref() else {
            return Ok(html.to_string());
        };
        let edge = &cfg.edge_headers;
        if !edge.is_enabled() || !ctx.site_dir.exists() {
            return Ok(html.to_string());
        }

        let Some(policy) = crate::csp::page_policy(html) else {
            // No inline blocks — the global `/*` policy applies.
            return Ok(html.to_string());
        };

        let url = url_path_for(path, &ctx.site_dir);

        // Insert + snapshot + write under one lock: a later snapshot
        // can then never be overwritten by an earlier one, so the
        // chronologically last write contains every page (module docs,
        // "Determinism").
        let mut registry = PAGE_CSP_REGISTRY
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let pages = registry.entry(ctx.site_dir.clone()).or_default();
        let _ = pages.insert(url, policy);
        emit_targets(ctx, edge, pages)?;

        Ok(html.to_string())
    }
}

/// Renders and writes every configured platform target.
///
/// `page_csp` maps site-relative URL paths (`/blog/post/`) to their
/// hash-strict per-page CSP; an empty map emits the global policy
/// only. Iteration order is the `BTreeMap`'s sorted order, keeping
/// the emitted files byte-deterministic across rebuilds.
fn emit_targets(
    ctx: &PluginContext,
    edge: &EdgeHeadersConfig,
    page_csp: &BTreeMap<String, String>,
) -> Result<(), SsgError> {
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
                let body = netlify::render(&headers, page_csp);
                fs::write(&out_path, body).with_path(&out_path)?;
                log::info!("[edge-headers] wrote {}", out_path.display());
            }
            "vercel" => {
                fs::create_dir_all(&edge_dir).with_path(&edge_dir)?;
                let out_path = edge_dir.join("vercel-headers.json");
                let body = vercel_render(&headers, page_csp).map_err(|e| {
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

/// Delegates to [`vercel::render`] with a fault-injection hook so
/// tests can drive the error-mapping branch (serialising the vercel
/// header JSON cannot fail in practice).
fn vercel_render(
    headers: &[(String, String)],
    page_csp: &BTreeMap<String, String>,
) -> Result<String, serde_json::Error> {
    fail_point!("postprocess::vercel-render", |_| Err(
        <serde_json::Error as serde::ser::Error>::custom(
            "injected: postprocess::vercel-render"
        )
    ));
    vercel::render(headers, page_csp)
}

/// Maps a built HTML file path to its served URL path.
///
/// `index.html` collapses to its directory (`blog/post/index.html` →
/// `/blog/post/`, root `index.html` → `/`); any other file keeps its
/// name (`about.html` → `/about.html`). Backslashes are normalised so
/// Windows builds emit identical files (determinism gate).
fn url_path_for(path: &Path, site_dir: &Path) -> String {
    let rel = path
        .strip_prefix(site_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    if rel == "index.html" {
        "/".to_string()
    } else if let Some(dir) = rel.strip_suffix("/index.html") {
        format!("/{dir}/")
    } else {
        format!("/{rel}")
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
            Path::new("/tmp/c"),
            Path::new("/tmp/b"),
            Path::new("/nonexistent/site-xyz"),
            Path::new("/tmp/t"),
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
        let mut edge = EdgeHeadersConfig::default();
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
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        let written = site.join(".ssg/edge/wrangler-headers.toml");
        assert!(written.exists(), "wrangler-headers.toml must be written");
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
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        let written = site.join("_headers");
        assert!(written.exists());
        let body = fs::read_to_string(&written).unwrap();
        assert!(body.contains("Strict-Transport-Security"));
    }

    #[test]
    // Failpoints are process-global: this test reaches `vercel_render`
    // expecting success, so it must never run concurrently with
    // `fault_tests`'s injected `postprocess::vercel-render` failure —
    // joins that test's `#[serial]` lock as `#[parallel]` on the same
    // key (mirrors the convention in `core::cache`'s fault-injection
    // tests).
    #[serial_test::parallel(vercel_render_fp)]
    fn after_compile_vercel_target_writes_json() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
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
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
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
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        EdgeHeadersPlugin.after_compile(&ctx).unwrap();
        assert!(site.join(".ssg/edge/wrangler-headers.toml").exists());
    }

    #[test]
    #[serial_test::parallel(vercel_render_fp)]
    fn after_compile_all_three_targets_emit_all_three_artefacts() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["cloudflare", "netlify", "vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
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

    // ── per-page CSP wiring (spec B4, plan §3 item 2.4) ─────────────

    #[test]
    fn url_path_for_maps_index_and_plain_pages() {
        let site = Path::new("/tmp/site");
        assert_eq!(url_path_for(&site.join("index.html"), site), "/");
        assert_eq!(
            url_path_for(&site.join("blog/post/index.html"), site),
            "/blog/post/"
        );
        assert_eq!(url_path_for(&site.join("about.html"), site), "/about.html");
    }

    #[test]
    #[serial_test::parallel(vercel_render_fp)]
    fn transform_records_page_policy_into_platform_files() {
        // spec B4 acceptance: a page with inline JSON-LD gets a
        // per-path entry carrying that block's exact sha256, in both
        // _headers and vercel-headers.json.
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify", "vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );

        let plugin = EdgeHeadersPlugin::new();
        plugin.after_compile(&ctx).unwrap();

        let jsonld = r#"{"@type":"BlogPosting","headline":"x"}"#;
        let html = format!(
            r#"<html><head><script type="application/ld+json">{jsonld}</script></head><body>b</body></html>"#
        );
        let page = site.join("blog/post/index.html");
        let out = plugin.transform_html(&html, &page, &ctx).unwrap();
        assert_eq!(out, html, "transform must be a pass-through");

        let expected_hash =
            crate::cmd::SriAlgorithm::Sha256.integrity(jsonld.as_bytes());

        let headers_body = fs::read_to_string(site.join("_headers")).unwrap();
        assert!(
            headers_body.contains("/blog/post/\n"),
            "per-path group missing: {headers_body}"
        );
        assert!(
            headers_body
                .contains(&format!("script-src 'self' '{expected_hash}'")),
            "exact sha256 source missing: {headers_body}"
        );
        // test_csp_strict analogue: hash-strict, no 'unsafe-inline'.
        assert!(!headers_body.contains("unsafe-inline"));

        let vercel_body =
            fs::read_to_string(site.join(".ssg/edge/vercel-headers.json"))
                .unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&vercel_body).unwrap();
        let groups = parsed["headers"].as_array().unwrap();
        let page_group = groups
            .iter()
            .find(|g| g["source"].as_str() == Some("/blog/post/"))
            .expect("per-page vercel route present");
        let value = page_group["headers"][0]["value"].as_str().unwrap();
        assert!(value.contains(&format!("'{expected_hash}'")));
        assert!(!value.contains("unsafe-inline"));
    }

    #[test]
    fn transform_without_inline_blocks_keeps_global_files_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        let plugin = EdgeHeadersPlugin::new();
        plugin.after_compile(&ctx).unwrap();
        let before = fs::read_to_string(site.join("_headers")).unwrap();

        let html = "<html><head></head><body>plain</body></html>";
        let out = plugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        assert_eq!(out, html);

        let after = fs::read_to_string(site.join("_headers")).unwrap();
        assert_eq!(before, after, "no inline blocks ⇒ no re-emit");
    }

    #[test]
    fn transform_is_noop_when_disabled_or_unconfigured() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        // No config at all.
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let html = "<script>x=1</script>";
        let out = EdgeHeadersPlugin::new()
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        assert_eq!(out, html);
        assert!(!site.join("_headers").exists());
    }

    #[test]
    fn after_compile_resets_previous_builds_page_registry() {
        // Watch-mode contract: a rebuild must not carry forward pages
        // from the previous build.
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        let plugin = EdgeHeadersPlugin::new();

        plugin.after_compile(&ctx).unwrap();
        let html = "<html><head><script>x=1</script></head></html>";
        let _ = plugin
            .transform_html(html, &site.join("old/index.html"), &ctx)
            .unwrap();
        assert!(fs::read_to_string(site.join("_headers"))
            .unwrap()
            .contains("/old/"));

        // Second build: after_compile resets; the stale /old/ entry
        // must be gone from the re-emitted global file.
        plugin.after_compile(&ctx).unwrap();
        let body = fs::read_to_string(site.join("_headers")).unwrap();
        assert!(!body.contains("/old/"), "stale page survived reset: {body}");
    }

    #[test]
    fn two_pages_accumulate_sorted_entries() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        let plugin = EdgeHeadersPlugin::new();
        plugin.after_compile(&ctx).unwrap();

        let html_a = "<html><head><script>a=1</script></head></html>";
        let html_b = "<html><head><script>b=2</script></head></html>";
        let _ = plugin
            .transform_html(html_b, &site.join("zeta/index.html"), &ctx)
            .unwrap();
        let _ = plugin
            .transform_html(html_a, &site.join("alpha/index.html"), &ctx)
            .unwrap();

        let body = fs::read_to_string(site.join("_headers")).unwrap();
        let i_alpha = body.find("/alpha/").unwrap();
        let i_zeta = body.find("/zeta/").unwrap();
        assert!(
            i_alpha < i_zeta,
            "entries must be sorted regardless of insertion order"
        );
    }

    // -----------------------------------------------------------------
    // emit_targets error paths (directory/file collisions)
    // -----------------------------------------------------------------

    fn edge_cfg(targets: &[&str]) -> EdgeHeadersConfig {
        let mut edge = EdgeHeadersConfig::default();
        edge.targets = targets.iter().map(|t| (*t).to_string()).collect();
        edge
    }

    #[test]
    fn emit_targets_cloudflare_errors_when_ssg_dir_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        // A file named .ssg blocks create_dir_all(.ssg/edge).
        fs::write(site.join(".ssg"), "not a dir").unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err =
            emit_targets(&ctx, &edge_cfg(&["cloudflare"]), &BTreeMap::new())
                .unwrap_err();
        assert!(format!("{err}").contains(".ssg"));
    }

    #[test]
    fn emit_targets_cloudflare_errors_when_output_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(site.join(".ssg/edge/wrangler-headers.toml"))
            .unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err =
            emit_targets(&ctx, &edge_cfg(&["cloudflare"]), &BTreeMap::new())
                .unwrap_err();
        assert!(format!("{err}").contains("wrangler-headers.toml"));
    }

    #[test]
    fn emit_targets_netlify_errors_when_headers_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(site.join("_headers")).unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err = emit_targets(&ctx, &edge_cfg(&["netlify"]), &BTreeMap::new())
            .unwrap_err();
        assert!(format!("{err}").contains("_headers"));
    }

    #[test]
    fn emit_targets_vercel_errors_when_ssg_dir_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        fs::write(site.join(".ssg"), "not a dir").unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err = emit_targets(&ctx, &edge_cfg(&["vercel"]), &BTreeMap::new())
            .unwrap_err();
        assert!(format!("{err}").contains(".ssg"));
    }

    #[test]
    fn emit_targets_vercel_errors_when_output_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(site.join(".ssg/edge/vercel-headers.json")).unwrap();
        let ctx = PluginContext::new(dir.path(), dir.path(), &site, dir.path());
        let err = emit_targets(&ctx, &edge_cfg(&["vercel"]), &BTreeMap::new())
            .unwrap_err();
        assert!(format!("{err}").contains("vercel-headers.json"));
    }

    // -----------------------------------------------------------------
    // transform_html: emit failure propagates
    // -----------------------------------------------------------------

    #[test]
    fn transform_html_propagates_emit_failure() {
        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        // `_headers` exists as a directory → netlify write fails.
        fs::create_dir_all(site.join("_headers")).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        // Inline script gives the page a hash-strict CSP so the
        // transform reaches emit_targets.
        let html = "<html><head><script>var x = 1;</script></head><body></body></html>";
        let err = EdgeHeadersPlugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap_err();
        assert!(format!("{err}").contains("_headers"));
    }

    // -----------------------------------------------------------------
    // PAGE_CSP_REGISTRY poisoned-lock recovery
    // -----------------------------------------------------------------

    #[test]
    fn after_compile_and_transform_recover_from_poisoned_registry_lock() {
        // `.lock().unwrap_or_else(PoisonError::into_inner)` is a
        // defensive recovery arm that only runs if some other thread
        // panicked while holding the lock — no ordinary single-threaded
        // test reaches it. Deliberately poison the (process-global)
        // registry from a spawned thread so both call sites
        // (after_compile's reset, transform_html's insert) exercise
        // their recovery branch instead of a real production bug.
        let poisoned = std::thread::spawn(|| {
            let _guard = PAGE_CSP_REGISTRY.lock().unwrap();
            panic!("intentional poison for coverage of the recovery arm");
        })
        .join();
        assert!(poisoned.is_err(), "spawned thread must have panicked");

        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["netlify"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        let plugin = EdgeHeadersPlugin::new();

        // Must not panic despite the poisoned lock.
        plugin.after_compile(&ctx).unwrap();

        let html = "<html><head><script>x=1</script></head></html>";
        let out = plugin
            .transform_html(html, &site.join("index.html"), &ctx)
            .unwrap();
        assert_eq!(out, html);
    }
}

#[cfg(all(test, feature = "test-fault-injection"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod fault_tests {
    use super::*;

    /// RAII guard that disables a failpoint on drop.
    struct FailGuard(&'static str);

    impl Drop for FailGuard {
        fn drop(&mut self) {
            let _ = fail::cfg(self.0, "off");
        }
    }

    fn cfg_with_targets(targets: Vec<&str>) -> crate::cmd::SsgConfig {
        let mut edge = EdgeHeadersConfig::default();
        edge.targets = targets.into_iter().map(String::from).collect();
        crate::cmd::SsgConfig::builder()
            .site_name("t".to_string())
            .base_url("http://example.com".to_string())
            .edge_headers(edge)
            .build()
            .unwrap()
    }

    #[test]
    #[serial_test::serial(vercel_render_fp)]
    fn after_compile_vercel_maps_serialize_failure_to_io_error() {
        let _guard = FailGuard("postprocess::vercel-render");
        fail::cfg("postprocess::vercel-render", "return")
            .expect("activate failpoint");

        let dir = tempfile::tempdir().unwrap();
        let site = dir.path().join("site");
        fs::create_dir_all(&site).unwrap();
        let cfg = cfg_with_targets(vec!["vercel"]);
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            &site,
            dir.path(),
            cfg,
        );
        let err = EdgeHeadersPlugin
            .after_compile(&ctx)
            .expect_err("injected serialize failure must propagate");
        let msg = format!("{err}");
        assert!(msg.contains("vercel-headers.json"), "got: {msg}");
        assert!(
            msg.contains("injected: postprocess::vercel-render"),
            "got: {msg}"
        );
    }
}
