// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;

use criterion::{black_box, Criterion};
use ssg::cli::build;

pub fn bench_cli(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");

    group.bench_function("valid arguments", |b| {
        b.iter(|| {
            // Call the build function to get the command-line arguments
            let result = black_box(build());

            // Check for errors
            if let Err(e) = result {
                panic!("Error: {}", e);
            }

            // Unwrap the result
            let args = result.unwrap();

            // Define the expected argument values
            let arg_specs = [
                ("new", None),
                ("content", Some("")),
                ("output", Some("")),
                ("help", None),
                ("version", None),
            ];

            // Iterate through the expected argument values
            for (arg_name, expected_value) in arg_specs.iter() {
                // Get the actual value for the argument
                let arg_value: Option<&String> = args.get_one(arg_name);

                // Compare the actual and expected values
                assert_eq!(
                    arg_value.map(String::as_str).unwrap_or_default(),
                    expected_value.unwrap_or_default()
                );
            }
        })
    });

    group.finish();
}
