#![allow(clippy::unwrap_used, clippy::expect_used)]
// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Search Example — Browser-native vector semantic search (v0.0.44)
//!
//! Demonstrates the `ssg-search` crate end-to-end:
//!
//! 1. Build a small corpus with [`ArtifactsBuilder`].
//! 2. Hand the resulting artifacts to a [`VectorEngine`].
//! 3. Run a top-3 cosine-similarity query and pretty-print the hits.
//!
//! Same pipeline the SSG search plugin uses at build time, scaled down
//! to four documents so the run completes in milliseconds.
//!
//! ## Run it
//!
//! ```sh
//! cargo run --example search_example
//! ```

use ssg_search::artifacts::{ArtifactsBuilder, InputDoc};
use ssg_search::engine::VectorEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build a tiny corpus — four pages.
    let docs = [
        InputDoc {
            url: "/posts/rust-async.html".into(),
            title: "Rust async/await primer".into(),
            body: "rust async await tokio futures runtime executor".into(),
            excerpt: "An introduction to async/await in Rust".into(),
        },
        InputDoc {
            url: "/posts/wasm-bundle.html".into(),
            title: "Shipping WASM bundles".into(),
            body: "wasm bundle size wasm-opt wasm-bindgen".into(),
            excerpt: "Tips to shrink your WebAssembly output".into(),
        },
        InputDoc {
            url: "/posts/iso20022.html".into(),
            title: "ISO 20022 payments primer".into(),
            body: "iban bic sepa payment instruction iso20022".into(),
            excerpt: "What ISO 20022 means for fintech engineers".into(),
        },
        InputDoc {
            url: "/posts/csp-hardening.html".into(),
            title: "CSP hardening for static sites".into(),
            body: "csp content security policy headers sri hash".into(),
            excerpt: "Lock down your static site with CSP and SRI".into(),
        },
    ];

    let mut builder = ArtifactsBuilder::default();
    for d in &docs {
        let _ = builder.add_doc(d.clone());
    }
    let arts = builder.build();
    let count = arts.count();
    println!(
        "[search] indexed {} docs at dim {} (model hash {})",
        count,
        arts.dim(),
        &arts.model_hash[..16],
    );

    // 2. Construct the runtime engine over the artifact bytes.
    let engine = VectorEngine::new(
        &arts.model,
        &arts.tokenizer,
        &arts.embeddings,
        count,
    )?;

    // 3. Query top-3 for a payments-shaped query.
    let query = "sepa iban payment";
    let hits = engine.search(query, 3);
    println!("[search] query: {query:?} → {} hits", hits.len() / 2);
    for pair in hits.chunks_exact(2) {
        let idx = pair[0] as usize;
        let score = pair[1];
        let entry = &arts.manifest.entries[idx];
        println!(
            "  rank score={score:.4}  {url}  — {title}",
            url = entry.url,
            title = entry.title,
        );
    }

    Ok(())
}
