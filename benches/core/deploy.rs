// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! Benchmarks for `ssg::deploy`.

use criterion::{criterion_group, BatchSize, Criterion};
use ssg::deploy::{DeployPlugin, DeployTarget};
use ssg::plugin::{Plugin, PluginContext};

fn bench_after_compile_netlify(c: &mut Criterion) {
    c.bench_function("deploy::after_compile_netlify", |b| {
        b.iter_batched(
            tempfile::tempdir,
            |dir| {
                let dir = dir.unwrap();
                let site = dir.path();
                std::fs::create_dir_all(site).unwrap();
                let ctx = PluginContext::new(site, site, site, site);
                let plugin = DeployPlugin::new(DeployTarget::Netlify);
                plugin.after_compile(&ctx).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_after_compile_netlify);
