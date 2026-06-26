#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Agentic Discovery Example — `agents.txt` + `ai-plugin.json` + MCP
//! (v0.0.44, issue #552)
//!
//! Demonstrates the three opt-in agentic-discovery emitters by:
//!
//! 1. Building an [`AgentsConfig`] enabling all three artefacts.
//! 2. Printing the body of `agents.txt` via [`render_agents_txt`].
//! 3. Printing `ai-plugin.json` via [`build_manifest`].
//! 4. Printing `mcp.json` via [`build_registry`].
//!
//! Pure functions — no filesystem writes here. The full plugin pipeline
//! ([`ssg::postprocess::AgenticDiscoveryPlugin`]) wires these into
//! `after_compile` and writes them to disk.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example agentic_discovery_example
//! ```

use ssg::cmd::SsgConfig;
use ssg::postprocess::agentic_discovery::{
    build_manifest, build_registry, render_agents_txt, AgentRule, AgentsConfig,
    McpConfig,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Site config — `base_url` flows into all three emitters.
    let mut cfg = SsgConfig::default();
    cfg.site_name = "Acme Docs".into();
    cfg.site_title = "Acme Engineering Documentation".into();
    cfg.site_description = "How Acme builds things, in writing.".into();
    cfg.base_url = "https://docs.acme.example".into();

    // 2. Agent policy. Allow most agents but block GPTBot from /private.
    let mut rules = HashMap::new();
    let _ = rules.insert(
        "gptbot".to_string(),
        AgentRule {
            allow: vec!["/blog/*".into()],
            disallow: vec!["/private/*".into()],
        },
    );

    let agents = AgentsConfig {
        agents_txt: true,
        ai_plugin: true,
        mcp: McpConfig {
            enabled: true,
            auto_resources: false,
            ..McpConfig::default()
        },
        rules,
        default_rule: None,
    };
    cfg.agents = Some(agents.clone());

    // 3. Render each artefact via its pure-function builder.
    let agents_txt = render_agents_txt(&agents, &cfg.base_url);
    println!("──── agents.txt ({} bytes) ────", agents_txt.len());
    println!("{agents_txt}");

    let ai_plugin = build_manifest(&cfg);
    let ai_plugin_pretty = serde_json::to_string_pretty(&ai_plugin)?;
    println!(
        "──── /.well-known/ai-plugin.json ({} bytes) ────",
        ai_plugin_pretty.len(),
    );
    println!("{ai_plugin_pretty}");

    let mcp = build_registry(&cfg, &agents, &[]);
    let mcp_pretty = serde_json::to_string_pretty(&mcp)?;
    println!(
        "──── /.well-known/mcp.json ({} bytes) ────",
        mcp_pretty.len(),
    );
    println!("{mcp_pretty}");

    println!("[agentic-discovery] rendered all 3 artefacts");

    Ok(())
}
