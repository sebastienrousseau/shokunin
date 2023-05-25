// Copyright © 2023 Shokunin (職人) Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use clap::{Arg, Command};
use criterion::{black_box, Criterion};
use ssg::process::args;

pub fn bench_parser(c: &mut Criterion) {
    // Test required arguments present
    let matches = Command::new("test")
        .arg(Arg::new("new"))
        .arg(Arg::new("content"))
        .arg(Arg::new("output"))
        .get_matches_from(vec!["test_name", "test_content", "output"]);

    c.bench_function("parse command line arguments", |b| {
        b.iter(|| {
            let result = args(black_box(&matches));
            assert!(result.is_ok());
        })
    });
}
