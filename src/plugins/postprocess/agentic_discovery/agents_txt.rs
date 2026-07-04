// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `agents.txt` emitter — robots.txt-shaped allow/disallow rules per
//! AI agent identifier.
//!
//! Output layout (AC1):
//!
//! ```text
//! User-agent: *
//! Allow: /
//! Disallow: /private/
//! Sitemap: https://example.com/sitemap.xml
//!
//! User-agent: GPTBot
//! Allow: /blog/*
//! Disallow: /
//! ```
//!
//! The default `User-agent: *` block is overridable via
//! `[agents.default_rule]`; per-agent overrides land via
//! `[agents.rules.<id>]`.

use super::{AgentRule, AgentsConfig};
use crate::error::{PathErrorExt, SsgError};
use crate::plugin::PluginContext;
use std::fs;

/// Render and write the `agents.txt` file under `ctx.site_dir`.
///
/// # Errors
///
/// Returns [`SsgError::Io`] if `agents.txt` cannot be written (e.g.
/// the site directory is read-only or full).
///
/// # Examples
///
/// ```
/// use ssg::plugin::PluginContext;
/// use ssg::postprocess::agentic_discovery::{AgentsConfig, write_agents_txt};
/// use std::fs;
/// let tmp = tempfile::tempdir().unwrap();
/// let ctx = PluginContext::new(tmp.path(), tmp.path(), tmp.path(), tmp.path());
/// let cfg = AgentsConfig::default();
/// write_agents_txt(&ctx, &cfg).unwrap();
/// let body = fs::read_to_string(tmp.path().join("agents.txt")).unwrap();
/// assert!(body.contains("User-agent: *"));
/// ```
pub fn write_agents_txt(
    ctx: &PluginContext,
    agents: &AgentsConfig,
) -> Result<(), SsgError> {
    let base_url = ctx
        .config
        .as_ref()
        .map(|c| c.base_url.trim_end_matches('/').to_string())
        .unwrap_or_default();

    let body = render_agents_txt(agents, &base_url);
    let path = ctx.site_dir.join("agents.txt");
    fs::write(&path, body).with_path(&path)?;
    Ok(())
}

/// Pure-function renderer kept separate from I/O so unit tests can
/// assert the exact byte layout without touching the filesystem.
///
/// # Examples
///
/// ```
/// use ssg::postprocess::agentic_discovery::{AgentsConfig, render_agents_txt};
/// let cfg = AgentsConfig::default();
/// let body = render_agents_txt(&cfg, "https://example.com");
/// assert!(body.contains("User-agent: *"));
/// assert!(body.contains("Sitemap: https://example.com/sitemap.xml"));
/// ```
#[must_use]
pub fn render_agents_txt(agents: &AgentsConfig, base_url: &str) -> String {
    let mut out = String::new();

    // ── Default block (User-agent: *) ───────────────────────────────
    out.push_str("User-agent: *\n");
    if let Some(ref default_rule) = agents.default_rule {
        render_rule_body(default_rule, &mut out);
    } else {
        // AC1 defaults — overridable but always present so consumers
        // get a sensible policy out of the box.
        out.push_str("Allow: /\n");
        out.push_str("Disallow: /private/\n");
    }
    if !base_url.is_empty() {
        out.push_str("Sitemap: ");
        out.push_str(base_url);
        out.push_str("/sitemap.xml\n");
    }

    // ── Per-agent blocks ────────────────────────────────────────────
    //
    // HashMap iteration order is non-deterministic; sort by canonical
    // agent identifier so consumers (and our golden tests) see a
    // stable byte layout across builds.
    let mut ids: Vec<&String> = agents.rules.keys().collect();
    ids.sort();
    for id in ids {
        // Empty IDs would render as `User-agent: ` which is a parse
        // failure for downstream consumers — drop them silently
        // rather than emitting broken stanzas.
        if id.trim().is_empty() {
            continue;
        }
        let rule = &agents.rules[id];
        out.push('\n');
        out.push_str("User-agent: ");
        out.push_str(&canonicalise_agent_id(id));
        out.push('\n');
        render_rule_body(rule, &mut out);
    }

    out
}

/// Render the `Allow:` / `Disallow:` lines for a single rule, in the
/// order specified by the issue (allow first, disallow second).
fn render_rule_body(rule: &AgentRule, out: &mut String) {
    for path in &rule.allow {
        out.push_str("Allow: ");
        out.push_str(path);
        out.push('\n');
    }
    for path in &rule.disallow {
        out.push_str("Disallow: ");
        out.push_str(path);
        out.push('\n');
    }
}

/// Map TOML-friendly lowercase identifiers (`gptbot`, `claudebot`,
/// `perplexitybot`) to the canonical mixed-case form most agent
/// runtimes ship under (e.g. `GPTBot`, `ClaudeBot`).
///
/// Falls back to a title-case transform for unknown agents so the
/// output is never just the raw lowercase ID.
fn canonicalise_agent_id(id: &str) -> String {
    match id.to_ascii_lowercase().as_str() {
        "gptbot" => "GPTBot".to_string(),
        "chatgpt-user" => "ChatGPT-User".to_string(),
        "oai-searchbot" => "OAI-SearchBot".to_string(),
        "anthropic-ai" => "Anthropic-AI".to_string(),
        "claudebot" => "ClaudeBot".to_string(),
        "claude-web" => "Claude-Web".to_string(),
        "perplexitybot" => "PerplexityBot".to_string(),
        "google-extended" => "Google-Extended".to_string(),
        "googleother" => "GoogleOther".to_string(),
        "bingbot" => "Bingbot".to_string(),
        "ccbot" => "CCBot".to_string(),
        "applebot-extended" => "Applebot-Extended".to_string(),
        "facebookbot" => "FacebookBot".to_string(),
        "meta-externalagent" => "Meta-ExternalAgent".to_string(),
        "amazonbot" => "Amazonbot".to_string(),
        "diffbot" => "Diffbot".to_string(),
        "cohere-ai" => "Cohere-AI".to_string(),
        // Title-case fallback: "fooBot" → "Foobot", preserving inner
        // hyphens. Good enough for unknown agents; site authors who
        // want exact casing can extend the match arms above.
        _ => title_case_agent_id(id),
    }
}

fn title_case_agent_id(id: &str) -> String {
    // Title-case each `-`-separated segment so identifiers like
    // `my-custom-bot` render as `My-Custom-Bot`.
    id.split('-')
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(first) => {
                    let mut s = String::new();
                    for c in first.to_uppercase() {
                        s.push(c);
                    }
                    for c in chars {
                        for lower in c.to_lowercase() {
                            s.push(lower);
                        }
                    }
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn rule(allow: &[&str], disallow: &[&str]) -> AgentRule {
        AgentRule {
            allow: allow.iter().map(|s| (*s).to_string()).collect(),
            disallow: disallow.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn renders_default_block_when_no_overrides() {
        // AC1: with no overrides we still emit the User-agent: *
        // stanza plus the sitemap line.
        let cfg = AgentsConfig::default();
        let txt = render_agents_txt(&cfg, "https://example.com");
        assert!(txt.contains("User-agent: *\n"));
        assert!(txt.contains("Allow: /\n"));
        assert!(txt.contains("Disallow: /private/\n"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml\n"));
    }

    #[test]
    fn omits_sitemap_when_base_url_empty() {
        // If the site has no base URL configured we omit the
        // Sitemap line rather than emit a broken `Sitemap: /sitemap.xml`.
        let cfg = AgentsConfig::default();
        let txt = render_agents_txt(&cfg, "");
        assert!(!txt.contains("Sitemap:"));
    }

    #[test]
    fn renders_per_agent_rule_in_canonical_case() {
        // AC2: the gptbot rule must be emitted with the canonical
        // `GPTBot` casing the real crawler ships under.
        let mut cfg = AgentsConfig::default();
        let _ = cfg
            .rules
            .insert("gptbot".to_string(), rule(&["/blog/*"], &["/"]));
        let txt = render_agents_txt(&cfg, "https://example.com");
        assert!(txt.contains("User-agent: GPTBot\n"));
        assert!(txt.contains("Allow: /blog/*\n"));
        assert!(txt.contains("Disallow: /\n"));
    }

    #[test]
    fn per_agent_rules_render_in_sorted_order() {
        // HashMap iteration is non-deterministic; we sort so the
        // output is byte-stable across builds. Without sorting, the
        // golden tests above would flake.
        let mut rules: HashMap<String, AgentRule> = HashMap::new();
        let _ = rules.insert("perplexitybot".to_string(), rule(&[], &["/"]));
        let _ = rules.insert("gptbot".to_string(), rule(&["/blog/*"], &[]));
        let _ = rules.insert("ccbot".to_string(), rule(&[], &["/"]));
        let cfg = AgentsConfig {
            agents_txt: true,
            rules,
            ..AgentsConfig::default()
        };
        let txt = render_agents_txt(&cfg, "https://x.example");
        // Sorted by lowercase key: ccbot, gptbot, perplexitybot.
        let ccb = txt.find("User-agent: CCBot").unwrap();
        let gpt = txt.find("User-agent: GPTBot").unwrap();
        let pxy = txt.find("User-agent: PerplexityBot").unwrap();
        assert!(ccb < gpt, "ccbot stanza must precede gptbot");
        assert!(gpt < pxy, "gptbot stanza must precede perplexitybot");
    }

    #[test]
    fn allow_lines_precede_disallow_lines() {
        // AC2 explicitly mandates "allow/disallow lines in the
        // correct order" — Allow before Disallow per robots.txt
        // convention.
        let mut cfg = AgentsConfig::default();
        let _ = cfg
            .rules
            .insert("gptbot".to_string(), rule(&["/a", "/b"], &["/c", "/d"]));
        let txt = render_agents_txt(&cfg, "https://x.example");
        let a_pos = txt.find("Allow: /a").unwrap();
        let b_pos = txt.find("Allow: /b").unwrap();
        let c_pos = txt.find("Disallow: /c").unwrap();
        let d_pos = txt.find("Disallow: /d").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
        assert!(c_pos < d_pos);
    }

    #[test]
    fn default_rule_override_replaces_defaults() {
        // Site author supplied an explicit default_rule — the baked-in
        // Allow: / + Disallow: /private/ pair must NOT be emitted.
        let cfg = AgentsConfig {
            default_rule: Some(rule(&["/public/*"], &["/admin/*"])),
            ..AgentsConfig::default()
        };
        let txt = render_agents_txt(&cfg, "https://x.example");
        assert!(txt.contains("Allow: /public/*"));
        assert!(txt.contains("Disallow: /admin/*"));
        assert!(
            !txt.contains("Disallow: /private/"),
            "baked-in default must not leak through when an override exists"
        );
    }

    #[test]
    fn canonicalise_unknown_agent_falls_back_to_title_case() {
        // Title-case keeps each `-` segment capitalised so identifiers
        // like `my-custom-bot` come out readable.
        assert_eq!(canonicalise_agent_id("my-custom-bot"), "My-Custom-Bot");
        assert_eq!(canonicalise_agent_id("foobot"), "Foobot");
    }

    #[test]
    fn known_agents_match_canonical_case() {
        // Pin the well-known crawler identifiers so reformats and
        // accidental deletions are caught by the unit suite.
        assert_eq!(canonicalise_agent_id("gptbot"), "GPTBot");
        assert_eq!(canonicalise_agent_id("claudebot"), "ClaudeBot");
        assert_eq!(canonicalise_agent_id("anthropic-ai"), "Anthropic-AI");
        assert_eq!(canonicalise_agent_id("perplexitybot"), "PerplexityBot");
        assert_eq!(canonicalise_agent_id("google-extended"), "Google-Extended");
    }

    #[test]
    fn empty_agent_id_is_skipped() {
        // An empty or whitespace key would render as `User-agent: `
        // which is unparseable downstream — drop the stanza.
        let mut cfg = AgentsConfig::default();
        let _ = cfg.rules.insert(String::new(), rule(&[], &["/"]));
        let _ = cfg.rules.insert("   ".to_string(), rule(&[], &["/"]));
        let txt = render_agents_txt(&cfg, "https://x.example");
        // Only the default `*` stanza should be present.
        assert_eq!(txt.matches("User-agent:").count(), 1);
    }

    #[test]
    fn trims_trailing_slash_in_sitemap_url() {
        // base_url comes from SsgConfig — the renderer is fed an
        // already-trimmed URL by `write_agents_txt`, but we still
        // exercise the contract here.
        let cfg = AgentsConfig::default();
        let txt = render_agents_txt(&cfg, "https://example.com");
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
        assert!(!txt.contains("//sitemap.xml"));
    }

    #[test]
    fn all_known_agent_ids_map_to_canonical_case() {
        // Pin the remaining well-known identifiers not covered above.
        let expected = [
            ("chatgpt-user", "ChatGPT-User"),
            ("oai-searchbot", "OAI-SearchBot"),
            ("claude-web", "Claude-Web"),
            ("googleother", "GoogleOther"),
            ("bingbot", "Bingbot"),
            ("ccbot", "CCBot"),
            ("applebot-extended", "Applebot-Extended"),
            ("facebookbot", "FacebookBot"),
            ("meta-externalagent", "Meta-ExternalAgent"),
            ("amazonbot", "Amazonbot"),
            ("diffbot", "Diffbot"),
            ("cohere-ai", "Cohere-AI"),
        ];
        for (id, canonical) in expected {
            assert_eq!(canonicalise_agent_id(id), canonical, "agent {id}");
        }
    }

    #[test]
    fn title_case_handles_empty_segments() {
        // `my--bot` has an empty middle segment; it must round-trip
        // without panicking and keep the double hyphen.
        assert_eq!(canonicalise_agent_id("my--bot"), "My--Bot");
    }

    #[test]
    fn write_agents_txt_errors_when_target_is_a_directory() {
        use crate::plugin::PluginContext;
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("agents.txt")).unwrap();
        let ctx =
            PluginContext::new(tmp.path(), tmp.path(), tmp.path(), tmp.path());
        let cfg = AgentsConfig::default();
        let err = write_agents_txt(&ctx, &cfg).unwrap_err();
        assert!(format!("{err}").contains("agents.txt"));
    }
}
