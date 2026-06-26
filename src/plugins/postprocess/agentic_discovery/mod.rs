// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Agentic discovery emitters (issue #552).
//!
//! Coordinates three modern agentic-discovery protocol files alongside
//! the existing `robots.txt` / `sitemap.xml` family:
//!
//! 1. **`agents.txt`** — a `robots.txt`-shaped plain-text spec listing
//!    AI agent identifiers and allow/disallow rules. Emitted at
//!    `/agents.txt`.
//! 2. **`.well-known/ai-plugin.json`** — the `OpenAI` plugin manifest
//!    spec (still the de-facto plugin descriptor across agent runtimes
//!    in 2026). Emitted at `/.well-known/ai-plugin.json`.
//! 3. **MCP registry** (`/.well-known/mcp.json`) — Model Context
//!    Protocol registry listing exposed `resources`, `tools`, and
//!    `prompts` over HTTP transport (the default delivery channel).
//!
//! Each emitter is **opt-in** per the `[agents]` section of `ssg.toml`.
//! When `[agents]` is absent (or every flag is `false`), this plugin is
//! a no-op — none of the three files are written. This preserves the
//! "you didn't ask for it" guarantee.
//!
//! # Configuration shape
//!
//! ```toml
//! [agents]
//! agents_txt = true
//! ai_plugin  = true
//!
//! [agents.mcp]
//! enabled         = true
//! transport       = "http"
//! auto_resources  = true   # walk content sidecars and emit MCP
//!                          # resources for every public page
//!
//! # Per-agent overrides for agents.txt (robots.txt style)
//! [agents.rules.gptbot]
//! allow    = ["/blog/*"]
//! disallow = ["/"]
//! ```
//!
//! Per-page frontmatter may also carry an `agents.disallow` list (read
//! from `.meta.json` sidecars by the MCP emitter), which causes the
//! page to be skipped when populating MCP resources.

mod agents_txt;
mod ai_plugin;
mod mcp;

use crate::error::SsgError;
use crate::plugin::{Plugin, PluginContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use agents_txt::{render_agents_txt, write_agents_txt};
pub use ai_plugin::{build_manifest, write_ai_plugin_json};
pub use mcp::{
    build_registry, collect_mcp_resources, write_mcp_registry, McpResource,
};

// =====================================================================
// Configuration types — surfaced from `ssg.toml` via `SsgConfig::agents`
// =====================================================================

/// The `[agents]` section of `ssg.toml`. Every field is optional;
/// absent values fall back to safe "off" defaults so that omitting
/// the section produces no new files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentsConfig {
    /// Emit `/agents.txt` (a `robots.txt`-shaped agent policy file).
    #[serde(default)]
    pub agents_txt: bool,

    /// Emit `/.well-known/ai-plugin.json` (`OpenAI` plugin manifest).
    #[serde(default)]
    pub ai_plugin: bool,

    /// MCP registry settings. Absent ⇒ disabled.
    #[serde(default)]
    pub mcp: McpConfig,

    /// Per-agent allow/disallow overrides keyed by agent identifier
    /// (e.g. `"gptbot"`, `"claudebot"`). The agent ID is preserved
    /// in TOML lowercase but rendered in canonical case
    /// (`User-agent: GPTBot`) by [`agents_txt::write_agents_txt`].
    #[serde(default)]
    pub rules: HashMap<String, AgentRule>,

    /// Default `User-agent: *` rule. When `None`, a permissive
    /// `Allow: /` + `Disallow: /private/` default is emitted.
    #[serde(default)]
    pub default_rule: Option<AgentRule>,
}

impl AgentsConfig {
    /// Returns `true` when at least one emitter is enabled. When this
    /// is `false`, the coordinator plugin is a complete no-op.
    #[must_use]
    pub const fn any_enabled(&self) -> bool {
        self.agents_txt || self.ai_plugin || self.mcp.enabled
    }
}

/// MCP registry settings. Mirrors the JSON shape consumed by clients
/// (Claude Desktop, IDE integrations, …) but expressed in TOML so site
/// authors can edit it alongside the rest of `ssg.toml`.
///
/// Hand-rolled `Default` so the `transport` and `protocol_version`
/// fields carry the same defaults that `#[serde(default = …)]` applies
/// during deserialisation — otherwise constructing the struct via
/// `McpConfig::default()` (rather than parsing TOML) would leave both
/// strings empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Master toggle. When `false`, no `mcp.json` is written.
    #[serde(default)]
    pub enabled: bool,

    /// Transport descriptor — currently always `"http"`. Held as a
    /// string so future transports (`"stdio"`, `"sse"`) can land
    /// without bumping the schema.
    #[serde(default = "default_transport")]
    pub transport: String,

    /// Override the URL the transport listens on. When omitted, the
    /// emitter synthesises `{base_url}/.well-known/mcp` so the
    /// registry is self-describing.
    #[serde(default)]
    pub url: Option<String>,

    /// MCP protocol version reported in the registry. Defaults to
    /// `"2025-03-26"`, the version pinned across the public MCP
    /// servers we tested against during the v0.0.44 cycle.
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,

    /// When `true`, walk `.meta.json` sidecars under `site_dir` and
    /// emit one MCP `resource` per public page. AC5 of #552.
    #[serde(default)]
    pub auto_resources: bool,

    /// Statically-declared MCP tools. Optional — most sites won't
    /// expose any. Authors may extend this list in `ssg.toml`.
    #[serde(default)]
    pub tools: Vec<McpToolDecl>,

    /// Statically-declared MCP prompt templates.
    #[serde(default)]
    pub prompts: Vec<McpPromptDecl>,
}

fn default_transport() -> String {
    "http".to_string()
}

fn default_protocol_version() -> String {
    "2025-03-26".to_string()
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transport: default_transport(),
            url: None,
            protocol_version: default_protocol_version(),
            auto_resources: false,
            tools: Vec::new(),
            prompts: Vec::new(),
        }
    }
}

/// A single `User-agent: …` block for `agents.txt`. Mirrors the
/// `robots.txt` grammar but is also reachable from per-page frontmatter
/// (via the `agents:` key on a `.meta.json` sidecar).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentRule {
    /// Allowed URL prefixes (e.g. `"/blog/*"`).
    #[serde(default)]
    pub allow: Vec<String>,

    /// Disallowed URL prefixes.
    #[serde(default)]
    pub disallow: Vec<String>,
}

/// Static MCP tool declaration. The emitter passes these through
/// verbatim to the registry JSON — schema validation is the consumer's
/// problem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDecl {
    /// Tool identifier (must be unique within the registry).
    pub name: String,
    /// One-line description shown in client UIs.
    pub description: String,
    /// Optional JSON Schema for the tool's input. Stored as a free-form
    /// `serde_json::Value` so authors can paste arbitrary schemas in
    /// `ssg.toml` without us re-modelling JSON Schema in Rust.
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// Static MCP prompt declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptDecl {
    /// Prompt identifier.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Optional argument schema (free-form JSON).
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

// =====================================================================
// Plugin
// =====================================================================

/// Coordinator plugin that fans out to the three agentic-discovery
/// emitters. Registered once in `register_default_plugins` and runs in
/// `after_compile`.
///
/// When no [`AgentsConfig`] is present on the context, or every flag is
/// `false`, the plugin is a no-op — none of the three files are
/// written. This means existing sites upgrading to v0.0.44 see no
/// behavioural change until they opt in.
#[derive(Debug, Clone, Copy, Default)]
pub struct AgenticDiscoveryPlugin;

impl Plugin for AgenticDiscoveryPlugin {
    fn name(&self) -> &'static str {
        "agentic-discovery"
    }

    fn after_compile(&self, ctx: &PluginContext) -> Result<(), SsgError> {
        if !ctx.site_dir.exists() {
            return Ok(());
        }

        let Some(cfg) = ctx.config.as_ref() else {
            return Ok(());
        };
        let Some(ref agents) = cfg.agents else {
            return Ok(());
        };

        if !agents.any_enabled() {
            return Ok(());
        }

        if agents.agents_txt {
            write_agents_txt(ctx, agents)?;
        }

        if agents.ai_plugin {
            write_ai_plugin_json(ctx, cfg)?;
        }

        if agents.mcp.enabled {
            write_mcp_registry(ctx, cfg, agents)?;
        }

        Ok(())
    }
}

// =====================================================================
// Tests — unit-level only; integration coverage lives in
// `tests/agentic_discovery.rs`.
// =====================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_ctx(dir: &Path) -> PluginContext {
        PluginContext::new(dir, dir, dir, dir)
    }

    #[test]
    fn plugin_name_is_stable() {
        // The name appears in log lines and the PluginManager API —
        // pin it so renames are deliberate.
        assert_eq!(AgenticDiscoveryPlugin.name(), "agentic-discovery");
    }

    #[test]
    fn no_config_is_no_op() {
        // A context with no SsgConfig must not crash and must not
        // produce any files.
        let dir = tempdir().unwrap();
        let ctx = test_ctx(dir.path());
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        assert!(!dir.path().join("agents.txt").exists());
        assert!(!dir.path().join(".well-known/ai-plugin.json").exists());
        assert!(!dir.path().join(".well-known/mcp.json").exists());
    }

    #[test]
    fn no_site_dir_is_no_op() {
        // Site dir missing → plugin succeeds silently.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope");
        let ctx = test_ctx(&missing);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        assert!(!missing.exists());
    }

    #[test]
    fn any_enabled_false_by_default() {
        // Empty config — every flag off → any_enabled() is false →
        // emitters are skipped.
        let cfg = AgentsConfig::default();
        assert!(!cfg.any_enabled());
    }

    #[test]
    fn any_enabled_reflects_each_flag() {
        let mut cfg = AgentsConfig::default();
        cfg.agents_txt = true;
        assert!(cfg.any_enabled());

        let mut cfg = AgentsConfig::default();
        cfg.ai_plugin = true;
        assert!(cfg.any_enabled());

        let mut cfg = AgentsConfig::default();
        cfg.mcp.enabled = true;
        assert!(cfg.any_enabled());
    }

    #[test]
    fn mcp_defaults_are_sane() {
        // The auto-populated MCP defaults match what we promise in
        // the issue body (HTTP transport, off-by-default).
        let mcp = McpConfig::default();
        assert!(!mcp.enabled);
        assert_eq!(mcp.transport, "http");
        assert_eq!(mcp.protocol_version, "2025-03-26");
        assert!(!mcp.auto_resources);
        assert!(mcp.tools.is_empty());
        assert!(mcp.prompts.is_empty());
        assert!(mcp.url.is_none());
    }

    #[test]
    fn parses_toml_with_all_three_emitters_enabled() {
        // The canonical config snippet from the issue body must
        // round-trip cleanly via toml::from_str.
        let toml_str = r#"
            agents_txt = true
            ai_plugin  = true

            [mcp]
            enabled = true
            transport = "http"
            auto_resources = true

            [rules.gptbot]
            allow    = ["/blog/*"]
            disallow = ["/"]
        "#;
        let cfg: AgentsConfig = toml::from_str(toml_str).unwrap();
        assert!(cfg.agents_txt);
        assert!(cfg.ai_plugin);
        assert!(cfg.mcp.enabled);
        assert!(cfg.mcp.auto_resources);
        assert_eq!(cfg.mcp.transport, "http");
        let rule = cfg.rules.get("gptbot").unwrap();
        assert_eq!(rule.allow, vec!["/blog/*"]);
        assert_eq!(rule.disallow, vec!["/"]);
    }

    #[test]
    fn empty_toml_parses_with_defaults() {
        // Absent fields must deserialise to the safe "off" defaults.
        let cfg: AgentsConfig = toml::from_str("").unwrap();
        assert!(!cfg.any_enabled());
        assert!(cfg.rules.is_empty());
        assert!(cfg.default_rule.is_none());
    }

    fn ctx_with_config(dir: &Path, agents: AgentsConfig) -> PluginContext {
        let mut cfg = crate::cmd::SsgConfig::default();
        cfg.base_url = "https://example.test".to_string();
        cfg.site_name = "Example".to_string();
        cfg.site_title = "Example".to_string();
        cfg.site_description = "A demo".to_string();
        cfg.agents = Some(agents);
        PluginContext::with_config(dir, dir, dir, dir, cfg)
    }

    #[test]
    fn agents_txt_enabled_writes_file() {
        let dir = tempdir().unwrap();
        let agents = AgentsConfig {
            agents_txt: true,
            ..AgentsConfig::default()
        };
        let ctx = ctx_with_config(dir.path(), agents);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        let body =
            std::fs::read_to_string(dir.path().join("agents.txt")).unwrap();
        assert!(body.contains("User-agent: *"));
        assert!(!dir.path().join(".well-known/ai-plugin.json").exists());
        assert!(!dir.path().join(".well-known/mcp.json").exists());
    }

    #[test]
    fn ai_plugin_enabled_writes_file() {
        let dir = tempdir().unwrap();
        let agents = AgentsConfig {
            ai_plugin: true,
            ..AgentsConfig::default()
        };
        let ctx = ctx_with_config(dir.path(), agents);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        let path = dir.path().join(".well-known/ai-plugin.json");
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"schema_version\""));
        assert!(!dir.path().join("agents.txt").exists());
    }

    #[test]
    fn mcp_enabled_writes_file() {
        let dir = tempdir().unwrap();
        let mut agents = AgentsConfig::default();
        agents.mcp.enabled = true;
        let ctx = ctx_with_config(dir.path(), agents);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        let path = dir.path().join(".well-known/mcp.json");
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"protocolVersion\""));
    }

    #[test]
    fn all_three_emitters_enabled_writes_all_files() {
        let dir = tempdir().unwrap();
        let mut agents = AgentsConfig {
            agents_txt: true,
            ai_plugin: true,
            ..AgentsConfig::default()
        };
        agents.mcp.enabled = true;
        let ctx = ctx_with_config(dir.path(), agents);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        assert!(dir.path().join("agents.txt").exists());
        assert!(dir.path().join(".well-known/ai-plugin.json").exists());
        assert!(dir.path().join(".well-known/mcp.json").exists());
    }

    #[test]
    fn config_present_but_agents_none_is_no_op() {
        let dir = tempdir().unwrap();
        let cfg = crate::cmd::SsgConfig::default();
        let ctx = PluginContext::with_config(
            dir.path(),
            dir.path(),
            dir.path(),
            dir.path(),
            cfg,
        );
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        assert!(!dir.path().join("agents.txt").exists());
    }

    #[test]
    fn all_flags_off_is_no_op_even_with_rules() {
        let dir = tempdir().unwrap();
        let mut agents = AgentsConfig::default();
        let _ = agents.rules.insert(
            "gptbot".to_string(),
            AgentRule {
                allow: vec!["/blog/*".to_string()],
                disallow: vec![],
            },
        );
        assert!(!agents.any_enabled());
        let ctx = ctx_with_config(dir.path(), agents);
        AgenticDiscoveryPlugin.after_compile(&ctx).unwrap();
        assert!(!dir.path().join("agents.txt").exists());
    }

    #[test]
    fn mcp_config_serde_round_trip() {
        let mcp = McpConfig {
            enabled: true,
            transport: "http".to_string(),
            url: Some("https://api.example/mcp".to_string()),
            protocol_version: "2025-03-26".to_string(),
            auto_resources: true,
            tools: vec![McpToolDecl {
                name: "search".to_string(),
                description: "Search the site".to_string(),
                input_schema: Some(serde_json::json!({"type":"object"})),
            }],
            prompts: vec![McpPromptDecl {
                name: "summary".to_string(),
                description: "Summarise".to_string(),
                arguments: None,
            }],
        };
        let json = serde_json::to_string(&mcp).unwrap();
        let back: McpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transport, "http");
        assert_eq!(back.tools.len(), 1);
        assert_eq!(back.tools[0].name, "search");
        assert_eq!(back.prompts[0].name, "summary");
    }

    #[test]
    fn plugin_default_and_copy_traits() {
        let a = AgenticDiscoveryPlugin;
        let b: AgenticDiscoveryPlugin = a;
        let _c = a;
        assert_eq!(a.name(), b.name());
        let default_plugin = <AgenticDiscoveryPlugin as Default>::default();
        assert_eq!(default_plugin.name(), "agentic-discovery");
        assert!(format!("{a:?}").contains("AgenticDiscoveryPlugin"));
    }
}
