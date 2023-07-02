// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::data::{IconData, ManifestData};
use ssg::json::manifest;

pub fn bench_json(c: &mut Criterion) {
    let options = ManifestData {
        background_color: "#ffffff".to_owned(),
        description: "My Web App".to_owned(),
        display: "standalone".to_owned(),
        icons: vec![IconData {
            src: "icons/icon-512x512.png".to_owned(),
            sizes: "512x512".to_string(),
            icon_type: Some("image/png".to_string()),
            purpose: Some("any maskable".to_string()),
        }],
        name: "My Web App".to_owned(),
        orientation: "portrait".to_owned(),
        scope: "/".to_owned(),
        short_name: "My App".to_owned(),
        start_url: "/index.html".to_owned(),
        theme_color: "#ffffff".to_owned(),
    };

    c.bench_function("generate manifest", |b| {
        b.iter(|| {
            let result = manifest(black_box(&options));
            assert!(
                result.contains("\"background_color\": \"#ffffff\"")
            );
            assert!(result.contains("\"description\": \"My Web App\""));
            assert!(result.contains("\"display\": \"standalone\""));
            assert!(result
                .contains("\"icons\": \"icons/icon-512x512.png\""));
            assert!(result.contains("\"name\": \"My Web App\""));
            assert!(result.contains("\"orientation\": \"portrait\""));
            assert!(result.contains("\"scope\": \"/\""));
            assert!(result.contains("\"short_name\": \"My App\""));
            assert!(result.contains("\"start_url\": \"/index.html\""));
            assert!(result.contains("\"theme_color\": \"#ffffff\""));
        })
    });
}
