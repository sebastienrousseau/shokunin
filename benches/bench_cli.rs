extern crate criterion;

use clap::{ArgMatches, Error};
use criterion::{criterion_group, criterion_main, Criterion};
use ssg::cli::build;
#[cfg(test)]
fn build_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");

    group.bench_function("valid arguments", |b| {
        b.iter(|| {
            // Call the build function to get the command-line arguments
            let result: Result<ArgMatches, Error> = build();

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

criterion_group!(benches, build_benchmark);
criterion_main!(benches);
