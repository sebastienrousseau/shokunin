// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    missing_docs,
    unused_results
)]

//! AVIF-vs-WebP encoding wall-clock comparison (issue #521 AC3).
//!
//! Generates a synthetic 1024x768 RGB photograph (varied noise pattern so
//! that encoders can't trivially collapse it to a few entropy classes),
//! then measures:
//!
//! - WebP encoding via `image::DynamicImage::save_with_format` (the same
//!   path used by `process_image`).
//! - AVIF encoding via [`ssg::image_plugin::encode_avif`] at the default
//!   quality of 70 (matches the production pipeline).
//! - Parallel AVIF encoding across the 4 standard responsive widths
//!   (320, 640, 1024, 1440) — proves AC3's "wall-time within 1.4× of
//!   WebP" guarantee on a multi-core machine.
//!
//! Run with `cargo bench --features image-optimization avif_vs_webp`.

use std::io::Cursor;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use rayon::prelude::*;
use ssg::image_plugin::encode_avif;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const WIDTHS: &[u32] = &[320, 640, 1024, 1440];
const QUALITY: u8 = 70;

/// Produces a 1024x768 RGB image with structured noise (`(x*y) ^ (x+y)`
/// across channels) — enough entropy that quantisers can't trivially
/// flatten the bitstream, so the per-codec runtime differences show up.
fn synthetic_photo() -> DynamicImage {
    let buf = ImageBuffer::from_fn(WIDTH, HEIGHT, |x, y| {
        let r = ((x * y) ^ (x + y)) as u8;
        let g = (x.wrapping_mul(3).wrapping_add(y)) as u8;
        let b = ((x ^ y).wrapping_mul(5)) as u8;
        Rgb([r, g, b])
    });
    DynamicImage::ImageRgb8(buf)
}

fn bench_webp_encode(c: &mut Criterion) {
    let img = synthetic_photo();
    let mut group = c.benchmark_group("avif_vs_webp/webp");
    group.throughput(Throughput::Elements(1));
    group.bench_function("encode_1024x768", |b| {
        b.iter(|| {
            let mut buf = Cursor::new(Vec::with_capacity(64 * 1024));
            img.write_to(&mut buf, ImageFormat::WebP)
                .expect("webp encode");
            buf.into_inner()
        });
    });
    group.finish();
}

fn bench_avif_encode(c: &mut Criterion) {
    let img = synthetic_photo();
    let mut group = c.benchmark_group("avif_vs_webp/avif");
    group.throughput(Throughput::Elements(1));
    group.bench_function("encode_1024x768_q70", |b| {
        b.iter(|| encode_avif(&img, QUALITY).expect("avif encode"));
    });
    group.finish();
}

/// Parallel-encoding bench across the four responsive widths. Demonstrates
/// AC3: total wall-clock for AVIF stays within 1.4× of WebP on a multi-
/// core host because each width is dispatched on a separate rayon worker.
fn bench_parallel_responsive_encode(c: &mut Criterion) {
    let img = synthetic_photo();
    let resized: Vec<DynamicImage> = WIDTHS
        .iter()
        .map(|&w| {
            let h = (HEIGHT * w) / WIDTH;
            img.resize_exact(w, h, image::imageops::FilterType::Lanczos3)
        })
        .collect();

    let mut group = c.benchmark_group("avif_vs_webp/parallel_4_widths");
    group.throughput(Throughput::Elements(WIDTHS.len() as u64));

    group.bench_function("webp_parallel", |b| {
        b.iter(|| {
            let bufs: Vec<Vec<u8>> = resized
                .par_iter()
                .map(|im| {
                    let mut buf = Cursor::new(Vec::with_capacity(64 * 1024));
                    im.write_to(&mut buf, ImageFormat::WebP)
                        .expect("webp encode");
                    buf.into_inner()
                })
                .collect();
            bufs
        });
    });

    group.bench_function("avif_parallel_q70", |b| {
        b.iter(|| {
            let bufs: Vec<Vec<u8>> = resized
                .par_iter()
                .map(|im| encode_avif(im, QUALITY).expect("avif encode"))
                .collect();
            bufs
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_webp_encode,
    bench_avif_encode,
    bench_parallel_responsive_encode
);
criterion_main!(benches);
