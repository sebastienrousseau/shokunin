// Copyright © 2023-2024-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use std::path::Path;

use criterion::{black_box, Criterion};
use ssg::models::data::{ManifestData, TxtData, CnameData, HumansData, SiteMapData};
use ssg::modules::json::{manifest, txt, cname, human, sitemap};

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

    let humans_data = HumansData {
        author: String::from("Test Author"),
        author_website: String::from("https://www.test.com"),
        author_twitter: String::from("Test Twitter"),
        author_location: String::from("Test Location"),
        thanks: String::from("Test Thanks"),
        site_last_updated: String::from("2022-01-01"),
        site_standards: String::from("Test Standards"),
        site_components: String::from("Test Components"),
        site_software: String::from("Test Software"),
    };

    let cname_data = CnameData {
        cname: String::from("test.com"),
    };

    let sitemap_data = SiteMapData {
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

    c.bench_function("humans_data", |b| {
        b.iter(|| human(black_box(&humans_data)))
    });

    c.bench_function("sitemap", |b| {
        b.iter(|| sitemap(black_box(sitemap_data.clone()), black_box(dir)))
    });

}
