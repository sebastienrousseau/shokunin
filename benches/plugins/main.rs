// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `src/plugins/`. Only a curated subset of plugins
//! exposes pure-function hot paths worth criterion-measuring; trait-impl
//! plugins are exercised end-to-end via integration tests instead.

mod i18n;
mod markdown_ext;
mod og_image;
mod seo;
mod shortcodes;

use criterion::criterion_main;

criterion_main!(
    i18n::benches,
    markdown_ext::benches,
    og_image::benches,
    seo::benches,
    shortcodes::benches
);
