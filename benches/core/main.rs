// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! # Benchmarks for `src/core/` — one binary, one submodule per source file.
//!
//! Wired via a single `[[bench]]` entry in the root `Cargo.toml`:
//!
//! ```toml
//! [[bench]]
//! name = "core"
//! path = "benches/core/main.rs"
//! ```
//!
//! Each `benches/core/<name>.rs` is a submodule that declares its own
//! `criterion_group!()` and exports it as `benches`. This file aggregates
//! all groups via `criterion_main!()`.
//!
//! Run with:
//!   * `cargo bench --bench core` — every core module's benchmarks
//!   * `cargo bench --bench core -- cache::` — only `cache` benchmarks

mod cache;

use criterion::criterion_main;

criterion_main!(cache::benches);
