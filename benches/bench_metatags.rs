// Copyright © 2023-2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use criterion::{black_box, Criterion};
use ssg::modules::metatags::generate_metatags;

/**
 * This function is used to benchmark the performance of generating metatags.
 * It takes a reference to a mutable `Criterion` instance and generates metatags based on the provided `meta` vector.
 * The `meta` vector contains tuples of tag names and their corresponding content.
 * The function then asserts that the generated metatags contain the expected content for each tag.
 *
 * # Arguments
 *
 * * `c` - A mutable reference to a `Criterion` instance.
 *
 * # Returns
 *
 * This function does not return any value. It is a benchmarking function and its purpose is to measure the performance of generating metatags.
 */
#[allow(dead_code)]
pub(crate) fn bench_metatags(c: &mut Criterion) {
    let meta = vec![
        ("description".to_owned(), "My web page".to_owned()),
        ("author".to_owned(), "John Doe".to_owned()),
        (
            "viewport".to_owned(),
            "width=device-width, initial-scale=1.0".to_owned(),
        ),
        ("robots".to_owned(), "noindex, nofollow".to_owned()),
    ];

    c.bench_function("generate metatags", |b| {
        b.iter(|| {
            let result = generate_metatags(black_box(&meta));
            assert!(result.contains("<meta name=\"description\" content=\"My web page\">"));
            assert!(result.contains("<meta name=\"author\" content=\"John Doe\">"));
            assert!(result.contains(
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
            ));
            assert!(result.contains("<meta name=\"robots\" content=\"noindex, nofollow\">"));
        })
    });
}
