// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use std::path::Path;

use criterion::{black_box, Criterion};
use ssg::data::{CnameData, ManifestData, SitemapData, TxtData};
use ssg::json::{manifest, cname, sitemap, txt};

pub fn bench_json(c: &mut Criterion) {
    let manifest_data = ManifestData {
        name: String::from("Test Name"),
        short_name: String::from("Test Short Name"),
        start_url: String::from("/"),
        display: String::from("standalone"),
        background_color: String::from("#000"),
        description: String::from("Test Description"),
        icons: Vec::new(),
        orientation: String::from("portrait"),
        scope: String::from("/"),
        theme_color: String::from("#000"),
    };

    let txt_data = TxtData {
        permalink: String::from("https://www.test.com"),
    };

    let cname_data = CnameData {
        cname: String::from("test.com"),
    };

    let sitemap_data = SitemapData {
        changefreq: String::from("always"),
        loc: String::from("https://www.test.com"),
        lastmod: String::from("2022-01-01"),
    };

    let dir = Path::new("./");

    c.bench_function("manifest", |b| {
        b.iter(|| manifest(black_box(&manifest_data)))
    });

    c.bench_function("txt", |b| b.iter(|| txt(black_box(&txt_data))));

    c.bench_function("cname", |b| {
        b.iter(|| cname(black_box(&cname_data)))
    });

    // This will be a file-system intensive benchmark
    c.bench_function("sitemap", |b| {
        b.iter(|| sitemap(black_box(&sitemap_data), black_box(dir)))
    });
}
