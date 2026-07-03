#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Agent JSON API + oEmbed Example (issue #586, ports 3 & 4)
//!
//! Demonstrates the two v0.0.47 "agent surface" emitters:
//!
//! 1. **[`ssg::agent_api::AgentApiPlugin`]** (default-on) — emits
//!    `/api/agents/{index,posts,topics,person}.json`, a stable JSON
//!    API for AI crawlers and agent toolchains: post metadata with
//!    word counts, a topic → URL map, and a JSON-LD `Person` entity
//!    for the site author.
//! 2. **[`ssg::oembed::OembedPlugin`]** (opt-in) — emits an oEmbed
//!    1.0 `link`-type document per shareable page plus the
//!    `<link rel="alternate" type="application/json+oembed">`
//!    discovery tag.
//!
//! The example builds a tiny three-post fixture site in a temp
//! directory, runs both plugins the same way the pipeline does
//! (`after_compile`, then the fused `transform_html` pass), and
//! prints every emitted document.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example agent_api_example
//! ```

use ssg::cmd::SsgConfig;
use ssg::plugin::{Plugin, PluginContext};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ----------------------------------------------------------------
    // 1. Fixture site: three pages with frontmatter sidecars — the
    //    `.meta.json` convention the compiler emits under
    //    `<build>/.meta/` for every Markdown source.
    // ----------------------------------------------------------------
    let tmp = tempfile::tempdir()?;
    let build_dir = tmp.path().join("build");
    let site_dir = tmp.path().join("public");
    let meta_dir = build_dir.join(".meta");
    fs::create_dir_all(&meta_dir)?;
    fs::create_dir_all(site_dir.join("blog"))?;

    let pages: &[(&str, &str, &str)] = &[
        (
            "index",
            r#"{
                "title": "Acme Engineering",
                "description": "How Acme builds things, in writing.",
                "author": "hello@acme.example (Acme Team)",
                "date": "2026-06-01",
                "tags": "engineering, home",
                "word_count": 180
            }"#,
            "<html><head><title>Acme Engineering</title></head>\
             <body><p>Welcome to the Acme engineering journal.</p></body></html>",
        ),
        (
            "blog/rust-pipelines",
            r#"{
                "title": "Rust build pipelines",
                "description": "Deterministic builds with plugins.",
                "author": "hello@acme.example (Acme Team)",
                "date": "2026-06-14",
                "tags": "rust, pipelines, engineering",
                "word_count": 950
            }"#,
            "<html><head><title>Rust build pipelines</title></head>\
             <body><p>Plugins run in registration order…</p></body></html>",
        ),
        (
            // No word_count in the sidecar — exercises the fallback
            // chain (JSON-LD wordCount, then stripped-HTML count).
            "blog/agent-readiness",
            r#"{
                "title": "Agent readiness for static sites",
                "description": "llms.txt, MCP, and now a JSON API.",
                "author": "hello@acme.example (Acme Team)",
                "date": "2026-06-28",
                "tags": "agents, rust"
            }"#,
            "<html><head><title>Agent readiness</title>\
             <script type=\"application/ld+json\">\
             {\"@type\":\"BlogPosting\",\"wordCount\":1234}\
             </script></head>\
             <body><p>Agents prefer structured surfaces.</p></body></html>",
        ),
    ];

    for (stem, meta, html) in pages {
        let sidecar = meta_dir.join(format!("{stem}.meta.json"));
        if let Some(parent) = sidecar.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(sidecar, meta)?;
        fs::write(site_dir.join(format!("{stem}.html")), html)?;
    }

    // ----------------------------------------------------------------
    // 2. Plugin context — same shape the pipeline hands every plugin.
    // ----------------------------------------------------------------
    let mut cfg = SsgConfig::default();
    cfg.site_name = "Acme".into();
    cfg.site_title = "Acme Engineering".into();
    cfg.site_description = "How Acme builds things, in writing.".into();
    cfg.base_url = "https://acme.example".into();
    cfg.language = "en".into();

    let ctx = PluginContext::with_config(
        tmp.path(),
        &build_dir,
        &site_dir,
        tmp.path(),
        cfg,
    );

    // ----------------------------------------------------------------
    // 3. AgentApiPlugin (#586 port 3) — after_compile emits the four
    //    /api/agents/*.json documents.
    // ----------------------------------------------------------------
    let agent_api = ssg::agent_api::AgentApiPlugin::default();
    agent_api.after_compile(&ctx)?;

    for doc in ["index.json", "posts.json", "topics.json", "person.json"] {
        let path = site_dir.join("api/agents").join(doc);
        let body = fs::read_to_string(&path)?;
        println!("──── /api/agents/{doc} ({} bytes) ────", body.len());
        println!("{body}");
    }

    // ----------------------------------------------------------------
    // 4. OembedPlugin (#586 port 4) — after_compile writes the
    //    per-page documents; transform_html injects the discovery
    //    <link> (the pipeline's fused transform pass does this for
    //    every page after after_compile).
    // ----------------------------------------------------------------
    let oembed = ssg::oembed::OembedPlugin;
    oembed.after_compile(&ctx)?;

    let page = site_dir.join("blog/rust-pipelines.html");
    let html = fs::read_to_string(&page)?;
    let html = oembed.transform_html(&html, &page, &ctx)?;
    fs::write(&page, &html)?;

    let oembed_doc =
        fs::read_to_string(site_dir.join("blog/rust-pipelines.oembed.json"))?;
    println!(
        "──── /blog/rust-pipelines.oembed.json ({} bytes) ────",
        oembed_doc.len()
    );
    println!("{oembed_doc}");

    let link_line = html
        .lines()
        .find(|l| l.contains("json+oembed"))
        .unwrap_or("<not injected>");
    println!("──── discovery link injected into <head> ────");
    println!("{}", link_line.trim());

    println!("\n[agent-api + oembed] rendered all artefacts");
    Ok(())
}
