// Copyright © 2023 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;
use criterion::Criterion;

mod bench_file;
mod bench_frontmatter;
mod bench_html;
mod bench_json;
mod bench_metatags;
mod bench_utilities;

criterion::criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
        bench_file::bench_file,
        bench_frontmatter::bench_extract,
        bench_frontmatter::bench_parse_yaml_document,
        bench_html::bench_generate_html,
        bench_json::bench_json,
        bench_metatags::bench_metatags,
        bench_utilities::bench_utilities,
}

criterion::criterion_main!(benches);
