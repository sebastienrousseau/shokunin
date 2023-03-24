use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use ssg::frontmatter::extract;

#[cfg(test)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let content = "---\n\
                   title: My Title\n\
                   date: 2000-01-01\n\
                   description: My Description\n\
                   keywords: foo, bar, baz\n\
                   permalink: /my-permalink\n\
                   layout: page\n\
                   ---\n\
                   My content";
    c.bench_function("extract front matter", |b| {
        b.iter(|| {
            let result = extract(black_box(content));
            assert_eq!(
                result,
                (
                    "My Title".to_owned(),
                    "2000-01-01".to_owned(),
                    "My Description".to_owned(),
                    "foo, bar, baz".to_owned(),
                    "/my-permalink".to_owned(),
                    "page".to_owned(),
                )
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
