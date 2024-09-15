// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

#![allow(missing_docs)]

//! # Translation Benchmark for SSG I18n
//!
//! This benchmark measures the performance of the translation functionality in the `ssg_i18n` library using the `criterion` crate.
//!

use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg_i18n::Translator;

/// Benchmark the translation of various strings using the `ssg_i18n` library.
fn benchmark_translation(c: &mut Criterion) {
    let translator = Translator::new("fr").unwrap();

    c.bench_function("translate hello to french", |b| {
        b.iter(|| translator.translate(black_box("Hello")).unwrap())
    });
}

criterion_group!(benches, benchmark_translation);
criterion_main!(benches);
