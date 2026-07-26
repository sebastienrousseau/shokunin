// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Integration tests for `src/core/` — one binary, one submodule per source file.
//!
//! Wired via a single `[[test]]` entry in the root `Cargo.toml`:
//!
//! ```toml
//! [[test]]
//! name = "core"
//! path = "tests/core/main.rs"
//! ```
//!
//! Each `tests/core/<name>.rs` is a submodule containing `#[test]` functions
//! that exercise the corresponding `src/core/<name>.rs` public API surface.
//!
//! Run with:
//!   * `cargo test --test core` — every core module's integration tests
//!   * `cargo test --test core cache::` — only `cache` module tests
//!   * `cargo test --test core -- some_fn_name` — specific test by name

#![allow(clippy::unwrap_used, clippy::expect_used)]
mod cache;
mod collections;
mod content;
mod content_stager;
mod dates;
mod depgraph;
mod deploy;
mod frontmatter;
mod fs_ops;
mod logging;
mod otel;
mod paths;
mod pipeline;
mod process;
mod scaffold;
mod schema;
mod stream;
mod streaming;
mod template_engine;
mod urls;
mod walk;
