// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

use criterion::{criterion_group, Criterion};
use ssg::i18n::{negotiate_locale, parse_accept_language};

fn bench_parse_accept_language(c: &mut Criterion) {
    let header = "en-GB,en;q=0.9,fr;q=0.8,de;q=0.7,es;q=0.6,it;q=0.5";
    c.bench_function("i18n::parse_accept_language", |b| {
        b.iter(|| parse_accept_language(header));
    });
}

fn bench_negotiate_locale(c: &mut Criterion) {
    let user = vec!["fr-FR".to_string(), "en-US".to_string()];
    let avail = vec![
        "en-US".to_string(),
        "fr-FR".to_string(),
        "de-DE".to_string(),
    ];
    c.bench_function("i18n::negotiate_locale", |b| {
        b.iter(|| negotiate_locale(&user, &avail, "en-US"));
    });
}

criterion_group!(benches, bench_parse_accept_language, bench_negotiate_locale);
