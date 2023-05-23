// Copyright © 2023 Shokunin (職人). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate criterion;
use criterion::Criterion;

mod bench_cli;
mod bench_file;
mod bench_frontmatter;
mod bench_html;
mod bench_json;
mod bench_metatags;
mod bench_parser;
mod bench_utilities;

criterion::criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
        bench_cli::bench_cli,
        bench_file::bench_file,
        bench_frontmatter::bench_frontmatter,
        bench_html::bench_html,
        bench_json::bench_json,
        bench_metatags::bench_metatags,
        bench_parser::bench_parser,
        bench_utilities::bench_utilities,
}

criterion::criterion_main!(benches);
